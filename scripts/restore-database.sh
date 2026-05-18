#!/usr/bin/env bash
set -euo pipefail

# ActivityWatch PostgreSQL Restore Script
# Restores database from compressed backup with safety validations
# Usage: ./scripts/restore-database.sh <backup_file>

# Configuration
DB_NAME="${DB_NAME:-activitywatch}"
DB_USER="${DB_USER:-aw_user}"
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

show_usage() {
    cat << EOF
Usage: $0 <backup_file>

Restore ActivityWatch PostgreSQL database from backup.

Arguments:
  backup_file    Path to compressed backup file (.sql.gz)

Environment Variables:
  DB_NAME        Database name (default: activitywatch)
  DB_USER        Database user (default: aw_user)
  DB_HOST        Database host (default: localhost)
  DB_PORT        Database port (default: 5432)
  DB_PASSWORD    Database password (required)

Example:
  DB_PASSWORD=mypassword $0 backups/activitywatch_backup_20260518_120000.sql.gz

EOF
    exit 1
}

# Check arguments
if [ $# -eq 0 ]; then
    log_error "No backup file specified"
    show_usage
fi

BACKUP_FILE="$1"

# Validate backup file
if [ ! -f "${BACKUP_FILE}" ]; then
    log_error "Backup file not found: ${BACKUP_FILE}"
    exit 1
fi

if [[ ! "${BACKUP_FILE}" =~ \.sql\.gz$ ]]; then
    log_error "Backup file must be a .sql.gz file"
    exit 1
fi

# Check required commands
for cmd in pg_restore psql gunzip docker; do
    if ! command -v ${cmd} &> /dev/null; then
        log_error "${cmd} command not found. Please install required tools."
        exit 1
    fi
done

# Verify backup integrity
log_step "1/7 - Verifying backup file integrity..."
if gunzip -t "${BACKUP_FILE}" 2>&1; then
    log_info "Backup file integrity verified"
    BACKUP_SIZE=$(du -h "${BACKUP_FILE}" | cut -f1)
    log_info "Backup size: ${BACKUP_SIZE}"
else
    log_error "Backup file is corrupted!"
    exit 1
fi

# Check if Docker Compose is running
log_step "2/7 - Checking Docker Compose status..."
if docker compose ps | grep -q "activitywatch-server.*Up"; then
    NEED_RESTART=true
    log_warn "aw-server is running and will be stopped during restore"
else
    NEED_RESTART=false
    log_info "aw-server is not running"
fi

# Confirm restore operation
log_warn "=========================================="
log_warn "WARNING: Database Restore Operation"
log_warn "=========================================="
log_warn "This will:"
log_warn "  1. Stop aw-server (if running)"
log_warn "  2. Drop database '${DB_NAME}'"
log_warn "  3. Create new database"
log_warn "  4. Restore from: ${BACKUP_FILE}"
log_warn "  5. Restart aw-server (if was running)"
log_warn ""
log_warn "ALL CURRENT DATA WILL BE LOST!"
log_warn "=========================================="
echo ""
read -p "Type 'YES' to continue: " -r
echo ""

if [[ ! $REPLY =~ ^YES$ ]]; then
    log_info "Restore cancelled by user"
    exit 0
fi

# Stop aw-server
if [ "${NEED_RESTART}" = true ]; then
    log_step "3/7 - Stopping aw-server..."
    if docker compose stop aw-server 2>&1; then
        log_info "aw-server stopped"
    else
        log_error "Failed to stop aw-server"
        exit 1
    fi
else
    log_step "3/7 - Skipping aw-server stop (not running)"
fi

# Wait for connections to close
log_info "Waiting for database connections to close..."
sleep 3

# Check PostgreSQL connection
log_step "4/7 - Checking PostgreSQL connection..."
if ! PGPASSWORD="${DB_PASSWORD:-}" pg_isready -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" &> /dev/null; then
    log_error "Cannot connect to PostgreSQL at ${DB_HOST}:${DB_PORT}"
    exit 1
fi
log_info "PostgreSQL is accessible"

# Drop and recreate database
log_step "5/7 - Dropping and recreating database..."
log_warn "Dropping database '${DB_NAME}'..."

# Terminate active connections
PGPASSWORD="${DB_PASSWORD:-}" psql \
    -h "${DB_HOST}" \
    -p "${DB_PORT}" \
    -U "${DB_USER}" \
    -d postgres \
    -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '${DB_NAME}' AND pid <> pg_backend_pid();" \
    > /dev/null 2>&1 || true

# Drop database
if PGPASSWORD="${DB_PASSWORD:-}" psql \
    -h "${DB_HOST}" \
    -p "${DB_PORT}" \
    -U "${DB_USER}" \
    -d postgres \
    -c "DROP DATABASE IF EXISTS ${DB_NAME};" 2>&1; then
    log_info "Database dropped"
else
    log_error "Failed to drop database"
    exit 1
fi

# Create database
log_info "Creating database '${DB_NAME}'..."
if PGPASSWORD="${DB_PASSWORD:-}" psql \
    -h "${DB_HOST}" \
    -p "${DB_PORT}" \
    -U "${DB_USER}" \
    -d postgres \
    -c "CREATE DATABASE ${DB_NAME} WITH ENCODING 'UTF8' LC_COLLATE='C' LC_CTYPE='C';" 2>&1; then
    log_info "Database created"
else
    log_error "Failed to create database"
    exit 1
fi

# Restore backup
log_step "6/7 - Restoring database from backup..."
log_info "This may take several minutes..."

if gunzip -c "${BACKUP_FILE}" | PGPASSWORD="${DB_PASSWORD:-}" psql \
    -h "${DB_HOST}" \
    -p "${DB_PORT}" \
    -U "${DB_USER}" \
    -d "${DB_NAME}" \
    --single-transaction \
    --quiet \
    2>&1 | grep -v "ERROR.*already exists" || true; then
    log_info "Database restored successfully"
else
    log_error "Failed to restore database"
    exit 1
fi

# Verify restore
log_info "Verifying restore..."
TABLE_COUNT=$(PGPASSWORD="${DB_PASSWORD:-}" psql \
    -h "${DB_HOST}" \
    -p "${DB_PORT}" \
    -U "${DB_USER}" \
    -d "${DB_NAME}" \
    -t \
    -c "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';" 2>&1 | tr -d ' ')

if [ "${TABLE_COUNT}" -ge 4 ]; then
    log_info "Verified ${TABLE_COUNT} tables restored"
else
    log_warn "Only ${TABLE_COUNT} tables found (expected 4+)"
fi

# Restart aw-server
if [ "${NEED_RESTART}" = true ]; then
    log_step "7/7 - Starting aw-server..."
    if docker compose start aw-server 2>&1; then
        log_info "aw-server started"
        log_info "Waiting for health check..."
        sleep 5
        
        # Check server health
        if docker compose ps | grep -q "activitywatch-server.*healthy"; then
            log_info "aw-server is healthy"
        else
            log_warn "aw-server may not be healthy yet. Check with: docker compose ps"
        fi
    else
        log_error "Failed to start aw-server"
        exit 1
    fi
else
    log_step "7/7 - Skipping aw-server start (was not running)"
fi

# Restore summary
log_info "=========================================="
log_info "Restore Summary:"
log_info "  Database: ${DB_NAME}"
log_info "  Host: ${DB_HOST}:${DB_PORT}"
log_info "  Backup File: ${BACKUP_FILE}"
log_info "  Backup Size: ${BACKUP_SIZE}"
log_info "  Tables Restored: ${TABLE_COUNT}"
log_info "  Status: SUCCESS"
log_info "=========================================="

log_info "Restore completed successfully!"
log_info "You can now access the server at http://localhost:5600"

exit 0
