# Logical Components - Unit 1: Database Layer

**Version**: 1.0  
**Date**: 2026-05-18  
**Unit**: Database Layer Migration  

---

## Overview

This document defines the logical components, their responsibilities, and interactions for the PostgreSQL database layer implementation.

---

## Component Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Rocket HTTP Layer                         │
│                    (API Endpoints - Existing)                    │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             │ API Calls
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Datastore Facade                            │
│              (High-level business operations)                    │
│  - create_bucket(), insert_event(), query_events()              │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             │ Async/Await
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Database Worker (NEW)                         │
│           (Connection pooling + Query execution)                 │
│  ┌───────────────┐  ┌──────────────┐  ┌────────────────┐       │
│  │  Connection   │  │    Retry     │  │   Metrics      │       │
│  │     Pool      │  │   Handler    │  │   Collector    │       │
│  │  (deadpool)   │  │  (backoff)   │  │  (prometheus)  │       │
│  └───────────────┘  └──────────────┘  └────────────────┘       │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             │ tokio-postgres
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                     PostgreSQL 15 Server                         │
│                   (Database + Schema + Data)                     │
└─────────────────────────────────────────────────────────────────┘
```

---

## 1. Core Components

### 1.1 Datastore Facade

**Purpose**: High-level business operations interface (unchanged public API)

**Responsibilities**:
- Provide bucket and event CRUD operations
- Maintain API compatibility with existing SQLite interface
- Coordinate transactions across multiple database operations
- Translate domain objects ↔ database rows

**Key Methods**:
```rust
pub trait DatastoreInterface {
    async fn create_bucket(&self, bucket: NewBucket) -> Result<Bucket, DatastoreError>;
    async fn get_bucket(&self, name: &str) -> Result<Bucket, DatastoreError>;
    async fn get_all_buckets(&self) -> Result<Vec<Bucket>, DatastoreError>;
    async fn delete_bucket(&self, name: &str) -> Result<(), DatastoreError>;
    
    async fn insert_event(&self, bucket_name: &str, event: Event) -> Result<i64, DatastoreError>;
    async fn insert_events(&self, bucket_name: &str, events: Vec<Event>) -> Result<Vec<i64>, DatastoreError>;
    async fn get_events(&self, bucket_name: &str, start: DateTime<Utc>, end: DateTime<Utc>, limit: Option<i64>) -> Result<Vec<Event>, DatastoreError>;
    async fn delete_events_by_id(&self, bucket_name: &str, event_ids: Vec<i64>) -> Result<(), DatastoreError>;
    
    async fn get_setting(&self, key: &str) -> Result<Option<String>, DatastoreError>;
    async fn set_setting(&self, key: &str, value: &str) -> Result<(), DatastoreError>;
}
```

**Dependencies**:
- DatabaseWorker (for connection pooling)
- RetryHandler (for transient error recovery)
- MetricsCollector (for observability)

**Location**: `aw-datastore/src/datastore.rs`

---

### 1.2 Database Worker

**Purpose**: Manage connection pool and execute queries

**Responsibilities**:
- Initialize and maintain PostgreSQL connection pool
- Provide connection acquisition interface
- Execute queries with timeout and error handling
- Manage connection lifecycle (creation, recycling, cleanup)

**Structure**:
```rust
pub struct DatabaseWorker {
    pool: Arc<deadpool_postgres::Pool>,
    retry_handler: RetryHandler,
    metrics: Arc<DbMetrics>,
    config: DbConfig,
}

impl DatabaseWorker {
    pub async fn new(config: DbConfig) -> Result<Self, DatastoreError> {
        let pool = Self::create_pool(&config)?;
        let retry_handler = RetryHandler::default();
        let metrics = Arc::new(DbMetrics::new());
        
        let worker = DatabaseWorker {
            pool: Arc::new(pool),
            retry_handler,
            metrics,
            config,
        };
        
        // Warm connections on startup
        worker.warm_connections().await?;
        
        Ok(worker)
    }
    
    pub async fn get_connection(&self) -> Result<PooledConnection, DatastoreError> {
        let start = Instant::now();
        
        let conn = self.pool.get().await
            .map_err(|e| DatastoreError::ConnectionPoolExhausted {
                timeout: 5000,
            })?;
        
        let duration = start.elapsed();
        self.metrics.record_connection_acquisition(duration);
        
        if duration.as_millis() > 100 {
            warn!("Slow connection acquisition: {}ms", duration.as_millis());
        }
        
        Ok(conn)
    }
    
    pub async fn execute_with_retry<F, T>(&self, operation: F) -> Result<T, DatastoreError>
    where
        F: Fn() -> std::pin::Pin<Box<dyn Future<Output = Result<T, DatastoreError>>>>
    {
        self.retry_handler.execute(operation).await
    }
}
```

**Location**: `aw-datastore/src/worker.rs` (refactored from SQLite version)

**Migration Notes**:
- Existing worker uses MPSC channels for thread-safe SQLite access
- New worker uses connection pool directly (PostgreSQL is thread-safe)
- MPSC channel pattern no longer needed (simpler architecture)

---

### 1.3 Connection Pool Manager

**Purpose**: Manage pool of PostgreSQL connections

**Implementation**: `deadpool-postgres` (external library)

**Configuration**:
```rust
pub struct PoolConfig {
    pub max_size: usize,           // 20 connections
    pub min_idle: usize,           // 5 idle connections
    pub connection_timeout: Duration, // 5 seconds
    pub idle_timeout: Duration,    // 10 minutes
    pub max_lifetime: Duration,    // 1 hour
}
```

**Pool Lifecycle**:
1. **Initialization**: Create min_idle connections on startup
2. **Acquisition**: Provide connection to requester (wait if all busy)
3. **Return**: Connection returns to pool after use (automatic on drop)
4. **Recycling**: Test connection health before reuse
5. **Cleanup**: Close idle connections after timeout
6. **Recreation**: Replace connections after max lifetime

**Health Checking**:
- Connections tested before reuse (`SELECT 1` query)
- Failed connections removed from pool
- New connections created to maintain min_idle

**Metrics**:
- Active connections (in use)
- Idle connections (available)
- Wait time for connection acquisition
- Connection creation failures

---

### 1.4 Retry Handler

**Purpose**: Implement exponential backoff retry logic

**Responsibilities**:
- Detect transient errors (connection failures, deadlocks)
- Retry operations with exponential backoff
- Add jitter to prevent thundering herd
- Fail fast on permanent errors (constraint violations)

**Structure**:
```rust
pub struct RetryHandler {
    policy: RetryPolicy,
}

pub struct RetryPolicy {
    max_attempts: u32,      // 5 attempts
    initial_delay_ms: u64,  // 100ms
    max_delay_ms: u64,      // 5000ms
}

impl RetryHandler {
    pub async fn execute<F, T>(&self, operation: F) -> Result<T, DatastoreError>
    where
        F: Fn() -> std::pin::Pin<Box<dyn Future<Output = Result<T, DatastoreError>>>>
    {
        // Retry logic implementation (see nfr-design-patterns.md)
    }
    
    fn is_transient_error(&self, error: &DatastoreError) -> bool {
        matches!(error,
            DatastoreError::ConnectionPoolExhausted { .. } |
            DatastoreError::ConnectionFailed { .. } |
            DatastoreError::QueryFailed { .. } // Check error code for deadlock, etc.
        )
    }
}
```

**Error Classification**:
- **Transient** (retry): Connection timeout, deadlock, serialization failure
- **Permanent** (fail immediately): Constraint violation, bucket not found, invalid data

**Location**: `aw-datastore/src/retry.rs` (new module)

---

### 1.5 Metrics Collector

**Purpose**: Collect and expose database metrics

**Responsibilities**:
- Track query execution metrics (count, duration, errors)
- Monitor connection pool utilization
- Expose metrics for Prometheus scraping
- Alert on threshold violations

**Structure**:
```rust
pub struct DbMetrics {
    queries_total: IntCounter,
    query_duration_seconds: Histogram,
    pool_connections_active: Gauge,
    pool_connections_idle: Gauge,
    pool_wait_duration_seconds: Histogram,
    errors_total: IntCounterVec, // Labeled by error type
}

impl DbMetrics {
    pub fn new() -> Self {
        DbMetrics {
            queries_total: IntCounter::new("db_queries_total", "Total database queries").unwrap(),
            query_duration_seconds: Histogram::with_opts(
                HistogramOpts::new("db_query_duration_seconds", "Query execution time")
                    .buckets(vec![0.001, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0])
            ).unwrap(),
            pool_connections_active: Gauge::new("db_pool_connections_active", "Active connections").unwrap(),
            pool_connections_idle: Gauge::new("db_pool_connections_idle", "Idle connections").unwrap(),
            pool_wait_duration_seconds: Histogram::with_opts(
                HistogramOpts::new("db_pool_wait_duration_seconds", "Time waiting for connection")
                    .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0])
            ).unwrap(),
            errors_total: IntCounterVec::new(
                Opts::new("db_errors_total", "Total database errors"),
                &["error_type"]
            ).unwrap(),
        }
    }
    
    pub fn record_query(&self, duration: Duration, success: bool, query_type: &str) {
        self.queries_total.inc();
        self.query_duration_seconds.observe(duration.as_secs_f64());
        
        if !success {
            self.errors_total.with_label_values(&[query_type]).inc();
        }
    }
    
    pub fn update_pool_stats(&self, status: PoolStatus) {
        self.pool_connections_active.set(status.size as f64);
        self.pool_connections_idle.set((status.max_size - status.size) as f64);
    }
}
```

**Metrics Endpoint**:
- Exposed via Rocket route: `GET /metrics`
- Prometheus text format
- Scrape interval: 15 seconds (recommended)

**Location**: `aw-datastore/src/metrics.rs` (new module)

---

### 1.6 Health Checker

**Purpose**: Verify database connectivity and health

**Responsibilities**:
- Execute health check queries
- Validate schema version
- Report health status to API layer
- Provide readiness/liveness checks for Docker

**Structure**:
```rust
pub struct HealthChecker {
    worker: Arc<DatabaseWorker>,
    timeout: Duration,
}

pub enum HealthStatus {
    Healthy { response_time: Duration },
    Unhealthy { reason: String },
    Degraded { reason: String },
}

impl HealthChecker {
    pub async fn check(&self) -> HealthStatus {
        let start = Instant::now();
        
        let result = tokio::time::timeout(
            self.timeout,
            self.check_database()
        ).await;
        
        let duration = start.elapsed();
        
        match result {
            Ok(Ok(())) => HealthStatus::Healthy { response_time: duration },
            Ok(Err(e)) => HealthStatus::Unhealthy { reason: e.to_string() },
            Err(_) => HealthStatus::Unhealthy { reason: "Timeout".to_string() },
        }
    }
    
    async fn check_database(&self) -> Result<(), DatastoreError> {
        let conn = self.worker.get_connection().await?;
        
        // Simple connectivity check
        conn.execute("SELECT 1", &[]).await?;
        
        // Schema version validation
        let version = self.get_schema_version(&conn).await?;
        if version < REQUIRED_MIN_VERSION {
            return Err(DatastoreError::OldDbVersion {
                current: version,
                required: REQUIRED_MIN_VERSION,
            });
        }
        
        Ok(())
    }
}
```

**Health Check Types**:
- **Liveness**: Is the service running? (Basic connectivity)
- **Readiness**: Can the service handle requests? (Full database check)

**Location**: `aw-datastore/src/health.rs` (new module)

---

### 1.7 Migration Manager

**Purpose**: Apply database schema migrations

**Responsibilities**:
- Track current schema version
- Apply pending migrations in order
- Validate migration success
- Rollback on failure (if possible)

**Structure**:
```rust
pub struct MigrationManager {
    worker: Arc<DatabaseWorker>,
}

impl MigrationManager {
    pub async fn run_migrations(&self) -> Result<(), DatastoreError> {
        let conn = self.worker.get_connection().await?;
        
        let current_version = self.get_current_version(&conn).await?;
        info!("Current schema version: {}", current_version);
        
        if current_version < 1 {
            self.migrate_v0_to_v1(&conn).await?;
        }
        // Future migrations: v1→v2, v2→v3, etc.
        
        Ok(())
    }
    
    async fn migrate_v0_to_v1(&self, conn: &Client) -> Result<(), DatastoreError> {
        info!("Applying migration v0 → v1 (PostgreSQL schema creation)");
        
        let tx = conn.transaction().await?;
        
        // Execute schema creation SQL
        tx.batch_execute(include_str!("../migrations/v1_initial_schema.sql")).await?;
        
        // Update schema version
        tx.execute("INSERT INTO schema_version (version) VALUES (1)", &[]).await?;
        
        tx.commit().await?;
        
        info!("Migration v0 → v1 complete");
        Ok(())
    }
    
    async fn get_current_version(&self, conn: &Client) -> Result<i32, DatastoreError> {
        // Check if schema_version table exists
        let result = conn.query_opt(
            "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1",
            &[]
        ).await;
        
        match result {
            Ok(Some(row)) => Ok(row.get(0)),
            Ok(None) => Ok(0), // No version = v0 (uninitialized)
            Err(_) => Ok(0),   // Table doesn't exist = v0
        }
    }
}
```

**Migration Files**:
- `aw-datastore/migrations/v1_initial_schema.sql` - PostgreSQL schema creation
- Future migrations: `v2_add_indexes.sql`, etc.

**Location**: `aw-datastore/src/migrations.rs` (new module)

---

## 2. Supporting Components

### 2.1 Configuration Manager

**Purpose**: Load and validate database configuration

**Structure**:
```rust
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub pool: PoolConfig,
}

impl DbConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load from environment variables (see nfr-design-patterns.md)
    }
    
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        // Load from TOML file
    }
    
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate required fields, ranges, etc.
    }
}
```

**Configuration Sources** (priority order):
1. Environment variables (highest priority)
2. Config file (`~/.config/activitywatch/aw-server.toml`)
3. Default values (lowest priority)

**Location**: `aw-server/src/config.rs` (extend existing)

---

### 2.2 Error Handler

**Purpose**: Convert database errors to appropriate HTTP responses

**Structure**:
```rust
impl From<DatastoreError> for rocket::http::Status {
    fn from(err: DatastoreError) -> Self {
        match err {
            DatastoreError::BucketNotFound { .. } => Status::NotFound,
            DatastoreError::ConstraintViolation { .. } => Status::Conflict,
            DatastoreError::ConnectionPoolExhausted { .. } => Status::ServiceUnavailable,
            DatastoreError::OldDbVersion { .. } => Status::InternalServerError,
            _ => Status::InternalServerError,
        }
    }
}

impl From<DatastoreError> for rocket::response::status::Custom<Json<ErrorResponse>> {
    fn from(err: DatastoreError) -> Self {
        let status = Status::from(&err);
        let response = ErrorResponse {
            error: err.to_string(),
            code: status.code,
        };
        rocket::response::status::Custom(status, Json(response))
    }
}
```

**Location**: `aw-server/src/endpoints/util.rs` (extend existing)

---

### 2.3 Query Builder (Optional)

**Purpose**: Build complex queries programmatically

**Status**: Not needed for initial version (queries are simple)

**Future Consideration**: If query complexity increases, consider using query builder library (e.g., `sqlbuilder` crate)

---

## 3. Component Interactions

### 3.1 Event Insertion Flow

```
1. Client → POST /api/0/buckets/{id}/events
2. Rocket Handler → datastore.insert_event(bucket_name, event)
3. Datastore → worker.get_connection()
4. Worker → pool.get() (with retry)
5. Pool → Returns connection
6. Datastore → conn.execute("INSERT INTO events ...")
7. PostgreSQL → Executes query, returns event_id
8. Datastore → metrics.record_query(duration, success)
9. Datastore → Returns event_id to handler
10. Handler → Returns HTTP 200 with event_id
```

**Error Path**:
- If pool exhausted (step 4): Retry with backoff
- If retry exhausted: Return 503 Service Unavailable
- If constraint violation (step 7): Return 409 Conflict
- If bucket not found: Return 404 Not Found

---

### 3.2 Health Check Flow

```
1. Docker → curl http://localhost:5600/api/0/health
2. Rocket Handler → health_checker.check()
3. HealthChecker → worker.get_connection() (with timeout)
4. HealthChecker → conn.execute("SELECT 1")
5. HealthChecker → get_schema_version(conn)
6. HealthChecker → Returns HealthStatus::Healthy
7. Handler → Returns HTTP 200 with JSON response
8. Docker → Marks container healthy
```

**Unhealthy Path**:
- If connection timeout: Return HTTP 503
- If schema version too old: Return HTTP 503
- Docker marks container unhealthy, may restart

---

### 3.3 Startup Flow

```
1. main() → Load configuration from env/file
2. main() → Create DatabaseWorker with config
3. Worker → Create connection pool (deadpool)
4. Worker → warm_connections() (acquire min_idle connections)
5. main() → Create MigrationManager
6. MigrationManager → run_migrations()
7. MigrationManager → Check current schema version
8. MigrationManager → Apply pending migrations (if any)
9. main() → Create HealthChecker, MetricsCollector
10. main() → Launch Rocket server with managed state
11. Rocket → Server ready, accepting requests
```

**Failure Points**:
- Configuration invalid: Exit with error message
- PostgreSQL unreachable: Exit with connection error
- Schema migration failed: Rollback, exit with error
- Connection pool creation failed: Exit with error

---

## 4. Deployment Components

### 4.1 Docker Container

**Purpose**: Package aw-server with all dependencies

**Dockerfile Structure**:
```dockerfile
# Build stage
FROM rust:1.75 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY aw-server/ ./aw-server/
COPY aw-datastore/ ./aw-datastore/
COPY aw-models/ ./aw-models/
# ... other crates
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq5 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/aw-server /usr/local/bin/
EXPOSE 5600
CMD ["aw-server", "--host", "0.0.0.0", "--port", "5600"]
```

**Dependencies**:
- `libpq5` - PostgreSQL client library (required by tokio-postgres)
- `ca-certificates` - SSL/TLS certificates (for HTTPS, future)

---

### 4.2 PostgreSQL Container

**Purpose**: Run PostgreSQL database server

**Docker Compose Configuration**:
```yaml
services:
  postgresql:
    image: postgres:15-alpine
    environment:
      POSTGRES_USER: aw_user
      POSTGRES_PASSWORD: ${DB_PASSWORD}
      POSTGRES_DB: activitywatch
    volumes:
      - pg_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U aw_user -d activitywatch"]
      interval: 10s
      timeout: 5s
      retries: 3
      start_period: 30s
    networks:
      - internal

volumes:
  pg_data:
    driver: local
```

---

### 4.3 Docker Network

**Purpose**: Isolate services, prevent external PostgreSQL access

**Configuration**:
```yaml
networks:
  internal:
    driver: bridge
    internal: false  # Allow external access to aw-server only
```

**Network Topology**:
- `aw-server` → Accessible from host (port 5600 exposed)
- `postgresql` → Only accessible from `aw-server` (not exposed)
- `aw-webui` → Accessible from host (port 80/443 exposed)

---

## 5. Monitoring Components

### 5.1 Prometheus Integration (Future)

**Purpose**: Scrape metrics from aw-server

**Configuration**:
```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'aw-server'
    static_configs:
      - targets: ['aw-server:5600']
    metrics_path: /metrics
    scrape_interval: 15s
```

**Metrics Exposed**:
- `db_queries_total` - Total queries
- `db_query_duration_seconds` - Query latency histogram
- `db_pool_connections_active` - Active connections
- `db_pool_connections_idle` - Idle connections
- `db_errors_total` - Total errors (by type)

---

### 5.2 Logging Aggregation (Future)

**Purpose**: Centralize logs from multiple containers

**Options**:
- Docker logs driver (json-file, syslog, fluentd)
- Loki + Grafana
- ELK stack

**Status**: Not implemented initially (stdout logs sufficient)

---

## 6. Component Summary

| Component | Purpose | Location | Status |
|-----------|---------|----------|--------|
| Datastore Facade | Public API interface | `aw-datastore/src/datastore.rs` | ✅ Refactor existing |
| Database Worker | Connection pooling | `aw-datastore/src/worker.rs` | ✅ Refactor existing |
| Connection Pool | Pool management | `deadpool-postgres` (external) | ✅ New dependency |
| Retry Handler | Exponential backoff | `aw-datastore/src/retry.rs` | ✅ New module |
| Metrics Collector | Observability | `aw-datastore/src/metrics.rs` | ✅ New module |
| Health Checker | Health checks | `aw-datastore/src/health.rs` | ✅ New module |
| Migration Manager | Schema migrations | `aw-datastore/src/migrations.rs` | ✅ New module |
| Configuration Manager | Config loading | `aw-server/src/config.rs` | ✅ Extend existing |
| Error Handler | Error mapping | `aw-server/src/endpoints/util.rs` | ✅ Extend existing |
| Docker Container | aw-server packaging | `Dockerfile` | ✅ New file |
| PostgreSQL Container | Database server | `docker-compose.yml` | ✅ New file |

**Legend**:
- ✅ - Included in initial implementation
- 🔮 - Future enhancement
- ⚠️ - External dependency

---

## 7. Data Flow Diagrams

### 7.1 Write Path (Event Insertion)

```
┌──────────┐
│  Client  │
└────┬─────┘
     │ POST /api/0/buckets/{id}/events
     ▼
┌────────────────┐
│ Rocket Handler │
└────┬───────────┘
     │ datastore.insert_event()
     ▼
┌─────────────────┐
│  Datastore      │ ◄─── Metrics Collection
│  Facade         │
└────┬────────────┘
     │ worker.get_connection()
     ▼
┌─────────────────┐
│ Database Worker │ ◄─── Retry Logic (if needed)
└────┬────────────┘
     │ pool.get()
     ▼
┌─────────────────┐
│ Connection Pool │
└────┬────────────┘
     │ Provides connection
     ▼
┌─────────────────┐
│   PostgreSQL    │
│   (INSERT)      │
└─────────────────┘
```

### 7.2 Read Path (Event Query)

```
┌──────────┐
│  Client  │
└────┬─────┘
     │ GET /api/0/buckets/{id}/events?start=...&end=...
     ▼
┌────────────────┐
│ Rocket Handler │
└────┬───────────┘
     │ datastore.get_events()
     ▼
┌─────────────────┐
│  Datastore      │ ◄─── Metrics Collection
│  Facade         │
└────┬────────────┘
     │ worker.get_connection()
     ▼
┌─────────────────┐
│ Database Worker │
└────┬────────────┘
     │ pool.get()
     ▼
┌─────────────────┐
│ Connection Pool │
└────┬────────────┘
     │ Provides connection
     ▼
┌─────────────────┐
│   PostgreSQL    │
│   (SELECT)      │
└─────────────────┘
     │
     │ Results returned through layers
     ▼
┌──────────┐
│  Client  │
└──────────┘
```

---

## 8. State Management

### 8.1 Application State (Rocket Managed State)

```rust
#[launch]
fn rocket() -> _ {
    let config = DbConfig::from_env().expect("Failed to load database config");
    let worker = DatabaseWorker::new(config).await.expect("Failed to create database worker");
    let datastore = Datastore::new(worker);
    let health_checker = HealthChecker::new(Arc::clone(&datastore.worker));
    let metrics = Arc::clone(&datastore.worker.metrics);
    
    rocket::build()
        .manage(datastore)
        .manage(health_checker)
        .manage(metrics)
        .mount("/api/0", routes![
            create_bucket,
            get_bucket,
            insert_event,
            // ... other routes
        ])
        .mount("/", routes![health_check, metrics_endpoint])
}
```

**Managed State**:
- `Datastore` - Database operations interface
- `HealthChecker` - Health check component
- `DbMetrics` - Metrics collector

---

### 8.2 Connection Pool State

**Lifecycle**:
- **Created**: On DatabaseWorker initialization
- **Warmed**: min_idle connections created on startup
- **Active**: Connections acquired/released throughout runtime
- **Recycled**: Connections tested and reused
- **Cleaned**: Idle connections closed after timeout
- **Destroyed**: Pool dropped on application shutdown

**State Transitions**:
```
[Created] → [Warming] → [Active] → [Recycling] → [Active]
                   ↓
              [Idle] → [Cleanup] → [Destroyed]
```

---

## 9. Scalability Considerations

### 9.1 Horizontal Scaling (Future)

**Current**: Single aw-server instance → Single PostgreSQL instance

**Future** (if needed):
- Multiple aw-server instances (load balanced)
- PostgreSQL read replicas (for query offloading)
- PgBouncer for connection pooling (if pool exhaustion occurs)

**Not Needed Initially**: 300 watchers well within single-instance capacity

---

### 9.2 Vertical Scaling

**Resource Allocation** (per container):

| Component | CPU | Memory | Disk |
|-----------|-----|--------|------|
| aw-server | 2-4 cores | 2-4 GB | 1 GB (binary + cache) |
| postgresql | 8-16 cores | 16-32 GB | 1-2 TB (data + indexes) |
| aw-webui | 0.5-1 core | 512 MB | 100 MB (static files) |

**Total**: 10-20 cores, 18-36 GB RAM, 1-2 TB storage

---

## 10. Future Enhancements

| Enhancement | Description | Priority | Trigger |
|-------------|-------------|----------|---------|
| Read Replicas | Offload queries to replicas | Low | Query latency >500ms |
| PgBouncer | External connection pooler | Low | Pool exhaustion |
| Cache Layer | Redis for hot data | Low | DB CPU >80% |
| Event Partitioning | Partition events by month | Medium | >1B events |
| Backup Automation | Automated pg_dump to S3 | High | Production deployment |
| Monitoring Dashboard | Grafana + Prometheus | Medium | Production deployment |
| Circuit Breaker | Protect from cascade failures | Low | PostgreSQL instability |

**Recommendation**: Deploy initial version, monitor metrics, enhance as needed.
