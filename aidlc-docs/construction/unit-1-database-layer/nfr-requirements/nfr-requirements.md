# NFR Requirements - Unit 1: Database Layer

**Version**: 1.0  
**Date**: 2026-05-18  
**Unit**: Database Layer Migration  

---

## Executive Summary

This document defines non-functional requirements (NFRs) for the PostgreSQL database layer migration, covering performance, scalability, availability, security, and operational considerations.

---

## 1. Performance Requirements

### 1.1 Query Response Times

| Operation | Target Latency (p50) | Target Latency (p95) | Target Latency (p99) |
|-----------|---------------------|---------------------|---------------------|
| Bucket lookup by name | < 10ms | < 25ms | < 50ms |
| Single event insert | < 15ms | < 30ms | < 60ms |
| Batch event insert (100 events) | < 100ms | < 200ms | < 400ms |
| Time-range query (1 day, ~1000 events) | < 50ms | < 150ms | < 300ms |
| Time-range query (1 week, ~10k events) | < 200ms | < 500ms | < 1000ms |
| Time-range query (1 month, ~50k events) | < 1000ms | < 2000ms | < 4000ms |

**Rationale**:
- ActivityWatch watchers typically query recent events (1 day - 1 week)
- UI dashboards expect sub-second response times
- Long-range queries (months) are acceptable at 1-4 seconds

### 1.2 Throughput Requirements

| Metric | Target | Peak Capacity |
|--------|--------|--------------|
| Events per second (sustained) | 50 eps | 300 eps |
| Concurrent API requests | 30 concurrent | 100 concurrent |
| Database connections | 20 active | 40 max |

**Rationale**:
- Expected deployment: 300 watchers × 1 event per 10 seconds = ~30 eps sustained
- 50 eps target provides 1.7x headroom for burst activity
- Peak capacity (300 eps) handles all watchers firing simultaneously or catch-up after disconnect
- Higher concurrency supports multiple API clients querying dashboards simultaneously

### 1.3 Connection Pool Performance

**Target**:
- Connection acquisition time: < 5ms (p95)
- Connection pool utilization: < 70% under normal load
- Connection timeout: 5 seconds (fail fast if pool exhausted)

**Configuration**:
```toml
[database.pool]
max_connections = 20
min_idle_connections = 5
connection_timeout_ms = 5000
idle_timeout_sec = 600
max_lifetime_sec = 3600
```

---

## 2. Scalability Requirements

### 2.1 Data Volume Scalability

| Metric | Initial | 1 Year | 5 Years | Design Capacity |
|--------|---------|--------|---------|-----------------|
| Events | 0 | 1.1B | 5.5B | 9.2 quintillion (BIGSERIAL) |
| Buckets | 300 | 500 | 1,000 | 10,000+ |
| Database size | 0 GB | 150 GB | 750 GB | 10 TB+ |

**Rationale**:
- Expected load: 3M events per day (300 watchers × 10k events/day)
- 1 year: 3M × 365 = ~1.1B events
- 5 years: ~5.5B events
- BIGSERIAL (64-bit) supports 9.2 quintillion events - ample headroom

### 2.2 Horizontal Scalability

**Current Scope**: Single PostgreSQL instance (vertical scaling)

**Future Considerations** (out of scope for initial migration):
- Read replicas for query offloading
- Connection pooling via PgBouncer
- Partitioning events table by time (monthly partitions)

**Vertical Scaling Targets**:
- 8-16 CPU cores recommended for 300-watcher deployment
- 16-32 GB RAM recommended (PostgreSQL shared_buffers + connection overhead + caching)
- NVMe SSD storage required (high IOPS for concurrent writes)
- 1-2 TB storage capacity for 5-year retention

### 2.3 Index Scalability

**Index Maintenance**:
- VACUUM ANALYZE scheduled nightly (cleanup + stats update)
- REINDEX monthly for heavily-written tables (events)
- Monitor index bloat (pg_stat_user_indexes)

**Index Size Estimates**:
- Primary key indexes: ~10% of table size
- Time-range composite index: ~15% of table size
- GIN indexes (JSONB): ~30-50% of data size (if used)

---

## 3. Availability Requirements

### 3.1 Uptime Targets

**Target**: 99.5% uptime (43.8 hours downtime per year)

**Rationale**:
- ActivityWatch is non-critical (not life-safety or financial)
- Watchers cache events locally during downtime
- Downtime primarily affects queries/dashboard views

**Acceptable Downtime Windows**:
- Planned maintenance: Monthly, 2-hour windows
- Emergency maintenance: As needed (< 4 hours total per year)

### 3.2 Recovery Time Objectives

| Scenario | RTO (Recovery Time) | RPO (Recovery Point) |
|----------|---------------------|---------------------|
| Database crash (automatic recovery) | < 5 minutes | 0 (no data loss) |
| Corruption recovery (restore from backup) | < 4 hours | < 24 hours |
| Disaster recovery (full rebuild) | < 12 hours | < 7 days |

**Rationale**:
- PostgreSQL WAL provides automatic crash recovery (no data loss)
- Daily backups acceptable for disaster recovery
- Activity tracking data is valuable but not critical (some loss acceptable)

### 3.3 Backup Strategy

**Frequency**:
- Full backup: Daily at 02:00 UTC (low-traffic period)
- Transaction log archiving: Continuous (if WAL archiving enabled)
- Backup retention: 30 days rolling window

**Backup Method**:
- PostgreSQL `pg_dump` for full logical backups
- Docker volume snapshots (if using Docker volumes)
- Offsite backup storage recommended (AWS S3, cloud storage)

**Restoration Testing**:
- Test restore monthly (verify backup integrity)
- Document restoration procedures

---

## 4. Security Requirements

### 4.1 Authentication & Authorization

**Database Access**:
- Username/password authentication (PostgreSQL native)
- No anonymous access
- Principle of least privilege (app user has only required permissions)

**Permissions Required**:
```sql
GRANT CONNECT ON DATABASE activitywatch TO aw_user;
GRANT SELECT, INSERT, DELETE ON ALL TABLES IN SCHEMA public TO aw_user;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO aw_user;
-- No UPDATE on events table (immutable)
-- No DROP, ALTER permissions (admin only)
```

### 4.2 Network Security

**Firewall Rules**:
- PostgreSQL port (5432) accessible only from:
  - `aw-server` container (Docker internal network)
  - Localhost (for admin access)
- No public internet access to PostgreSQL

**Docker Network Isolation**:
```yaml
networks:
  internal:
    driver: bridge
    internal: true  # No external access
```

### 4.3 Data Protection

**Encryption at Rest**: Optional (not required for initial deployment)
- Activity data is not highly sensitive (application usage logs)
- Consider enabling if storing personally identifiable information (PII)
- PostgreSQL supports transparent data encryption (TDE) via extensions

**Encryption in Transit**: 
- Within Docker network: Not required (trusted internal network)
- External connections: SSL/TLS recommended if exposing PostgreSQL externally

**Data Sanitization**:
- Privacy filter available (existing aw-datastore feature)
- Can redact sensitive window titles, URLs

### 4.4 SQL Injection Prevention

**Mitigation**:
- **Parameterized queries only** (no string concatenation)
- Use tokio-postgres/sqlx with parameter binding
- Never construct SQL with user input

**Example (Safe)**:
```rust
conn.execute(
    "SELECT * FROM events WHERE bucketrow = $1 AND starttime >= $2",
    &[&bucket_id, &start_time]
).await?;
```

**Example (Unsafe - NEVER DO THIS)**:
```rust
// NEVER DO THIS - SQL injection vulnerability
let query = format!("SELECT * FROM events WHERE bucketrow = {}", bucket_id);
conn.execute(&query, &[]).await?;
```

---

## 5. Reliability Requirements

### 5.1 Error Handling

**Database Connection Failures**:
- Retry with exponential backoff (100ms, 200ms, 400ms, 800ms, 1600ms)
- Maximum 5 retries before failing request
- Return 503 Service Unavailable to API layer

**Query Failures**:
- Transient errors: Retry (connection timeout, deadlock)
- Permanent errors: Return immediately (constraint violation, syntax error)
- Log all errors with context (query, parameters, error code)

**Transaction Failures**:
- Automatic rollback on error
- Propagate error to API layer (return 400/500 with details)

### 5.2 Health Monitoring

**Health Check Endpoint**: `/api/0/info` (existing)
- Check database connectivity (simple SELECT 1 query)
- Return 200 OK if healthy, 503 if database unavailable

**Docker Health Check**:
```yaml
healthcheck:
  test: ["CMD", "pg_isready", "-U", "aw_user", "-d", "activitywatch"]
  interval: 10s
  timeout: 5s
  retries: 3
  start_period: 30s
```

### 5.3 Monitoring Metrics

**Critical Metrics** (log or expose via metrics endpoint):
- Database connection pool utilization (%)
- Query execution time (histogram: p50, p95, p99)
- Error rate (errors per minute)
- Transaction rollback rate
- Active connections count

**Performance Metrics**:
- Events inserted per second
- Queries per second
- Cache hit ratio (PostgreSQL buffer cache)
- Index usage statistics

**Alert Thresholds**:
- Connection pool > 80% utilized: Warning
- Connection pool > 95% utilized: Critical
- Query latency p95 > 1 second: Warning
- Error rate > 10 per minute: Critical
- Database unavailable: Critical (immediate alert)

---

## 6. Maintainability Requirements

### 6.1 Code Quality

**Rust Best Practices**:
- Use `async`/`await` for database operations (tokio-postgres or sqlx)
- Error handling with `Result<T, E>` (no unwrap/panic in production code)
- Type safety: Strong typing for database queries

**Database Library Choice**:
- **Recommended**: `tokio-postgres` with `deadpool-postgres` (connection pooling)
- **Alternative**: `sqlx` (compile-time query validation)

**Rationale**:
- tokio-postgres: Mature, well-tested, excellent async support
- deadpool: Battle-tested connection pooling
- sqlx: Optional upgrade for compile-time safety

### 6.2 Schema Migrations

**Migration Strategy**:
- Version-tracked migrations (schema_version table)
- Forward-only migrations (no rollbacks)
- Idempotent migrations (safe to re-run)

**Migration Execution**:
- Automatic on startup (check version, apply pending migrations)
- Manual override flag: `--skip-migrations` (for debugging)

**Migration Safety**:
- Wrap migrations in transactions (atomic)
- Test migrations on copy of production data
- Document migration procedures

### 6.3 Logging

**Log Levels**:
- ERROR: Database connection failures, query errors, transaction failures
- WARN: Slow queries (> 100ms), high connection pool utilization
- INFO: Connection pool stats, migration execution
- DEBUG: All SQL queries with parameters (development only)

**Structured Logging**:
```rust
error!(
    query = %query,
    params = ?params,
    error = %err,
    "Database query failed"
);
```

### 6.4 Documentation

**Required Documentation**:
- Schema diagram (ERD showing tables, relationships)
- Migration history (what each version changed)
- Operational runbook (backup, restore, troubleshooting)
- Performance tuning guide (indexes, VACUUM, configuration)

---

## 7. Operational Requirements

### 7.1 Database Configuration

**PostgreSQL Configuration** (postgresql.conf):
```ini
# Memory
shared_buffers = 4GB             # 25% of RAM (for 16GB RAM system)
effective_cache_size = 12GB      # 75% of RAM
work_mem = 32MB                  # Per-query memory

# Connections
max_connections = 50             # Total connections allowed (app pool + admin)

# Performance
random_page_cost = 1.1           # SSD storage (lower = better for SSD)
effective_io_concurrency = 200   # SSD concurrent I/O

# WAL
wal_level = replica              # Enable replication (future)
max_wal_size = 1GB
min_wal_size = 80MB

# Autovacuum
autovacuum = on
autovacuum_vacuum_scale_factor = 0.1
autovacuum_analyze_scale_factor = 0.05
```

### 7.2 Capacity Planning

**Growth Projections**:
- Database growth: ~150 GB per year (300-watcher deployment)
- Plan for 1-2 TB storage minimum (5-10 years capacity)
- Monitor disk usage weekly (alert at 80% full)
- Consider partitioning events table by month after 1 year (improve query performance)

**Resource Monitoring**:
- CPU: Alert if sustained > 80%
- Memory: Alert if PostgreSQL memory > 80% of allocated
- Disk I/O: Monitor IOPS and latency

### 7.3 Disaster Recovery Procedures

**Recovery Scenarios**:

1. **Database Corruption** (partial):
   - Identify corrupted tables (pg_dump errors)
   - Restore from most recent backup
   - Replay WAL logs if available (minimize data loss)

2. **Complete Data Loss**:
   - Restore from daily backup (up to 24 hours data loss)
   - Notify users of data loss window
   - Watchers will re-submit recent events (if still in local cache)

3. **Docker Volume Loss**:
   - Recreate volume from backup
   - Restart PostgreSQL container
   - Verify data integrity (row counts, spot checks)

---

## 8. Testing Requirements

### 8.1 Unit Testing

**Coverage Target**: > 80% code coverage for database layer

**Test Scenarios**:
- Connection pool acquisition/release
- Query execution (SELECT, INSERT, DELETE)
- Transaction commit/rollback
- Error handling (connection failures, constraint violations)
- Timestamp conversion (SQLite INTEGER ↔ PostgreSQL TIMESTAMP)

**Test Infrastructure**:
- Use `testcontainers-rs` for PostgreSQL test containers
- Spin up PostgreSQL instance per test suite
- Clean database state between tests

### 8.2 Integration Testing

**Test Scenarios**:
- Full API workflow (create bucket → insert events → query events)
- Concurrent event insertions (multiple threads)
- Large batch insertions (1000+ events)
- Time-range queries (various date ranges)
- Cascade deletes (delete bucket → verify events deleted)

**Performance Benchmarks**:
- Measure query latency under load (JMeter or similar)
- Verify performance targets met (see section 1.1)

### 8.3 Load Testing

**Test Scenarios**:
- Sustained load: 100 events/second for 10 minutes
- Burst load: 500 events/second for 1 minute
- Concurrent queries: 50 simultaneous time-range queries

**Success Criteria**:
- No errors or timeouts
- Latency targets met (p95, p99)
- Database remains stable (no connection leaks)

---

## 9. Compliance Requirements

**Data Retention**: No specific requirements (user-configurable)

**GDPR Considerations** (if applicable):
- Users can delete their data (DELETE bucket API)
- Privacy filter can redact sensitive information
- Export functionality available (JSON export)

**Audit Logging**: Optional (not required for initial deployment)
- Log all database operations (who, what, when)
- Retention: 90 days

---

## 10. NFR Priority Matrix

| NFR Category | Priority | Impact | Effort |
|--------------|----------|--------|--------|
| Performance (query latency) | High | High | Medium |
| Scalability (data volume) | High | High | Low (BIGSERIAL) |
| Reliability (error handling) | High | High | Medium |
| Security (SQL injection prevention) | High | High | Low (parameterized queries) |
| Availability (uptime) | Medium | Medium | Low (PostgreSQL default) |
| Monitoring (metrics) | Medium | Medium | Medium |
| Backup/Recovery | Medium | High | Low (pg_dump) |
| Documentation | Low | Medium | Low |

**Focus Areas for Initial Deployment**:
1. Performance: Query optimization, indexing
2. Reliability: Error handling, retry logic
3. Security: Parameterized queries, network isolation
4. Scalability: BIGSERIAL, connection pooling

---

## 11. Success Criteria

NFR Requirements are met when:

✅ All performance targets documented (latency, throughput)  
✅ Scalability projections defined (data volume, growth)  
✅ Availability targets specified (uptime, RTO, RPO)  
✅ Security requirements defined (authentication, SQL injection prevention)  
✅ Monitoring metrics identified (what to track)  
✅ Testing strategy documented (unit, integration, load)  
✅ Operational procedures outlined (backup, recovery, configuration)  
✅ Tech stack decisions made (see tech-stack-decisions.md)
