# Business Logic Model - Unit 1: Database Layer

**Version**: 1.0  
**Date**: 2026-05-18  
**Unit**: Database Layer Migration  

---

## Overview

The database layer provides persistent storage for ActivityWatch data through a worker-thread architecture. All database operations are channeled through an MPSC (multi-producer single-consumer) queue to ensure thread-safe access.

---

## Architecture Pattern: Worker Thread Model

### Current Pattern (SQLite)
```
API Layer (Multiple Threads)
    ↓ (MPSC Channel)
Database Worker Thread (Single)
    ↓ (Single Connection)
SQLite Database
```

### New Pattern (PostgreSQL)
```
API Layer (Multiple Threads)
    ↓ (MPSC Channel)
Database Worker Thread (Single)
    ↓ (Connection Pool)
PostgreSQL Database
```

**Key Change**: Replace single SQLite connection with PostgreSQL connection pool (deadpool-postgres or bb8).

---

## Core Business Workflows

### Workflow 1: Create Bucket

**Trigger**: API call to `POST /api/0/buckets/{bucket_id}`

**Business Logic**:
1. Validate bucket metadata (name, type, client, hostname)
2. Check if bucket with same name already exists
3. If exists: Return existing bucket (idempotent)
4. If not exists: Insert new bucket with current timestamp
5. Return bucket object with generated ID

**SQL Operations**:
```sql
-- Check existence
SELECT id, name, type, client, hostname, created, data 
FROM buckets 
WHERE name = $1;

-- Insert if not exists (using ON CONFLICT for idempotency)
INSERT INTO buckets (name, type, client, hostname, created, data)
VALUES ($1, $2, $3, $4, NOW(), $5)
ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
RETURNING id, name, type, client, hostname, created, data;
```

**Transaction**: Single statement (implicit transaction, or explicit if needed)

---

### Workflow 2: Insert Event(s)

**Trigger**: API call to `POST /api/0/buckets/{bucket_id}/events`

**Business Logic**:
1. Resolve bucket name → bucket ID (lookup)
2. Validate event data (timestamps, JSON structure)
3. Ensure `endtime >= starttime`
4. Insert event(s) in batch (if multiple events submitted)
5. Return event ID(s)

**SQL Operations**:
```sql
-- Lookup bucket ID
SELECT id FROM buckets WHERE name = $1;

-- Insert single event
INSERT INTO events (bucketrow, starttime, endtime, data)
VALUES ($1, $2, $3, $4)
RETURNING id;

-- Batch insert (if multiple events)
INSERT INTO events (bucketrow, starttime, endtime, data)
VALUES 
    ($1, $2, $3, $4),
    ($1, $5, $6, $7),
    ...
RETURNING id;
```

**Transaction**: 
- Single insert: No explicit transaction needed
- Batch insert: Wrap in explicit transaction for atomicity

**Performance Optimization**: Use prepared statements for event insertion (frequent operation).

---

### Workflow 3: Query Events by Time Range

**Trigger**: API call to `GET /api/0/buckets/{bucket_id}/events?start=...&end=...`

**Business Logic**:
1. Resolve bucket name → bucket ID
2. Parse start/end timestamps (ISO 8601 format)
3. Query events within time range
4. Apply limit if specified (optional pagination)
5. Order by starttime (ascending)
6. Return event list

**SQL Operations**:
```sql
-- Query events in time range
SELECT id, bucketrow, starttime, endtime, data
FROM events
WHERE bucketrow = $1
  AND starttime >= $2
  AND endtime <= $3
ORDER BY starttime ASC
LIMIT $4;
```

**Index Usage**: `idx_events_timerange` (composite index on bucketrow + starttime + endtime)

**Transaction**: Read-only, no transaction needed (autocommit)

---

### Workflow 4: Delete Bucket

**Trigger**: API call to `DELETE /api/0/buckets/{bucket_id}`

**Business Logic**:
1. Resolve bucket name → bucket ID
2. Delete bucket (CASCADE will delete all events)
3. Return success confirmation

**SQL Operations**:
```sql
-- Delete bucket (events cascade automatically)
DELETE FROM buckets WHERE name = $1;
```

**Transaction**: Single statement (implicit transaction)

**Cascade Behavior**: PostgreSQL foreign key constraint with `ON DELETE CASCADE` automatically removes all associated events.

---

### Workflow 5: Get All Buckets

**Trigger**: API call to `GET /api/0/buckets`

**Business Logic**:
1. Query all buckets (no filtering)
2. Return bucket metadata list

**SQL Operations**:
```sql
-- List all buckets
SELECT id, name, type, client, hostname, created, data
FROM buckets
ORDER BY name ASC;
```

**Transaction**: Read-only, no transaction needed

---

### Workflow 6: Get/Set Key-Value Settings

**Trigger**: API calls to `/api/0/settings`

**Business Logic (Get)**:
1. Query specific key or all keys
2. Return value(s)

**Business Logic (Set)**:
1. Insert or update key-value pair (UPSERT)
2. Return confirmation

**SQL Operations**:
```sql
-- Get single key
SELECT value FROM key_value WHERE key = $1;

-- Get all keys
SELECT key, value FROM key_value;

-- Set key (UPSERT)
INSERT INTO key_value (key, value)
VALUES ($1, $2)
ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value;
```

---

## Connection Management

### Connection Pool Configuration

**Library**: `deadpool-postgres` (recommended) or `bb8-postgres`

**Pool Settings**:
```rust
PoolConfig {
    max_size: 20,           // Maximum connections in pool
    min_idle: 5,            // Minimum idle connections
    connection_timeout: 5s, // Wait timeout for acquiring connection
    idle_timeout: Some(600s), // Close idle connections after 10min
    max_lifetime: Some(3600s), // Recreate connections after 1 hour
}
```

**Rationale**:
- `max_size: 20` - Supports high-scale deployment (300 watchers, 30-50 concurrent API requests)
- `min_idle: 5` - Keep 5 connections warm for immediate use under load
- Timeouts prevent connection leaks and stale connections

### Connection Acquisition Pattern

```rust
async fn execute_query<T>(&self, query: Query) -> Result<T> {
    // Acquire connection from pool
    let conn = self.pool.get().await?;
    
    // Execute query
    let result = conn.query(&query.sql, &query.params).await?;
    
    // Connection automatically returns to pool on drop
    Ok(result)
}
```

---

## Transaction Strategy

### Explicit Transactions (When Needed)

**Use Cases**:
- Batch event insertions (atomicity required)
- Complex operations spanning multiple queries
- Schema migrations

**Pattern**:
```rust
async fn insert_batch_events(&self, events: Vec<Event>) -> Result<()> {
    let mut conn = self.pool.get().await?;
    let transaction = conn.transaction().await?;
    
    for event in events {
        transaction.execute(
            "INSERT INTO events (bucketrow, starttime, endtime, data) VALUES ($1, $2, $3, $4)",
            &[&event.bucketrow, &event.starttime, &event.endtime, &event.data]
        ).await?;
    }
    
    transaction.commit().await?;
    Ok(())
}
```

### Isolation Levels

**Default**: `READ COMMITTED` (PostgreSQL default)
- Sufficient for ActivityWatch workload
- Prevents dirty reads
- Allows concurrent writes

**Alternative**: `SERIALIZABLE` (if stricter isolation needed)
- Not required for current use cases

---

## Error Handling & Retry Logic

### Connection Errors

**Strategy**: Retry with exponential backoff

```rust
async fn execute_with_retry<T>(&self, query: Query, max_retries: u32) -> Result<T> {
    let mut attempts = 0;
    let mut delay = Duration::from_millis(100);
    
    loop {
        match self.execute_query(query.clone()).await {
            Ok(result) => return Ok(result),
            Err(e) if is_transient_error(&e) && attempts < max_retries => {
                attempts += 1;
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
}

fn is_transient_error(error: &Error) -> bool {
    // Connection timeout, network errors, etc.
    matches!(error.code(), Some("08000") | Some("08003") | Some("08006"))
}
```

**Transient Errors** (retry):
- Connection timeout
- Network failure
- Database temporarily unavailable

**Permanent Errors** (no retry):
- Constraint violations (foreign key, unique)
- Data type mismatches
- Authorization failures

### Graceful Error Propagation

**Pattern**: Return errors to API layer (don't panic)

```rust
pub enum DatastoreError {
    ConnectionError(String),
    QueryError(String),
    NotFound(String),
    ConstraintViolation(String),
    TransactionFailed(String),
}
```

API layer converts DatastoreError to HTTP status codes:
- `NotFound` → 404
- `ConstraintViolation` → 409 Conflict
- `ConnectionError` → 503 Service Unavailable

---

## Concurrent Access & Locking

### PostgreSQL Concurrency Model

**Advantage**: PostgreSQL supports multiple concurrent writers (MVCC)

**Strategy**: **Allow concurrent writes** (leverage PostgreSQL capabilities)

**No Application-Level Locking**: PostgreSQL handles row-level locking automatically

**Concurrent Event Insertion**:
```
Watcher 1 → Insert Event → PostgreSQL (Row Lock)
Watcher 2 → Insert Event → PostgreSQL (Row Lock)
Watcher 3 → Insert Event → PostgreSQL (Row Lock)
```

All insertions proceed concurrently without blocking.

### Conflict Resolution

**Bucket Creation**: Use `ON CONFLICT` clause for idempotency
```sql
INSERT INTO buckets (name, ...) VALUES (...)
ON CONFLICT (name) DO NOTHING;
```

**Event Insertion**: No conflicts (events are immutable, append-only)

---

## Logging & Observability

### SQL Query Logging

**Configuration**: Environment variable `DB_LOG_LEVEL`

**Levels**:
- `OFF` - No SQL logging (production default)
- `ERROR` - Log errors only
- `WARN` - Log slow queries (> 100ms)
- `INFO` - Log all queries (debug/testing)
- `DEBUG` - Log queries + parameters (verbose)

**Implementation**:
```rust
if env::var("DB_LOG_LEVEL").unwrap_or_else(|_| "OFF".to_string()) == "INFO" {
    info!("Executing query: {}", query.sql);
}
```

### Metrics (Future Enhancement)

Track:
- Query execution time (histogram)
- Connection pool utilization
- Transaction success/failure rate
- Database errors (by type)

---

## Schema Versioning & Migrations

### Version Tracking

**Table**: `schema_version` (new table)
```sql
CREATE TABLE schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
```

**Current Version**: 1 (initial PostgreSQL schema)

### Migration Functions

**Pattern**: Similar to SQLite approach
```rust
fn get_db_version(conn: &Connection) -> Result<i32> {
    let row = conn.query_one("SELECT MAX(version) FROM schema_version", &[]).await?;
    Ok(row.get(0))
}

fn apply_migrations(conn: &mut Connection) -> Result<()> {
    let current_version = get_db_version(conn)?;
    
    if current_version < 1 {
        migrate_v0_to_v1(conn)?;
    }
    // Future migrations here
    
    Ok(())
}

fn migrate_v0_to_v1(conn: &mut Connection) -> Result<()> {
    conn.batch_execute("
        CREATE TABLE buckets (...);
        CREATE TABLE events (...);
        CREATE TABLE key_value (...);
        CREATE INDEX idx_events_bucketrow ON events(bucketrow);
        -- etc.
        INSERT INTO schema_version (version) VALUES (1);
    ").await?;
    Ok(())
}
```

**Auto-Migration**: On startup, check version and apply pending migrations.

---

## Performance Characteristics

### Read Performance
- **Time-range queries**: Fast with composite index `idx_events_timerange`
- **Bucket lookup**: Fast with unique index on `name`
- **Full table scans**: Avoided (all queries use indexes)

### Write Performance
- **Single event insert**: ~1-5ms (with connection pool)
- **Batch insert (100 events)**: ~10-50ms (transaction overhead amortized)
- **Concurrent writes**: PostgreSQL handles gracefully (no contention)

### Storage Growth
- **Events table**: Primary growth area (millions of rows over time)
- **Buckets table**: Stable (5-50 rows typically)
- **KeyValue table**: Negligible (< 100 rows)

**Optimization**: Consider partitioning events table by time range (future enhancement for very large datasets).
