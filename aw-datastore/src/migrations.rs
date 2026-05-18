use std::sync::Arc;
use deadpool_postgres::Pool;
use chrono::Utc;

/// Schema migration manager
pub struct MigrationManager {
    pool: Arc<Pool>,
}

#[derive(Debug)]
pub enum MigrationError {
    DatabaseError(String),
    MigrationFailed(String),
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            MigrationError::MigrationFailed(msg) => write!(f, "Migration failed: {}", msg),
        }
    }
}

impl std::error::Error for MigrationError {}

impl MigrationManager {
    pub fn new(pool: Arc<Pool>) -> Self {
        MigrationManager { pool }
    }

    /// Run all pending migrations
    pub async fn run_migrations(&self) -> Result<(), MigrationError> {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| MigrationError::DatabaseError(e.to_string()))?;

        // Check current schema version
        let current_version = match self.get_current_version(&client).await {
            Ok(v) => v,
            Err(_) => 0, // No schema_version table yet, assume v0
        };

        log::info!("Current database schema version: {}", current_version);

        // Apply migrations sequentially
        if current_version < 1 {
            log::info!("Applying migration v0 -> v1: Initial PostgreSQL schema");
            let tx = client
                .transaction()
                .await
                .map_err(|e| MigrationError::DatabaseError(e.to_string()))?;

            self.migrate_v0_to_v1(&tx).await?;

            tx.commit()
                .await
                .map_err(|e| MigrationError::MigrationFailed(e.to_string()))?;

            log::info!("Migration v0 -> v1 completed successfully");
        }

        // Future migrations would go here:
        // if current_version < 2 { self.migrate_v1_to_v2(&mut tx).await?; }
        // if current_version < 3 { self.migrate_v2_to_v3(&mut tx).await?; }

        Ok(())
    }

    /// Get current schema version from database
    async fn get_current_version(
        &self,
        client: &deadpool_postgres::Client,
    ) -> Result<i32, MigrationError> {
        // Check if schema_version table exists
        let table_exists = client
            .query_opt(
                "SELECT 1 FROM information_schema.tables 
                 WHERE table_schema = 'public' AND table_name = 'schema_version'",
                &[],
            )
            .await
            .map_err(|e| MigrationError::DatabaseError(e.to_string()))?;

        if table_exists.is_none() {
            return Ok(0); // No migrations applied yet
        }

        // Get max version
        let row = client
            .query_one("SELECT MAX(version) as version FROM schema_version", &[])
            .await
            .map_err(|e| MigrationError::DatabaseError(e.to_string()))?;

        let version: Option<i32> = row.get("version");
        Ok(version.unwrap_or(0))
    }

    /// Initial migration: Create PostgreSQL schema (v0 -> v1)
    async fn migrate_v0_to_v1(
        &self,
        tx: &deadpool_postgres::Transaction<'_>,
    ) -> Result<(), MigrationError> {
        // Create schema_version table first
        tx.execute(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied TIMESTAMP WITH TIME ZONE NOT NULL
            )",
            &[],
        )
        .await
        .map_err(|e| MigrationError::MigrationFailed(e.to_string()))?;

        // Create buckets table
        tx.execute(
            "CREATE TABLE IF NOT EXISTS buckets (
                id SERIAL PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                type TEXT NOT NULL,
                client TEXT NOT NULL,
                hostname TEXT NOT NULL,
                created TIMESTAMP WITH TIME ZONE NOT NULL,
                data JSONB NOT NULL DEFAULT '{}'::jsonb
            )",
            &[],
        )
        .await
        .map_err(|e| MigrationError::MigrationFailed(e.to_string()))?;

        // Create index on buckets
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_buckets_client_hostname 
             ON buckets(client, hostname)",
            &[],
        )
        .await
        .map_err(|e| MigrationError::MigrationFailed(e.to_string()))?;

        // Create events table with composite index
        tx.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id BIGSERIAL PRIMARY KEY,
                bucketrow INTEGER NOT NULL REFERENCES buckets(id) ON DELETE CASCADE,
                starttime TIMESTAMP WITH TIME ZONE NOT NULL,
                endtime TIMESTAMP WITH TIME ZONE NOT NULL,
                data JSONB NOT NULL
            )",
            &[],
        )
        .await
        .map_err(|e| MigrationError::MigrationFailed(e.to_string()))?;

        // Create composite index for time-range queries (critical for performance)
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_events_timerange 
             ON events(bucketrow, starttime, endtime)",
            &[],
        )
        .await
        .map_err(|e| MigrationError::MigrationFailed(e.to_string()))?;

        // Create index on events starttime (for chronological queries)
        tx.execute(
            "CREATE INDEX IF NOT EXISTS idx_events_starttime 
             ON events(starttime)",
            &[],
        )
        .await
        .map_err(|e| MigrationError::MigrationFailed(e.to_string()))?;

        // Create key_value table
        tx.execute(
            "CREATE TABLE IF NOT EXISTS key_value (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            &[],
        )
        .await
        .map_err(|e| MigrationError::MigrationFailed(e.to_string()))?;

        // Record migration
        let now = Utc::now();
        tx.execute(
            "INSERT INTO schema_version (version, applied) VALUES ($1, $2)",
            &[&1i32, &now],
        )
        .await
        .map_err(|e| MigrationError::MigrationFailed(e.to_string()))?;

        Ok(())
    }

    /// Check if database is initialized
    pub async fn is_initialized(&self) -> bool {
        let client = match self.pool.get().await {
            Ok(c) => c,
            Err(_) => return false,
        };

        match self.get_current_version(&client).await {
            Ok(version) => version > 0,
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running PostgreSQL instance
    // In production, use testcontainers for integration tests

    #[test]
    fn test_migration_error_display() {
        let error = MigrationError::DatabaseError("connection failed".to_string());
        assert_eq!(error.to_string(), "Database error: connection failed");

        let error = MigrationError::MigrationFailed("table exists".to_string());
        assert_eq!(error.to_string(), "Migration failed: table exists");
    }
}
