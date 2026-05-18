# Code Generation Plan - Unit 1: Database Layer Migration

**Version**: 1.0  
**Date**: 2026-05-18  
**Unit**: Database Layer Migration (aw-datastore PostgreSQL refactoring)  
**Type**: Brownfield (Modify existing + Create new)

---

## Unit Context

**Stories Implemented**: Database Layer Migration
- Replace SQLite with PostgreSQL 15
- Implement connection pooling (deadpool-postgres)
- Add retry logic with exponential backoff
- Add metrics collection (Prometheus-style)
- Add health checks
- Add schema migrations manager
- Maintain 100% API compatibility with existing aw-datastore interface

**Dependencies**:
- None (Unit 1 is foundational, other units depend on it)

**Expected Interfaces**:
- Public API: Maintain existing `Datastore` struct methods (get_bucket, get_buckets, create_bucket, delete_bucket, insert_events, get_events, etc.)
- Internal: New `DatabaseWorker` with connection pool, `RetryHandler`, `HealthChecker`, `MigrationManager`

**Database Entities**:
- buckets (id SERIAL PRIMARY KEY, client_id TEXT, type TEXT, hostname TEXT, created TIMESTAMP)
- events (id BIGSERIAL PRIMARY KEY, bucketrow INTEGER REFERENCES buckets ON DELETE CASCADE, starttime TIMESTAMP WITH TIME ZONE, endtime TIMESTAMP WITH TIME ZONE, data JSONB)
- key_value (key TEXT PRIMARY KEY, value TEXT)
- schema_version (version INTEGER PRIMARY KEY, applied TIMESTAMP)

**Service Boundaries**:
- aw-datastore crate: Database abstraction layer only
- aw-server crate: Configuration management (DbConfig), CLI args, server startup

---

## Code Generation Steps

### Step 1: Project Dependencies Update (aw-datastore/Cargo.toml)
**Action**: Modify existing file  
**File**: `aw-datastore/Cargo.toml`  
**Changes**:
- Remove: `rusqlite = "0.30"`
- Add: `tokio-postgres = { version = "0.7", features = ["with-chrono-0_4", "with-serde_json-1"] }`
- Add: `deadpool-postgres = "0.10"`
- Add: `thiserror = "1.0"`
- Add (dev-dependencies): `testcontainers = "0.15"`
- Keep existing: `chrono`, `serde`, `serde_json`, `log`

**Rationale**: Switch from synchronous rusqlite to asynchronous tokio-postgres with connection pooling

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 2: Database Worker Refactoring (aw-datastore/src/worker.rs)
**Action**: Modify existing file  
**File**: `aw-datastore/src/worker.rs`  
**Current Implementation**: MPSC channel + single SQLite connection in worker thread  
**New Implementation**: Connection pool manager with direct async access  
**Changes**:
- Remove: MPSC channel structs (`DatastoreWorker`, `Request`, `Response` enums)
- Remove: Worker thread loop
- Add: `DatabaseWorker` struct with `Arc<Pool<PostgresConnectionManager>>`
- Add: `get_connection()` method returning `PooledConnection`
- Add: `create_pool()` function (initializes pool with config)
- Add: Retry handler integration for transient errors
- Keep: Public API wrapper methods (but delegate to pool instead of MPSC)

**Rationale**: PostgreSQL is thread-safe, no need for MPSC pattern. Direct connection pool access with async/await.

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 3: Schema Adaptation (aw-datastore/src/datastore.rs)
**Action**: Modify existing file  
**File**: `aw-datastore/src/datastore.rs`  
**Current Implementation**: SQLite schema with INTEGER timestamps, TEXT JSON, INTEGER AUTOINCREMENT  
**New Implementation**: PostgreSQL schema with TIMESTAMP WITH TIME ZONE, JSONB, BIGSERIAL  
**Changes**:
- Update `create_tables()`: Convert SQLite DDL to PostgreSQL DDL
  - `INTEGER AUTOINCREMENT` → `SERIAL` (buckets.id), `BIGSERIAL` (events.id)
  - `TEXT` JSON → `JSONB`
  - `INTEGER` timestamps → `TIMESTAMP WITH TIME ZONE`
  - Add `ON DELETE CASCADE` for foreign keys
  - Add composite index: `CREATE INDEX idx_events_timerange ON events(bucketrow, starttime, endtime)`
- Update migration functions (_migrate_v0_to_v1 through _migrate_v3_to_v4):
  - Convert SQLite syntax to PostgreSQL syntax
  - Use PostgreSQL-specific features (e.g., `ALTER TABLE ... ADD CONSTRAINT`)
- Update query methods (get_events, insert_events, etc.):
  - Replace `?` placeholders with `$1, $2, $3...` (PostgreSQL numbered parameters)
  - Convert chrono DateTime<Utc> to/from PostgreSQL TIMESTAMP WITH TIME ZONE
  - Use `RETURNING` clause for insert operations (PostgreSQL-specific)
- Keep: Public method signatures unchanged (API compatibility)

**Rationale**: Adapt schema and queries for PostgreSQL while maintaining API compatibility

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 4: Retry Handler Component (NEW FILE)
**Action**: Create new file  
**File**: `aw-datastore/src/retry.rs`  
**Implementation**:
```rust
// Exponential backoff retry handler for transient database errors
pub struct RetryPolicy {
    pub max_attempts: u32,           // Default: 5
    pub initial_delay_ms: u64,       // Default: 100
    pub max_delay_ms: u64,           // Default: 5000
}

impl RetryPolicy {
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: Fn() -> Future<Output = Result<T, E>>,
        E: Debug,
    {
        // Implement exponential backoff with jitter
        // Classify errors as transient/permanent
        // Retry only transient errors (connection timeout, deadlock, serialization failure)
    }
    
    fn is_transient_error(error: &E) -> bool {
        // Check PostgreSQL error codes: 40001, 40P01, 08006, etc.
    }
}
```

**Rationale**: Implement resilience pattern for transient database errors

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 5: Metrics Collector Component (NEW FILE)
**Action**: Create new file  
**File**: `aw-datastore/src/metrics.rs`  
**Implementation**:
```rust
// Prometheus-style metrics for database operations
pub struct DbMetrics {
    queries_total: Counter,                      // Total queries executed
    query_duration_seconds: Histogram,           // Query latency distribution
    pool_connections_active: Gauge,              // Active connections
    pool_connections_idle: Gauge,                // Idle connections
    errors_total: Counter,                       // Total errors
}

impl DbMetrics {
    pub fn new() -> Self { /* ... */ }
    
    pub fn record_query(&self, operation: &str, duration: Duration) { /* ... */ }
    
    pub fn update_pool_stats(&self, active: usize, idle: usize) { /* ... */ }
    
    pub fn record_error(&self, error_type: &str) { /* ... */ }
}
```

**Rationale**: Implement observability pattern for monitoring database performance

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 6: Health Check Component (NEW FILE)
**Action**: Create new file  
**File**: `aw-datastore/src/health.rs`  
**Implementation**:
```rust
// Health check component with timeout
pub struct HealthChecker {
    pool: Arc<Pool<PostgresConnectionManager>>,
    timeout: Duration,  // Default: 5 seconds
}

impl HealthChecker {
    pub async fn check(&self) -> HealthStatus {
        // Execute health check with timeout
        // Check connectivity: SELECT 1
        // Check schema version: SELECT version FROM schema_version
        // Return Healthy/Unhealthy/Degraded
    }
    
    pub async fn check_database(&self, conn: &Connection) -> Result<(), Error> {
        // Validate connection + schema version
    }
}

pub enum HealthStatus {
    Healthy,
    Unhealthy(String),   // Error message
    Degraded(String),    // Warning message
}
```

**Rationale**: Implement health check pattern for Docker healthchecks and monitoring

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 7: Migration Manager Component (NEW FILE)
**Action**: Create new file  
**File**: `aw-datastore/src/migrations.rs`  
**Implementation**:
```rust
// Schema migration manager
pub struct MigrationManager {
    pool: Arc<Pool<PostgresConnectionManager>>,
}

impl MigrationManager {
    pub async fn run_migrations(&self) -> Result<(), Error> {
        // Check current schema version
        // Apply pending migrations in transactions
        // Update schema_version table
    }
    
    async fn get_current_version(&self) -> Result<i32, Error> {
        // Query schema_version table
    }
    
    async fn migrate_v0_to_v1(&self, conn: &mut Connection) -> Result<(), Error> {
        // Create PostgreSQL schema (initial migration)
        // Create tables: buckets, events, key_value, schema_version
        // Create indexes: idx_events_timerange
        // All in transaction
    }
}
```

**Rationale**: Implement migration pattern for schema versioning and upgrades

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 8: Update Module Exports (aw-datastore/src/lib.rs)
**Action**: Modify existing file  
**File**: `aw-datastore/src/lib.rs`  
**Changes**:
- Add: `pub mod retry;`
- Add: `pub mod metrics;`
- Add: `pub mod health;`
- Add: `pub mod migrations;`
- Keep: Existing exports (`pub mod datastore;`, `pub mod worker;`, etc.)

**Rationale**: Expose new modules to aw-server crate

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 9: Database Configuration (aw-server/src/config.rs)
**Action**: Modify existing file  
**File**: `aw-server/src/config.rs`  
**Current Implementation**: `default_address()` returns "127.0.0.1"  
**New Implementation**: Add DB config + change default address to "0.0.0.0"  
**Changes**:
- Add: `DbConfig` struct with fields:
  ```rust
  pub struct DbConfig {
      pub host: String,        // Default: "localhost"
      pub port: u16,           // Default: 5432
      pub user: String,        // Default: "aw_user"
      pub password: String,    // From env var or file
      pub database: String,    // Default: "activitywatch"
  }
  ```
- Add: `impl DbConfig` with methods:
  - `from_env()` - Load from environment variables (DB_HOST, DB_PORT, DB_USER, DB_PASSWORD, DB_NAME)
  - `from_password_file()` - Read password from file (Docker secrets support)
  - `default()` - Provide sensible defaults
- Modify: `default_address()` to return "0.0.0.0" (was "127.0.0.1")
- Keep: Existing `AWConfig` struct and methods

**Rationale**: Add database configuration management + allow external access

**Stories**: Database Layer Migration, Network Configuration  
**Checkbox**: [x]

---

### Step 10: CLI Arguments Update (aw-server/src/main.rs)
**Action**: Modify existing file  
**File**: `aw-server/src/main.rs`  
**Current Implementation**: Accepts `--host`, `--port`, `--dbpath` CLI flags  
**New Implementation**: Replace `--dbpath` with database connection parameters  
**Changes**:
- Remove: `--dbpath` argument
- Add: `--db-host`, `--db-port`, `--db-user`, `--db-password`, `--db-name` arguments
- Update: Argument parsing to construct `DbConfig`
- Update: DatabaseWorker initialization to use `DbConfig`
- Keep: Existing `--host`, `--port`, `--cors` arguments

**Rationale**: Support PostgreSQL connection parameters instead of SQLite file path

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 11: Unit Tests - Retry Handler (aw-datastore/tests/)
**Action**: Create new file  
**File**: `aw-datastore/tests/test_retry.rs`  
**Tests**:
- `test_retry_transient_error_succeeds()` - Verify retry succeeds after transient errors
- `test_retry_permanent_error_fails_immediately()` - Verify no retry on permanent errors
- `test_retry_max_attempts_exceeded()` - Verify max attempts limit
- `test_exponential_backoff()` - Verify delay increases exponentially
- `test_jitter_variation()` - Verify jitter prevents thundering herd

**Rationale**: Validate retry logic correctness

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 12: Unit Tests - Metrics Collector (aw-datastore/tests/)
**Action**: Create new file  
**File**: `aw-datastore/tests/test_metrics.rs`  
**Tests**:
- `test_query_counter_increments()` - Verify queries_total counter increments
- `test_query_duration_recorded()` - Verify duration histogram records latencies
- `test_pool_stats_updated()` - Verify pool gauges update correctly
- `test_error_counter_increments()` - Verify errors_total counter increments

**Rationale**: Validate metrics collection correctness

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 13: Unit Tests - Health Checker (aw-datastore/tests/)
**Action**: Create new file  
**File**: `aw-datastore/tests/test_health.rs`  
**Tests**:
- `test_health_check_healthy()` - Verify healthy status when database accessible
- `test_health_check_unhealthy()` - Verify unhealthy status when database down
- `test_health_check_timeout()` - Verify timeout handling
- `test_schema_version_validation()` - Verify schema version check

**Rationale**: Validate health check logic correctness

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 14: Integration Tests - Database Worker (aw-datastore/tests/)
**Action**: Modify existing file  
**File**: `aw-datastore/tests/datastore.rs`  
**Current Implementation**: Tests with SQLite in-memory database  
**New Implementation**: Tests with PostgreSQL testcontainers  
**Changes**:
- Add: `testcontainers` setup - start PostgreSQL 15 container
- Update: All test fixtures to use PostgreSQL connection string
- Update: Test assertions to expect PostgreSQL-specific behavior (BIGSERIAL, JSONB, etc.)
- Add: Connection pool tests (concurrent access, pool exhaustion)
- Add: Transaction tests (rollback, commit)
- Keep: Existing test logic (bucket CRUD, event CRUD, query operations)

**Rationale**: Validate database layer works with PostgreSQL (not SQLite)

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 15: Integration Tests - Migration Manager (aw-datastore/tests/)
**Action**: Create new file  
**File**: `aw-datastore/tests/test_migrations.rs`  
**Tests**:
- `test_initial_migration()` - Verify v0→v1 migration creates all tables
- `test_migration_idempotency()` - Verify running migrations twice is safe
- `test_schema_version_tracking()` - Verify schema_version table updated correctly
- `test_migration_transaction_rollback()` - Verify migration failure rolls back

**Rationale**: Validate migration manager correctness

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 16: Documentation - Code Summary (aw-datastore)
**Action**: Create new file  
**File**: `aidlc-docs/construction/unit-1-database-layer/code/datastore-implementation.md`  
**Content**:
- Overview of database layer architecture
- Modified files summary with before/after comparison
- New components description (RetryHandler, MetricsCollector, HealthChecker, MigrationManager)
- Connection pool configuration details
- Migration strategy explanation
- API compatibility notes
- Testing strategy summary

**Rationale**: Document implementation details for future reference

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 17: Docker Infrastructure - Dockerfile
**Action**: Create new file  
**File**: `Dockerfile` (workspace root)  
**Implementation**: Multi-stage build as specified in deployment-architecture.md
- Stage 1: Builder (rust:1.75-bookworm, cargo build --release)
- Stage 2: Runtime (debian:bookworm-slim, copy binary, install libpq5 + ca-certificates)
- USER: awuser (non-root)
- EXPOSE: 5600
- CMD: ["aw-server"]

**Rationale**: Container image for aw-server deployment

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 18: Docker Infrastructure - docker-compose.yml
**Action**: Create new file  
**File**: `docker-compose.yml` (workspace root)  
**Implementation**: As specified in deployment-architecture.md
- Services: postgresql (postgres:15-alpine), aw-server
- Networks: internal (bridge)
- Volumes: pg_data (named volume)
- Secrets: db_password (file-based)
- Healthchecks: pg_isready (10s), /api/0/info (30s)
- Resource limits: PostgreSQL (8 CPU, 24 GB), aw-server (4 CPU, 4 GB)

**Rationale**: Orchestration for production deployment

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 19: Docker Infrastructure - PostgreSQL Configuration
**Action**: Create new file  
**File**: `docker/postgresql.conf` (workspace root)  
**Implementation**: As specified in deployment-architecture.md
- Memory tuning: shared_buffers 4GB, effective_cache_size 12GB
- Connection tuning: max_connections 50
- Performance tuning: random_page_cost 1.1, effective_io_concurrency 200
- WAL configuration: wal_level replica, max_wal_size 2GB
- Autovacuum: enabled with 3 workers
- Logging: slow queries > 1s

**Rationale**: PostgreSQL performance optimization for 300 watchers

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 20: Docker Infrastructure - Database Init Script
**Action**: Create new file  
**File**: `docker/init-db.sh` (workspace root)  
**Implementation**: As specified in deployment-architecture.md
- Create pg_stat_statements extension
- Log initialization status

**Rationale**: Automated database setup on first container startup

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 21: Docker Infrastructure - .dockerignore
**Action**: Create new file  
**File**: `.dockerignore` (workspace root)  
**Implementation**: As specified in deployment-architecture.md
- Exclude: target/, aw-webui/node_modules/, .git/, *.md, docs/, aidlc-docs/, secrets/

**Rationale**: Optimize Docker build context size

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 22: Operations Scripts - Backup Script
**Action**: Create new file  
**File**: `scripts/backup-database.sh` (workspace root)  
**Implementation**: As specified in deployment-architecture.md
- pg_dump with gzip compression
- 30-day retention cleanup
- Timestamp-based naming

**Rationale**: Automated database backup for production

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 23: Operations Scripts - Restore Script
**Action**: Create new file  
**File**: `scripts/restore-database.sh` (workspace root)  
**Implementation**: As specified in deployment-architecture.md
- Stop aw-server
- Drop and recreate database
- Restore from SQL dump
- Restart aw-server

**Rationale**: Database restoration procedure for disaster recovery

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 24: Documentation - Deployment Guide
**Action**: Create new file  
**File**: `aidlc-docs/construction/unit-1-database-layer/code/deployment-guide.md`  
**Content**:
- Prerequisites checklist
- Initial deployment steps
- Update procedure
- Backup/restore procedures
- Monitoring commands
- Troubleshooting common issues
- Links to deployment-architecture.md

**Rationale**: Complete deployment documentation for operators

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

### Step 25: Documentation - README Update
**Action**: Modify existing file  
**File**: `README.md` (workspace root)  
**Changes**:
- Add: PostgreSQL requirement (replace SQLite mention)
- Add: Docker Compose deployment instructions
- Add: Environment variable documentation (DB_HOST, DB_PORT, etc.)
- Update: Build instructions (if changed)
- Keep: Existing sections (project description, features, etc.)

**Rationale**: Update project documentation for new database backend

**Stories**: Database Layer Migration  
**Checkbox**: [x]

---

## Plan Summary

**Total Steps**: 25  
**Modified Files**: 7 (Cargo.toml, worker.rs, datastore.rs, lib.rs, config.rs, main.rs, README.md)  
**New Files**: 18 (4 Rust modules, 5 test files, 9 deployment/operations files)  
**Tests**: 5 test suites (retry, metrics, health, integration, migrations)

**Execution Strategy**: Sequential (dependencies must be resolved top-down)
1. Dependencies first (Cargo.toml)
2. Core components (worker, datastore, retry, metrics, health, migrations)
3. Configuration (config.rs, main.rs)
4. Tests (unit + integration)
5. Documentation (code summaries)
6. Deployment artifacts (Docker, scripts)
7. Final documentation (README)

**API Compatibility**: 100% - No breaking changes to public Datastore interface

**Testing Coverage**: Unit tests for all new components + integration tests with testcontainers

**Deployment Readiness**: Complete Docker infrastructure with healthchecks, backups, monitoring
