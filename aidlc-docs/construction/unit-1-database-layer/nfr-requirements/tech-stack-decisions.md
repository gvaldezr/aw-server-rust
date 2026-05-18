# Tech Stack Decisions - Unit 1: Database Layer

**Version**: 1.0  
**Date**: 2026-05-18  
**Unit**: Database Layer Migration  

---

## Decision Summary

| Component | Technology Choice | Alternative Considered | Rationale |
|-----------|------------------|----------------------|-----------|
| Database | PostgreSQL 15 LTS | PostgreSQL 16, MySQL, MongoDB | LTS support, maturity, SQL standard |
| Rust Driver | tokio-postgres | sqlx, diesel | Async-first, mature, flexible |
| Connection Pooling | deadpool-postgres | bb8, mobc | Battle-tested, tokio ecosystem |
| JSON Handling | serde_json | - | De facto standard for Rust JSON |
| Date/Time | chrono | time crate | Existing dependency, rich API |
| Error Handling | thiserror | anyhow | Structured errors for library code |
| Async Runtime | tokio | async-std | Existing dependency (Rocket uses tokio) |

---

## D1: Database Technology

### Decision: PostgreSQL 15 LTS

**Alternatives Considered**:
- PostgreSQL 16 (latest stable)
- PostgreSQL 14 (older LTS)
- MySQL/MariaDB
- MongoDB (NoSQL)

**Rationale**:
1. **PostgreSQL 15 LTS**: Long-term support (5 years), stable, proven
2. **Native JSON support**: JSONB type perfect for ActivityWatch event data
3. **Strong typing**: Better data integrity than SQLite loose typing
4. **ACID compliance**: Full transactional guarantees
5. **Mature ecosystem**: Excellent Rust drivers, tooling, documentation
6. **Open source**: No licensing concerns (PostgreSQL License)

**Trade-offs**:
- ➕ More robust than SQLite (multi-user, concurrent writes)
- ➕ Better performance for large datasets (indexing, query optimization)
- ➖ More complex deployment (requires separate server process)
- ➖ Higher resource usage (memory, CPU vs SQLite)

**Migration Impact**: Medium (schema mapping required, but straightforward)

---

## D2: Rust Database Driver

### Decision: tokio-postgres

**Alternatives Considered**:
- **sqlx**: Compile-time query validation, async
- **diesel**: ORM, compile-time query validation, sync/async
- **rust-postgres**: Synchronous (blocking)

**Rationale**:
1. **Async-first**: Native tokio integration (Rocket uses tokio)
2. **Mature**: Production-tested, actively maintained
3. **Low-level control**: Direct SQL queries (no ORM abstraction)
4. **Type safety**: Strong typing with Rust type system
5. **Performance**: Minimal overhead, efficient connection reuse
6. **Existing patterns**: Can mirror current rusqlite patterns

**Comparison**:

| Feature | tokio-postgres | sqlx | diesel |
|---------|---------------|------|--------|
| Async | ✅ Native | ✅ Native | ⚠️ Optional |
| Compile-time checks | ❌ Runtime | ✅ Compile-time | ✅ Compile-time |
| ORM | ❌ No | ❌ No | ✅ Yes |
| Learning curve | Low | Medium | High |
| Flexibility | High | High | Medium (ORM constraints) |

**Trade-offs**:
- ➕ Simple, direct SQL (familiar for SQLite users)
- ➕ Low learning curve (no ORM complexity)
- ➖ No compile-time query validation (manual testing required)

**Code Example**:
```rust
use tokio_postgres::{Client, Error};

async fn insert_event(client: &Client, event: &Event) -> Result<i64, Error> {
    let row = client.query_one(
        "INSERT INTO events (bucketrow, starttime, endtime, data) 
         VALUES ($1, $2, $3, $4) RETURNING id",
        &[&event.bucketrow, &event.starttime, &event.endtime, &event.data]
    ).await?;
    
    Ok(row.get(0))
}
```

---

## D3: Connection Pooling

### Decision: deadpool-postgres

**Alternatives Considered**:
- **bb8**: Generic connection pool (supports multiple backends)
- **mobc**: Another generic pool
- **r2d2**: Synchronous pool (not async-compatible)

**Rationale**:
1. **Tokio ecosystem**: Native tokio integration
2. **Battle-tested**: Used in production by many projects
3. **PostgreSQL-specific**: Optimized for PostgreSQL
4. **Configuration**: Flexible pool sizing, timeouts, health checks
5. **Error handling**: Clear error types (timeout, unavailable)

**Comparison**:

| Feature | deadpool-postgres | bb8 | mobc |
|---------|------------------|-----|------|
| Async | ✅ Tokio | ✅ Generic | ✅ Generic |
| PostgreSQL-specific | ✅ Yes | ❌ No | ❌ No |
| Maturity | High | High | Medium |
| Documentation | Excellent | Good | Good |

**Configuration Example**:
```rust
use deadpool_postgres::{Config, Runtime};

let mut cfg = Config::new();
cfg.host = Some(env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()));
cfg.port = Some(env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string()).parse()?);
cfg.user = Some(env::var("DB_USER").unwrap_or_else(|_| "aw_user".to_string()));
cfg.password = Some(env::var("DB_PASSWORD")?);
cfg.dbname = Some(env::var("DB_NAME").unwrap_or_else(|_| "activitywatch".to_string()));

cfg.manager = Some(ManagerConfig {
    recycling_method: RecyclingMethod::Fast,
});

cfg.pool = Some(PoolConfig {
    max_size: 20,
    timeouts: Timeouts {
        wait: Some(Duration::from_secs(5)),
        create: Some(Duration::from_secs(5)),
        recycle: Some(Duration::from_secs(5)),
    },
});

let pool = cfg.create_pool(Some(Runtime::Tokio1))?;
```

**Trade-offs**:
- ➕ Optimized for PostgreSQL (better performance)
- ➕ Clear error handling (timeout vs unavailable)
- ➖ PostgreSQL-specific (can't switch backends easily)

---

## D4: JSON Serialization

### Decision: serde_json

**Alternatives Considered**:
- **simd-json**: Faster JSON parsing (SIMD optimizations)
- **json**: Older JSON library

**Rationale**:
1. **Existing dependency**: Already used in aw-server (no new dependency)
2. **De facto standard**: Most common Rust JSON library
3. **Excellent serde integration**: Seamless serialization/deserialization
4. **tokio-postgres support**: Native support for serde_json::Value
5. **Mature**: Production-tested, stable API

**Code Example**:
```rust
use serde_json::{json, Value};

let event_data: Value = json!({
    "app": "chrome",
    "title": "GitHub - ActivityWatch",
    "url": "https://github.com/ActivityWatch"
});

// tokio-postgres handles Value → JSONB automatically
client.execute(
    "INSERT INTO events (data) VALUES ($1)",
    &[&event_data]
).await?;
```

**Trade-offs**:
- ➕ Zero learning curve (already familiar)
- ➕ Seamless integration with tokio-postgres
- ➖ Slightly slower than simd-json (acceptable trade-off)

---

## D5: Date/Time Handling

### Decision: chrono

**Alternatives Considered**:
- **time**: Modern alternative to chrono
- **jiff**: New time library (experimental)

**Rationale**:
1. **Existing dependency**: Already used throughout aw-server
2. **PostgreSQL support**: tokio-postgres has native chrono support
3. **Rich API**: Comprehensive date/time operations
4. **DateTime<Utc>**: Perfect for PostgreSQL TIMESTAMP WITH TIME ZONE
5. **Backward compatibility**: Minimal code changes from current SQLite code

**Type Mapping**:
```rust
use chrono::{DateTime, Utc};

// PostgreSQL TIMESTAMP WITH TIME ZONE ↔ DateTime<Utc>
let timestamp: DateTime<Utc> = row.get("starttime");
let timestamp: DateTime<Utc> = Utc::now();

// tokio-postgres handles conversion automatically
client.execute(
    "INSERT INTO events (starttime) VALUES ($1)",
    &[&timestamp]
).await?;
```

**Trade-offs**:
- ➕ Zero migration effort (already using chrono)
- ➕ Proven reliability (used in production everywhere)
- ➖ Slightly larger dependency than `time` crate

---

## D6: Error Handling

### Decision: thiserror

**Alternatives Considered**:
- **anyhow**: Context-rich errors (good for applications)
- **Custom error enums**: Manual implementation

**Rationale**:
1. **Structured errors**: Clear error types for library code
2. **Zero-cost abstraction**: Compile-time error derivation
3. **Type-safe**: Error types checked at compile time
4. **Pattern matching**: Easy to handle specific error cases
5. **Library best practice**: Recommended for libraries (vs anyhow for apps)

**Error Type Definition**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatastoreError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Query failed: {0}")]
    QueryError(String),
    
    #[error("Bucket not found: {bucket_name}")]
    BucketNotFound { bucket_name: String },
    
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] tokio_postgres::Error),
}
```

**Trade-offs**:
- ➕ Type-safe error handling (compile-time checks)
- ➕ Easy to convert to HTTP status codes
- ➖ More boilerplate than anyhow (acceptable for library code)

---

## D7: Async Runtime

### Decision: tokio (Existing)

**Alternatives Considered**:
- **async-std**: Alternative async runtime
- **smol**: Lightweight async runtime

**Rationale**:
1. **Existing dependency**: Rocket 0.5 uses tokio
2. **Ecosystem**: Most Rust async libraries target tokio
3. **Mature**: Production-tested, actively maintained
4. **Performance**: Excellent performance characteristics
5. **No migration**: Already using tokio, no changes needed

**Trade-offs**:
- ➕ No migration effort (already using tokio)
- ➕ Largest ecosystem (most libraries support tokio)
- ➖ Larger binary size than smol (acceptable trade-off)

---

## D8: Testing Infrastructure

### Decision: testcontainers-rs + tokio::test

**Alternatives Considered**:
- **Docker Compose**: Manual container management
- **In-memory PostgreSQL**: Limited availability
- **Mock database**: Not testing real PostgreSQL

**Rationale**:
1. **Real PostgreSQL**: Test against actual PostgreSQL instance
2. **Isolated tests**: Each test gets clean database
3. **Automatic cleanup**: Containers destroyed after tests
4. **CI/CD friendly**: Works in CI environments (Docker available)

**Test Setup Example**:
```rust
use testcontainers::{clients, images::postgres::Postgres};

#[tokio::test]
async fn test_insert_event() {
    let docker = clients::Cli::default();
    let postgres = docker.run(Postgres::default());
    let port = postgres.get_host_port_ipv4(5432);
    
    let config = format!(
        "host=localhost port={} user=postgres password=postgres dbname=postgres",
        port
    );
    
    let (client, connection) = tokio_postgres::connect(&config, NoTls).await.unwrap();
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    
    // Run migration
    apply_schema(&client).await.unwrap();
    
    // Test event insertion
    let event = Event { /* ... */ };
    let event_id = insert_event(&client, &event).await.unwrap();
    assert!(event_id > 0);
}
```

**Trade-offs**:
- ➕ High confidence (testing real database)
- ➕ Isolated (no shared test state)
- ➖ Slower than mocks (acceptable for integration tests)

---

## D9: Schema Migration Strategy

### Decision: Custom migration functions (mirror SQLite approach)

**Alternatives Considered**:
- **diesel-cli**: Diesel migration tool
- **sqlx-cli**: SQLx migration tool
- **refinery**: Standalone migration framework

**Rationale**:
1. **Consistency**: Mirror existing SQLite migration pattern
2. **Simplicity**: No external tools required
3. **Flexibility**: Full control over migration logic
4. **Version tracking**: schema_version table (same as SQLite)

**Migration Pattern**:
```rust
async fn apply_migrations(client: &Client) -> Result<(), DatastoreError> {
    let version = get_schema_version(client).await?;
    
    if version < 1 {
        migrate_v0_to_v1(client).await?;
    }
    // Future migrations here
    
    Ok(())
}

async fn migrate_v0_to_v1(client: &Client) -> Result<(), DatastoreError> {
    let tx = client.transaction().await?;
    
    tx.batch_execute("
        CREATE TABLE buckets (...);
        CREATE TABLE events (...);
        CREATE TABLE key_value (...);
        CREATE INDEX idx_events_bucketrow ON events(bucketrow);
        -- ... more schema ...
        CREATE TABLE schema_version (version INTEGER PRIMARY KEY, applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW());
        INSERT INTO schema_version (version) VALUES (1);
    ").await?;
    
    tx.commit().await?;
    Ok(())
}
```

**Trade-offs**:
- ➕ Consistent with existing SQLite approach
- ➕ No external dependencies
- ➖ Manual SQL (no compile-time validation)

---

## Dependency Summary

### New Dependencies (Cargo.toml)

```toml
[dependencies]
# PostgreSQL driver + connection pooling
tokio-postgres = { version = "0.7", features = ["with-chrono-0_4", "with-serde_json-1"] }
deadpool-postgres = "0.10"

# Error handling
thiserror = "1.0"

# Existing dependencies (no changes)
tokio = { version = "1", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[dev-dependencies]
# Testing infrastructure
testcontainers = "0.15"
```

### Removed Dependencies

```toml
# Remove SQLite dependencies
# rusqlite = { version = "0.30", features = ["chrono", "serde_json", "bundled"] }
```

---

## Implementation Timeline

| Phase | Deliverable | Estimated Effort |
|-------|------------|-----------------|
| Phase 1 | Update Cargo.toml dependencies | 1 hour |
| Phase 2 | Implement connection pool setup | 4 hours |
| Phase 3 | Port datastore.rs schema to PostgreSQL | 8 hours |
| Phase 4 | Port worker.rs query logic | 8 hours |
| Phase 5 | Write unit tests | 8 hours |
| Phase 6 | Write integration tests | 8 hours |
| Phase 7 | Performance testing | 4 hours |
| **Total** | **Complete database migration** | **~40 hours** |

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Type conversion bugs (timestamp) | Medium | High | Comprehensive unit tests |
| Connection pool exhaustion | Low | Medium | Monitor pool utilization, alerting |
| Query performance regression | Medium | Medium | Benchmark queries, index optimization |
| Docker networking issues | Low | Low | Standard Docker Compose networking |

---

## Success Criteria

Tech stack decisions are successful when:

✅ All dependencies identified and justified  
✅ Trade-offs documented for each technology choice  
✅ Implementation complexity estimated  
✅ Risk mitigation strategies defined  
✅ Testing infrastructure specified  
✅ Migration timeline estimated  
✅ Backward compatibility plan defined (API unchanged)
