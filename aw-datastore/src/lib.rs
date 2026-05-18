#[macro_use]
extern crate log;

#[macro_export]
macro_rules! json_map {
    { $( $key:literal : $value:expr),* } => {{
        use serde_json::Value;
        use serde_json::map::Map;
        #[allow(unused_mut)]
        let mut map : Map<String, Value> = Map::new();
        $(
          map.insert( $key.to_string(), json!($value) );
        )*
        map
    }};
}

mod datastore;
mod datastore_pg;  // PostgreSQL implementations
mod legacy_import;
mod privacy_filter;
mod worker;

// PostgreSQL-specific modules
pub mod retry;
pub mod metrics;
pub mod health;
pub mod migrations;

pub use self::datastore::DatastoreInstance;
pub use self::worker::{Datastore, DbConfig};
pub use self::retry::RetryPolicy;
pub use self::metrics::DbMetrics;
pub use self::health::{HealthChecker, HealthStatus};
pub use self::migrations::MigrationManager;

#[derive(Debug, Clone)]
pub enum DatastoreMethod {
    Memory(),
    File(String),
}

/* TODO: Implement this as a proper error */
#[derive(Debug, Clone)]
pub enum DatastoreError {
    NoSuchBucket(String),
    BucketAlreadyExists(String),
    NoSuchKey(String),
    MpscError,
    InternalError(String),
    // Errors specific to when migrate is disabled
    Uninitialized(String),
    OldDbVersion(String),
}
