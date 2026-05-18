# Code Structure

## Build System

- **Type**: Cargo (Rust package manager)
- **Workspace**: Monorepo with 7 member crates
- **Build Profile**: Release for optimized builds
- **Key Build Files**:
  - `Cargo.toml` (workspace root)
  - `Cargo.lock` (dependency lock)
  - `.github/workflows/` (CI/CD pipelines)

## Project Structure

```
aw-server-rust/
├── aw-server/              # Main HTTP server application
│   ├── src/
│   │   ├── main.rs        # Entry point, CLI argument parsing
│   │   ├── lib.rs         # Library exports
│   │   ├── config.rs      # Configuration management
│   │   ├── dirs.rs        # Directory paths
│   │   ├── logging.rs     # Logger setup
│   │   ├── device_id.rs   # Device identification
│   │   ├── macros.rs      # Utility macros
│   │   ├── endpoints/     # REST API endpoints
│   │   │   ├── bucket.rs  # Bucket CRUD operations
│   │   │   ├── events.rs  # Event queries
│   │   │   ├── query.rs   # Advanced query endpoint
│   │   │   ├── export.rs  # Data export
│   │   │   ├── import.rs  # Data import
│   │   │   ├── settings.rs # Configuration endpoints
│   │   │   ├── apikey.rs  # API key validation
│   │   │   ├── cors.rs    # CORS handling
│   │   │   └── util.rs    # Endpoint utilities
│   │   └── android/       # Android-specific code
│   ├── tests/
│   │   ├── api.rs         # API endpoint tests
│   │   └── macros.rs      # Test macros
│   └── Cargo.toml
│
├── aw-datastore/           # Persistence layer
│   ├── src/
│   │   ├── lib.rs         # Library exports
│   │   ├── datastore.rs   # SQLite schema & migrations
│   │   ├── worker.rs      # Database worker thread
│   │   ├── privacy_filter.rs # Data filtering
│   │   └── legacy_import.rs # Import from Python version
│   ├── tests/
│   │   └── datastore.rs   # Datastore tests
│   └── Cargo.toml
│
├── aw-models/              # Data models & serialization
│   ├── src/
│   │   ├── lib.rs         # Exports
│   │   ├── bucket.rs      # Bucket model
│   │   ├── event.rs       # Event model
│   │   ├── timeinterval.rs # Time interval type
│   │   ├── duration.rs    # Duration type
│   │   ├── query.rs       # Query models
│   │   ├── settings.rs    # Settings model
│   │   ├── info.rs        # Server info model
│   │   ├── export.rs      # Export format models
│   │   └── tryvec.rs      # Utility type
│   └── Cargo.toml
│
├── aw-query/               # Query engine
│   ├── src/
│   │   ├── lib.rs         # Exports
│   │   ├── lexer.rs       # Tokenization
│   │   ├── parser.rs      # AST parsing
│   │   ├── ast.rs         # Abstract syntax tree
│   │   ├── interpret.rs   # Query execution
│   │   ├── functions.rs   # Built-in functions
│   │   ├── datatype.rs    # Query data types
│   │   └── grammar.rs     # Grammar rules
│   ├── tests/
│   │   └── query.rs       # Query engine tests
│   ├── benches/
│   │   └── benchmark.rs   # Performance benchmarks
│   └── Cargo.toml
│
├── aw-transform/           # Data transformation
│   ├── src/
│   │   ├── lib.rs
│   │   ├── chunk.rs
│   │   ├── merge.rs       # Merge operations
│   │   ├── heartbeat.rs   # Heartbeat merging
│   │   ├── classify.rs    # Event classification
│   │   ├── filter_period.rs # Period filtering
│   │   ├── flood.rs       # Flood detection
│   │   ├── sort.rs        # Sorting
│   │   ├── union_no_overlap.rs
│   │   ├── period_union.rs
│   │   └── find_bucket.rs
│   ├── benches/
│   │   └── bench.rs       # Transform benchmarks
│   └── Cargo.toml
│
├── aw-client-rust/         # Rust client library
│   ├── src/
│   │   ├── lib.rs
│   │   ├── blocking.rs    # Blocking client
│   │   ├── classes.rs     # Client classes
│   │   ├── queries.rs     # Query helpers
│   │   └── single_instance.rs
│   ├── tests/
│   │   └── test.rs
│   └── Cargo.toml
│
├── aw-sync/                # Sync between servers
│   ├── src/
│   │   ├── lib.rs
│   │   ├── main.rs
│   │   ├── sync.rs
│   │   ├── sync_wrapper.rs
│   │   ├── android.rs
│   │   ├── accessmethod.rs
│   │   ├── util.rs
│   │   └── dirs.rs
│   ├── tests/
│   │   └── sync.rs
│   └── Cargo.toml
│
├── aw-webui/               # Web UI (TypeScript/React)
│   └── [React components, assets]
│
├── Cargo.toml              # Workspace root
├── Cargo.lock              # Locked dependencies
├── Makefile                # Build targets
└── README.md               # Project documentation
```

## Key Source Files (Candidates for Modification)

### Database Layer (Primary focus for PostgreSQL migration)
- `aw-datastore/src/worker.rs` - Database connection handling (currently rusqlite/SQLite)
- `aw-datastore/src/datastore.rs` - Schema definition and migrations
- `aw-datastore/Cargo.toml` - Database dependencies (currently rusqlite)

### Network Configuration (Primary focus for 0.0.0.0 binding)
- `aw-server/src/config.rs` - Default address configuration (currently "127.0.0.1")
- `aw-server/src/main.rs` - CLI argument handling for --host parameter

### API Endpoints
- `aw-server/src/endpoints/bucket.rs` - Bucket CRUD operations
- `aw-server/src/endpoints/events.rs` - Event submission and retrieval
- `aw-server/src/endpoints/query.rs` - Advanced query interface

## Critical Dependencies

| Dependency | Version | Usage | Files |
|------------|---------|-------|-------|
| Rocket | 0.5.0 | Web framework | aw-server |
| rusqlite | 0.30 | SQLite driver | aw-datastore |
| Serde | 1.0 | Serialization | aw-models, aw-server |
| Chrono | 0.4 | Date/time handling | aw-models |
| Tokio | (implied by Rocket) | Async runtime | aw-server |

## Design Patterns

### Worker Thread Pattern (Database Access)
- Location: `aw-datastore/src/worker.rs`
- Purpose: Single writer, multiple readers for SQLite (MPSC channel-based requests)
- Pattern: Actor model for thread-safe database access

### Module Separation
- Clear module boundaries between query, transform, models, and storage
- Each module has a specific responsibility

### Configuration via TOML
- Location: `~/.config/activitywatch/aw-server-rust/config.toml`
- Extensible configuration system

## Testing Structure

- Unit tests in `tests/` directories of each crate
- Integration tests for API endpoints
- Benchmark tests for performance-critical code (query, transform)
