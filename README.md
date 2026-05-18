aw-server-rust
==============

[![Build Status](https://github.com/ActivityWatch/aw-server-rust/workflows/Build/badge.svg?branch=master)](https://github.com/ActivityWatch/aw-server-rust/actions?query=workflow%3ABuild+branch%3Amaster)
[![Coverage Status](https://codecov.io/gh/ActivityWatch/aw-server-rust/branch/master/graph/badge.svg)](https://codecov.io/gh/ActivityWatch/aw-server-rust)
[![Dependency Status](https://deps.rs/repo/github/activitywatch/aw-server-rust/status.svg)](https://deps.rs/repo/github/activitywatch/aw-server-rust)

A reimplementation of aw-server in Rust.

Features missing compared to the Python implementation of aw-server:

 - API explorer (Swagger/OpenAPI)

### How to compile

Build with `cargo`:

```sh
cargo build --release
```

You can also build with make, which will build the web assets as well:

```
make build
```

Your built executable will be located in `./target/release/aw-server-rust`. If you want to use it with a development version of `aw-qt` you'll want to copy this binary into your `venv`:

```shell
cp target/release/aw-server ../venv/bin/aw-server-rust
```

### Database Backend

**PostgreSQL Support**: This version of aw-server-rust uses PostgreSQL instead of SQLite for improved concurrency, scalability, and production readiness.

#### Requirements

- **PostgreSQL 15+**: Required for all deployments
- **Connection Details**: Configured via environment variables or command-line arguments

#### Configuration

**Via Environment Variables**:
```bash
export DB_HOST=localhost
export DB_PORT=5432
export DB_USER=aw_user
export DB_PASSWORD=your_password  # Or use DB_PASSWORD_FILE for Docker secrets
export DB_NAME=activitywatch
```

**Via Command-Line Arguments**:
```bash
aw-server --db-host localhost --db-port 5432 --db-user aw_user \
  --db-password your_password --db-name activitywatch
```

#### Docker Deployment (Recommended)

The easiest way to run aw-server-rust with PostgreSQL is using Docker Compose:

**Quick Start**:
```bash
# 1. Create database password
mkdir -p secrets
openssl rand -base64 32 > secrets/db_password.txt
chmod 600 secrets/db_password.txt

# 2. Start services
docker compose up -d

# 3. Verify deployment
curl http://localhost:5600/api/0/info

# 4. Access Web UI
open http://localhost:8080
```

**Services Included**:
- **postgresql**: PostgreSQL 15 database (internal: 172.20.0.2:5432)
- **aw-server**: ActivityWatch API server (exposed: 0.0.0.0:5600)
- **aw-webui**: ActivityWatch Web UI (exposed: 0.0.0.0:8080)

**Resource Limits** (adjust in docker-compose.yml):
- PostgreSQL: 4 CPU cores, 16 GB RAM
- aw-server: 4 CPU cores, 4 GB RAM
- aw-webui: 1 CPU core, 512 MB RAM

For detailed deployment instructions, see [Deployment Guide](./aidlc-docs/construction/unit-1-database-layer/code/deployment-guide.md).

#### Database Operations

**Backup Database**:
```bash
DB_PASSWORD=$(cat secrets/db_password.txt) \
  ./scripts/backup-database.sh ./backups
```

**Restore Database**:
```bash
DB_PASSWORD=$(cat secrets/db_password.txt) \
  ./scripts/restore-database.sh backups/activitywatch_backup_<timestamp>.sql.gz
```

**Manual Database Access**:
```bash
# Via Docker Compose
docker compose exec postgresql psql -U aw_user -d activitywatch

# Via psql client
PGPASSWORD=your_password psql -h localhost -p 5432 -U aw_user -d activitywatch
```

#### Migration from SQLite

**Note**: The SQLite backend has been removed in this version. To migrate from an existing SQLite database:

1. Export data from your SQLite database using the Python version of aw-server
2. Use the API to re-import data into the PostgreSQL backend
3. Alternatively, create a custom migration script based on your data structure

For assistance with migration, see [GitHub Issues](https://github.com/ActivityWatch/aw-server-rust/issues).

#### Performance Characteristics

**Designed for**:
- **300 concurrent watchers**
- **50 events/second sustained, 300 events/second peak**
- **1.1 billion events/year (~150 GB data)**

**Features**:
- ✅ Connection pooling (20 max connections)
- ✅ Automatic retry with exponential backoff
- ✅ Health checks (liveness + readiness)
- ✅ Prometheus-style metrics
- ✅ Async I/O throughout


### How to run

If you want to quick-compile for debugging, run cargo run from the project root:

```sh
cargo run --bin aw-server
```

*NOTE:* This will start aw-server-rust in testing mode (on port 5666 instead of port 5600).

### Configuration

The server reads its configuration from `~/.config/activitywatch/aw-server-rust/config.toml` (or `config-testing.toml` in testing mode).

Available options:

```toml
# Address to listen on
#address = "127.0.0.1"

# Port to listen on (default: 5600, testing: 5666)
#port = 5600

# Additional exact CORS origins to allow (e.g. for custom web interfaces)
#cors = ["http://localhost:3000"]

# Additional regex CORS origins to allow (e.g. for sideloaded browser extensions)
#cors_regex = ["chrome-extension://yourextensionidhere"]
```

#### Custom CORS Origins

**Default Behavior (Production Deployment)**:
By default, the server **allows requests from ANY origin** when no custom CORS configuration is provided. This is suitable for:
- Private network deployments with multiple watchers
- Internal/corporate LANs
- Docker deployments accessed from multiple IPs

**Default allowed origins** (when CORS is not restricted):
- All origins via `AllowedOrigins::all()`
- Includes WebUI access from any IP on port 8080
- Suitable for 300+ concurrent watchers across a network

**To Restrict Origins** (optional, for added security):
If you need to restrict access to specific origins, configure them in `config.toml`:

```toml
# Allow specific origins only
cors = ["http://192.168.1.100:8080", "http://192.168.1.101:8080"]

# Or use regex patterns
cors_regex = ["http://192\\.168\\.1\\..*:8080"]
```

When `cors` or `cors_regex` are configured, the server switches to restricted mode and only allows:
- The server's own origin (`http://127.0.0.1:<port>`, `http://localhost:<port>`)
- WebUI origins (`http://localhost:8080`, `http://127.0.0.1:8080`)
- The official Chrome extension (`chrome-extension://nglaklhklhcoonedhgnpgddginnjdadi`)
- All Firefox extensions (`moz-extension://.*`)
- Your custom configured origins

**Network Access**:
To allow access from other computers on your network:
1. Server binds to `0.0.0.0` by default (listens on all interfaces)
2. Find your server IP: `ifconfig | grep "inet "`
3. Access from other computers: `http://<server-ip>:8080` (WebUI) or `http://<server-ip>:5600` (API)

### Syncing

For details about aw-sync-rust, see the [README](./aw-sync/README.md) in its subdirectory.
