# Requirements Document - ActivityWatch Server PostgreSQL Migration & Docker Deployment

**Version**: 1.0  
**Date**: 2026-05-18  
**Project Type**: Brownfield - Database Migration + Network Configuration + Docker Deployment  
**Status**: Requirements Approved  

---

## Executive Summary

Migrar ActivityWatch Server Rust de SQLite a PostgreSQL 15 (LTS), permitir que escuche en todas las interfaces de red (0.0.0.0), e incluir un docker-compose para despliegue en producción con PostgreSQL, aw-server, y aw-webui en contenedores separados con persistencia de volúmenes.

---

## 1. Request Analysis

### 1.1 Request Type
- **Primary**: Technology Migration (SQLite → PostgreSQL)
- **Secondary**: Infrastructure Enhancement (Network, Docker)

### 1.2 Scope Estimate
- **Multiple Components**: Changes across aw-datastore, aw-server/config, new Docker infrastructure

### 1.3 Complexity Estimate
- **Complex**: Significant database layer refactoring with multiple considerations for production deployment

---

## 2. Functional Requirements

### FR-1: PostgreSQL Database Migration

#### FR-1.1 Database Version
- **Requirement**: Migrate to PostgreSQL 15 LTS
- **Rationale**: Long-term support, stable, widely adopted
- **Implementation**: Update aw-datastore to use PostgreSQL driver (sqlx or tokio-postgres)

#### FR-1.2 Connection Configuration
- **Requirement**: Support environment variables for database connection
- **Variables Required**:
  - `DB_HOST` - PostgreSQL server hostname (default: localhost)
  - `DB_PORT` - PostgreSQL port (default: 5432)
  - `DB_USER` - Database user (default: aw_user)
  - `DB_PASSWORD` - Database password
  - `DB_NAME` - Database name (default: activitywatch)
- **Implementation**: Read from environment in aw-server/src/config.rs, pass to datastore

#### FR-1.3 Schema Migration
- **Requirement**: Adapt SQLite schema to PostgreSQL equivalents
- **Tables to Migrate**:
  - `buckets` - bucket metadata
  - `events` - activity events
  - `key_value` - settings key-value pairs
- **Data Types**: Map SQLite types to PostgreSQL (TEXT→VARCHAR, INTEGER→SERIAL/BIGINT, etc.)
- **Constraints**: Preserve foreign keys and indexes

#### FR-1.4 Database Initialization
- **Requirement**: Auto-create schema on first run if not exists
- **Implementation**: Schema creation logic in datastore initialization
- **No Legacy Import**: Start with empty database (no SQLite migration needed)

### FR-2: Network Configuration

#### FR-2.1 Binding Address
- **Requirement**: Server must listen on 0.0.0.0 (all interfaces)
- **Purpose**: Allow connections from any machine on the network
- **Implementation**: Change default binding address in config.rs from "127.0.0.1" to "0.0.0.0"
- **Backward Compatibility**: CLI flag `--host` remains configurable for override

#### FR-2.2 Port Configuration
- **Requirement**: Maintain current port mapping
  - Production: 5600
  - Testing: 5666
- **Implementation**: No changes needed, current logic preserved

### FR-3: Docker Deployment

#### FR-3.1 Docker Compose Services
- **Services Required**:
  1. **postgresql** - PostgreSQL 15 database
  2. **aw-server** - ActivityWatch server
  3. **aw-webui** - Web UI interface (separate container)

#### FR-3.2 PostgreSQL Service Configuration
- **Image**: postgres:15-alpine
- **Environment Variables**:
  - `POSTGRES_USER=aw_user`
  - `POSTGRES_PASSWORD=<password>`
  - `POSTGRES_DB=activitywatch`
- **Volume**: Named volume for data persistence (`pg_data`)
- **Port**: 5432 (internal only, not exposed)
- **Healthcheck**: Required for service startup ordering

#### FR-3.3 aw-server Service Configuration
- **Build Context**: Build from current Cargo project
- **Environment Variables**: DB_HOST, DB_PORT, DB_USER, DB_PASSWORD, DB_NAME
- **Port Mapping**: 5600:5600 (exposed for API)
- **Depends On**: postgresql service
- **Volume**: Named volume for persistent data if needed
- **Healthcheck**: /api/0/info endpoint

#### FR-3.4 aw-webui Service Configuration
- **Service Type**: Separate container (Nginx or Node.js server)
- **Purpose**: Serve React UI static files
- **Port Mapping**: 80:3000 or similar (expose web UI)
- **Build Context**: Build from aw-webui directory with production build
- **Dependencies**: Connects to aw-server via API

#### FR-3.5 Data Persistence
- **Strategy**: Docker named volumes
  - `pg_data` - PostgreSQL data directory
  - `aw_data` - ActivityWatch server data (if needed)
- **Benefits**: Survives container restarts, volume management via Docker
- **No Bind Mounts**: Use named volumes for cleaner production setup

#### FR-3.6 Environment Configuration
- **Method**: Hardcoded values in docker-compose.yml (not .env)
- **Default Credentials**: 
  - User: `aw_user`
  - Password: `activitywatch_secure` (can be changed in compose file)
  - Database: `activitywatch`

### FR-4: API Compatibility

#### FR-4.1 API Endpoints
- **Requirement**: All REST endpoints must remain identical
- **Constraint**: No changes to endpoint paths, methods, request/response schemas
- **Impact**: Database layer changes transparent to API layer

---

## 3. Non-Functional Requirements

### NFR-1: Performance

#### NFR-1.1 Query Performance
- **Target**: Equivalence or improvement over SQLite
- **Optimization**: PostgreSQL indexes on frequently queried columns (bucket_id, timestamp ranges)
- **Rationale**: Balanced read/write workload for activity tracking

#### NFR-1.2 Write Throughput
- **Target**: Support concurrent event submissions from multiple watchers
- **Optimization**: Connection pooling in datastore worker thread

### NFR-2: Reliability

#### NFR-2.1 Service Startup Order
- **Requirement**: PostgreSQL must be healthy before aw-server connects
- **Implementation**: Docker healthchecks + depends_on with condition

#### NFR-2.2 Data Persistence
- **Requirement**: Data survives container restarts
- **Implementation**: Named volumes + automatic schema recreation

### NFR-3: Operability

#### NFR-3.1 Logging
- **Strategy**: stdout logging (Docker standard)
- **Integration**: Docker logs accessible via `docker-compose logs`
- **No File Rotation**: Keep simple for containerized deployment

#### NFR-3.2 Testing
- **Unit Tests**: Test database layer with PostgreSQL (testcontainers or Docker)
- **Integration Tests**: Test full API with PostgreSQL backend
- **Strategy**: Automated via CI/CD

### NFR-4: Deployment

#### NFR-4.1 Single-Machine Deployment
- **Target Scenario**: One physical/virtual machine running all services
- **Docker Compose**: Sufficient (not Kubernetes)
- **Resource Requirements**: 2GB RAM minimum (PostgreSQL + server + UI)

### NFR-5: Security

#### NFR-5.1 Database Credentials
- **Storage**: Environment variables (best practice)
- **Exposure**: Not hardcoded in code, configurable per environment
- **Access**: Only aw-server container needs database credentials

---

## 4. Architecture Changes

### 4.1 Database Layer Refactoring

```
BEFORE (SQLite):
├── aw-datastore/src/worker.rs      [rusqlite Connection]
├── aw-datastore/src/datastore.rs   [SQLite schema]
└── Cargo.toml                        [rusqlite dependency]

AFTER (PostgreSQL):
├── aw-datastore/src/worker.rs      [PostgreSQL connection pool]
├── aw-datastore/src/datastore.rs   [PostgreSQL schema]
└── Cargo.toml                        [sqlx or tokio-postgres dependency]
```

### 4.2 Configuration Changes

```
BEFORE:
- Default address: 127.0.0.1
- Database: File-based SQLite at ~/.local/share/...

AFTER:
- Default address: 0.0.0.0
- Database: Environment variables (DB_HOST, DB_PORT, DB_USER, etc.)
- Docker: Environment-driven via docker-compose.yml
```

### 4.3 Deployment Infrastructure (New)

```
docker-compose.yml
├── postgresql (PostgreSQL 15 container)
├── aw-server (Rust binary container)
└── aw-webui (Web UI container)

Environment:
- Named volumes for persistence
- Internal networking between services
- Health checks for startup sequencing
```

---

## 5. Implementation Scope

### Phase 1: Database Migration
- [ ] Update Cargo.toml dependencies (remove rusqlite, add PostgreSQL driver)
- [ ] Refactor aw-datastore/src/worker.rs for PostgreSQL
- [ ] Adapt aw-datastore/src/datastore.rs schema
- [ ] Update aw-server/src/config.rs for DB environment variables
- [ ] Test schema creation and basic operations

### Phase 2: Network Configuration
- [ ] Update aw-server/src/config.rs default binding to 0.0.0.0
- [ ] Verify CLI flag --host override still works
- [ ] Test server accessibility from other machines

### Phase 3: Docker Infrastructure
- [ ] Create Dockerfile for aw-server build
- [ ] Create docker-compose.yml with all services
- [ ] Configure PostgreSQL service with healthcheck
- [ ] Configure aw-webui container (build + serve)
- [ ] Test full stack startup and data persistence
- [ ] Document usage and configuration

### Phase 4: Validation
- [ ] Unit tests pass with PostgreSQL backend
- [ ] Integration tests pass
- [ ] Docker Compose stack runs cleanly
- [ ] API endpoints work as before
- [ ] Data persists across container restarts

---

## 6. Success Criteria

✅ **Done When**:
1. aw-server connects to PostgreSQL 15 via environment variables
2. All data is persisted in PostgreSQL (not SQLite)
3. Server listens on 0.0.0.0 by default
4. docker-compose.yml deploys complete stack with PostgreSQL, server, and UI
5. All REST API endpoints work identically (backward compatible)
6. Data persists across container restarts
7. Automated tests pass (unit + integration)

---

## 7. Dependencies & Constraints

### Technology Dependencies
- PostgreSQL 15 (not negotiable)
- Docker & Docker Compose (required for deployment)
- Rust 1.70+ (for async PostgreSQL drivers)

### File Modifications
- aw-datastore/Cargo.toml
- aw-datastore/src/worker.rs
- aw-datastore/src/datastore.rs
- aw-server/src/config.rs
- docker-compose.yml (new file)
- Dockerfile (new file)

### External Services
- PostgreSQL container (provided by docker-compose)
- No external APIs or services required

---

## 8. Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Data loss during migration | High | No legacy migration needed (start fresh); test with sample data |
| Network exposure on 0.0.0.0 | Medium | Document security implications; recommend firewall/VPN |
| PostgreSQL driver compatibility | Medium | Choose well-maintained driver (sqlx or tokio-postgres); test thoroughly |
| API regression | High | Comprehensive test suite; maintain exact endpoint compatibility |

---

## 9. Assumptions

1. No existing data needs to be migrated from SQLite
2. Single-machine Docker Compose deployment is sufficient
3. PostgreSQL 15 is available in production environment
4. API contract must remain 100% identical
5. Logging to stdout is acceptable (no external logging system)

---

## 10. Out of Scope

❌ **NOT Included**:
- Multi-region or multi-container orchestration (Kubernetes)
- External logging/monitoring systems
- SSL/TLS setup for PostgreSQL
- Advanced backup/recovery procedures
- Data migration from legacy SQLite databases
- Performance optimization beyond basic indexing
