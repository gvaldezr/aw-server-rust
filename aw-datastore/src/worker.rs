use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use chrono::DateTime;
use chrono::Utc;

use deadpool_postgres::{Manager, Pool};
use tokio_postgres::{Config as PgConfig, NoTls};

use aw_models::Bucket;
use aw_models::Event;

use crate::privacy_filter::PrivacyFilterEngine;
use crate::retry::RetryPolicy;
use crate::metrics::DbMetrics;
use crate::migrations::MigrationManager;
use crate::DatastoreError;
use crate::DatastoreInstance;

/// Database configuration for PostgreSQL
#[derive(Clone, Debug)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl DbConfig {
    /// Load database configuration from environment variables
    pub fn from_env() -> Self {
        DbConfig {
            host: std::env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("DB_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(5432),
            user: std::env::var("DB_USER").unwrap_or_else(|_| "aw_user".to_string()),
            password: Self::load_password(),
            database: std::env::var("DB_NAME").unwrap_or_else(|_| "activitywatch".to_string()),
        }
    }

    /// Load password from file (Docker secrets support) or environment variable
    fn load_password() -> String {
        // Try loading from file first (Docker secrets pattern)
        if let Ok(password_file) = std::env::var("DB_PASSWORD_FILE") {
            if let Ok(password) = std::fs::read_to_string(&password_file) {
                return password.trim().to_string();
            }
            eprintln!("Warning: DB_PASSWORD_FILE specified but could not read: {}", password_file);
        }

        // Fall back to environment variable
        std::env::var("DB_PASSWORD").unwrap_or_else(|_| {
            eprintln!("Warning: No DB_PASSWORD or DB_PASSWORD_FILE specified, using default");
            "activitywatch".to_string()
        })
    }

    /// Get PostgreSQL connection string
    pub fn connection_string(&self) -> String {
        format!(
            "host={} port={} user={} password={} dbname={}",
            self.host, self.port, self.user, self.password, self.database
        )
    }

    pub fn to_postgres_config(&self) -> PgConfig {
        let mut config = PgConfig::new();
        config.host(&self.host);
        config.port(self.port);
        config.user(&self.user);
        config.password(&self.password);
        config.dbname(&self.database);
        
        // Set connection timeouts
        config.connect_timeout(std::time::Duration::from_secs(10));
        config.keepalives(true);
        config.keepalives_idle(std::time::Duration::from_secs(30));
        
        config
    }
}

impl Default for DbConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

#[derive(Clone)]
pub struct Datastore {
    pool: Arc<Pool>,
    retry_policy: Arc<RetryPolicy>,
    metrics: Arc<DbMetrics>,
    privacy_engine: Arc<tokio::sync::RwLock<PrivacyFilterEngine>>,
}

impl fmt::Debug for Datastore {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Datastore(PostgreSQL)")
    }
}

impl Datastore {
    /// Create a new Datastore with PostgreSQL configuration
    pub async fn new_with_config(
        db_config: DbConfig,
        legacy_import: bool,
    ) -> Result<Self, DatastoreError> {
        if legacy_import {
            warn!("Legacy import from aw-server-python is not supported with PostgreSQL backend");
        }

        // Log database configuration (without password)
        info!(
            "Connecting to PostgreSQL at {}:{}/{} as user {}",
            db_config.host, db_config.port, db_config.database, db_config.user
        );

        // Initial delay to allow network to stabilize (Docker)
        info!("Waiting 3 seconds for network initialization...");
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        let pg_config = db_config.to_postgres_config();
        let manager = Manager::new(pg_config, NoTls);
        
        let pool = Pool::builder(manager)
            .max_size(20)
            .build()
            .map_err(|e| {
                DatastoreError::InternalError(format!("Failed to create connection pool: {}", e))
            })?;

        // Test connection with retries (for Docker network initialization)
        info!("Testing database connection...");
        let max_retries = 5;
        let mut last_error = String::new();
        
        for attempt in 1..=max_retries {
            match pool.get().await {
                Ok(_client) => {
                    info!("Database connection successful on attempt {}", attempt);
                    break;
                }
                Err(e) => {
                    last_error = format!("{:?}", e);  // Use Debug format for more details
                    if attempt < max_retries {
                        warn!("Connection attempt {} failed: {}. Retrying in 2s...", attempt, last_error);
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    } else {
                        error!("Failed to connect to database after {} attempts: {}", max_retries, last_error);
                        return Err(DatastoreError::InternalError(format!("Failed to connect to database: {}", last_error)));
                    }
                }
            }
        }

        let migration_manager = MigrationManager::new(Arc::new(pool.clone()));
        migration_manager
            .run_migrations()
            .await
            .map_err(|e| DatastoreError::InternalError(format!("Migration failed: {}", e)))?;

        info!(
            "PostgreSQL connection pool initialized: {}:{}/{}",
            db_config.host, db_config.port, db_config.database
        );

        Ok(Datastore {
            pool: Arc::new(pool),
            retry_policy: Arc::new(RetryPolicy::default()),
            metrics: Arc::new(DbMetrics::new()),
            privacy_engine: Arc::new(tokio::sync::RwLock::new(PrivacyFilterEngine::new(vec![]))),
        })
    }

    async fn get_connection(&self) -> Result<deadpool_postgres::Client, DatastoreError> {
        let pool = Arc::clone(&self.pool);
        let metrics = Arc::clone(&self.metrics);
        
        let start = std::time::Instant::now();
        match pool.get().await {
            Ok(client) => {
                metrics.record_query("pool_get", start.elapsed());
                Ok(client)
            }
            Err(e) => {
                metrics.record_error("pool_get");
                Err(DatastoreError::InternalError(format!(
                    "Failed to get connection from pool: {}",
                    e
                )))
            }
        }
    }

    pub async fn create_bucket(&self, bucket: &Bucket) -> Result<(), DatastoreError> {
        let client = self.get_connection().await?;
        let start = std::time::Instant::now();
        
        let result = DatastoreInstance::create_bucket_pg(&client, bucket).await;
        
        self.metrics.record_query("create_bucket", start.elapsed());
        if result.is_err() {
            self.metrics.record_error("create_bucket");
        }
        
        result
    }

    pub async fn delete_bucket(&self, bucket_id: &str) -> Result<(), DatastoreError> {
        let client = self.get_connection().await?;
        let start = std::time::Instant::now();
        
        let result = DatastoreInstance::delete_bucket_pg(&client, bucket_id).await;
        
        self.metrics.record_query("delete_bucket", start.elapsed());
        if result.is_err() {
            self.metrics.record_error("delete_bucket");
        }
        
        result
    }

    pub async fn get_bucket(&self, bucket_id: &str) -> Result<Bucket, DatastoreError> {
        let client = self.get_connection().await?;
        let start = std::time::Instant::now();
        
        let result = DatastoreInstance::get_bucket_pg(&client, bucket_id).await;
        
        self.metrics.record_query("get_bucket", start.elapsed());
        if result.is_err() {
            self.metrics.record_error("get_bucket");
        }
        
        result
    }

    pub async fn get_buckets(&self) -> Result<HashMap<String, Bucket>, DatastoreError> {
        let client = self.get_connection().await?;
        let start = std::time::Instant::now();
        
        let result = DatastoreInstance::get_buckets_pg(&client).await;
        
        self.metrics.record_query("get_buckets", start.elapsed());
        if result.is_err() {
            self.metrics.record_error("get_buckets");
        }
        
        result
    }

    pub async fn insert_events(
        &self,
        bucket_id: &str,
        events: Vec<Event>,
    ) -> Result<Vec<Event>, DatastoreError> {
        let privacy_engine = self.privacy_engine.read().await;
        let filtered = privacy_engine.filter_events(bucket_id, events);
        drop(privacy_engine);
        
        if filtered.is_empty() {
            return Ok(vec![]);
        }

        let client = self.get_connection().await?;
        let start = std::time::Instant::now();
        
        let result = DatastoreInstance::insert_events_pg(&client, bucket_id, filtered).await;
        
        self.metrics.record_query("insert_events", start.elapsed());
        if result.is_err() {
            self.metrics.record_error("insert_events");
        }
        
        result
    }

    pub async fn heartbeat(
        &self,
        bucket_id: &str,
        event: Event,
        pulsetime: f64,
    ) -> Result<Event, DatastoreError> {
        let privacy_engine = self.privacy_engine.read().await;
        let filtered = match privacy_engine.filter_event(bucket_id, event.clone()) {
            Some(e) => e,
            None => {
                drop(privacy_engine);
                return Ok(event);
            }
        };
        drop(privacy_engine);

        let client = self.get_connection().await?;
        let start = std::time::Instant::now();
        
        let result = DatastoreInstance::heartbeat_pg(&client, bucket_id, filtered, pulsetime).await;
        
        self.metrics.record_query("heartbeat", start.elapsed());
        if result.is_err() {
            self.metrics.record_error("heartbeat");
        }
        
        result
    }

    pub async fn get_event(&self, bucket_id: &str, event_id: i64) -> Result<Event, DatastoreError> {
        let client = self.get_connection().await?;
        let start = std::time::Instant::now();
        
        let result = DatastoreInstance::get_event_pg(&client, bucket_id, event_id).await;
        
        self.metrics.record_query("get_event", start.elapsed());
        if result.is_err() {
            self.metrics.record_error("get_event");
        }
        
        result
    }

    pub async fn get_events(
        &self,
        bucket_id: &str,
        starttime: Option<DateTime<Utc>>,
        endtime: Option<DateTime<Utc>>,
        limit: Option<u64>,
    ) -> Result<Vec<Event>, DatastoreError> {
        let client = self.get_connection().await?;
        let start = std::time::Instant::now();
        
        let result = DatastoreInstance::get_events_pg(
            &client,
            bucket_id,
            starttime,
            endtime,
            limit,
            false,
        )
        .await;
        
        self.metrics.record_query("get_events", start.elapsed());
        if result.is_err() {
            self.metrics.record_error("get_events");
        }
        
        result
    }

    pub async fn get_events_unclipped(
        &self,
        bucket_id: &str,
        starttime: Option<DateTime<Utc>>,
        endtime: Option<DateTime<Utc>>,
        limit: Option<u64>,
    ) -> Result<Vec<Event>, DatastoreError> {
        let client = self.get_connection().await?;
        let start = std::time::Instant::now();
        
        let result = DatastoreInstance::get_events_pg(
            &client,
            bucket_id,
            starttime,
            endtime,
            limit,
            true,
        )
        .await;
        
        self.metrics.record_query("get_events_unclipped", start.elapsed());
        if result.is_err() {
            self.metrics.record_error("get_events_unclipped");
        }
        
        result
    }

    pub async fn get_event_count(
        &self,
        bucket_id: &str,
        starttime: Option<DateTime<Utc>>,
        endtime: Option<DateTime<Utc>>,
    ) -> Result<i64, DatastoreError> {
        let client = self.get_connection().await?;
        let start = std::time::Instant::now();
        
        let result = DatastoreInstance::get_event_count_pg(&client, bucket_id, starttime, endtime).await;
        
        self.metrics.record_query("get_event_count", start.elapsed());
        if result.is_err() {
            self.metrics.record_error("get_event_count");
        }
        
        result
    }

    pub async fn delete_events_by_id(
        &self,
        bucket_id: &str,
        event_ids: Vec<i64>,
    ) -> Result<(), DatastoreError> {
        let client = self.get_connection().await?;
        let start = std::time::Instant::now();
        
        let result = DatastoreInstance::delete_events_by_id_pg(&client, bucket_id, event_ids).await;
        
        self.metrics.record_query("delete_events_by_id", start.elapsed());
        if result.is_err() {
            self.metrics.record_error("delete_events_by_id");
        }
        
        result
    }

    pub async fn force_commit(&self) -> Result<(), DatastoreError> {
        Ok(())
    }

    pub async fn get_key_values(&self, pattern: &str) -> Result<HashMap<String, String>, DatastoreError> {
        let client = self.get_connection().await?;
        let start = std::time::Instant::now();
        
        let result = DatastoreInstance::get_key_values_pg(&client, pattern).await;
        
        self.metrics.record_query("get_key_values", start.elapsed());
        if result.is_err() {
            self.metrics.record_error("get_key_values");
        }
        
        result
    }

    pub async fn get_key_value(&self, key: &str) -> Result<String, DatastoreError> {
        let client = self.get_connection().await?;
        let start = std::time::Instant::now();
        
        let result = DatastoreInstance::get_key_value_pg(&client, key).await;
        
        self.metrics.record_query("get_key_value", start.elapsed());
        if result.is_err() {
            self.metrics.record_error("get_key_value");
        }
        
        result
    }

    pub async fn set_key_value(&self, key: &str, data: &str) -> Result<(), DatastoreError> {
        let client = self.get_connection().await?;
        let start = std::time::Instant::now();
        
        let result = DatastoreInstance::set_key_value_pg(&client, key, data).await;
        
        self.metrics.record_query("set_key_value", start.elapsed());
        if result.is_err() {
            self.metrics.record_error("set_key_value");
        }
        
        result
    }

    pub async fn delete_key_value(&self, key: &str) -> Result<(), DatastoreError> {
        let client = self.get_connection().await?;
        let start = std::time::Instant::now();
        
        let result = DatastoreInstance::delete_key_value_pg(&client, key).await;
        
        self.metrics.record_query("delete_key_value", start.elapsed());
        if result.is_err() {
            self.metrics.record_error("delete_key_value");
        }
        
        result
    }

    pub async fn refresh_privacy_filter(&self) -> Result<(), DatastoreError> {
        let client = self.get_connection().await?;
        
        match DatastoreInstance::get_key_value_pg(&client, "settings.privacy_filters").await {
            Ok(json_str) => match PrivacyFilterEngine::from_json(&json_str) {
                Ok(engine) => {
                    let mut privacy_engine = self.privacy_engine.write().await;
                    *privacy_engine = engine;
                    Ok(())
                }
                Err(e) => {
                    warn!("Failed to parse privacy_filters setting: {}", e);
                    Ok(())
                }
            },
            Err(_) => {
                let mut privacy_engine = self.privacy_engine.write().await;
                *privacy_engine = PrivacyFilterEngine::new(vec![]);
                Ok(())
            }
        }
    }

    pub async fn close(&self) {
        info!("Closing PostgreSQL datastore");
    }

    pub fn get_metrics(&self) -> Arc<DbMetrics> {
        Arc::clone(&self.metrics)
    }

    pub fn get_pool_status(&self) -> (usize, usize) {
        let status = self.pool.status();
        (status.size, status.available.max(0) as usize)
    }
}
