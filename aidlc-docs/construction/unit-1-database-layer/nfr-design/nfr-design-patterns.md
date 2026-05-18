# NFR Design Patterns - Unit 1: Database Layer

**Version**: 1.0  
**Date**: 2026-05-18  
**Unit**: Database Layer Migration  

---

## Overview

This document defines design patterns and architectural approaches for meeting the non-functional requirements (NFR) of the PostgreSQL database layer.

---

## 1. Resilience Patterns

### 1.1 Connection Pool Pattern

**Pattern**: Object Pool with Health Checking

**Implementation**: `deadpool-postgres` with automatic connection recycling

**Design**:
```rust
use deadpool_postgres::{Config, Pool, Runtime, ManagerConfig, RecyclingMethod};
use std::time::Duration;

pub struct DatabasePool {
    pool: Pool,
}

impl DatabasePool {
    pub fn new(config: &DbConfig) -> Result<Self, DatastoreError> {
        let mut cfg = Config::new();
        cfg.host = Some(config.host.clone());
        cfg.port = Some(config.port);
        cfg.user = Some(config.user.clone());
        cfg.password = Some(config.password.clone());
        cfg.dbname = Some(config.database.clone());
        
        // Manager configuration
        cfg.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });
        
        // Pool configuration
        cfg.pool = Some(PoolConfig {
            max_size: 20,
            timeouts: Timeouts {
                wait: Some(Duration::from_secs(5)),
                create: Some(Duration::from_secs(5)),
                recycle: Some(Duration::from_secs(5)),
            },
        });
        
        let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
        Ok(DatabasePool { pool })
    }
    
    pub async fn get_connection(&self) -> Result<PooledConnection, DatastoreError> {
        self.pool.get().await
            .map_err(|e| DatastoreError::ConnectionPoolExhausted(e.to_string()))
    }
}
```

**Benefits**:
- ✅ Connection reuse (avoid connection overhead)
- ✅ Automatic health checking (detect stale connections)
- ✅ Resource limits (prevent connection exhaustion)
- ✅ Fast failure (timeout if pool exhausted)

**Metrics to Monitor**:
- Pool utilization (connections in use / max connections)
- Wait time for connection acquisition
- Connection creation failures

---

### 1.2 Retry Pattern with Exponential Backoff

**Pattern**: Retry with Exponential Backoff and Jitter

**Use Cases**: Transient connection failures, deadlocks, temporary unavailability

**Design**:
```rust
use tokio::time::{sleep, Duration};
use rand::Rng;

pub struct RetryPolicy {
    max_attempts: u32,
    initial_delay_ms: u64,
    max_delay_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        RetryPolicy {
            max_attempts: 5,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
        }
    }
}

impl RetryPolicy {
    pub async fn execute<F, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> std::pin::Pin<Box<dyn Future<Output = Result<T, E>>>>,
        E: std::fmt::Display,
    {
        let mut attempt = 1;
        
        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(err) if attempt >= self.max_attempts => {
                    error!("Operation failed after {} attempts: {}", attempt, err);
                    return Err(err);
                }
                Err(err) if !is_transient_error(&err) => {
                    // Don't retry permanent errors
                    warn!("Non-transient error, not retrying: {}", err);
                    return Err(err);
                }
                Err(err) => {
                    // Calculate delay with exponential backoff + jitter
                    let delay_ms = self.calculate_delay(attempt);
                    warn!("Transient error on attempt {}: {}. Retrying in {}ms", 
                          attempt, err, delay_ms);
                    sleep(Duration::from_millis(delay_ms)).await;
                    attempt += 1;
                }
            }
        }
    }
    
    fn calculate_delay(&self, attempt: u32) -> u64 {
        let exponential_delay = self.initial_delay_ms * 2u64.pow(attempt - 1);
        let capped_delay = exponential_delay.min(self.max_delay_ms);
        
        // Add jitter (±25%)
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(-0.25..=0.25);
        let jittered = capped_delay as f64 * (1.0 + jitter);
        
        jittered as u64
    }
}

fn is_transient_error<E: std::fmt::Display>(error: &E) -> bool {
    let err_str = error.to_string().to_lowercase();
    err_str.contains("connection refused") ||
    err_str.contains("connection reset") ||
    err_str.contains("timeout") ||
    err_str.contains("deadlock") ||
    err_str.contains("could not serialize")
}
```

**Retry Schedule Example**:
- Attempt 1: Immediate
- Attempt 2: ~100ms delay (+ jitter)
- Attempt 3: ~200ms delay (+ jitter)
- Attempt 4: ~400ms delay (+ jitter)
- Attempt 5: ~800ms delay (+ jitter)
- Fail after 5 attempts (~1.5 seconds total)

**Benefits**:
- ✅ Automatic recovery from transient failures
- ✅ Jitter prevents thundering herd
- ✅ Fast failure for permanent errors
- ✅ Configurable retry limits

---

### 1.3 Circuit Breaker Pattern (Future Enhancement)

**Status**: Not implemented in initial version (future consideration)

**Use Case**: Protect against cascading failures if PostgreSQL becomes unavailable

**Design Approach** (if needed):
- Monitor error rate over rolling window
- Open circuit if error rate exceeds threshold (e.g., 50% failures in 10 seconds)
- Half-open state after cooldown period (e.g., 30 seconds)
- Close circuit if requests succeed in half-open state

**Recommendation**: Monitor production metrics first; implement if PostgreSQL failures cause API instability.

---

## 2. Performance Patterns

### 2.1 Bulk Insert Pattern

**Pattern**: Batch Operations with Transactions

**Use Case**: Inserting multiple events in single API call

**Design**:
```rust
pub async fn insert_events_batch(
    &self,
    bucket_id: i32,
    events: Vec<Event>,
) -> Result<Vec<i64>, DatastoreError> {
    if events.is_empty() {
        return Ok(vec![]);
    }
    
    let conn = self.pool.get_connection().await?;
    let tx = conn.transaction().await?;
    
    let mut event_ids = Vec::with_capacity(events.len());
    
    // Use prepared statement for efficiency
    let stmt = tx.prepare(
        "INSERT INTO events (bucketrow, starttime, endtime, data) 
         VALUES ($1, $2, $3, $4) RETURNING id"
    ).await?;
    
    for event in events {
        let row = tx.query_one(
            &stmt,
            &[&bucket_id, &event.starttime, &event.endtime, &event.data]
        ).await?;
        
        event_ids.push(row.get(0));
    }
    
    tx.commit().await?;
    Ok(event_ids)
}
```

**Benefits**:
- ✅ Single transaction (atomicity)
- ✅ Reduced network round-trips
- ✅ Prepared statement reuse

**Performance**:
- Single insert: ~15ms per event
- Batch insert (100 events): ~100ms total (~1ms per event)
- **7-15x faster for batch operations**

---

### 2.2 Query Optimization Pattern

**Pattern**: Indexed Queries with Prepared Statements

**Design**:
```rust
// Leverage composite index: idx_events_timerange (bucketrow, starttime, endtime)
pub async fn query_events_by_timerange(
    &self,
    bucket_id: i32,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    limit: Option<i64>,
) -> Result<Vec<Event>, DatastoreError> {
    let conn = self.pool.get_connection().await?;
    
    let limit_val = limit.unwrap_or(1000).min(10000); // Cap at 10k
    
    let rows = conn.query(
        "SELECT id, bucketrow, starttime, endtime, data 
         FROM events 
         WHERE bucketrow = $1 
           AND starttime >= $2 
           AND endtime <= $3 
         ORDER BY starttime ASC 
         LIMIT $4",
        &[&bucket_id, &start, &end, &limit_val]
    ).await?;
    
    Ok(rows.iter().map(|row| Event::from_row(row)).collect())
}
```

**Index Strategy**:
- Composite index `(bucketrow, starttime, endtime)` - covers most common query
- PostgreSQL uses index for filtering + sorting
- LIMIT applied after index scan (efficient)

**Query Plan** (expected):
```
Index Scan using idx_events_timerange on events
  Index Cond: ((bucketrow = 123) AND (starttime >= '...') AND (endtime <= '...'))
  Limit: 1000
```

---

### 2.3 Connection Warming Pattern

**Pattern**: Pre-warm idle connections on startup

**Design**:
```rust
impl DatabasePool {
    pub async fn warm_connections(&self) -> Result<(), DatastoreError> {
        info!("Warming connection pool (min_idle connections)...");
        
        // Acquire min_idle connections simultaneously
        let futures: Vec<_> = (0..5).map(|_| {
            let pool = self.pool.clone();
            async move {
                let conn = pool.get().await?;
                // Execute simple query to ensure connection is ready
                conn.execute("SELECT 1", &[]).await?;
                Ok::<_, DatastoreError>(())
            }
        }).collect();
        
        futures::future::join_all(futures).await;
        
        info!("Connection pool warmed");
        Ok(())
    }
}
```

**Benefits**:
- ✅ First requests don't wait for connection creation
- ✅ Validates PostgreSQL connectivity on startup
- ✅ Fails fast if PostgreSQL unavailable

---

### 2.4 Lazy Loading Pattern

**Pattern**: Load data only when accessed

**Use Case**: Bucket data (JSONB) may be large; don't load unless needed

**Design**:
```rust
pub struct Bucket {
    pub id: i32,
    pub name: String,
    pub bucket_type: String,
    pub client: String,
    pub hostname: String,
    pub created: DateTime<Utc>,
    // Lazy-loaded data field
    data: Option<serde_json::Value>,
}

impl Bucket {
    pub async fn get_data(&mut self, conn: &Client) -> Result<&serde_json::Value, DatastoreError> {
        if self.data.is_none() {
            let row = conn.query_one(
                "SELECT data FROM buckets WHERE id = $1",
                &[&self.id]
            ).await?;
            self.data = Some(row.get(0));
        }
        Ok(self.data.as_ref().unwrap())
    }
}
```

**Benefits**:
- ✅ Reduces payload size for bucket listing APIs
- ✅ Load on demand (only when needed)

---

## 3. Scalability Patterns

### 3.1 Read-Heavy Workload Pattern

**Pattern**: Connection pool tuned for high read concurrency

**Rationale**:
- ActivityWatch queries dominate writes (10:1 ratio typical)
- Dashboard views query historical data frequently
- Time-range queries are read-only

**Configuration**:
```rust
// Pool sized for concurrent reads (300 watchers)
PoolConfig {
    max_size: 20,      // Support 20 concurrent queries
    min_idle: 5,       // Keep 5 connections warm
}

// PostgreSQL configuration
// max_connections = 50 (pool + admin connections)
```

**Benefits**:
- ✅ High read throughput (multiple queries simultaneously)
- ✅ PostgreSQL MVCC handles concurrent reads efficiently
- ✅ No locking on read operations

---

### 3.2 Write Batching Pattern

**Pattern**: Aggregate writes when possible

**Use Case**: Watchers submitting events in bursts

**Design**:
```rust
pub struct EventBuffer {
    events: Vec<(i32, Event)>, // (bucket_id, event)
    max_size: usize,
    flush_interval: Duration,
}

impl EventBuffer {
    pub async fn add_event(&mut self, bucket_id: i32, event: Event) {
        self.events.push((bucket_id, event));
        
        if self.events.len() >= self.max_size {
            self.flush().await;
        }
    }
    
    async fn flush(&mut self) {
        if self.events.is_empty() {
            return;
        }
        
        // Group events by bucket_id
        let mut events_by_bucket: HashMap<i32, Vec<Event>> = HashMap::new();
        for (bucket_id, event) in self.events.drain(..) {
            events_by_bucket.entry(bucket_id).or_default().push(event);
        }
        
        // Batch insert per bucket
        for (bucket_id, events) in events_by_bucket {
            if let Err(e) = self.datastore.insert_events_batch(bucket_id, events).await {
                error!("Failed to flush events: {}", e);
            }
        }
    }
}
```

**Benefits**:
- ✅ Reduce transaction overhead
- ✅ Fewer network round-trips
- ✅ Better throughput under load

**Trade-offs**:
- ⚠️ Introduces latency (events buffered)
- ⚠️ Complexity (buffer management, error handling)

**Recommendation**: Implement if write throughput becomes bottleneck (monitor first).

---

### 3.3 Partitioning Pattern (Future)

**Pattern**: Partition events table by time (monthly partitions)

**Status**: Not needed for initial deployment (implement after 1 year)

**Trigger**: When events table exceeds 1B rows (~500 GB)

**Design Approach**:
```sql
-- Create parent table (existing events table becomes template)
CREATE TABLE events_y2027m01 PARTITION OF events
    FOR VALUES FROM ('2027-01-01') TO ('2027-02-01');

CREATE TABLE events_y2027m02 PARTITION OF events
    FOR VALUES FROM ('2027-02-01') TO ('2027-03-01');

-- ... etc
```

**Benefits** (when implemented):
- ✅ Faster queries (prune partitions outside time range)
- ✅ Easier archival (drop old partitions)
- ✅ Parallel vacuum/analyze per partition

---

## 4. Security Patterns

### 4.1 Secure Credential Management Pattern

**Pattern**: Environment Variables with Validation

**Design**:
```rust
use std::env;

pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl DbConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let config = DbConfig {
            host: env::var("DB_HOST")
                .unwrap_or_else(|_| "localhost".to_string()),
            port: env::var("DB_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidPort)?,
            user: env::var("DB_USER")
                .unwrap_or_else(|_| "aw_user".to_string()),
            password: env::var("DB_PASSWORD")
                .map_err(|_| ConfigError::MissingPassword)?,
            database: env::var("DB_NAME")
                .unwrap_or_else(|_| "activitywatch".to_string()),
        };
        
        // Validate required fields
        if config.password.is_empty() {
            return Err(ConfigError::MissingPassword);
        }
        
        Ok(config)
    }
}
```

**Security Rules**:
- ✅ Never hardcode credentials in source code
- ✅ Never log credentials (redact in logs)
- ✅ Validate on startup (fail fast if missing)
- ✅ Use Docker secrets in production (not plain env vars)

**Docker Secrets Integration** (production):
```yaml
services:
  aw-server:
    secrets:
      - db_password
    environment:
      DB_PASSWORD_FILE: /run/secrets/db_password

secrets:
  db_password:
    external: true
```

---

### 4.2 SQL Injection Prevention Pattern

**Pattern**: Parameterized Queries Only

**Rule**: **NEVER concatenate user input into SQL strings**

**Safe Example**:
```rust
// ✅ SAFE - Parameterized query
let rows = conn.query(
    "SELECT * FROM events WHERE bucketrow = $1 AND data->>'app' = $2",
    &[&bucket_id, &app_name]
).await?;
```

**Unsafe Example** (NEVER DO THIS):
```rust
// ❌ UNSAFE - SQL injection vulnerability
let query = format!(
    "SELECT * FROM events WHERE bucketrow = {} AND data->>'app' = '{}'",
    bucket_id, app_name
);
let rows = conn.query(&query, &[]).await?;
```

**Enforcement**:
- All queries use tokio-postgres parameter binding (`$1`, `$2`, etc.)
- Code review checklist: No string interpolation in SQL
- Static analysis: Lint for SQL string concatenation

---

### 4.3 Least Privilege Pattern

**Pattern**: Database User with Minimal Permissions

**PostgreSQL Setup**:
```sql
-- Create application user (not superuser)
CREATE USER aw_user WITH PASSWORD 'secure_password';

-- Grant only required permissions
GRANT CONNECT ON DATABASE activitywatch TO aw_user;
GRANT SELECT, INSERT, DELETE ON ALL TABLES IN SCHEMA public TO aw_user;
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO aw_user;

-- Explicitly deny dangerous operations
REVOKE UPDATE ON events FROM aw_user;  -- Events are immutable
REVOKE DROP, ALTER ON ALL TABLES FROM aw_user;
REVOKE CREATE ON SCHEMA public FROM aw_user;
```

**Benefits**:
- ✅ Limits blast radius of compromise
- ✅ Prevents accidental schema changes
- ✅ Enforces immutability (no UPDATE on events)

---

## 5. Observability Patterns

### 5.1 Structured Logging Pattern

**Pattern**: Contextual, Structured Logs

**Design**:
```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(self, event), fields(bucket_id = %bucket_id, event_id))]
pub async fn insert_event(
    &self,
    bucket_id: i32,
    event: Event,
) -> Result<i64, DatastoreError> {
    let start = Instant::now();
    
    let result = self.retry_policy.execute(|| {
        Box::pin(async {
            let conn = self.pool.get_connection().await?;
            conn.execute(
                "INSERT INTO events (bucketrow, starttime, endtime, data) VALUES ($1, $2, $3, $4) RETURNING id",
                &[&bucket_id, &event.starttime, &event.endtime, &event.data]
            ).await
        })
    }).await;
    
    let duration = start.elapsed();
    
    match result {
        Ok(event_id) => {
            info!(
                event_id = %event_id,
                duration_ms = %duration.as_millis(),
                "Event inserted successfully"
            );
            Ok(event_id)
        }
        Err(e) => {
            error!(
                error = %e,
                duration_ms = %duration.as_millis(),
                "Failed to insert event"
            );
            Err(e)
        }
    }
}
```

**Log Levels**:
- **ERROR**: Database failures, retry exhaustion, connection pool exhaustion
- **WARN**: Slow queries (>100ms), retries, high pool utilization (>80%)
- **INFO**: Connection pool stats, migration execution, startup/shutdown
- **DEBUG**: All SQL queries with parameters (development only)

**Structured Fields**:
- `bucket_id`, `event_id` - Entity identifiers
- `duration_ms` - Operation timing
- `error` - Error details
- `retry_attempt` - Retry count

---

### 5.2 Metrics Collection Pattern

**Pattern**: Prometheus-style Metrics

**Design**:
```rust
use prometheus::{IntCounter, Histogram, Gauge};

pub struct DbMetrics {
    pub queries_total: IntCounter,
    pub query_duration: Histogram,
    pub pool_connections_active: Gauge,
    pub pool_connections_idle: Gauge,
    pub errors_total: IntCounter,
}

impl DbMetrics {
    pub fn record_query(&self, duration: Duration, success: bool) {
        self.queries_total.inc();
        self.query_duration.observe(duration.as_secs_f64());
        
        if !success {
            self.errors_total.inc();
        }
    }
    
    pub fn update_pool_stats(&self, active: usize, idle: usize) {
        self.pool_connections_active.set(active as f64);
        self.pool_connections_idle.set(idle as f64);
    }
}
```

**Metrics Exposed**:
- `db_queries_total` - Total queries executed
- `db_query_duration_seconds` - Query latency histogram
- `db_pool_connections_active` - Active connections
- `db_pool_connections_idle` - Idle connections
- `db_errors_total` - Total database errors

---

### 5.3 Health Check Pattern

**Pattern**: Deep Health Check with Timeout

**Design**:
```rust
pub struct HealthChecker {
    pool: Arc<DatabasePool>,
    timeout: Duration,
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
            Err(_) => HealthStatus::Unhealthy { reason: "Health check timeout".to_string() },
        }
    }
    
    async fn check_database(&self) -> Result<(), DatastoreError> {
        let conn = self.pool.get_connection().await?;
        
        // Simple query to verify connectivity
        conn.execute("SELECT 1", &[]).await?;
        
        // Optional: Check schema version
        let version = self.get_schema_version(&conn).await?;
        if version < REQUIRED_MIN_VERSION {
            return Err(DatastoreError::OldDbVersion { current: version });
        }
        
        Ok(())
    }
}
```

**Health Check Endpoint** (Rocket):
```rust
#[get("/api/0/health")]
async fn health_check(health_checker: &State<HealthChecker>) -> Result<Json<HealthResponse>, Status> {
    match health_checker.check().await {
        HealthStatus::Healthy { response_time } => {
            Ok(Json(HealthResponse {
                status: "healthy",
                database: "ok",
                response_time_ms: response_time.as_millis() as u64,
            }))
        }
        HealthStatus::Unhealthy { reason } => {
            Err(Status::ServiceUnavailable)
        }
    }
}
```

---

## 6. Error Handling Patterns

### 6.1 Error Classification Pattern

**Pattern**: Typed Errors with Context

**Design**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatastoreError {
    #[error("Connection pool exhausted (timeout: {timeout}ms)")]
    ConnectionPoolExhausted { timeout: u64 },
    
    #[error("Database connection failed: {source}")]
    ConnectionFailed { source: tokio_postgres::Error },
    
    #[error("Query failed: {query}")]
    QueryFailed { query: String, source: tokio_postgres::Error },
    
    #[error("Transaction failed: {reason}")]
    TransactionFailed { reason: String },
    
    #[error("Bucket not found: {bucket_name}")]
    BucketNotFound { bucket_name: String },
    
    #[error("Constraint violation: {constraint}")]
    ConstraintViolation { constraint: String },
    
    #[error("Database schema too old: current={current}, required={required}")]
    OldDbVersion { current: i32, required: i32 },
}
```

**Error Mapping to HTTP Status**:
```rust
impl From<DatastoreError> for Status {
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
```

---

### 6.2 Graceful Degradation Pattern

**Pattern**: Fallback Behavior on Failure

**Example**: If database unavailable, return cached data (if applicable)

**Design**:
```rust
pub async fn get_bucket(&self, name: &str) -> Result<Bucket, DatastoreError> {
    // Try database first
    match self.get_bucket_from_db(name).await {
        Ok(bucket) => Ok(bucket),
        Err(DatastoreError::ConnectionPoolExhausted { .. }) => {
            // Fallback to cache (if implemented)
            warn!("Database unavailable, checking cache for bucket: {}", name);
            self.get_bucket_from_cache(name).await
        }
        Err(e) => Err(e),
    }
}
```

**Status**: Cache not implemented in initial version (future enhancement if needed).

---

## 7. Testing Patterns

### 7.1 Testcontainers Pattern

**Pattern**: Isolated PostgreSQL Containers per Test Suite

**Design**:
```rust
use testcontainers::{clients, images::postgres::Postgres};

#[tokio::test]
async fn test_insert_event() {
    // Spin up PostgreSQL container
    let docker = clients::Cli::default();
    let postgres = docker.run(Postgres::default());
    let port = postgres.get_host_port_ipv4(5432);
    
    // Create datastore with test database
    let config = DbConfig {
        host: "localhost".to_string(),
        port,
        user: "postgres".to_string(),
        password: "postgres".to_string(),
        database: "postgres".to_string(),
    };
    
    let datastore = Datastore::new(config).await.unwrap();
    
    // Run migrations
    datastore.run_migrations().await.unwrap();
    
    // Test event insertion
    let bucket = create_test_bucket(&datastore).await;
    let event = create_test_event();
    let event_id = datastore.insert_event(bucket.id, event).await.unwrap();
    
    assert!(event_id > 0);
}
```

**Benefits**:
- ✅ Real PostgreSQL (no mocks)
- ✅ Isolated (no shared state between tests)
- ✅ Automatic cleanup (container destroyed after test)

---

## 8. Pattern Summary

| Pattern | Category | Priority | Status |
|---------|----------|----------|--------|
| Connection Pool | Resilience | High | ✅ Initial |
| Retry with Backoff | Resilience | High | ✅ Initial |
| Circuit Breaker | Resilience | Low | 🔮 Future |
| Bulk Insert | Performance | High | ✅ Initial |
| Query Optimization | Performance | High | ✅ Initial |
| Connection Warming | Performance | Medium | ✅ Initial |
| Lazy Loading | Performance | Low | 🔮 Optional |
| Read-Heavy Tuning | Scalability | High | ✅ Initial |
| Write Batching | Scalability | Medium | 🔮 Future |
| Partitioning | Scalability | Low | 🔮 Future (>1 year) |
| Secure Credentials | Security | High | ✅ Initial |
| SQL Injection Prevention | Security | High | ✅ Initial |
| Least Privilege | Security | High | ✅ Initial |
| Structured Logging | Observability | High | ✅ Initial |
| Metrics Collection | Observability | Medium | ✅ Initial |
| Health Checks | Observability | High | ✅ Initial |
| Error Classification | Error Handling | High | ✅ Initial |
| Graceful Degradation | Error Handling | Low | 🔮 Future |
| Testcontainers | Testing | High | ✅ Initial |

**Legend**:
- ✅ Initial - Implement in initial migration
- 🔮 Future - Consider after production monitoring
- 🔮 Optional - Implement only if needed
