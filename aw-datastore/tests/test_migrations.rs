use std::sync::Arc;
use aw_datastore::migrations::{MigrationError, MigrationManager};
use deadpool_postgres::{Config, Manager, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;

async fn create_test_pool() -> Pool {
    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.port = Some(5432);
    cfg.user = Some("aw_user".to_string());
    cfg.password = Some("activitywatch".to_string());
    cfg.dbname = Some("activitywatch_test_migrations".to_string());
    
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    
    cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap()
}

async fn cleanup_test_db(pool: &Pool) {
    let client = pool.get().await.unwrap();
    
    // Drop all tables
    let _ = client.execute("DROP TABLE IF EXISTS events CASCADE", &[]).await;
    let _ = client.execute("DROP TABLE IF EXISTS buckets CASCADE", &[]).await;
    let _ = client.execute("DROP TABLE IF EXISTS key_value CASCADE", &[]).await;
    let _ = client.execute("DROP TABLE IF EXISTS schema_version CASCADE", &[]).await;
}

#[tokio::test]
#[ignore] // Requires running PostgreSQL instance
async fn test_migration_manager_creation() {
    let pool = create_test_pool().await;
    let manager = MigrationManager::new(Arc::new(pool));
    
    // Manager should be created successfully
    assert!(std::ptr::addr_of!(manager) as usize != 0);
}

#[tokio::test]
#[ignore] // Requires running PostgreSQL instance
async fn test_initial_migration() {
    let pool = create_test_pool().await;
    cleanup_test_db(&pool).await;
    
    let manager = MigrationManager::new(Arc::new(pool.clone()));
    
    // Run migrations
    let result = manager.run_migrations().await;
    assert!(result.is_ok(), "Migration failed: {:?}", result.err());
    
    // Verify tables were created
    let client = pool.get().await.unwrap();
    
    let tables = client
        .query(
            "SELECT table_name FROM information_schema.tables 
             WHERE table_schema = 'public' 
             ORDER BY table_name",
            &[],
        )
        .await
        .unwrap();
    
    let table_names: Vec<String> = tables.iter().map(|row| row.get(0)).collect();
    
    assert!(table_names.contains(&"buckets".to_string()));
    assert!(table_names.contains(&"events".to_string()));
    assert!(table_names.contains(&"key_value".to_string()));
    assert!(table_names.contains(&"schema_version".to_string()));
}

#[tokio::test]
#[ignore] // Requires running PostgreSQL instance
async fn test_schema_version_tracking() {
    let pool = create_test_pool().await;
    cleanup_test_db(&pool).await;
    
    let manager = MigrationManager::new(Arc::new(pool.clone()));
    
    // Run migrations
    manager.run_migrations().await.unwrap();
    
    // Check schema version
    let client = pool.get().await.unwrap();
    let row = client
        .query_one("SELECT MAX(version) FROM schema_version", &[])
        .await
        .unwrap();
    
    let version: i32 = row.get(0);
    assert_eq!(version, 1, "Expected schema version 1");
}

#[tokio::test]
#[ignore] // Requires running PostgreSQL instance
async fn test_buckets_table_structure() {
    let pool = create_test_pool().await;
    cleanup_test_db(&pool).await;
    
    let manager = MigrationManager::new(Arc::new(pool.clone()));
    manager.run_migrations().await.unwrap();
    
    let client = pool.get().await.unwrap();
    
    // Check buckets table columns
    let columns = client
        .query(
            "SELECT column_name, data_type, is_nullable 
             FROM information_schema.columns 
             WHERE table_name = 'buckets' 
             ORDER BY ordinal_position",
            &[],
        )
        .await
        .unwrap();
    
    let column_names: Vec<String> = columns.iter().map(|row| row.get(0)).collect();
    
    assert!(column_names.contains(&"id".to_string()));
    assert!(column_names.contains(&"name".to_string()));
    assert!(column_names.contains(&"type".to_string()));
    assert!(column_names.contains(&"client".to_string()));
    assert!(column_names.contains(&"hostname".to_string()));
    assert!(column_names.contains(&"created".to_string()));
    assert!(column_names.contains(&"data".to_string()));
}

#[tokio::test]
#[ignore] // Requires running PostgreSQL instance
async fn test_events_table_structure() {
    let pool = create_test_pool().await;
    cleanup_test_db(&pool).await;
    
    let manager = MigrationManager::new(Arc::new(pool.clone()));
    manager.run_migrations().await.unwrap();
    
    let client = pool.get().await.unwrap();
    
    // Check events table columns
    let columns = client
        .query(
            "SELECT column_name, data_type 
             FROM information_schema.columns 
             WHERE table_name = 'events' 
             ORDER BY ordinal_position",
            &[],
        )
        .await
        .unwrap();
    
    let column_names: Vec<String> = columns.iter().map(|row| row.get(0)).collect();
    
    assert!(column_names.contains(&"id".to_string()));
    assert!(column_names.contains(&"bucketrow".to_string()));
    assert!(column_names.contains(&"starttime".to_string()));
    assert!(column_names.contains(&"endtime".to_string()));
    assert!(column_names.contains(&"data".to_string()));
}

#[tokio::test]
#[ignore] // Requires running PostgreSQL instance
async fn test_indexes_created() {
    let pool = create_test_pool().await;
    cleanup_test_db(&pool).await;
    
    let manager = MigrationManager::new(Arc::new(pool.clone()));
    manager.run_migrations().await.unwrap();
    
    let client = pool.get().await.unwrap();
    
    // Check indexes exist
    let indexes = client
        .query(
            "SELECT indexname FROM pg_indexes 
             WHERE schemaname = 'public' 
             ORDER BY indexname",
            &[],
        )
        .await
        .unwrap();
    
    let index_names: Vec<String> = indexes.iter().map(|row| row.get(0)).collect();
    
    assert!(index_names.contains(&"idx_buckets_client_hostname".to_string()));
    assert!(index_names.contains(&"idx_events_timerange".to_string()));
    assert!(index_names.contains(&"idx_events_starttime".to_string()));
}

#[tokio::test]
#[ignore] // Requires running PostgreSQL instance
async fn test_foreign_key_constraints() {
    let pool = create_test_pool().await;
    cleanup_test_db(&pool).await;
    
    let manager = MigrationManager::new(Arc::new(pool.clone()));
    manager.run_migrations().await.unwrap();
    
    let client = pool.get().await.unwrap();
    
    // Check foreign key constraint exists
    let constraints = client
        .query(
            "SELECT constraint_name, table_name, constraint_type 
             FROM information_schema.table_constraints 
             WHERE constraint_type = 'FOREIGN KEY' 
             AND table_name = 'events'",
            &[],
        )
        .await
        .unwrap();
    
    assert!(!constraints.is_empty(), "No foreign key constraints found");
    
    let constraint_names: Vec<String> = constraints.iter().map(|row| row.get(0)).collect();
    assert!(constraint_names.contains(&"events_bucketrow_fkey".to_string()));
}

#[tokio::test]
#[ignore] // Requires running PostgreSQL instance
async fn test_idempotent_migrations() {
    let pool = create_test_pool().await;
    cleanup_test_db(&pool).await;
    
    let manager = MigrationManager::new(Arc::new(pool.clone()));
    
    // Run migrations first time
    manager.run_migrations().await.unwrap();
    
    // Run migrations again (should be idempotent)
    let result = manager.run_migrations().await;
    assert!(result.is_ok(), "Second migration run failed: {:?}", result.err());
    
    // Verify still at version 1
    let client = pool.get().await.unwrap();
    let row = client
        .query_one("SELECT MAX(version) FROM schema_version", &[])
        .await
        .unwrap();
    
    let version: i32 = row.get(0);
    assert_eq!(version, 1, "Version should still be 1 after second run");
}

#[tokio::test]
#[ignore] // Requires running PostgreSQL instance
async fn test_is_initialized() {
    let pool = create_test_pool().await;
    cleanup_test_db(&pool).await;
    
    let manager = MigrationManager::new(Arc::new(pool.clone()));
    
    // Should not be initialized before migrations
    assert!(!manager.is_initialized().await);
    
    // Run migrations
    manager.run_migrations().await.unwrap();
    
    // Should be initialized after migrations
    assert!(manager.is_initialized().await);
}

#[test]
fn test_migration_error_types() {
    let db_error = MigrationError::DatabaseError("Connection failed".to_string());
    let migration_error = MigrationError::MigrationFailed("Schema creation failed".to_string());
    
    // Test Display implementation
    assert!(db_error.to_string().contains("Database error"));
    assert!(migration_error.to_string().contains("Migration failed"));
}

#[tokio::test]
#[ignore] // Requires running PostgreSQL instance
async fn test_cascade_delete_behavior() {
    let pool = create_test_pool().await;
    cleanup_test_db(&pool).await;
    
    let manager = MigrationManager::new(Arc::new(pool.clone()));
    manager.run_migrations().await.unwrap();
    
    let client = pool.get().await.unwrap();
    
    // Insert test bucket
    let bucket_id: i32 = client
        .query_one(
            "INSERT INTO buckets (name, type, client, hostname, created, data) 
             VALUES ($1, $2, $3, $4, NOW(), '{}'::jsonb) 
             RETURNING id",
            &[&"test-bucket", &"test", &"test-client", &"test-host"],
        )
        .await
        .unwrap()
        .get(0);
    
    // Insert test event
    client
        .execute(
            "INSERT INTO events (bucketrow, starttime, endtime, data) 
             VALUES ($1, NOW(), NOW() + interval '5 seconds', '{}'::jsonb)",
            &[&bucket_id],
        )
        .await
        .unwrap();
    
    // Verify event exists
    let event_count: i64 = client
        .query_one("SELECT COUNT(*) FROM events WHERE bucketrow = $1", &[&bucket_id])
        .await
        .unwrap()
        .get(0);
    assert_eq!(event_count, 1);
    
    // Delete bucket
    client
        .execute("DELETE FROM buckets WHERE id = $1", &[&bucket_id])
        .await
        .unwrap();
    
    // Verify event was cascade deleted
    let event_count_after: i64 = client
        .query_one("SELECT COUNT(*) FROM events WHERE bucketrow = $1", &[&bucket_id])
        .await
        .unwrap()
        .get(0);
    assert_eq!(event_count_after, 0, "Events should be cascade deleted");
}
