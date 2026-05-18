# Deployment Architecture - Unit 1: Database Layer

**Version**: 1.0  
**Date**: 2026-05-18  
**Unit**: Database Layer Migration  
**Deployment Target**: Docker Compose (single-machine)

---

## Overview

Complete deployment architecture for PostgreSQL database migration, including Docker Compose orchestration, Dockerfile specifications, and deployment procedures.

---

## 1. Complete docker-compose.yml

**File Location**: `<workspace-root>/docker-compose.yml`

```yaml
version: '3.8'

services:
  # PostgreSQL 15 Database
  postgresql:
    image: postgres:15-alpine
    container_name: activitywatch-postgresql
    restart: unless-stopped
    
    environment:
      POSTGRES_USER: aw_user
      POSTGRES_PASSWORD_FILE: /run/secrets/db_password
      POSTGRES_DB: activitywatch
      POSTGRES_INITDB_ARGS: "--encoding=UTF8 --locale=C"
      # PostgreSQL performance tuning
      POSTGRES_SHARED_BUFFERS: 4GB
      POSTGRES_EFFECTIVE_CACHE_SIZE: 12GB
      POSTGRES_MAX_CONNECTIONS: 50
    
    secrets:
      - db_password
    
    volumes:
      # Data persistence
      - pg_data:/var/lib/postgresql/data
      # Custom configuration
      - ./docker/postgresql.conf:/etc/postgresql/postgresql.conf:ro
      # Optional: Custom init scripts
      - ./docker/init-db.sh:/docker-entrypoint-initdb.d/init-db.sh:ro
    
    command: postgres -c config_file=/etc/postgresql/postgresql.conf
    
    networks:
      - internal
    
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U aw_user -d activitywatch"]
      interval: 10s
      timeout: 5s
      retries: 3
      start_period: 30s
    
    deploy:
      resources:
        limits:
          cpus: '8.0'
          memory: 24G
        reservations:
          cpus: '4.0'
          memory: 16G
    
    logging:
      driver: "json-file"
      options:
        max-size: "100m"
        max-file: "5"

  # ActivityWatch Server
  aw-server:
    build:
      context: .
      dockerfile: Dockerfile
      args:
        RUST_VERSION: "1.75"
    image: activitywatch/aw-server:latest
    container_name: activitywatch-server
    restart: unless-stopped
    
    environment:
      # Database connection
      DB_HOST: postgresql
      DB_PORT: 5432
      DB_USER: aw_user
      DB_PASSWORD_FILE: /run/secrets/db_password
      DB_NAME: activitywatch
      
      # Server configuration
      AW_HOST: "0.0.0.0"
      AW_PORT: 5600
      
      # Logging
      RUST_LOG: info
      DB_LOG_LEVEL: warn
    
    secrets:
      - db_password
    
    ports:
      - "5600:5600"
    
    networks:
      - internal
    
    depends_on:
      postgresql:
        condition: service_healthy
    
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:5600/api/0/info"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s
    
    deploy:
      resources:
        limits:
          cpus: '4.0'
          memory: 4G
        reservations:
          cpus: '2.0'
          memory: 2G
    
    logging:
      driver: "json-file"
      options:
        max-size: "100m"
        max-file: "5"

# Networks
networks:
  internal:
    driver: bridge
    ipam:
      driver: default
      config:
        - subnet: 172.20.0.0/16

# Volumes
volumes:
  pg_data:
    driver: local
    driver_opts:
      type: none
      o: bind
      device: /data/activitywatch/postgres

# Secrets
secrets:
  db_password:
    file: ./secrets/db_password.txt
```

---

## 2. Complete Dockerfile (aw-server)

**File Location**: `<workspace-root>/Dockerfile`

```dockerfile
#################################################
# Stage 1: Builder
#################################################
ARG RUST_VERSION=1.75
FROM rust:${RUST_VERSION}-bookworm AS builder

# Set working directory
WORKDIR /app

# Install build dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo workspace manifests first (for layer caching)
COPY Cargo.toml Cargo.lock ./

# Create dummy main files to build dependencies first (cache optimization)
RUN mkdir -p aw-server/src && \
    mkdir -p aw-datastore/src && \
    mkdir -p aw-models/src && \
    mkdir -p aw-query/src && \
    mkdir -p aw-transform/src && \
    mkdir -p aw-client-rust/src && \
    mkdir -p aw-sync/src && \
    echo "fn main() {}" > aw-server/src/main.rs && \
    echo "pub fn dummy() {}" > aw-datastore/src/lib.rs && \
    echo "pub fn dummy() {}" > aw-models/src/lib.rs && \
    echo "pub fn dummy() {}" > aw-query/src/lib.rs && \
    echo "pub fn dummy() {}" > aw-transform/src/lib.rs && \
    echo "pub fn dummy() {}" > aw-client-rust/src/lib.rs && \
    echo "pub fn dummy() {}" > aw-sync/src/lib.rs

# Copy crate manifests
COPY aw-server/Cargo.toml ./aw-server/
COPY aw-datastore/Cargo.toml ./aw-datastore/
COPY aw-models/Cargo.toml ./aw-models/
COPY aw-query/Cargo.toml ./aw-query/
COPY aw-transform/Cargo.toml ./aw-transform/
COPY aw-client-rust/Cargo.toml ./aw-client-rust/
COPY aw-sync/Cargo.toml ./aw-sync/

# Build dependencies only (cached layer)
RUN cargo build --release --bin aw-server

# Remove dummy source files and build artifacts
RUN rm -rf aw-server/src aw-datastore/src aw-models/src aw-query/src aw-transform/src aw-client-rust/src aw-sync/src && \
    rm -rf target/release/deps/aw_* target/release/aw-server*

# Copy actual source code
COPY aw-server/ ./aw-server/
COPY aw-datastore/ ./aw-datastore/
COPY aw-models/ ./aw-models/
COPY aw-query/ ./aw-query/
COPY aw-transform/ ./aw-transform/
COPY aw-client-rust/ ./aw-client-rust/
COPY aw-sync/ ./aw-sync/

# Build release binary with all optimizations
RUN cargo build --release --bin aw-server

# Strip debug symbols to reduce binary size
RUN strip target/release/aw-server

#################################################
# Stage 2: Runtime
#################################################
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libpq5 \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 -s /bin/bash awuser && \
    mkdir -p /home/awuser/.local/share/activitywatch && \
    chown -R awuser:awuser /home/awuser

# Copy binary from builder stage
COPY --from=builder /app/target/release/aw-server /usr/local/bin/aw-server

# Ensure binary is executable
RUN chmod +x /usr/local/bin/aw-server

# Switch to non-root user
USER awuser
WORKDIR /home/awuser

# Expose port
EXPOSE 5600

# Health check (using curl installed in runtime image)
HEALTHCHECK --interval=30s --timeout=10s --start-period=60s --retries=3 \
  CMD curl -f http://localhost:5600/api/0/info || exit 1

# Set default environment variables
ENV RUST_LOG=info \
    DB_LOG_LEVEL=warn \
    AW_HOST=0.0.0.0 \
    AW_PORT=5600

# Run server
CMD ["aw-server"]
```

---

## 3. PostgreSQL Configuration File

**File Location**: `<workspace-root>/docker/postgresql.conf`

```ini
#################################################
# PostgreSQL 15 Configuration
# Optimized for: 300 watchers, 24 GB RAM, 8 CPU
#################################################

#-----------------------
# Memory Configuration
#-----------------------
shared_buffers = 4GB                    # 25% of total RAM
effective_cache_size = 12GB             # 50% of total RAM
work_mem = 32MB                         # Per query operation
maintenance_work_mem = 512MB            # For VACUUM, CREATE INDEX
huge_pages = try                        # Try to use huge pages

#-----------------------
# Connection Configuration
#-----------------------
max_connections = 50                    # Max concurrent connections
superuser_reserved_connections = 3      # Reserved for superuser
shared_preload_libraries = 'pg_stat_statements'  # Query statistics

#-----------------------
# Performance Tuning
#-----------------------
random_page_cost = 1.1                  # SSD storage (lower for SSD)
effective_io_concurrency = 200          # SSD concurrent I/O capability
seq_page_cost = 1.0                     # Sequential page fetch cost

#-----------------------
# Query Planner
#-----------------------
default_statistics_target = 100         # Statistics detail level
constraint_exclusion = partition        # Enable for partitioned tables
cursor_tuple_fraction = 0.1             # Optimize for cursor queries

#-----------------------
# Write-Ahead Logging (WAL)
#-----------------------
wal_level = replica                     # Enable replication (future)
wal_buffers = 16MB                      # WAL buffer size
min_wal_size = 80MB                     # Minimum WAL size
max_wal_size = 2GB                      # Maximum WAL size before checkpoint
checkpoint_completion_target = 0.9      # Spread checkpoint I/O

#-----------------------
# Archiving (for backup)
#-----------------------
archive_mode = off                      # Enable for backup (future)
# archive_command = 'cp %p /archive/%f' # Command for archiving

#-----------------------
# Replication (future)
#-----------------------
max_wal_senders = 5                     # Max replication connections
wal_keep_size = 128MB                   # Keep WAL for replication

#-----------------------
# Autovacuum
#-----------------------
autovacuum = on                         # Enable automatic VACUUM
autovacuum_max_workers = 3              # Max concurrent workers
autovacuum_vacuum_scale_factor = 0.1    # Trigger VACUUM at 10% dead tuples
autovacuum_analyze_scale_factor = 0.05  # Trigger ANALYZE at 5% changes
autovacuum_vacuum_cost_limit = 200      # I/O cost limit per worker
autovacuum_naptime = 1min               # Check interval

#-----------------------
# Logging
#-----------------------
logging_collector = on                  # Enable log collection
log_destination = 'stderr'              # Log to stderr (captured by Docker)
log_directory = 'log'                   # Log directory
log_filename = 'postgresql-%Y-%m-%d_%H%M%S.log'  # Log file pattern
log_rotation_age = 1d                   # Rotate logs daily
log_rotation_size = 100MB               # Rotate when file exceeds size
log_truncate_on_rotation = off          # Append to rotated logs
log_line_prefix = '%t [%p]: [%l-1] user=%u,db=%d,app=%a,client=%h '  # Log format

# Log slow queries
log_min_duration_statement = 1000       # Log queries > 1 second
log_checkpoints = on                    # Log checkpoint activity
log_connections = on                    # Log new connections
log_disconnections = on                 # Log disconnections
log_lock_waits = on                     # Log lock waits
log_temp_files = 0                      # Log all temp files

# Don't log every statement (too verbose)
log_statement = 'none'                  # Options: none, ddl, mod, all

#-----------------------
# Performance Statistics
#-----------------------
shared_preload_libraries = 'pg_stat_statements'  # Query stats extension
pg_stat_statements.max = 10000          # Track up to 10k queries
pg_stat_statements.track = all          # Track all queries
track_activities = on                   # Track running queries
track_counts = on                       # Track table statistics
track_io_timing = on                    # Track I/O timing

#-----------------------
# Locale and Formatting
#-----------------------
datestyle = 'iso, mdy'
timezone = 'UTC'                        # Use UTC for consistency
lc_messages = 'C'                       # Use C locale for messages
lc_monetary = 'C'
lc_numeric = 'C'
lc_time = 'C'
default_text_search_config = 'pg_catalog.english'

#-----------------------
# Security
#-----------------------
ssl = off                               # SSL disabled (internal network)
# ssl_cert_file = 'server.crt'         # Enable for SSL
# ssl_key_file = 'server.key'
password_encryption = scram-sha-256     # Use SCRAM-SHA-256 for passwords
```

---

## 4. Optional Database Initialization Script

**File Location**: `<workspace-root>/docker/init-db.sh`

```bash
#!/bin/bash
set -e

# This script runs automatically during PostgreSQL initialization
# (only on first container startup when data directory is empty)

echo "Running ActivityWatch database initialization..."

# Create pg_stat_statements extension for query performance monitoring
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    -- Enable query statistics extension
    CREATE EXTENSION IF NOT EXISTS pg_stat_statements;
    
    -- Create index for pg_stat_statements (optional, improves query performance)
    -- Already built-in, but documenting for reference
    
    -- Log initialization complete
    SELECT 'ActivityWatch PostgreSQL initialization complete' AS status;
EOSQL

echo "Database initialization complete."
```

**Make executable**:
```bash
chmod +x docker/init-db.sh
```

---

## 5. .dockerignore File

**File Location**: `<workspace-root>/.dockerignore`

```
# Build artifacts
target/
**/*.rs.bk
*.pdb

# Development files
.git/
.gitignore
.env
.env.*

# Documentation
*.md
!README.md
docs/
aidlc-docs/
.aidlc-rule-details/

# Web UI (not needed for aw-server image)
aw-webui/node_modules/
aw-webui/dist/
aw-webui/build/
aw-webui/.cache/

# Docker files (avoid recursion)
Dockerfile
docker-compose.yml
docker-compose.*.yml
.dockerignore

# IDE files
.vscode/
.idea/
*.swp
*.swo
*~

# OS files
.DS_Store
Thumbs.db

# Secrets
secrets/
*.key
*.crt
*.pem

# Logs
*.log
logs/

# Testing
coverage/
.pytest_cache/
.coverage

# Temporary files
tmp/
temp/
*.tmp
```

---

## 6. Secrets Directory Structure

**Directory Structure**:
```
<workspace-root>/
├── secrets/
│   ├── .gitignore           # Exclude all secrets from Git
│   └── db_password.txt      # PostgreSQL password (single line)
```

**secrets/.gitignore**:
```
# Exclude all secrets from version control
*
!.gitignore
```

**Generate db_password.txt**:
```bash
mkdir -p secrets
openssl rand -base64 32 > secrets/db_password.txt
chmod 600 secrets/db_password.txt
```

**Security Notes**:
- ✅ Secrets directory excluded from Git
- ✅ File permissions restricted (600 = owner read/write only)
- ✅ Password generated with strong randomness (256-bit)
- ❌ Never commit secrets to version control
- ❌ Never share secrets via insecure channels

---

## 7. Deployment Procedures

### 7.1 Initial Deployment

**Prerequisites**:
- Docker 20.10+ installed
- Docker Compose 1.29+ installed
- Git repository cloned
- 16-32 GB RAM available
- 1-2 TB disk space available

**Steps**:

```bash
# 1. Clone repository
git clone https://github.com/ActivityWatch/aw-server-rust.git
cd aw-server-rust

# 2. Create required directories
mkdir -p docker secrets
mkdir -p /data/activitywatch/postgres

# 3. Generate database password
openssl rand -base64 32 > secrets/db_password.txt
chmod 600 secrets/db_password.txt

# 4. Copy PostgreSQL configuration (if customizing)
# (postgresql.conf should be committed to repo in docker/ directory)

# 5. Build Docker images
docker-compose build

# 6. Start services (detached mode)
docker-compose up -d

# 7. Monitor startup logs
docker-compose logs -f

# 8. Wait for services to become healthy
# (healthchecks will show services as "healthy" after ~30-60 seconds)

# 9. Verify deployment
curl http://localhost:5600/api/0/info
# Expected: JSON response with version info

# 10. Check database connectivity
docker exec activitywatch-postgresql psql -U aw_user -d activitywatch -c "SELECT version();"
# Expected: PostgreSQL version info
```

**Expected Startup Timeline**:
- **T+0s**: Services start
- **T+5s**: PostgreSQL starts accepting connections
- **T+10s**: PostgreSQL healthcheck passes
- **T+15s**: aw-server starts (depends_on satisfied)
- **T+30s**: Schema migrations complete
- **T+45s**: aw-server healthcheck passes
- **T+60s**: Deployment fully operational

---

### 7.2 Updates and Rollouts

**Zero-downtime Rolling Update** (for minor changes):

```bash
# 1. Pull latest code
git pull origin master

# 2. Rebuild images
docker-compose build

# 3. Rolling restart (recreate containers with new image)
docker-compose up -d --no-deps --build aw-server
# --no-deps: Don't restart dependent services (postgresql)
# --build: Rebuild image before starting

# 4. Verify new version
curl http://localhost:5600/api/0/info
docker-compose logs --tail=50 aw-server
```

**Full Restart** (for database schema changes):

```bash
# 1. Stop services gracefully
docker-compose down

# 2. Pull updates
git pull origin master

# 3. Rebuild all images
docker-compose build

# 4. Start services
docker-compose up -d

# 5. Monitor logs
docker-compose logs -f
```

**Downtime**: ~30-60 seconds (time for services to restart and pass healthchecks)

---

### 7.3 Backup Procedures

**Database Backup (pg_dump)**:

```bash
#!/bin/bash
# File: scripts/backup-database.sh

BACKUP_DIR="/backup/activitywatch"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/aw-${TIMESTAMP}.sql.gz"

# Create backup directory
mkdir -p "${BACKUP_DIR}"

# Dump database with compression
docker exec activitywatch-postgresql pg_dump -U aw_user activitywatch | gzip > "${BACKUP_FILE}"

# Verify backup file exists and has size > 0
if [ -s "${BACKUP_FILE}" ]; then
    echo "✅ Backup successful: ${BACKUP_FILE}"
    
    # Delete backups older than 30 days
    find "${BACKUP_DIR}" -name "aw-*.sql.gz" -mtime +30 -delete
    echo "🗑️ Cleaned up backups older than 30 days"
else
    echo "❌ Backup failed!"
    exit 1
fi
```

**Schedule with cron**:
```bash
# Run daily at 2 AM
0 2 * * * /path/to/scripts/backup-database.sh >> /var/log/aw-backup.log 2>&1
```

**Volume Backup (alternative)**:

```bash
#!/bin/bash
# File: scripts/backup-volume.sh

BACKUP_DIR="/backup/activitywatch"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="${BACKUP_DIR}/pg_data-${TIMESTAMP}.tar.gz"

mkdir -p "${BACKUP_DIR}"

# Backup entire PostgreSQL data volume
docker run --rm \
  -v activitywatch_pg_data:/data:ro \
  -v "${BACKUP_DIR}:/backup" \
  alpine \
  tar czf "/backup/pg_data-${TIMESTAMP}.tar.gz" /data

echo "✅ Volume backup complete: ${BACKUP_FILE}"
```

---

### 7.4 Restore Procedures

**Restore from SQL Dump**:

```bash
#!/bin/bash
# File: scripts/restore-database.sh

BACKUP_FILE="$1"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <backup-file.sql.gz>"
    exit 1
fi

# Stop aw-server (prevent write conflicts)
docker-compose stop aw-server

# Drop existing database and recreate (careful!)
docker exec activitywatch-postgresql psql -U aw_user postgres -c "DROP DATABASE IF EXISTS activitywatch;"
docker exec activitywatch-postgresql psql -U aw_user postgres -c "CREATE DATABASE activitywatch;"

# Restore from backup
gunzip -c "${BACKUP_FILE}" | docker exec -i activitywatch-postgresql psql -U aw_user activitywatch

# Restart aw-server
docker-compose start aw-server

echo "✅ Database restored from ${BACKUP_FILE}"
```

**Restore from Volume Backup**:

```bash
#!/bin/bash
# File: scripts/restore-volume.sh

BACKUP_FILE="$1"

if [ -z "$BACKUP_FILE" ]; then
    echo "Usage: $0 <pg_data-backup.tar.gz>"
    exit 1
fi

# Stop all services
docker-compose down

# Remove existing volume
docker volume rm activitywatch_pg_data

# Recreate volume
docker volume create activitywatch_pg_data

# Restore data
docker run --rm \
  -v activitywatch_pg_data:/data \
  -v "$(dirname ${BACKUP_FILE}):/backup" \
  alpine \
  tar xzf "/backup/$(basename ${BACKUP_FILE})" -C /data --strip-components=1

# Start services
docker-compose up -d

echo "✅ Volume restored from ${BACKUP_FILE}"
```

---

## 8. Monitoring and Operations

### 8.1 Health Monitoring

**Check Service Status**:
```bash
# View all services with health status
docker-compose ps

# Expected output:
# NAME                            STATUS              PORTS
# activitywatch-postgresql        Up (healthy)        5432/tcp
# activitywatch-server            Up (healthy)        0.0.0.0:5600->5600/tcp
```

**Check Individual Service Health**:
```bash
# PostgreSQL health
docker exec activitywatch-postgresql pg_isready -U aw_user -d activitywatch
# Expected: "activitywatch:5432 - accepting connections"

# aw-server health
curl http://localhost:5600/api/0/info
# Expected: JSON response
```

---

### 8.2 Resource Monitoring

**Real-time Resource Usage**:
```bash
# Show CPU, memory, network, disk I/O
docker stats

# Expected output columns:
# CONTAINER ID   NAME                   CPU %   MEM USAGE / LIMIT   MEM %   NET I/O       BLOCK I/O
# abc123         activitywatch-postgresql   5.23%   8.5GiB / 24GiB    35.42%  1.2MB / 3.4MB  500MB / 2GB
# def456         activitywatch-server       2.14%   1.2GiB / 4GiB     30.00%  3.4MB / 1.2MB  10MB / 50MB
```

**Historical Resource Usage** (requires metrics collection):
```bash
# Query Docker API for historical stats (requires external tool like cAdvisor)
# Or: Use Prometheus + Grafana for visualization
```

---

### 8.3 Log Management

**View Logs**:
```bash
# All services (follow mode)
docker-compose logs -f

# Specific service
docker-compose logs -f postgresql
docker-compose logs -f aw-server

# Last 100 lines
docker-compose logs --tail=100 aw-server

# Since timestamp
docker-compose logs --since=2026-05-18T10:00:00 aw-server

# Filter by keyword
docker-compose logs aw-server | grep ERROR
```

**Log Rotation**:
- Automatic rotation configured in docker-compose.yml
- Max size: 100 MB per log file
- Max files: 5 (500 MB total per container)
- Older logs automatically deleted

**Export Logs**:
```bash
# Export last 24 hours to file
docker-compose logs --since=24h > logs-$(date +%Y%m%d).txt
```

---

### 8.4 Database Operations

**Connect to PostgreSQL**:
```bash
# Interactive psql session
docker exec -it activitywatch-postgresql psql -U aw_user activitywatch

# Run single query
docker exec activitywatch-postgresql psql -U aw_user activitywatch -c "SELECT COUNT(*) FROM events;"
```

**Common Database Queries**:

```sql
-- Check database size
SELECT pg_size_pretty(pg_database_size('activitywatch'));

-- Check table sizes
SELECT 
  tablename,
  pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

-- Check event count
SELECT COUNT(*) FROM events;

-- Check bucket count
SELECT COUNT(*) FROM buckets;

-- Check recent events
SELECT id, bucket_id, starttime, endtime 
FROM events 
ORDER BY starttime DESC 
LIMIT 10;

-- Check connection pool usage (from aw-server metrics)
-- (Requires metrics endpoint implementation in aw-server)
```

**Performance Monitoring**:

```sql
-- Enable pg_stat_statements extension (if not enabled)
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- View slow queries
SELECT 
  mean_exec_time,
  calls,
  query
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;

-- View most frequent queries
SELECT 
  calls,
  mean_exec_time,
  query
FROM pg_stat_statements
ORDER BY calls DESC
LIMIT 10;

-- Reset statistics
SELECT pg_stat_statements_reset();
```

---

## 9. Troubleshooting

### 9.1 Common Issues

**Issue: PostgreSQL fails to start**

Symptoms:
```
activitywatch-postgresql | initdb: error: directory "/var/lib/postgresql/data" exists but is not empty
```

Solution:
```bash
# Remove corrupted data volume
docker-compose down -v
docker volume rm activitywatch_pg_data

# Restore from backup or start fresh
docker-compose up -d
```

---

**Issue: aw-server cannot connect to PostgreSQL**

Symptoms:
```
activitywatch-server | Error: Failed to connect to database: connection refused
```

Solution:
```bash
# 1. Check PostgreSQL health
docker-compose ps

# 2. Check network connectivity
docker exec activitywatch-server ping postgresql

# 3. Check PostgreSQL logs
docker-compose logs postgresql

# 4. Verify environment variables
docker exec activitywatch-server env | grep DB_

# 5. Restart services
docker-compose restart
```

---

**Issue: Services unhealthy after startup**

Symptoms:
```
activitywatch-server | unhealthy
```

Solution:
```bash
# 1. Check healthcheck logs
docker inspect activitywatch-server --format='{{json .State.Health}}' | jq

# 2. Manually test healthcheck
docker exec activitywatch-server curl -f http://localhost:5600/api/0/info

# 3. Check application logs
docker-compose logs --tail=100 aw-server

# 4. Increase start_period if migrations take longer
# Edit docker-compose.yml: healthcheck.start_period: 120s
```

---

**Issue: Disk space exhausted**

Symptoms:
```
PostgreSQL | ERROR:  could not extend file: No space left on device
```

Solution:
```bash
# 1. Check disk usage
df -h /data/activitywatch/postgres

# 2. Remove old backups
find /backup/activitywatch -name "*.sql.gz" -mtime +7 -delete

# 3. Vacuum database (reclaim space)
docker exec activitywatch-postgresql psql -U aw_user activitywatch -c "VACUUM FULL;"

# 4. Resize volume or add more disk space
```

---

### 9.2 Performance Troubleshooting

**Slow Query Performance**:

```bash
# 1. Enable query logging (temporarily)
docker exec activitywatch-postgresql psql -U aw_user activitywatch -c "ALTER SYSTEM SET log_min_duration_statement = 100;"
docker exec activitywatch-postgresql psql -U aw_user activitywatch -c "SELECT pg_reload_conf();"

# 2. Monitor logs for slow queries
docker-compose logs -f postgresql | grep "duration:"

# 3. Analyze slow queries
docker exec activitywatch-postgresql psql -U aw_user activitywatch
# Then: EXPLAIN ANALYZE <slow_query>;

# 4. Check missing indexes
SELECT 
  schemaname, tablename, attname, n_distinct, correlation
FROM pg_stats
WHERE schemaname = 'public'
ORDER BY abs(correlation) DESC;

# 5. Disable query logging
docker exec activitywatch-postgresql psql -U aw_user activitywatch -c "ALTER SYSTEM RESET log_min_duration_statement;"
docker exec activitywatch-postgresql psql -U aw_user activitywatch -c "SELECT pg_reload_conf();"
```

---

**High Memory Usage**:

```bash
# 1. Check memory usage
docker stats --no-stream

# 2. Check PostgreSQL buffer usage
docker exec activitywatch-postgresql psql -U aw_user activitywatch -c "
  SELECT
    setting AS shared_buffers,
    pg_size_pretty(setting::bigint * 8192) AS size
  FROM pg_settings
  WHERE name = 'shared_buffers';
"

# 3. Reduce shared_buffers if needed
# Edit docker/postgresql.conf: shared_buffers = 2GB
docker-compose restart postgresql
```

---

## 10. Deployment Architecture Summary

### 10.1 Component Diagram

```
┌────────────────────────────────────────────────────────────────────┐
│                        Docker Host (macOS)                         │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │              Docker Bridge Network (internal)                │ │
│  │              Subnet: 172.20.0.0/16                           │ │
│  │                                                              │ │
│  │  ┌────────────────────────┐      ┌───────────────────────┐  │ │
│  │  │   PostgreSQL           │      │   aw-server          │  │ │
│  │  │   postgres:15-alpine   │◄─────┤   Custom Rust Build  │  │ │
│  │  │                        │      │                       │  │ │
│  │  │   Resources:           │      │   Resources:          │  │ │
│  │  │   - CPU: 4-8 cores     │      │   - CPU: 2-4 cores    │  │ │
│  │  │   - RAM: 16-24 GB      │      │   - RAM: 2-4 GB       │  │ │
│  │  │   - Port: 5432         │      │   - Port: 5600        │  │ │
│  │  │                        │      │                       │  │ │
│  │  │   Volume:              │      │   Secrets:            │  │ │
│  │  │   pg_data mounted      │      │   db_password         │  │ │
│  │  │   /var/lib/postgresql  │      │                       │  │ │
│  │  │                        │      │                       │  │ │
│  │  │   Health: pg_isready   │      │   Health: /api/0/info │  │ │
│  │  │   10s interval         │      │   30s interval        │  │ │
│  │  └────────────────────────┘      └───────────┬───────────┘  │ │
│  │              ▲                                │              │ │
│  │              │                                │              │ │
│  │              │                                │              │ │
│  │              │ (no external access)           │ Port Forward │ │
│  │              │                                │              │ │
│  └──────────────┼────────────────────────────────┼──────────────┘ │
│                 │                                │                │
│                 X                                ▼                │
│            (blocked)                      Host :5600              │
│                                     (accessible externally)       │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │                  Docker Named Volume                         │ │
│  │                                                              │ │
│  │  pg_data                                                     │ │
│  │  Bind mount: /data/activitywatch/postgres                    │ │
│  │  Size: 1-2 TB capacity                                       │ │
│  │  Backup: Daily via pg_dump                                   │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │                  Docker Secrets                              │ │
│  │                                                              │ │
│  │  db_password                                                 │ │
│  │  File: ./secrets/db_password.txt                             │ │
│  │  Mounted: /run/secrets/db_password                           │ │
│  └──────────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────┘
```

---

### 10.2 Deployment Checklist

**Pre-deployment**:
- [ ] Docker and Docker Compose installed
- [ ] 16-32 GB RAM available
- [ ] 1-2 TB disk space available (SSD recommended)
- [ ] 8-16 CPU cores available
- [ ] Firewall configured (allow port 5600)
- [ ] Git repository cloned
- [ ] Secrets generated (`secrets/db_password.txt`)
- [ ] PostgreSQL configuration reviewed (`docker/postgresql.conf`)
- [ ] Backup strategy planned (cron job, backup location)

**Deployment**:
- [ ] Docker Compose build successful
- [ ] Services started (`docker-compose up -d`)
- [ ] PostgreSQL healthcheck passing
- [ ] aw-server healthcheck passing
- [ ] API accessible (`curl http://localhost:5600/api/0/info`)
- [ ] Database schema created (check `schema_version` table)
- [ ] Logs show no errors

**Post-deployment**:
- [ ] Monitoring configured (docker stats, Prometheus, etc.)
- [ ] Backup cron job scheduled
- [ ] Log rotation working (check log file sizes)
- [ ] Performance baseline established
- [ ] Documentation updated
- [ ] Team notified

**Ongoing Operations**:
- [ ] Daily health checks
- [ ] Weekly backup verification
- [ ] Monthly performance review
- [ ] Quarterly capacity planning

---

## 11. File Location Summary

| File | Location | Purpose |
|------|----------|---------|
| docker-compose.yml | `<workspace-root>/` | Service orchestration |
| Dockerfile | `<workspace-root>/` | aw-server image build |
| postgresql.conf | `<workspace-root>/docker/` | PostgreSQL tuning |
| init-db.sh | `<workspace-root>/docker/` | Database initialization |
| .dockerignore | `<workspace-root>/` | Build context exclusions |
| db_password.txt | `<workspace-root>/secrets/` | Database password |
| backup-database.sh | `<workspace-root>/scripts/` | Backup automation |
| restore-database.sh | `<workspace-root>/scripts/` | Restore automation |

**Total Files**: 8 files to create/modify for complete deployment infrastructure.

---

## 12. Next Steps

After Infrastructure Design approval:
1. **Code Generation (Unit 1)**: Implement database migration code
2. **Infrastructure Design (Unit 2)**: Add aw-webui service to docker-compose.yml
3. **Code Generation (Unit 2)**: Finalize network configuration
4. **Build & Test**: Comprehensive testing of full stack
