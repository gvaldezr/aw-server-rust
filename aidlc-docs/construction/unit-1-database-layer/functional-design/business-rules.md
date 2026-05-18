# Business Rules - Unit 1: Database Layer

**Version**: 1.0  
**Date**: 2026-05-18  
**Unit**: Database Layer Migration  

---

## Overview

This document defines business rules, validation logic, constraints, and data integrity rules for the ActivityWatch database layer.

---

## BR1: Bucket Naming Rules

### Rule Definition
Bucket names must follow the convention: `{bucket_type}_{hostname}`

### Validation Logic
```rust
fn validate_bucket_name(name: &str, bucket_type: &str, hostname: &str) -> Result<()> {
    let expected_name = format!("{}_{}", bucket_type, hostname);
    if name != expected_name {
        return Err(ValidationError::InvalidBucketName {
            provided: name.to_string(),
            expected: expected_name,
        });
    }
    Ok(())
}
```

### Enforcement
- **Level**: Application-level (soft constraint, not DB constraint)
- **Rationale**: Legacy compatibility - existing clients follow convention

### Examples
- ✅ Valid: `aw-watcher-window_hostname123`
- ✅ Valid: `currentwindow_mycomputer`
- ❌ Invalid: `bucket-name-without-hostname`

---

## BR2: Bucket Name Uniqueness

### Rule Definition
Each bucket name must be globally unique across the database.

### Database Constraint
```sql
ALTER TABLE buckets ADD CONSTRAINT buckets_name_unique UNIQUE (name);
```

### Behavior
- **Insert**: If bucket exists, return existing bucket (idempotent)
- **Update**: Not applicable (buckets are immutable)
- **Delete**: Cascade delete to events

### Error Handling
```rust
match db.insert_bucket(bucket).await {
    Ok(bucket_id) => Ok(bucket_id),
    Err(DatastoreError::ConstraintViolation(_)) => {
        // Bucket already exists, fetch and return
        db.get_bucket_by_name(&bucket.name).await
    }
    Err(e) => Err(e),
}
```

---

## BR3: Event Timestamp Validity

### Rule Definition
For every event: `endtime >= starttime`

### Validation Logic
```rust
fn validate_event_timestamps(event: &Event) -> Result<()> {
    if event.endtime < event.starttime {
        return Err(ValidationError::InvalidTimeRange {
            start: event.starttime,
            end: event.endtime,
        });
    }
    
    // Optional: Warn if duration is unreasonably long (e.g., > 24 hours)
    let duration = event.endtime - event.starttime;
    if duration > Duration::hours(24) {
        warn!("Event duration exceeds 24 hours: {:?}", duration);
    }
    
    Ok(())
}
```

### Enforcement
- **Level**: Application-level (validated before insert)
- **Not enforced in DB**: PostgreSQL CHECK constraint could be added, but not required

### Edge Cases
- **Zero duration** (`starttime == endtime`): Valid (instant event)
- **Negative duration**: Invalid (rejected)

---

## BR4: Event Immutability

### Rule Definition
Events are **append-only**. Once created, events cannot be updated.

### Implementation
- No UPDATE operations on events table
- Only INSERT and DELETE allowed

### Rationale
- Historical data integrity
- Audit trail preservation
- Simplifies concurrency (no update conflicts)

### API Behavior
- `POST /api/0/buckets/{id}/events` - Create event ✅
- `PUT /api/0/buckets/{id}/events/{event_id}` - Not implemented ❌
- `DELETE /api/0/buckets/{id}/events/{event_id}` - Delete event ✅

---

## BR5: Referential Integrity (Bucket-Event Relationship)

### Rule Definition
Every event must reference a valid bucket. Deleting a bucket deletes all associated events.

### Database Constraint
```sql
ALTER TABLE events 
ADD CONSTRAINT fk_events_bucketrow 
FOREIGN KEY (bucketrow) REFERENCES buckets(id) 
ON DELETE CASCADE;
```

### Behavior
- **Insert Event**: Fails if bucket doesn't exist (foreign key violation)
- **Delete Bucket**: All events automatically deleted (cascade)
- **Delete Event**: Allowed (bucket remains)

### Error Handling
```rust
match db.insert_event(event).await {
    Ok(event_id) => Ok(event_id),
    Err(DatastoreError::ForeignKeyViolation(_)) => {
        Err(ApiError::BucketNotFound {
            bucket_id: event.bucket_name.clone(),
        })
    }
    Err(e) => Err(e),
}
```

---

## BR6: JSON Data Validation

### Rule Definition
Event data and bucket data fields must contain valid JSON.

### PostgreSQL Enforcement
- **Type**: JSONB (automatically validates JSON structure)
- **Invalid JSON**: Insert fails with parse error

### Application-Level Validation
```rust
fn validate_event_data(data: &serde_json::Value) -> Result<()> {
    // Ensure data is an object (not array, string, etc.)
    if !data.is_object() {
        return Err(ValidationError::InvalidEventData {
            reason: "Event data must be a JSON object".to_string(),
        });
    }
    
    // Optional: Validate required fields for specific bucket types
    // (e.g., "app" field for window watchers)
    
    Ok(())
}
```

### Examples
- ✅ Valid: `{"app": "chrome", "title": "GitHub"}`
- ✅ Valid: `{}`
- ❌ Invalid: `"string"` (not an object)
- ❌ Invalid: `[1, 2, 3]` (not an object)

---

## BR7: Bucket Metadata Requirements

### Rule Definition
Buckets must have non-empty `type`, `client`, and `hostname` fields.

### Validation Logic
```rust
fn validate_bucket_metadata(bucket: &Bucket) -> Result<()> {
    if bucket.bucket_type.is_empty() {
        return Err(ValidationError::MissingField("type"));
    }
    if bucket.client.is_empty() {
        return Err(ValidationError::MissingField("client"));
    }
    if bucket.hostname.is_empty() {
        return Err(ValidationError::MissingField("hostname"));
    }
    Ok(())
}
```

### Enforcement
- **Level**: Application-level (validated before insert)
- **Database**: NOT NULL constraints on columns

### Database Constraints
```sql
ALTER TABLE buckets ALTER COLUMN type SET NOT NULL;
ALTER TABLE buckets ALTER COLUMN client SET NOT NULL;
ALTER TABLE buckets ALTER COLUMN hostname SET NOT NULL;
```

---

## BR8: Timestamp Timezone Handling

### Rule Definition
All timestamps must be stored in UTC (TIMESTAMP WITH TIME ZONE).

### Implementation
```rust
use chrono::{DateTime, Utc};

fn ensure_utc_timestamp(dt: DateTime<Utc>) -> DateTime<Utc> {
    dt // Already UTC, no conversion needed
}

fn parse_api_timestamp(timestamp_str: &str) -> Result<DateTime<Utc>> {
    // Parse ISO 8601 format, convert to UTC if timezone provided
    DateTime::parse_from_rfc3339(timestamp_str)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| ValidationError::InvalidTimestamp)
}
```

### API Contract
- **Input**: Accept ISO 8601 with timezone (e.g., `2026-05-18T10:30:00Z`)
- **Output**: Always return UTC timestamps
- **Storage**: PostgreSQL TIMESTAMP WITH TIME ZONE (stores UTC internally)

---

## BR9: Bucket Data Field Default

### Rule Definition
If bucket `data` field is not provided, default to empty JSON object `{}`.

### Database Default
```sql
ALTER TABLE buckets ALTER COLUMN data SET DEFAULT '{}'::jsonb;
```

### Application Behavior
```rust
let bucket_data = bucket.data.unwrap_or_else(|| json!({}));
```

---

## BR10: Connection Pool Limits

### Rule Definition
Database connection pool must respect configured limits to prevent resource exhaustion.

### Configuration
```rust
struct PoolConfig {
    max_size: usize,        // Maximum connections (default: 10)
    min_idle: usize,        // Minimum idle connections (default: 2)
    connection_timeout: Duration, // Timeout acquiring connection (default: 5s)
}
```

### Behavior
- **Max connections reached**: New requests wait up to `connection_timeout`
- **Timeout exceeded**: Return error to API layer (503 Service Unavailable)
- **Idle connections**: Closed after idle timeout (default: 10 minutes)

### Monitoring
Log warnings when:
- Connection pool utilization exceeds 80%
- Connection acquisition takes > 1 second
- Connections are timing out

---

## BR11: Transaction Atomicity for Batch Operations

### Rule Definition
Batch event insertions must be atomic (all succeed or all fail).

### Implementation
```rust
async fn insert_events_batch(&self, events: Vec<Event>) -> Result<Vec<i64>> {
    let mut conn = self.pool.get().await?;
    let tx = conn.transaction().await?;
    
    let mut event_ids = Vec::new();
    for event in events {
        let row = tx.query_one(
            "INSERT INTO events (bucketrow, starttime, endtime, data) VALUES ($1, $2, $3, $4) RETURNING id",
            &[&event.bucketrow, &event.starttime, &event.endtime, &event.data]
        ).await?;
        event_ids.push(row.get(0));
    }
    
    tx.commit().await?;
    Ok(event_ids)
}
```

### Error Handling
- **Any insert fails**: Rollback entire batch (no partial inserts)
- **Return error**: API layer returns 400/500 with error details

---

## BR12: Schema Version Consistency

### Rule Definition
Database schema version must match expected version before executing queries.

### Validation
```rust
async fn validate_schema_version(&self) -> Result<()> {
    let db_version = self.get_schema_version().await?;
    
    if db_version < REQUIRED_MIN_VERSION {
        return Err(DatastoreError::OldDbVersion {
            current: db_version,
            required: REQUIRED_MIN_VERSION,
        });
    }
    
    if db_version > KNOWN_MAX_VERSION {
        warn!("Database schema version {} is newer than known version {}", 
              db_version, KNOWN_MAX_VERSION);
    }
    
    Ok(())
}
```

### Behavior
- **Version too old**: Fail startup, require migration
- **Version too new**: Warn but continue (forward compatibility)

---

## BR13: Idempotent Bucket Creation

### Rule Definition
Creating a bucket with an existing name returns the existing bucket (no error).

### Implementation
```sql
INSERT INTO buckets (name, type, client, hostname, created, data)
VALUES ($1, $2, $3, $4, NOW(), $5)
ON CONFLICT (name) DO NOTHING
RETURNING id, name, type, client, hostname, created, data;
```

### Rationale
- Watchers may attempt to recreate buckets on restart
- API should be idempotent (safe to call multiple times)

---

## BR14: Event Query Pagination

### Rule Definition
Event queries must support pagination to prevent memory exhaustion on large result sets.

### Implementation
```rust
async fn get_events(&self, bucket_id: i32, start: DateTime<Utc>, end: DateTime<Utc>, limit: Option<i64>) -> Result<Vec<Event>> {
    let limit = limit.unwrap_or(1000); // Default limit: 1000 events
    let max_limit = 10000; // Maximum allowed limit
    
    let effective_limit = limit.min(max_limit);
    
    let rows = self.conn.query(
        "SELECT id, bucketrow, starttime, endtime, data 
         FROM events 
         WHERE bucketrow = $1 AND starttime >= $2 AND endtime <= $3 
         ORDER BY starttime ASC 
         LIMIT $4",
        &[&bucket_id, &start, &end, &effective_limit]
    ).await?;
    
    Ok(rows.iter().map(|row| Event::from_row(row)).collect())
}
```

### Limits
- **Default limit**: 1000 events
- **Maximum limit**: 10,000 events
- **No limit specified**: Apply default

---

## BR15: Concurrent Write Safety

### Rule Definition
Multiple concurrent event insertions must not cause data corruption or constraint violations.

### PostgreSQL Guarantees
- **MVCC (Multi-Version Concurrency Control)**: PostgreSQL handles concurrent writes automatically
- **Row-level locking**: Prevents conflicts on same row
- **Serializable transactions**: Optional for stricter isolation

### Application Implementation
- **No application-level locking required**
- **Trust PostgreSQL concurrency model**
- **Use connection pool**: Each request gets isolated connection

---

## Data Integrity Checklist

✅ **Foreign Keys**: Events reference valid buckets (ON DELETE CASCADE)  
✅ **Unique Constraints**: Bucket names are unique  
✅ **NOT NULL Constraints**: Required fields cannot be null  
✅ **JSON Validation**: JSONB type validates JSON structure  
✅ **Timestamp Validity**: Application validates endtime >= starttime  
✅ **Immutability**: No UPDATE operations on events  
✅ **Atomicity**: Batch operations wrapped in transactions  
✅ **Idempotency**: Duplicate bucket creation handled gracefully  
✅ **Connection Pooling**: Resource limits enforced  
✅ **Schema Versioning**: Version checks before operations
