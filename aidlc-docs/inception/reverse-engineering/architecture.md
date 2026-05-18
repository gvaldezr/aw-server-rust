# System Architecture

## System Overview

ActivityWatch Server is a Rust-based backend service that manages activity tracking data. It provides a RESTful API for receiving events from watchers and querying historical activity data. The architecture follows a modular design with clear separation of concerns.

## Architecture Components

```
┌─────────────────────────────────────────────────────────┐
│                 aw-webui (Web Interface)                │
│              React/TypeScript Frontend UI                │
└────────────────────────┬────────────────────────────────┘
                         │ HTTP
┌─────────────────────────────────────────────────────────┐
│              aw-server (Main Application)               │
│  ┌──────────────────────────────────────────────────┐   │
│  │ Rocket Framework (HTTP Server)                   │   │
│  │ - REST Endpoints                                 │   │
│  │ - CORS Handling                                  │   │
│  │ - Authentication (API key)                       │   │
│  └────────────────────┬─────────────────────────────┘   │
│                       │                                   │
│  ┌────────────────────────────────────────────────────┐  │
│  │          Endpoints Module                          │  │
│  │ ├─ /api/0/buckets/* (bucket CRUD)                │  │
│  │ ├─ /api/0/events/* (event queries)               │  │
│  │ ├─ /api/0/query/* (advanced queries)             │  │
│  │ ├─ /api/0/export/* (data export)                 │  │
│  │ ├─ /api/0/import/* (data import)                 │  │
│  │ └─ /api/0/settings/* (configuration)             │  │
│  └────────────────────┬─────────────────────────────┘   │
│                       │                                   │
│  ┌────────────────────────────────────────────────────┐  │
│  │ aw-query Module (Query Engine)                     │  │
│  │ - Query Parser (Lexer → AST)                      │  │
│  │ - Query Interpreter                               │  │
│  │ - Advanced filtering & aggregation               │  │
│  └────────────────────┬─────────────────────────────┘   │
│                       │                                   │
│  ┌────────────────────────────────────────────────────┐  │
│  │ aw-transform Module (Data Transformation)         │  │
│  │ - Merge heartbeats                                │  │
│  │ - Filter by period                                │  │
│  │ - Classify events                                 │  │
│  │ - Sort & union operations                         │  │
│  └────────────────────┬─────────────────────────────┘   │
│                       │                                   │
│  ┌────────────────────────────────────────────────────┐  │
│  │      Datastore (aw-datastore module)              │  │
│  │  Currently: SQLite with rusqlite                  │  │
│  │  - In-memory mode (testing)                       │  │
│  │  - File-based mode (production)                   │  │
│  │  - Connection pooling via worker threads          │  │
│  │  - Schema versioning (v0-v4)                      │  │
│  └────────────────────┬─────────────────────────────┘   │
│                       │                                   │
│  ┌────────────────────────────────────────────────────┐  │
│  │ aw-models Module (Data Models)                     │  │
│  │ - Bucket, Event, TimeInterval models              │  │
│  │ - Settings, Info, Export models                   │  │
│  └────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
                         │ File System
┌─────────────────────────────────────────────────────────┐
│              SQLite Database File                        │
│         ~/.local/share/activitywatch/aw-server.db       │
└─────────────────────────────────────────────────────────┘
```

## Key Modules

| Module | Purpose | Language | Current Technology |
|--------|---------|----------|-------------------|
| aw-server | Main application, HTTP API, configuration | Rust | Rocket 0.5.0, rusqlite |
| aw-datastore | Data persistence layer | Rust | SQLite 3 (rusqlite) |
| aw-query | Query engine with parsing & interpretation | Rust | Custom parser/lexer |
| aw-transform | Data transformation operations | Rust | Pure Rust |
| aw-models | Data models & serialization | Rust | Serde JSON |
| aw-client-rust | Client library for interacting with server | Rust | - |
| aw-sync | Synchronization between servers | Rust | - |
| aw-webui | Web user interface | TypeScript/React | React, Vite |

## Current Database Technology

**SQLite 3** (via rusqlite crate)
- Single-file database at `~/.local/share/activitywatch/aw-server.db`
- Schema version: 4
- Tables: `buckets`, `events`, `key_value`
- Indexes on bucket_id and bucketrow
- Constraints with foreign keys

## Network Configuration

**Current State:**
- Listens on **127.0.0.1** (localhost only) by default
- Port: 5600 (production), 5666 (testing)
- All configuration via `config.toml`

## Data Flow

### Event Submission
```
Watcher → POST /api/0/buckets/{bucket_id}/events → Server → Datastore (SQLite)
```

### Event Query
```
Client → GET /api/0/buckets/{bucket_id}/events → Query Engine → Datastore (SQLite) → Transform → Response
```

### Advanced Query
```
Client → POST /api/0/query → Query Parser → Query Interpreter → Datastore → Transform → Response
```
