#!/usr/bin/env bash
set -euo pipefail

# ActivityWatch PostgreSQL Backup Script
# Performs compressed database backups with 30-day retention policy
# Usage: ./scripts/backup-database.sh [backup_dir]

# Configuration
BACKUP_DIR="${1:-./backups}"
RETENTION_DAYS=30
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
DB_NAME="${DB_NAME:-activitywatch}"
DB_USER="${DB_USER:-aw_user}"
DB_HOST="${DB_HOST:-localhost}"
DB_PORT="${DB_PORT:-5432}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
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

# Create backup directory if it doesn't exist
mkdir -p "${BACKUP_DIR}"

# Backup filename
BACKUP_FILE="${BACKUP_DIR}/activitywatch_backup_${TIMESTAMP}.sql.gz"

log_info "Starting backup of database '${DB_NAME}'..."
log_info "Backup location: ${BACKUP_FILE}"

# Check if pg_dump is available
if ! command -v pg_dump &> /dev/null; then
    log_error "pg_dump command not found. Please install PostgreSQL client tools."
    exit 1
fi

# Check if database is accessible
if ! PGPASSWORD="${DB_PASSWORD:-}" pg_isready -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" -d "${DB_NAME}" &> /dev/null; then
    log_error "Cannot connect to database '${DB_NAME}' at ${DB_HOST}:${DB_PORT}"
    log_error "Please check your database connection settings."
    exit 1
fi

# Perform backup
log_info "Dumping database..."
if PGPASSWORD="${DB_PASSWORD:-}" pg_dump \
    -h "${DB_HOST}" \
    -p "${DB_PORT}" \
    -U "${DB_USER}" \
    -d "${DB_NAME}" \
    --format=plain \
    --no-owner \
    --no-acl \
    --verbose \
    2>&1 | gzip > "${BACKUP_FILE}"; then
    
    BACKUP_SIZE=$(du -h "${BACKUP_FILE}" | cut -f1)
    log_info "Backup completed successfully!"
    log_info "Backup size: ${BACKUP_SIZE}"
else
    log_error "Backup failed!"
    rm -f "${BACKUP_FILE}"
    exit 1
fi

# Verify backup file
if [ ! -s "${BACKUP_FILE}" ]; then
    log_error "Backup file is empty!"
    rm -f "${BACKUP_FILE}"
    exit 1
fi

# Test backup integrity
log_info "Verifying backup integrity..."
if gunzip -t "${BACKUP_FILE}" 2>&1; then
    log_info "Backup integrity verified successfully"
else
    log_error "Backup file is corrupted!"
    rm -f "${BACKUP_FILE}"
    exit 1
fi

# Cleanup old backups (retention policy)
log_info "Cleaning up backups older than ${RETENTION_DAYS} days..."
OLD_BACKUPS=$(find "${BACKUP_DIR}" -name "activitywatch_backup_*.sql.gz" -type f -mtime +${RETENTION_DAYS} 2>/dev/null || true)

if [ -n "${OLD_BACKUPS}" ]; then
    echo "${OLD_BACKUPS}" | while read -r old_backup; do
        log_info "Removing old backup: $(basename "${old_backup}")"
        rm -f "${old_backup}"
    done
    REMOVED_COUNT=$(echo "${OLD_BACKUPS}" | wc -l | tr -d ' ')
    log_info "Removed ${REMOVED_COUNT} old backup(s)"
else
    log_info "No old backups to remove"
fi

# List remaining backups
log_info "Current backups:"
ls -lh "${BACKUP_DIR}"/activitywatch_backup_*.sql.gz 2>/dev/null || log_warn "No backups found in ${BACKUP_DIR}"

# Backup summary
TOTAL_BACKUPS=$(ls -1 "${BACKUP_DIR}"/activitywatch_backup_*.sql.gz 2>/dev/null | wc -l | tr -d ' ')
log_info "=========================================="
log_info "Backup Summary:"
log_info "  Database: ${DB_NAME}"
log_info "  Host: ${DB_HOST}:${DB_PORT}"
log_info "  Backup File: ${BACKUP_FILE}"
log_info "  Backup Size: ${BACKUP_SIZE}"
log_info "  Total Backups: ${TOTAL_BACKUPS}"
log_info "  Retention: ${RETENTION_DAYS} days"
log_info "=========================================="

exit 0
