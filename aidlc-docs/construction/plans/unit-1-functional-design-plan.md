# Functional Design Plan - Unit 1: Database Layer Migration

**Unit**: Unit 1 - Database Layer (aw-datastore PostgreSQL refactoring)  
**Status**: Planning Phase  

---

## Design Objectives

Transform the SQLite-based persistence layer (rusqlite) to PostgreSQL while maintaining identical API interfaces and business logic.

---

## Functional Design Checkpoints

Please complete each [Answer]: tag with your decision or confirmation:

### CP1: Data Type Mapping Strategy

**Question**: SQLite has loose typing, PostgreSQL is strongly-typed. How should we handle data type mapping?

**Context**: 
- SQLite: TEXT, INTEGER, BLOB storage classes
- PostgreSQL: VARCHAR, BIGINT, BYTEA, UUID, TIMESTAMP types

**Options**:
- A) Strict mapping (TEXT→VARCHAR with constraints, INTEGER→BIGINT, ensure all types match PostgreSQL strictly)
- B) Flexible mapping (TEXT→TEXT in PostgreSQL, allow implicit conversions, simpler but less safe)
- C) Custom mapping per field (analyze each column and decide individually)

[Answer]: 

---

### CP2: Schema Versioning & Migrations

**Question**: How should we manage schema versioning and handle future migrations?

**Current State**: SQLite uses `user_version` pragma (v0-v4 currently)

**Options**:
- A) Continue using schema version number + migration functions (parallel SQLite approach)
- B) Use flyway/liquibase-style versioned SQL migration files
- C) Use Rust migration framework (sqlx migrations or diesel)
- D) Manual migrations with git history tracking

[Answer]: 

---

### CP3: Schema Initialization & First-Run Setup

**Question**: When aw-server connects to PostgreSQL for the first time, what should happen?

**Options**:
- A) Auto-create all tables if they don't exist (like current SQLite behavior)
- B) Require DBA to run migration scripts manually
- C) Provide migration tool that must be run separately
- D) Both auto-create and provide manual option

[Answer]: 

---

### CP4: Foreign Keys & Constraints

**Question**: How strictly should we enforce referential integrity?

**Current SQLite**: Foreign keys disabled by default

**PostgreSQL Options**:
- A) Enforce foreign keys strictly (ON DELETE RESTRICT) - fail if referential integrity violated
- B) Enforce with cascading deletes (ON DELETE CASCADE) - cascade deletes to events when bucket is deleted
- C) Soft referential checks in application code (less strict)

[Answer]: 

---

### CP5: Connection Pooling Strategy

**Question**: How should we manage database connections in the worker thread?

**Current SQLite**: Single connection per database (rusqlite)

**Options**:
- A) Single connection (maintain current pattern for simplicity)
- B) Connection pool (deadpool or bb8) with configurable pool size
- C) Hybrid (pool in production, single connection in testing)

[Answer]: 

---

### CP6: Transaction Handling

**Question**: How should transactions be handled?

**Current Approach**: SQLite uses explicit transactions in worker.rs (IMMEDIATE)

**PostgreSQL Options**:
- A) Mirror current approach (explicit transactions with isolation levels)
- B) Implicit transactions (autocommit mode, no explicit BEGIN/COMMIT)
- C) Async transactions with tokio (if using tokio-postgres)

[Answer]: 

---

### CP7: Indexes & Query Optimization

**Question**: What indexes are critical for performance?

**Current SQLite Indexes**:
- bucket_id_index on buckets(id)
- events_bucketrow_index on events(bucketrow)

**Options**:
- A) Keep existing indexes, add additional indexes for timestamp ranges (common queries)
- B) Minimal indexing (only primary/foreign key constraints)
- C) Comprehensive indexing (include columns for WHERE/JOIN clauses)

[Answer]: 

---

### CP8: Concurrent Access & Locking

**Question**: How should we handle concurrent access and locking?

**SQLite Limitation**: Single writer, multiple readers

**PostgreSQL Capability**: Multiple concurrent writers

**Options**:
- A) Maintain single-writer pattern (keep current behavior for compatibility)
- B) Allow concurrent writes (leverage PostgreSQL capabilities)
- C) Configurable per environment (single-writer in testing, concurrent in production)

[Answer]: 

---

### CP9: Error Handling & Connection Failures

**Question**: How should we handle database connection failures?

**Current SQLite**: panic on connection errors

**Options**:
- A) Panic on errors (maintain current behavior)
- B) Retry logic with exponential backoff
- C) Graceful degradation (return errors to API layer)
- D) Combination (retry internally, graceful error to API if retries exhausted)

[Answer]: 

---

### CP10: Logging & Debugging

**Question**: What level of SQL query logging is needed?

**Options**:
- A) No query logging (production only)
- B) Log slow queries (> 100ms)
- C) Log all queries (debug/testing only)
- D) Configurable via environment variable

[Answer]: 

---

## Expected Outputs

Once these questions are answered, I will generate:

1. **domain-entities.md** - Domain model (Bucket, Event, KeyValue entities)
2. **business-logic-model.md** - Persistence layer logic and workflows
3. **business-rules.md** - Validation rules, constraints, data integrity rules
4. **schema-mapping.md** - Detailed SQLite → PostgreSQL type mapping

---

## Instructions

Please complete all [Answer]: tags above with your selection (A, B, C, D) or custom response. Once completed, respond with "LISTO" and I'll generate the detailed functional design.
