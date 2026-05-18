# Infrastructure Design - Unit 1: Database Layer

**Version**: 1.0  
**Date**: 2026-05-18  
**Unit**: Database Layer Migration  
**Deployment Target**: Docker Compose (single-machine deployment)

---

## Overview

This document maps logical components from NFR Design to actual Docker infrastructure services for PostgreSQL database deployment.

---

## 1. Infrastructure Mapping

### 1.1 Component to Service Mapping

| Logical Component | Infrastructure Service | Deployment Method | Justification |
|-------------------|----------------------|-------------------|---------------|
| PostgreSQL Server | `postgres:15-alpine` Docker image | Docker container | Official PostgreSQL image, alpine for smaller size |
| Database Worker (Connection Pool) | Embedded in aw-server | N/A (application code) | tokio-postgres + deadpool run in-process |
| aw-server Application | Custom Rust Docker image | Docker container | Multi-stage build for optimized binary |
| Persistent Storage | Docker named volume | Volume mount | Survives container restarts/recreations |
| Internal Network | Docker bridge network | Docker network | Isolate PostgreSQL from external access |
| Health Monitoring | Docker healthchecks | Container config | Built-in Docker health checking |

---

## 2. Compute Infrastructure

### 2.1 PostgreSQL Container

**Base Image**: `postgres:15-alpine`

**Rationale**:
- Official PostgreSQL image (trusted, maintained)
- Alpine Linux base (smaller image size: ~250MB vs ~380MB for debian)
- PostgreSQL 15 LTS (long-term support until 2027)

**Resource Allocation**:
```yaml
resources:
  limits:
    cpus: '8.0'       # Up to 8 CPU cores
    memory: 24G       # 24 GB RAM (PostgreSQL + caching)
  reservations:
    cpus: '4.0'       # Minimum 4 CPU cores
    memory: 16G       # Minimum 16 GB RAM
```

**Rationale**:
- 300 watchers → 30 eps sustained, 300 eps peak
- 16-32 GB RAM needed for shared_buffers (4GB) + effective_cache_size (12GB) + overhead
- 8-16 CPU cores for concurrent query processing

**Environment Variables**:
```yaml
environment:
  POSTGRES_USER: aw_user
  POSTGRES_PASSWORD_FILE: /run/secrets/db_password  # Docker secret
  POSTGRES_DB: activitywatch
  POSTGRES_INITDB_ARGS: "--encoding=UTF8 --locale=C"
```

**Configuration Tuning**:
- Custom `postgresql.conf` mounted as volume (see section 2.3)
- Overrides for shared_buffers, effective_cache_size, max_connections

---

### 2.2 aw-server Container

**Base Image**: Custom multi-stage Rust build

**Dockerfile**:
```dockerfile
# Stage 1: Build
FROM rust:1.75-bookworm AS builder

WORKDIR /app

# Copy workspace manifest
COPY Cargo.toml Cargo.lock ./

# Copy all crate directories
COPY aw-server/ ./aw-server/
COPY aw-datastore/ ./aw-datastore/
COPY aw-models/ ./aw-models/
COPY aw-query/ ./aw-query/
COPY aw-transform/ ./aw-transform/
COPY aw-client-rust/ ./aw-client-rust/
COPY aw-sync/ ./aw-sync/

# Build release binary
RUN cargo build --release --bin aw-server

# Stage 2: Runtime
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/aw-server /usr/local/bin/

# Expose port
EXPOSE 5600

# Run as non-root user
RUN useradd -m -u 1000 awuser
USER awuser

CMD ["aw-server", "--host", "0.0.0.0", "--port", "5600"]
```

**Image Size**: ~150 MB (Rust binary + minimal debian + libpq)

**Resource Allocation**:
```yaml
resources:
  limits:
    cpus: '4.0'       # Up to 4 CPU cores
    memory: 4G        # 4 GB RAM
  reservations:
    cpus: '2.0'       # Minimum 2 CPU cores
    memory: 2G        # Minimum 2 GB RAM
```

**Rationale**:
- aw-server is lightweight (mostly I/O-bound)
- 2-4 CPU cores sufficient for 30-50 concurrent API requests
- 2-4 GB RAM for Rocket framework + connection pool + request handling

**Environment Variables**:
```yaml
environment:
  DB_HOST: postgresql                # Docker service name
  DB_PORT: 5432
  DB_USER: aw_user
  DB_PASSWORD_FILE: /run/secrets/db_password
  DB_NAME: activitywatch
  RUST_LOG: info                     # Logging level
  DB_LOG_LEVEL: warn                 # Database query logging
```

---

### 2.3 Configuration Files as Volumes

**PostgreSQL Configuration Override**:

File: `docker/postgresql.conf`

```ini
# Memory Configuration (for 24 GB RAM system)
shared_buffers = 4GB
effective_cache_size = 12GB
work_mem = 32MB
maintenance_work_mem = 512MB

# Connection Configuration
max_connections = 50
superuser_reserved_connections = 3

# Performance Configuration
random_page_cost = 1.1                # SSD storage
effective_io_concurrency = 200        # SSD concurrent I/O
checkpoint_completion_target = 0.9
wal_buffers = 16MB

# WAL Configuration
wal_level = replica                   # Enable replication (future)
max_wal_size = 2GB
min_wal_size = 80MB

# Autovacuum Configuration
autovacuum = on
autovacuum_max_workers = 3
autovacuum_vacuum_scale_factor = 0.1
autovacuum_analyze_scale_factor = 0.05

# Query Planner Configuration
default_statistics_target = 100
```

**Mount in docker-compose.yml**:
```yaml
volumes:
  - ./docker/postgresql.conf:/etc/postgresql/postgresql.conf:ro
command: postgres -c config_file=/etc/postgresql/postgresql.conf
```

---

## 3. Storage Infrastructure

### 3.1 Named Volume for PostgreSQL Data

**Volume Definition**:
```yaml
volumes:
  pg_data:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /data/activitywatch/postgres  # Host path
```

**Rationale**:
- **Named volume**: Survives `docker-compose down` (data persistence)
- **Bind mount**: Store data on host filesystem (easier backup/restore)
- **Explicit path**: Control exact storage location

**Volume Mount**:
```yaml
services:
  postgresql:
    volumes:
      - pg_data:/var/lib/postgresql/data
```

**Storage Requirements**:
- **Initial**: ~1 GB (PostgreSQL base + initial schema)
- **1 year**: ~150 GB (1.1B events)
- **5 years**: ~750 GB (5.5B events)
- **Recommended**: 1-2 TB filesystem capacity

**Backup Strategy**:
```bash
# Daily backup script (cron)
#!/bin/bash
docker exec activitywatch-postgresql-1 pg_dump -U aw_user activitywatch | gzip > /backup/aw-$(date +%Y%m%d).sql.gz

# Retention: Keep 30 days of backups
find /backup -name "aw-*.sql.gz" -mtime +30 -delete
```

---

### 3.2 Secrets Management

**Docker Secrets** (for production):

```yaml
secrets:
  db_password:
    file: ./secrets/db_password.txt  # Single-line file with password
```

**Secret File Creation**:
```bash
# Create secrets directory
mkdir -p secrets
chmod 700 secrets

# Generate secure password
openssl rand -base64 32 > secrets/db_password.txt
chmod 600 secrets/db_password.txt
```

**Secret Consumption**:
```yaml
services:
  postgresql:
    secrets:
      - db_password
    environment:
      POSTGRES_PASSWORD_FILE: /run/secrets/db_password
  
  aw-server:
    secrets:
      - db_password
    environment:
      DB_PASSWORD_FILE: /run/secrets/db_password
```

**Alternative (Development)**: Plain environment variable
```yaml
environment:
  POSTGRES_PASSWORD: dev_password_123  # NOT FOR PRODUCTION
```

---

## 4. Networking Infrastructure

### 4.1 Internal Network Topology

**Network Definition**:
```yaml
networks:
  internal:
    driver: bridge
    internal: false  # Allow external access to aw-server (not PostgreSQL)
```

**Network Assignment**:
```yaml
services:
  postgresql:
    networks:
      - internal
    # No ports exposed to host
  
  aw-server:
    networks:
      - internal
    ports:
      - "5600:5600"  # Expose to host
```

**Topology**:
```
┌─────────────────────────────────────────────────┐
│              Docker Host (macOS)                │
│                                                 │
│  ┌───────────────────────────────────────────┐ │
│  │      Docker Bridge Network (internal)     │ │
│  │                                           │ │
│  │  ┌─────────────┐      ┌───────────────┐  │ │
│  │  │ postgresql  │◄─────┤   aw-server   │  │ │
│  │  │   :5432     │      │     :5600     │  │ │
│  │  └─────────────┘      └───────┬───────┘  │ │
│  │         ▲                      │          │ │
│  └─────────┼──────────────────────┼──────────┘ │
│            │                      │            │
│            │ (no external         │ Port       │
│            │  access)             │ Forward    │
└────────────┼──────────────────────┼────────────┘
             │                      │
             X                      ▼
       (blocked)            Host :5600
                        (accessible externally)
```

**Security Benefits**:
- PostgreSQL not accessible from host or internet
- Only aw-server can connect to PostgreSQL (via internal network)
- aw-server exposed on 5600 (API access)

---

### 4.2 Service Discovery

**DNS Resolution**:
- Docker Compose provides automatic DNS resolution
- Service name = hostname (`postgresql`, `aw-server`)

**Connection String**:
```rust
// In aw-server code
let db_host = env::var("DB_HOST").unwrap_or("postgresql".to_string());
// Resolves to PostgreSQL container IP automatically
```

**Benefits**:
- No hardcoded IP addresses
- Services can restart without breaking connections
- Load balancing (if multiple containers, future)

---

## 5. Health Monitoring Infrastructure

### 5.1 PostgreSQL Health Check

**Healthcheck Configuration**:
```yaml
services:
  postgresql:
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U aw_user -d activitywatch"]
      interval: 10s
      timeout: 5s
      retries: 3
      start_period: 30s
```

**Behavior**:
- **Test**: `pg_isready` checks PostgreSQL connectivity
- **Interval**: Check every 10 seconds
- **Timeout**: Fail if no response in 5 seconds
- **Retries**: Mark unhealthy after 3 consecutive failures
- **Start Period**: Grace period during initial startup (30 seconds)

**Health States**:
- `starting` - Container starting, within start_period
- `healthy` - Healthcheck passing
- `unhealthy` - Healthcheck failing (after retries)

**Actions on Unhealthy**:
- Docker logs warning
- Dependent services won't start (if depends_on: condition: service_healthy)
- Orchestrators (Kubernetes, Swarm) may restart container

---

### 5.2 aw-server Health Check

**Healthcheck Configuration**:
```yaml
services:
  aw-server:
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:5600/api/0/info"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s
    depends_on:
      postgresql:
        condition: service_healthy
```

**Behavior**:
- **Test**: HTTP request to `/api/0/info` endpoint (existing endpoint)
- **Interval**: Check every 30 seconds
- **Timeout**: Fail if no response in 10 seconds
- **Retries**: Mark unhealthy after 3 consecutive failures
- **Start Period**: Grace period for migrations + startup (60 seconds)

**Dependency**:
- `depends_on.postgresql.condition: service_healthy` - Wait for PostgreSQL before starting
- Ensures migrations can run successfully

---

### 5.3 Monitoring Dashboard (Future)

**Options for Production Monitoring**:

1. **Prometheus + Grafana**:
   - Scrape `/metrics` endpoint from aw-server
   - Visualize query latency, pool utilization, error rates
   - Alert on threshold violations

2. **Docker Stats**:
   - Built-in: `docker stats` command
   - Shows CPU, memory, network I/O per container
   - Lightweight, no external dependencies

3. **Logs Aggregation**:
   - Collect stdout logs from all containers
   - Options: Loki, ELK stack, Fluentd
   - Centralized log searching and analysis

**Status**: Not implemented initially (stdout logs + docker stats sufficient)

---

## 6. Deployment Architecture Decisions

### 6.1 Single-Machine Deployment

**Choice**: Docker Compose on single host

**Rationale**:
- 300 watchers well within single-machine capacity
- Simpler operations (no orchestrator complexity)
- Lower resource overhead (no control plane)

**Scaling Considerations**:
- Vertical scaling sufficient up to ~1000 watchers
- Horizontal scaling requires orchestrator (Kubernetes, Swarm)
- Not needed for initial deployment

---

### 6.2 Container Restart Policy

**Configuration**:
```yaml
services:
  postgresql:
    restart: unless-stopped
  
  aw-server:
    restart: unless-stopped
```

**Policy**: `unless-stopped`

**Behavior**:
- Restart on failure (exit code != 0)
- Restart on Docker daemon restart
- Don't restart if manually stopped (`docker stop`)

**Alternative Policies**:
- `no` - Never restart (not recommended)
- `always` - Always restart (even if manually stopped)
- `on-failure` - Restart only on non-zero exit (may not restart on daemon restart)

**Rationale**: `unless-stopped` provides automatic recovery while respecting manual stops.

---

### 6.3 Logging Configuration

**Configuration**:
```yaml
services:
  postgresql:
    logging:
      driver: "json-file"
      options:
        max-size: "100m"
        max-file: "5"
  
  aw-server:
    logging:
      driver: "json-file"
      options:
        max-size: "100m"
        max-file: "5"
```

**Log Rotation**:
- **max-size**: Rotate when log file exceeds 100 MB
- **max-file**: Keep 5 most recent log files
- **Total storage**: Up to 500 MB per container

**Log Access**:
```bash
# View logs
docker logs activitywatch-postgresql-1
docker logs activitywatch-aw-server-1

# Follow logs (tail -f)
docker logs -f activitywatch-aw-server-1

# View logs with timestamps
docker logs -t activitywatch-aw-server-1
```

**Log Retention**: 5 files × 100 MB = 500 MB per container (reasonable for troubleshooting)

---

## 7. Build Infrastructure

### 7.1 Multi-Stage Docker Build

**Stage 1: Builder** (Rust compilation)
- Base: `rust:1.75-bookworm` (Debian with Rust toolchain)
- Size: ~2 GB (includes build dependencies)
- Output: `/app/target/release/aw-server` binary

**Stage 2: Runtime** (Final image)
- Base: `debian:bookworm-slim` (minimal Debian)
- Size: ~150 MB (binary + libpq + ca-certificates)
- Output: Optimized runtime image

**Benefits**:
- ✅ Small final image (150 MB vs 2 GB)
- ✅ No build tools in production image (security)
- ✅ Faster deployment (smaller image transfer)
- ✅ Reproducible builds (pinned Rust version)

---

### 7.2 Build Context Optimization

**.dockerignore**:
```
target/
aw-webui/node_modules/
aw-webui/dist/
.git/
*.md
.gitignore
docker-compose.yml
Dockerfile
aidlc-docs/
.aidlc-rule-details/
```

**Benefits**:
- Smaller build context (faster uploads to Docker daemon)
- Avoid invalidating cache with unrelated file changes

---

## 8. Infrastructure as Code

### 8.1 docker-compose.yml

**Primary deployment artifact** (see deployment-architecture.md for complete file)

**Key Features**:
- Version: `3.8` (supports healthchecks, secrets)
- Services: `postgresql`, `aw-server` (aw-webui in Unit 2)
- Networks: `internal` (bridge network)
- Volumes: `pg_data` (named volume)
- Secrets: `db_password` (file-based secret)

---

### 8.2 Infrastructure Versioning

**Git Repository Structure**:
```
aw-server-rust/
├── Dockerfile               # aw-server image build
├── docker-compose.yml       # Deployment orchestration
├── docker/
│   ├── postgresql.conf      # PostgreSQL tuning
│   └── init-db.sh           # Optional init script
├── secrets/
│   └── .gitignore           # Exclude secrets from Git
└── ...
```

**Version Control**:
- ✅ Commit: Dockerfile, docker-compose.yml, config files
- ❌ Exclude: secrets/, volumes/, logs/

---

## 9. Operations Infrastructure

### 9.1 Deployment Commands

**Initial Deployment**:
```bash
# Generate secret
mkdir -p secrets
openssl rand -base64 32 > secrets/db_password.txt
chmod 600 secrets/db_password.txt

# Build images
docker-compose build

# Start services
docker-compose up -d

# Check health
docker-compose ps
docker-compose logs -f aw-server
```

**Updates** (rolling update):
```bash
# Pull latest code
git pull

# Rebuild images
docker-compose build

# Restart services
docker-compose up -d
```

**Backup**:
```bash
# Backup database
docker exec activitywatch-postgresql-1 pg_dump -U aw_user activitywatch > backup.sql

# Backup volume (alternative)
docker run --rm -v activitywatch_pg_data:/data -v $(pwd):/backup alpine tar czf /backup/pg_data.tar.gz /data
```

**Restore**:
```bash
# Restore from SQL dump
cat backup.sql | docker exec -i activitywatch-postgresql-1 psql -U aw_user activitywatch

# Restore from volume backup
docker run --rm -v activitywatch_pg_data:/data -v $(pwd):/backup alpine tar xzf /backup/pg_data.tar.gz -C /
```

---

### 9.2 Monitoring Commands

**Health Status**:
```bash
docker-compose ps  # Shows health status per service
```

**Resource Usage**:
```bash
docker stats  # Live CPU, memory, network I/O
```

**Logs**:
```bash
docker-compose logs postgresql  # PostgreSQL logs
docker-compose logs aw-server   # aw-server logs
docker-compose logs -f          # Follow all logs
```

**Database Inspection**:
```bash
# Connect to PostgreSQL
docker exec -it activitywatch-postgresql-1 psql -U aw_user activitywatch

# Run queries
SELECT COUNT(*) FROM events;
SELECT pg_size_pretty(pg_database_size('activitywatch'));
```

---

## 10. Infrastructure Scaling Strategy

### 10.1 Vertical Scaling (Current)

**Resource Increase**:
- Increase CPU limits in docker-compose.yml
- Increase memory limits
- Increase PostgreSQL shared_buffers, effective_cache_size

**When to Scale Up**:
- CPU utilization > 80% sustained
- Memory pressure (swapping)
- Query latency degradation

---

### 10.2 Horizontal Scaling (Future)

**Not Needed Initially**, but future options:

1. **Read Replicas** (PostgreSQL streaming replication):
   - Offload queries to read-only replicas
   - Requires wal_level=replica (already configured)

2. **Load Balancing** (multiple aw-server instances):
   - Add nginx/traefik load balancer
   - Run multiple aw-server containers
   - Each with own connection pool

3. **Orchestrator Migration** (Kubernetes):
   - Convert docker-compose.yml to Kubernetes manifests
   - StatefulSet for PostgreSQL
   - Deployment for aw-server
   - Horizontal Pod Autoscaler (HPA)

**Trigger**: When single-machine capacity exhausted (>1000 watchers)

---

## 11. Infrastructure Security

### 11.1 Container Security

**Non-root User**:
- aw-server runs as `awuser` (UID 1000)
- PostgreSQL runs as `postgres` user (built-in)

**Read-only Filesystem** (future enhancement):
```yaml
read_only: true
tmpfs:
  - /tmp
```

**Capability Dropping** (future enhancement):
```yaml
cap_drop:
  - ALL
cap_add:
  - NET_BIND_SERVICE  # Only if needed for port <1024
```

---

### 11.2 Network Security

**Firewall Rules** (host-level):
```bash
# Allow only port 5600 (aw-server API)
ufw allow 5600/tcp

# Block PostgreSQL port (redundant, but defense-in-depth)
ufw deny 5432/tcp
```

**Internal Network Isolation**:
- PostgreSQL only accessible via Docker internal network
- No port mapping to host (`ports:` not defined for postgresql)

---

## 12. Infrastructure Cost Optimization

### 12.1 Resource Allocation

**Right-sizing**:
- Start with minimum reservations (4 CPU, 16 GB RAM)
- Monitor actual usage
- Scale up only when needed

**Storage Efficiency**:
- Use alpine images (smaller)
- Multi-stage builds (remove build artifacts)
- Volume bind mounts (no image bloat)

---

### 12.2 Infrastructure Alternatives

**Cost Comparison** (hypothetical cloud deployment):

| Option | Monthly Cost (est.) | Notes |
|--------|---------------------|-------|
| On-premise (current) | $0 (hardware owned) | Recommended for initial deployment |
| AWS EC2 (t3.2xlarge) | ~$120 | 8 vCPU, 32 GB RAM |
| AWS RDS PostgreSQL | ~$300 | Managed database |
| Azure VM (Standard D8s v3) | ~$280 | 8 vCPU, 32 GB RAM |
| GCP Compute (n2-standard-8) | ~$240 | 8 vCPU, 32 GB RAM |

**Recommendation**: On-premise deployment sufficient for initial 300-watcher scale.

---

## 13. Infrastructure Validation

### 13.1 Pre-deployment Checklist

✅ Docker and Docker Compose installed  
✅ Sufficient disk space (1-2 TB)  
✅ Sufficient RAM (16-32 GB)  
✅ Sufficient CPU (8-16 cores)  
✅ Secrets generated and secured  
✅ Firewall rules configured  
✅ Backup strategy defined  
✅ Monitoring plan in place  

---

### 13.2 Post-deployment Validation

```bash
# 1. Check services are running
docker-compose ps
# Expected: postgresql and aw-server both "Up" and "healthy"

# 2. Check PostgreSQL connectivity
docker exec activitywatch-postgresql-1 pg_isready -U aw_user
# Expected: "accepting connections"

# 3. Check aw-server API
curl http://localhost:5600/api/0/info
# Expected: JSON response with version info

# 4. Check database schema
docker exec -it activitywatch-postgresql-1 psql -U aw_user activitywatch -c "\dt"
# Expected: List of tables (buckets, events, key_value, schema_version)

# 5. Check logs for errors
docker-compose logs --tail=100 | grep -i error
# Expected: No critical errors
```

---

## 14. Infrastructure Summary

| Infrastructure Component | Service/Technology | Configuration | Rationale |
|-------------------------|-------------------|---------------|-----------|
| **Database Server** | postgres:15-alpine | 8 CPU, 24 GB RAM | LTS version, optimized for 300 watchers |
| **Application Server** | Custom Rust image | 4 CPU, 4 GB RAM | Multi-stage build for small image |
| **Persistent Storage** | Named Docker volume | 1-2 TB capacity | Bind mount for easier backup |
| **Networking** | Docker bridge | Internal network | Isolate PostgreSQL from external |
| **Health Monitoring** | Docker healthchecks | 10s/30s intervals | Auto-recovery on failures |
| **Secrets** | Docker secrets | File-based | Secure credential management |
| **Logging** | JSON file driver | 100 MB × 5 files | Automatic log rotation |
| **Restart Policy** | unless-stopped | Auto-restart | Survive failures + daemon restarts |
| **Build** | Multi-stage Dockerfile | Builder + runtime | Small image size (~150 MB) |
| **Orchestration** | Docker Compose 3.8 | Single-machine | Simple, sufficient for scale |

**Total Infrastructure Footprint**:
- **CPU**: 12 cores (8 PostgreSQL + 4 aw-server)
- **Memory**: 28 GB (24 GB PostgreSQL + 4 GB aw-server)
- **Storage**: 1-2 TB (database + backups)
- **Network**: 1 Gbps (adequate for API traffic)

**Deployment Complexity**: Low (Docker Compose, no orchestrator)

**Operational Overhead**: Low (Docker handles restart, health checks, logs)

**Scalability**: Vertical scaling to ~1000 watchers, then consider horizontal scaling
