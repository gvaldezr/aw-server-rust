# Domain Entities - Unit 1: Database Layer

**Version**: 1.0  
**Date**: 2026-05-18  
**Unit**: Database Layer Migration  

---

## Overview

The ActivityWatch persistence layer manages three core domain entities: Buckets, Events, and KeyValue settings. These entities represent the fundamental data model for activity tracking.

---

## Entity 1: Bucket

**Purpose**: Logical container for activity events from a specific watcher source.

### Attributes

| Attribute | Type | PostgreSQL Type | Constraints | Description |
|-----------|------|----------------|-------------|-------------|
| id | Integer | SERIAL | PRIMARY KEY, AUTO_INCREMENT | Unique bucket identifier |
| name | String | VARCHAR(255) | UNIQUE, NOT NULL | Bucket name (e.g., "aw-watcher-window_hostname") |
| type | String | VARCHAR(100) | NOT NULL | Bucket type (e.g., "currentwindow", "afkstatus") |
| client | String | VARCHAR(100) | NOT NULL | Client application name |
| hostname | String | VARCHAR(255) | NOT NULL | Hostname where watcher is running |
| created | DateTime | TIMESTAMP WITH TIME ZONE | NOT NULL, DEFAULT NOW() | Creation timestamp |
| data | JSON | JSONB | DEFAULT '{}' | Metadata key-value pairs |

### Business Rules

- `name` must be unique across all buckets
- `name` format convention: `{type}_{hostname}` (enforced by application)
- `created` timestamp is immutable once set
- `data` field stores arbitrary metadata as JSON
- Deleting a bucket cascades to all associated events

### SQLite → PostgreSQL Mapping

```sql
-- SQLite Schema
CREATE TABLE buckets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    type TEXT NOT NULL,
    client TEXT NOT NULL,
    hostname TEXT NOT NULL,
    created TEXT NOT NULL,
    data TEXT DEFAULT '{}'
);

-- PostgreSQL Schema
CREATE TABLE buckets (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    type VARCHAR(100) NOT NULL,
    client VARCHAR(100) NOT NULL,
    hostname VARCHAR(255) NOT NULL,
    created TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    data JSONB DEFAULT '{}'::jsonb
);
```

---

## Entity 2: Event

**Purpose**: Single activity record with timestamp, duration, and associated data.

### Attributes

| Attribute | Type | PostgreSQL Type | Constraints | Description |
|-----------|------|----------------|-------------|-------------|
| id | Integer | BIGSERIAL | PRIMARY KEY, AUTO_INCREMENT | Unique event identifier |
| bucketrow | Integer | INTEGER | NOT NULL, FOREIGN KEY → buckets(id) | Reference to parent bucket |
| starttime | DateTime | TIMESTAMP WITH TIME ZONE | NOT NULL | Event start timestamp |
| endtime | DateTime | TIMESTAMP WITH TIME ZONE | NOT NULL | Event end timestamp |
| data | JSON | JSONB | NOT NULL | Event data payload |

### Business Rules

- `bucketrow` must reference a valid bucket (foreign key constraint)
- `endtime` must be >= `starttime` (enforced by application)
- `data` contains event-specific attributes (app name, window title, etc.)
- Events are immutable once created (no updates, only inserts/deletes)
- Deleting a bucket automatically deletes all associated events (CASCADE)

### Indexes

```sql
CREATE INDEX idx_events_bucketrow ON events(bucketrow);
CREATE INDEX idx_events_starttime ON events(starttime);
CREATE INDEX idx_events_endtime ON events(endtime);
CREATE INDEX idx_events_timerange ON events(bucketrow, starttime, endtime);
```

**Index Rationale**:
- `bucketrow`: Filter events by bucket (frequent query)
- `starttime`, `endtime`: Time-range queries (most common query pattern)
- Composite index on `(bucketrow, starttime, endtime)`: Optimize bucket-specific time-range queries

### SQLite → PostgreSQL Mapping

```sql
-- SQLite Schema
CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    bucketrow INTEGER NOT NULL,
    starttime INTEGER NOT NULL,  -- Unix timestamp (milliseconds)
    endtime INTEGER NOT NULL,
    data TEXT NOT NULL,
    FOREIGN KEY (bucketrow) REFERENCES buckets(id)
);

-- PostgreSQL Schema
CREATE TABLE events (
    id BIGSERIAL PRIMARY KEY,
    bucketrow INTEGER NOT NULL,
    starttime TIMESTAMP WITH TIME ZONE NOT NULL,
    endtime TIMESTAMP WITH TIME ZONE NOT NULL,
    data JSONB NOT NULL,
    FOREIGN KEY (bucketrow) REFERENCES buckets(id) ON DELETE CASCADE
);
```

**Critical Change**: SQLite stores timestamps as INTEGER (Unix epoch milliseconds), PostgreSQL uses TIMESTAMP WITH TIME ZONE. Conversion required in application layer.

---

## Entity 3: KeyValue

**Purpose**: Store configuration settings and persistent key-value pairs.

### Attributes

| Attribute | Type | PostgreSQL Type | Constraints | Description |
|-----------|------|----------------|-------------|-------------|
| key | String | VARCHAR(255) | PRIMARY KEY | Setting key (unique identifier) |
| value | String | TEXT | NOT NULL | Setting value (arbitrary string data) |

### Business Rules

- `key` must be unique (primary key)
- Values are stored as TEXT (application parses as needed)
- Used for server settings, feature flags, metadata

### SQLite → PostgreSQL Mapping

```sql
-- SQLite Schema
CREATE TABLE key_value (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- PostgreSQL Schema
CREATE TABLE key_value (
    key VARCHAR(255) PRIMARY KEY,
    value TEXT NOT NULL
);
```

---

## Entity Relationships

```
┌──────────────────┐
│     Bucket       │
│  (id, name, ...) │
└────────┬─────────┘
         │ 1
         │
         │ *
┌────────▼─────────┐
│      Event       │
│ (id, bucketrow,  │
│  starttime, ...)  │
└──────────────────┘

┌──────────────────┐
│    KeyValue      │
│  (key, value)    │
└──────────────────┘
```

**Relationship**: One-to-Many (Bucket → Events)
- One bucket contains zero or many events
- Each event belongs to exactly one bucket
- Cascade delete: Deleting bucket removes all events

---

## Type Conversions

### Timestamp Handling

**SQLite**: Stores as INTEGER (Unix epoch milliseconds)
```rust
// SQLite storage
let timestamp_ms: i64 = chrono::Utc::now().timestamp_millis();
```

**PostgreSQL**: Native TIMESTAMP WITH TIME ZONE
```rust
// PostgreSQL storage
let timestamp: DateTime<Utc> = chrono::Utc::now();
```

**Conversion Strategy**: 
- Application layer converts `chrono::DateTime<Utc>` to/from PostgreSQL TIMESTAMP
- Use `tokio-postgres` or `sqlx` type conversion (automatic with chrono feature)

### JSON Handling

**SQLite**: TEXT column with JSON string
```rust
let json_str = serde_json::to_string(&data)?;
```

**PostgreSQL**: JSONB native type (binary, indexed, queryable)
```rust
let json_value = serde_json::to_value(&data)?;
// tokio-postgres/sqlx handles JSONB conversion automatically
```

**Benefits of JSONB**:
- Binary storage (faster)
- Indexable (GIN indexes for JSON queries)
- Queryable with PostgreSQL JSON operators
- Validates JSON structure on insert

---

## Data Integrity Rules

1. **Referential Integrity**: Events must reference valid buckets (enforced by foreign key)
2. **Cascade Deletes**: Deleting a bucket deletes all events (ON DELETE CASCADE)
3. **Immutability**: Events are append-only (no UPDATE operations)
4. **Time Validity**: `endtime >= starttime` (enforced in application)
5. **JSON Validation**: Data fields must be valid JSON (PostgreSQL validates JSONB)

---

## Migration Considerations

### Data Volume Estimates
- **Buckets**: Typically 5-50 per installation (low volume)
- **Events**: Can reach millions over time (high volume, time-series data)
- **KeyValue**: < 100 entries (low volume)

### Performance Impact
- **Read-heavy workload**: Queries for historical events (time-range queries)
- **Write pattern**: Continuous event insertion (one event per watcher heartbeat)
- **Indexes critical**: Time-range queries must be fast (user experience)

### Schema Evolution
- Current schema version: 4 (SQLite)
- PostgreSQL schema version: 1 (fresh start, no legacy migration)
- Future migrations managed via version number + migration functions
