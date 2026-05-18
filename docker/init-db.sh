#!/bin/bash
set -e

# This script runs automatically during PostgreSQL initialization
# (only on first container startup when data directory is empty)

echo "Running ActivityWatch database initialization..."

# Create pg_stat_statements extension for query performance monitoring
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    -- Enable query statistics extension
    CREATE EXTENSION IF NOT EXISTS pg_stat_statements;
    
    -- Log initialization complete
    SELECT 'ActivityWatch PostgreSQL initialization complete' AS status;
EOSQL

echo "Database initialization complete."
