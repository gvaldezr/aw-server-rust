# Schema Mapping - SQLite to PostgreSQL

**Version**: 1.0  
**Date**: 2026-05-18  
**Unit**: Database Layer Migration  

---

## Overview

This document provides detailed type mappings, schema transformations, and migration strategies for converting the ActivityWatch SQLite schema to PostgreSQL.

---

## Table 1: Buckets

### SQLite Schema (Current)
```sql
CREATE TABLE buckets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    type TEXT NOT NULL,
    client TEXT NOT NULL,
    hostname TEXT NOT NULL,
    created TEXT NOT NULL,
    data TEXT DEFAULT '{}'
);

CREATE INDEX bucket_id_index ON buckets(id);
```

### PostgreSQL Schema (Target)
```sql
CREATE TABLE buckets (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    type VARCHAR(100) NOT NULL,
    client VARCHAR(100) NOT NULL,
    hostname VARCHAR(255) NOT NULL,
    created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    data JSONB DEFAULT '{}'::jsonb
);

-- Note: SERIAL automatically creates index on primary key
-- No need for explicit bucket_id_index
```

### Type Mapping

| Column | SQLite Type | PostgreSQL Type | Notes |
|--------|-------------|-----------------|-------|
| id | INTEGER AUTOINCREMENT | SERIAL | Auto-incrementing integer (1, 2, 3...) |
| name | TEXT | VARCHAR(255) | Limited length for bucket names |
| type | TEXT | VARCHAR(100) | Bucket type identifier |
| client | TEXT | VARCHAR(100) | Client application name |
| hostname | TEXT | VARCHAR(255) | Hostname identifier |
| created | TEXT (ISO 8601) | TIMESTAMP WITH TIME ZONE | Native timestamp type |
| data | TEXT (JSON string) | JSONB | Binary JSON with indexing support |

### Key Changes

1. **id column**: `INTEGER PRIMARY KEY AUTOINCREMENT` → `SERIAL PRIMARY KEY`
   - Both auto-increment, equivalent behavior
   - PostgreSQL SERIAL creates sequence automatically

2. **TEXT → VARCHAR**: 
   - SQLite TEXT has no length limit
   - PostgreSQL VARCHAR with explicit lengths for better validation
   - Lengths chosen based on typical values (255 for names, 100 for types)

3. **created column**: `TEXT` → `TIMESTAMP WITH TIME ZONE`
   - SQLite stores ISO 8601 strings: `"2026-05-18T10:30:00.000000Z"`
   - PostgreSQL native timestamp with timezone awareness
   - **Migration**: Parse ISO 8601 string → DateTime<Utc>

4. **data column**: `TEXT` → `JSONB`
   - SQLite stores JSON as TEXT string
   - PostgreSQL JSONB: Binary format, indexable, queryable
   - **Benefits**: Faster queries, GIN indexing, JSON operators

---

## Table 2: Events

### SQLite Schema (Current)
```sql
CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bucketrow INTEGER NOT NULL,
    starttime INTEGER NOT NULL,
    endtime INTEGER NOT NULL,
    data TEXT NOT NULL,
    FOREIGN KEY (bucketrow) REFERENCES buckets(id)
);

CREATE INDEX events_bucketrow_index ON events(bucketrow);
```

### PostgreSQL Schema (Target)
```sql
CREATE TABLE events (
    id BIGSERIAL PRIMARY KEY,
    bucketrow INTEGER NOT NULL,
    starttime TIMESTAMP WITH TIME ZONE NOT NULL,
    endtime TIMESTAMP WITH TIME ZONE NOT NULL,
    data JSONB NOT NULL,
    FOREIGN KEY (bucketrow) REFERENCES buckets(id) ON DELETE CASCADE
);

CREATE INDEX idx_events_bucketrow ON events(bucketrow);
CREATE INDEX idx_events_starttime ON events(starttime);
CREATE INDEX idx_events_endtime ON events(endtime);
CREATE INDEX idx_events_timerange ON events(bucketrow, starttime, endtime);
```

### Type Mapping

| Column | SQLite Type | PostgreSQL Type | Notes |
|--------|-------------|-----------------|-------|
| id | INTEGER AUTOINCREMENT | BIGSERIAL | 64-bit for large event volumes |
| bucketrow | INTEGER | INTEGER | Foreign key to buckets(id) |
| starttime | INTEGER (Unix ms) | TIMESTAMP WITH TIME ZONE | Native timestamp |
| endtime | INTEGER (Unix ms) | TIMESTAMP WITH TIME ZONE | Native timestamp |
| data | TEXT (JSON string) | JSONB | Binary JSON |

### Key Changes

1. **id column**: `INTEGER` → `BIGSERIAL`
   - **Rationale**: Events table grows to millions of rows
   - INTEGER max: 2.1 billion (~2^31)
   - BIGSERIAL max: 9.2 quintillion (~2^63)
   - Future-proof for high-volume installations

2. **starttime/endtime**: `INTEGER` → `TIMESTAMP WITH TIME ZONE`
   - **SQLite format**: Unix epoch milliseconds (e.g., `1716028200000`)
   - **PostgreSQL format**: Native timestamp with timezone
   - **Conversion**:
     ```rust
     // SQLite → Application
     let timestamp = DateTime::<Utc>::from_timestamp_millis(sqlite_value)?;
     
     // Application → PostgreSQL
     // tokio-postgres/sqlx handles DateTime<Utc> automatically
     ```

3. **Foreign Key Constraint**: Added `ON DELETE CASCADE`
   - **Behavior**: Deleting bucket automatically deletes all events
   - **SQLite**: Foreign keys not enforced by default
   - **PostgreSQL**: Foreign keys enforced by default

4. **Additional Indexes**:
   - `idx_events_starttime`: Time-range queries (start)
   - `idx_events_endtime`: Time-range queries (end)
   - `idx_events_timerange`: Composite index for bucket+time queries
   - **Rationale**: Most common query pattern is "get events in time range for bucket"

---

## Table 3: Key-Value

### SQLite Schema (Current)
```sql
CREATE TABLE key_value (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

### PostgreSQL Schema (Target)
```sql
CREATE TABLE key_value (
    key VARCHAR(255) PRIMARY KEY,
    value TEXT NOT NULL
);
```

### Type Mapping

| Column | SQLite Type | PostgreSQL Type | Notes |
|--------|-------------|-----------------|-------|
| key | TEXT | VARCHAR(255) | Limited length for keys |
| value | TEXT | TEXT | Unlimited length for values |

### Key Changes

1. **key column**: `TEXT` → `VARCHAR(255)`
   - Keys are identifiers (short strings)
   - 255 character limit is reasonable
   - Helps prevent accidental very long keys

2. **value column**: `TEXT` → `TEXT`
   - No change, TEXT to TEXT
   - PostgreSQL TEXT has no length limit (like SQLite)

---

## Timestamp Conversion Strategy

### SQLite Representation
```rust
// SQLite stores timestamps as INTEGER (milliseconds since Unix epoch)
let sqlite_timestamp: i64 = 1716028200000; // May 18, 2026 10:30:00 UTC
```

### PostgreSQL Representation
```rust
// PostgreSQL stores native TIMESTAMP WITH TIME ZONE
use chrono::{DateTime, Utc};
let pg_timestamp: DateTime<Utc> = DateTime::from_timestamp_millis(1716028200000)?;
```

### Conversion Functions

#### Read from Database
```rust
// tokio-postgres Row → DateTime<Utc>
let starttime: DateTime<Utc> = row.get("starttime");
// Automatic conversion, no manual parsing needed
```

#### Write to Database
```rust
// DateTime<Utc> → PostgreSQL TIMESTAMP
conn.execute(
    "INSERT INTO events (starttime, endtime, ...) VALUES ($1, $2, ...)",
    &[&event.starttime, &event.endtime, ...]
).await?;
// Automatic conversion, no manual formatting needed
```

#### API Layer (JSON Serialization)
```rust
// DateTime<Utc> ↔ ISO 8601 string (for JSON API)
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Event {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    starttime: DateTime<Utc>, // Serializes to milliseconds for API compatibility
    
    #[serde(with = "chrono::serde::ts_milliseconds")]
    endtime: DateTime<Utc>,
}
```

---

## JSON Handling: TEXT vs JSONB

### SQLite Approach (Current)
```rust
// Store as TEXT string
let json_str = serde_json::to_string(&data)?;
conn.execute("INSERT INTO events (data) VALUES (?)", &[&json_str])?;

// Read as TEXT, parse manually
let json_str: String = row.get("data")?;
let data: serde_json::Value = serde_json::from_str(&json_str)?;
```

### PostgreSQL Approach (New)
```rust
// Store as serde_json::Value (tokio-postgres handles JSONB conversion)
use serde_json::Value;
let data: Value = json!({"app": "chrome", "title": "GitHub"});
conn.execute("INSERT INTO events (data) VALUES ($1)", &[&data]).await?;

// Read as serde_json::Value (automatic deserialization)
let data: Value = row.get("data");
```

### JSONB Benefits
- **Storage**: Binary format (smaller, faster)
- **Querying**: JSON operators (`->`, `->>`, `@>`, `?`)
  ```sql
  -- Query events where app = "chrome"
  SELECT * FROM events WHERE data->>'app' = 'chrome';
  ```
- **Indexing**: GIN indexes on JSONB columns
  ```sql
  CREATE INDEX idx_events_data_gin ON events USING GIN (data);
  ```
- **Validation**: PostgreSQL validates JSON structure on insert

---

## Index Strategy

### SQLite Indexes (Current)
```sql
CREATE INDEX bucket_id_index ON buckets(id);
CREATE INDEX events_bucketrow_index ON events(bucketrow);
```

### PostgreSQL Indexes (Enhanced)
```sql
-- Buckets table
-- No explicit index needed - SERIAL PRIMARY KEY auto-creates index

-- Events table
CREATE INDEX idx_events_bucketrow ON events(bucketrow);
CREATE INDEX idx_events_starttime ON events(starttime);
CREATE INDEX idx_events_endtime ON events(endtime);
CREATE INDEX idx_events_timerange ON events(bucketrow, starttime, endtime);

-- Optional: GIN index for JSONB queries (if needed)
CREATE INDEX idx_events_data_gin ON events USING GIN (data);
```

### Index Rationale

1. **idx_events_bucketrow**: Filter by bucket (frequent)
2. **idx_events_starttime**: Time-range queries (start bound)
3. **idx_events_endtime**: Time-range queries (end bound)
4. **idx_events_timerange**: Composite index for most common query:
   ```sql
   SELECT * FROM events 
   WHERE bucketrow = ? AND starttime >= ? AND endtime <= ?
   ORDER BY starttime;
   ```
5. **idx_events_data_gin** (optional): JSON queries (if querying event data by specific fields)

---

## Foreign Key Constraints

### SQLite (Current)
```sql
-- Foreign key defined but NOT enforced by default
FOREIGN KEY (bucketrow) REFERENCES buckets(id)
```

**SQLite behavior**: Foreign keys must be explicitly enabled:
```sql
PRAGMA foreign_keys = ON;
```

### PostgreSQL (Target)
```sql
-- Foreign key enforced by default
FOREIGN KEY (bucketrow) REFERENCES buckets(id) ON DELETE CASCADE
```

**PostgreSQL behavior**:
- **ON DELETE CASCADE**: Deleting bucket deletes all events
- **Enforced automatically**: No need to enable
- **Referential integrity**: Insert fails if bucket doesn't exist

---

## Data Type Size Comparison

| Type | SQLite Storage | PostgreSQL Storage | Notes |
|------|---------------|-------------------|-------|
| INTEGER | Variable (1-8 bytes) | 4 bytes (INT) or 8 bytes (BIGINT) | Fixed size in PG |
| TEXT | Variable | Variable | Similar behavior |
| BLOB | Variable | BYTEA (variable) | Equivalent |
| REAL | 8 bytes | 8 bytes (DOUBLE PRECISION) | Equivalent |
| TIMESTAMP | N/A (stored as TEXT/INTEGER) | 8 bytes + timezone | Native type in PG |
| JSON | TEXT (variable) | JSONB (~same size, compressed) | JSONB is binary |

---

## Migration Script Template

```sql
-- PostgreSQL schema creation (v1)
BEGIN;

-- Create tables
CREATE TABLE buckets (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    type VARCHAR(100) NOT NULL,
    client VARCHAR(100) NOT NULL,
    hostname VARCHAR(255) NOT NULL,
    created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    data JSONB DEFAULT '{}'::jsonb
);

CREATE TABLE events (
    id BIGSERIAL PRIMARY KEY,
    bucketrow INTEGER NOT NULL,
    starttime TIMESTAMP WITH TIME ZONE NOT NULL,
    endtime TIMESTAMP WITH TIME ZONE NOT NULL,
    data JSONB NOT NULL,
    FOREIGN KEY (bucketrow) REFERENCES buckets(id) ON DELETE CASCADE
);

CREATE TABLE key_value (
    key VARCHAR(255) PRIMARY KEY,
    value TEXT NOT NULL
);

-- Create indexes
CREATE INDEX idx_events_bucketrow ON events(bucketrow);
CREATE INDEX idx_events_starttime ON events(starttime);
CREATE INDEX idx_events_endtime ON events(endtime);
CREATE INDEX idx_events_timerange ON events(bucketrow, starttime, endtime);

-- Schema version tracking
CREATE TABLE schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

INSERT INTO schema_version (version) VALUES (1);

COMMIT;
```

---

## Compatibility Matrix

| Feature | SQLite | PostgreSQL | Compatible? |
|---------|--------|-----------|-------------|
| Auto-increment IDs | INTEGER AUTOINCREMENT | SERIAL/BIGSERIAL | ✅ Yes |
| JSON storage | TEXT | JSONB | ✅ Yes (with conversion) |
| Timestamps | INTEGER/TEXT | TIMESTAMP WITH TIME ZONE | ✅ Yes (with conversion) |
| Foreign keys | Defined but not enforced | Enforced by default | ✅ Yes (behavior change) |
| Transactions | Supported | Supported | ✅ Yes |
| Concurrent writes | Single writer | Multiple writers | ⚠️ Improved (more concurrent) |
| Indexes | B-tree | B-tree, GIN, etc. | ✅ Yes (enhanced) |

---

## Performance Implications

### Read Performance
- **Time-range queries**: Faster with composite index
- **JSON queries**: Much faster with JSONB + GIN index
- **Large result sets**: Similar (limited by network)

### Write Performance
- **Single insert**: Comparable (~1-5ms)
- **Batch insert**: Faster with explicit transactions
- **Concurrent inserts**: Much better (PostgreSQL MVCC)

### Storage
- **Disk space**: JSONB may use slightly more space than TEXT
- **Compression**: PostgreSQL TOAST compression helps
- **Indexes**: More indexes = more space (trade-off for speed)

---

## Rollback Strategy (If Needed)

If PostgreSQL migration fails and rollback to SQLite is required:

1. **Schema**: SQLite schema remains unchanged
2. **Data**: No legacy data migration (starting fresh)
3. **Code**: Revert to rusqlite dependency
4. **Config**: Switch database connection string

**Note**: Since no legacy migration is required (starting fresh), rollback is straightforward - just revert code changes.
