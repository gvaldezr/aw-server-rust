# ActivityWatch PostgreSQL Deployment Guide

**Version**: 1.0  
**Date**: 2026-05-18  
**Status**: Production Ready

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Initial Deployment](#initial-deployment)
3. [Configuration](#configuration)
4. [Update Procedure](#update-procedure)
5. [Backup & Restore](#backup--restore)
6. [Monitoring](#monitoring)
7. [Troubleshooting](#troubleshooting)
8. [Security Recommendations](#security-recommendations)
9. [Performance Tuning](#performance-tuning)

---

## Prerequisites

### System Requirements

**Minimum**:
- CPU: 2 cores
- RAM: 8 GB
- Disk: 100 GB (SSD recommended)
- OS: Linux, macOS, or Windows with WSL2

**Recommended** (for 300 watchers):
- CPU: 4 cores
- RAM: 16 GB
- Disk: 1 TB SSD
- OS: Ubuntu 22.04 LTS or similar

### Software Requirements

1. **Docker** (20.10+)
   ```bash
   docker --version
   # Docker version 20.10.0 or higher
   ```

2. **Docker Compose** (2.0+)
   ```bash
   docker compose version
   # Docker Compose version v2.0.0 or higher
   ```

3. **Git** (for cloning repository)
   ```bash
   git --version
   ```

### Network Requirements

- Port 5432: PostgreSQL (internal only, optional external for backups)
- Port 5600: ActivityWatch API (exposed to clients)
- Port 8080: ActivityWatch Web UI (HTTP interface)
- Internet: For Docker image pulls (initial setup only)

### Pre-Deployment Checklist

- [ ] Docker and Docker Compose installed
- [ ] Sufficient disk space (100 GB minimum)
- [ ] Firewall configured (ports 5600, optionally 5432)
- [ ] Backup strategy defined
- [ ] Database password generated (strong password)

---

## Initial Deployment

### Step 1: Clone Repository

```bash
git clone https://github.com/ActivityWatch/aw-server-rust.git
cd aw-server-rust
```

### Step 2: Create Secrets

```bash
# Create secrets directory
mkdir -p secrets
chmod 700 secrets

# Generate strong password
openssl rand -base64 32 > secrets/db_password.txt
chmod 600 secrets/db_password.txt

# Verify secret
cat secrets/db_password.txt
```

**Important**: Store this password securely. You'll need it for backups and manual database access.

### Step 3: Review Configuration

**docker-compose.yml** - Main configuration:
```yaml
# Key settings to review:
services:
  postgresql:
    ports: ["5432:5432"]  # Expose for backups (optional)
    deploy:
      resources:
        limits:
          cpus: '4'       # Adjust based on your hardware
          memory: 16G     # Adjust based on your hardware
  
  aw-server:
    ports: ["5600:5600"]  # API port
    deploy:
      resources:
        limits:
          cpus: '4'       # Adjust based on your hardware
          memory: 4G      # Adjust based on your hardware
  
  aw-webui:
    ports: ["8080:80"]     # Web UI port
    deploy:
      resources:
        limits:
          cpus: '1'       # Adjust based on your hardware
          memory: 512M    # Adjust based on your hardware
```

**docker/postgresql.conf** - Database tuning:
```ini
# Adjust these based on your system RAM:
shared_buffers = 4GB              # 25% of RAM (16GB system)
effective_cache_size = 12GB       # 75% of RAM (16GB system)
max_connections = 50              # Adjust for your workload
```

### Step 4: Build and Deploy

```bash
# Build Docker image
docker compose build

# Start services
docker compose up -d

# Check status
docker compose ps
```

Expected output:
```
NAME                       STATUS              PORTS
activitywatch-postgresql   Up (healthy)        0.0.0.0:5432->5432/tcp
activitywatch-server       Up (healthy)        0.0.0.0:5600->5600/tcp
activitywatch-webui        Up (healthy)        0.0.0.0:8080->80/tcp
```

### Step 5: Verify Deployment

**Check Web UI**:
```bash
# Open in browser
open http://localhost:8080

# Or check health endpoint
curl http://localhost:8080/health
```

**Check server health**:
```bash
curl http://localhost:5600/api/0/info | jq .
```

Expected response:
```json
{
  "hostname": "container-id",
  "version": "v0.14.0 (rust)",
  "testing": false,
  "device_id": "uuid"
}
```

**Check database**:
```bash
docker compose exec postgresql psql -U aw_user -d activitywatch -c "\dt"
```

Expected tables:
- buckets
- events
- key_value
- schema_version

**Test API operations**:
```bash
# Create bucket
curl -X POST http://localhost:5600/api/0/buckets/test-bucket \
  -H "Content-Type: application/json" \
  -d '{"type":"test","client":"test","hostname":"localhost"}'

# List buckets
curl http://localhost:5600/api/0/buckets/ | jq .

# Delete test bucket
curl -X DELETE http://localhost:5600/api/0/buckets/test-bucket
```

---

## Configuration

### Environment Variables

All configuration via environment variables in `docker-compose.yml`:

**PostgreSQL Configuration**:
```yaml
environment:
  POSTGRES_USER: aw_user                    # Database user
  POSTGRES_PASSWORD_FILE: /run/secrets/db_password  # Password from secret
  POSTGRES_DB: activitywatch                # Database name
  POSTGRES_SHARED_BUFFERS: 4GB              # Memory allocation
  POSTGRES_EFFECTIVE_CACHE_SIZE: 12GB       # Query optimizer hint
  POSTGRES_MAX_CONNECTIONS: 50              # Connection limit
```

**aw-server Configuration**:
```yaml
environment:
  DB_HOST: postgresql                       # Database hostname
  DB_PORT: 5432                             # Database port
  DB_USER: aw_user                          # Database user
  DB_PASSWORD_FILE: /run/secrets/db_password  # Password from secret
  DB_NAME: activitywatch                    # Database name
  RUST_LOG: info                            # Log level (debug, info, warn, error)
```

### Changing Configuration

1. Edit `docker-compose.yml`
2. Restart services:
   ```bash
   docker compose down
   docker compose up -d
   ```

### Custom PostgreSQL Configuration

Edit `docker/postgresql.conf` for advanced tuning:
```ini
# Memory settings
shared_buffers = 4GB
effective_cache_size = 12GB
work_mem = 32MB
maintenance_work_mem = 1GB

# Connection settings
max_connections = 50
superuser_reserved_connections = 3

# Performance settings
random_page_cost = 1.1              # For SSD
effective_io_concurrency = 200      # For SSD
```

Restart PostgreSQL after changes:
```bash
docker compose restart postgresql
```

---

## Update Procedure

### Minor Updates (Code Changes Only)

```bash
# 1. Pull latest code
git pull origin master

# 2. Rebuild image
docker compose build aw-server

# 3. Restart server (PostgreSQL stays running)
docker compose up -d aw-server
```

### Major Updates (Schema Changes)

```bash
# 1. Backup database first!
./scripts/backup-database.sh

# 2. Pull latest code
git pull origin master

# 3. Rebuild and restart all services
docker compose down
docker compose build
docker compose up -d

# 4. Check migration logs
docker compose logs aw-server | grep -i migration
```

### Rollback Procedure

If deployment fails:

```bash
# 1. Stop new version
docker compose down

# 2. Checkout previous version
git checkout <previous-commit>

# 3. Restore database if needed
DB_PASSWORD=$(cat secrets/db_password.txt) \
  ./scripts/restore-database.sh backups/activitywatch_backup_<timestamp>.sql.gz

# 4. Deploy previous version
docker compose up -d
```

---

## Backup & Restore

### Automated Backups

**Set up cron job** (recommended):
```bash
# Edit crontab
crontab -e

# Add daily backup at 2 AM
0 2 * * * cd /path/to/aw-server-rust && DB_PASSWORD=$(cat secrets/db_password.txt) ./scripts/backup-database.sh /path/to/backups >> /var/log/aw-backup.log 2>&1
```

**Manual backup**:
```bash
DB_PASSWORD=$(cat secrets/db_password.txt) \
  ./scripts/backup-database.sh ./backups
```

Backup features:
- ✅ Compressed with gzip
- ✅ Timestamp-based naming
- ✅ Integrity verification
- ✅ 30-day retention policy
- ✅ Automatic cleanup

### Restore from Backup

**Full restore procedure**:
```bash
# List available backups
ls -lh backups/activitywatch_backup_*.sql.gz

# Restore specific backup
DB_PASSWORD=$(cat secrets/db_password.txt) \
  ./scripts/restore-database.sh backups/activitywatch_backup_20260518_120000.sql.gz
```

**What happens during restore**:
1. Stops aw-server (if running)
2. Drops existing database
3. Creates new database
4. Restores from backup
5. Starts aw-server (if was running)

**Safety features**:
- ⚠️ Requires explicit "YES" confirmation
- ✅ Backup integrity verification
- ✅ Connection termination handling
- ✅ Single transaction (all-or-nothing)

### Backup Best Practices

1. **Schedule** daily backups during low-traffic hours
2. **Store** backups on separate disk/server
3. **Test** restore procedure monthly
4. **Monitor** backup success/failure
5. **Retain** at least 30 days of backups
6. **Encrypt** backups if stored off-site

---

## Monitoring

### Health Checks

**Docker Compose health**:
```bash
# Check service status
docker compose ps

# Expected output:
#   STATUS: Up (healthy)
```

**API health endpoint**:
```bash
curl http://localhost:5600/api/0/info
```

**Database health**:
```bash
docker compose exec postgresql pg_isready -U aw_user -d activitywatch
```

### Logs

**View real-time logs**:
```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f aw-server
docker compose logs -f postgresql
```

**Check for errors**:
```bash
# Server errors
docker compose logs aw-server | grep -i error | tail -20

# Database errors
docker compose logs postgresql | grep -i error | tail -20
```

### Resource Usage

**Container stats**:
```bash
docker stats
```

**Database size**:
```bash
docker compose exec postgresql psql -U aw_user -d activitywatch -c "
  SELECT 
    pg_size_pretty(pg_database_size('activitywatch')) as db_size,
    (SELECT COUNT(*) FROM buckets) as buckets,
    (SELECT COUNT(*) FROM events) as events;"
```

**Connection pool status**:
```bash
docker compose exec postgresql psql -U aw_user -d activitywatch -c "
  SELECT COUNT(*) as active_connections 
  FROM pg_stat_activity 
  WHERE datname = 'activitywatch';"
```

### Metrics Collection

**Export Prometheus metrics** (future enhancement):
```bash
curl http://localhost:5600/metrics
```

**Database statistics**:
```bash
docker compose exec postgresql psql -U aw_user -d activitywatch -c "
  SELECT schemaname, tablename, 
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
  FROM pg_tables 
  WHERE schemaname = 'public' 
  ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;"
```

### Alerting

Set up monitoring alerts for:
- 🚨 Container down (health check failed)
- 🚨 Disk space < 10 GB
- 🚨 Database size growing too fast
- 🚨 Connection pool exhausted
- 🚨 Backup failures

---

## Troubleshooting

### Common Issues

#### 1. Port Already in Use

**Error**:
```
Error: bind: address already in use
```

**Solution**:
```bash
# Check what's using the port
sudo lsof -i :5600
sudo lsof -i :5432

# Kill the process or change port in docker-compose.yml
```

#### 2. Database Connection Refused

**Error**:
```
ERROR: connection refused
```

**Checks**:
```bash
# 1. Is PostgreSQL running?
docker compose ps postgresql

# 2. Is PostgreSQL healthy?
docker compose exec postgresql pg_isready

# 3. Check PostgreSQL logs
docker compose logs postgresql | tail -50

# 4. Verify listen_addresses
docker compose exec postgresql grep listen_addresses /etc/postgresql/postgresql.conf
```

**Solution**:
```bash
# Restart PostgreSQL
docker compose restart postgresql
```

#### 3. Migration Failures

**Error**:
```
ERROR: Failed to run migrations
```

**Solution**:
```bash
# 1. Check migration logs
docker compose logs aw-server | grep -i migration

# 2. Check schema version
docker compose exec postgresql psql -U aw_user -d activitywatch -c "
  SELECT * FROM schema_version ORDER BY version DESC LIMIT 1;"

# 3. If corrupted, restore from backup
DB_PASSWORD=$(cat secrets/db_password.txt) \
  ./scripts/restore-database.sh backups/latest_backup.sql.gz
```

#### 4. Out of Disk Space

**Error**:
```
ERROR: No space left on device
```

**Solution**:
```bash
# 1. Check disk usage
df -h

# 2. Clean Docker volumes
docker system prune -a --volumes

# 3. Remove old backups
find backups/ -name "*.sql.gz" -mtime +30 -delete

# 4. Archive old events (manual SQL)
docker compose exec postgresql psql -U aw_user -d activitywatch -c "
  DELETE FROM events WHERE starttime < NOW() - INTERVAL '1 year';"
```

#### 5. Slow Query Performance

**Solution**:
```bash
# 1. Check slow queries
docker compose exec postgresql psql -U aw_user -d activitywatch -c "
  SELECT query, calls, mean_exec_time 
  FROM pg_stat_statements 
  ORDER BY mean_exec_time DESC 
  LIMIT 10;"

# 2. Analyze tables
docker compose exec postgresql psql -U aw_user -d activitywatch -c "
  VACUUM ANALYZE;"

# 3. Reindex if needed
docker compose exec postgresql psql -U aw_user -d activitywatch -c "
  REINDEX DATABASE activitywatch;"
```

#### 6. Container Won't Start

**Solution**:
```bash
# 1. Check logs
docker compose logs aw-server

# 2. Check dependencies
docker compose ps postgresql

# 3. Rebuild image
docker compose build --no-cache aw-server

# 4. Reset containers
docker compose down -v
docker compose up -d
```

### Debug Mode

Enable verbose logging:
```yaml
# docker-compose.yml
services:
  aw-server:
    environment:
      RUST_LOG: debug  # Change from 'info' to 'debug'
```

Restart and check logs:
```bash
docker compose restart aw-server
docker compose logs -f aw-server
```

### Getting Help

1. **Check logs**: Always start with `docker compose logs`
2. **Review documentation**: See [datastore-implementation.md](./datastore-implementation.md)
3. **GitHub Issues**: https://github.com/ActivityWatch/aw-server-rust/issues
4. **Community Forum**: https://forum.activitywatch.net/

---

## Security Recommendations

### 1. Strong Passwords

Generate secure password:
```bash
openssl rand -base64 32 > secrets/db_password.txt
chmod 600 secrets/db_password.txt
```

**Never**:
- Commit secrets to Git
- Use default passwords
- Share passwords in plaintext

### 2. Network Security

**Firewall rules** (example with ufw):
```bash
# Allow API port
sudo ufw allow 5600/tcp

# Deny external PostgreSQL access (local only)
sudo ufw deny 5432/tcp
```

**Docker network isolation**:
- PostgreSQL only accessible to aw-server
- No external PostgreSQL exposure (unless needed for backups)

### 3. File Permissions

```bash
# Secure secrets
chmod 700 secrets/
chmod 600 secrets/db_password.txt

# Secure backups
chmod 700 backups/
chmod 600 backups/*.sql.gz

# Secure scripts
chmod 755 scripts/*.sh
```

### 4. Regular Updates

```bash
# Update Docker images monthly
docker compose pull
docker compose up -d

# Update PostgreSQL config as needed
# Review security advisories
```

### 5. Audit Logging

Enable PostgreSQL audit logging:
```ini
# docker/postgresql.conf
log_statement = 'all'
log_connections = on
log_disconnections = on
```

### 6. SSL/TLS (Optional)

For external API access, use reverse proxy with TLS:
```bash
# Example with nginx
server {
    listen 443 ssl;
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    location / {
        proxy_pass http://localhost:5600;
    }
}
```

---

## Performance Tuning

### PostgreSQL Tuning

**For 16 GB RAM system** (recommended for 300 watchers):
```ini
# docker/postgresql.conf

# Memory
shared_buffers = 4GB                  # 25% of RAM
effective_cache_size = 12GB           # 75% of RAM
work_mem = 32MB                       # Per-operation memory
maintenance_work_mem = 1GB            # For VACUUM, CREATE INDEX

# Connections
max_connections = 50                  # Connection limit
superuser_reserved_connections = 3    # Reserved for admin

# Query Planning
random_page_cost = 1.1                # For SSD (default: 4.0)
effective_io_concurrency = 200        # For SSD (default: 1)

# Write Performance
wal_buffers = 16MB
checkpoint_completion_target = 0.9
max_wal_size = 2GB

# Logging
log_min_duration_statement = 1000     # Log queries > 1s
```

### Connection Pool Tuning

**Current settings** (in aw-datastore/src/worker.rs):
```rust
max_size: 20,        // Max connections
min_idle: 5,         // Keep-alive connections
```

**For higher load**, increase max_size:
- 300 watchers: 20 connections (current)
- 600 watchers: 30 connections
- 1000 watchers: 50 connections

### Index Optimization

**Check index usage**:
```sql
SELECT 
    schemaname, tablename, indexname, idx_scan as scans
FROM pg_stat_user_indexes 
ORDER BY idx_scan;
```

**Create additional indexes** if needed:
```sql
-- Example: Index for specific query patterns
CREATE INDEX idx_events_bucket_time 
ON events (bucketrow, starttime DESC);
```

### Query Optimization

**Enable pg_stat_statements**:
```sql
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;
```

**Find slow queries**:
```sql
SELECT 
    query, 
    calls, 
    mean_exec_time, 
    max_exec_time
FROM pg_stat_statements 
ORDER BY mean_exec_time DESC 
LIMIT 10;
```

### Scaling Considerations

**Vertical Scaling** (recommended first):
- Increase CPU cores: 4 → 8
- Increase RAM: 16 GB → 32 GB
- Use faster SSD: SATA → NVMe

**Horizontal Scaling** (future):
- Read replicas for queries
- Partitioning events table by time
- Caching layer (Redis)

---

## Additional Resources

### Related Documentation

- [Datastore Implementation](./datastore-implementation.md)
- [Deployment Architecture](../infrastructure-design/deployment-architecture.md)
- [NFR Design](../nfr-design/nfr-design-patterns.md)

### External Links

- PostgreSQL Documentation: https://www.postgresql.org/docs/15/
- Docker Compose: https://docs.docker.com/compose/
- ActivityWatch: https://activitywatch.net/

### Support

- GitHub Issues: https://github.com/ActivityWatch/aw-server-rust/issues
- Forum: https://forum.activitywatch.net/
- Documentation: https://docs.activitywatch.net/

---

**Last Updated**: 2026-05-18  
**Version**: 1.0  
**Status**: Production Ready
