use aw_datastore::{Datastore, DbConfig};
use aw_models::{Bucket, BucketMetadata, Event};
use chrono::{Duration, Utc};
use serde_json::Map;

// Helper to create test database config
fn create_test_config() -> DbConfig {
    DbConfig {
        host: "localhost".to_string(),
        port: 5432,
        user: "aw_user".to_string(),
        password: "activitywatch".to_string(),
        database: "activitywatch_test".to_string(),
    }
}

// Helper to create test bucket
fn create_test_bucket(id: &str) -> Bucket {
    Bucket {
        bid: None,
        id: id.to_string(),
        _type: "test".to_string(),
        client: "test-client".to_string(),
        hostname: "test-host".to_string(),
        created: Some(Utc::now()),
        data: Map::new(),
        metadata: BucketMetadata::default(),
        events: None,
        last_updated: None,
    }
}

// Helper to create test event
fn create_test_event(data: serde_json::Value) -> Event {
    let now = Utc::now();
    Event {
        id: None,
        timestamp: now,
        duration: Duration::seconds(5),
        data: data.as_object().unwrap().clone(),
    }
}

#[tokio::test]
#[ignore] // Requires PostgreSQL instance
async fn test_bucket_lifecycle() {
    let config = create_test_config();
    
    // Initialize datastore (false = no legacy import)
    let datastore = Datastore::new_with_config(config, false)
        .await
        .expect("Failed to initialize datastore");
    
    // Create bucket
    let bucket = create_test_bucket("test-bucket-lifecycle");
    datastore
        .create_bucket(&bucket)
        .await
        .expect("Failed to create bucket");
    
    // Verify bucket exists
    let retrieved_bucket = datastore
        .get_bucket("test-bucket-lifecycle")
        .await
        .expect("Failed to get bucket");
    assert_eq!(retrieved_bucket.id, "test-bucket-lifecycle");
    
    // Delete bucket
    datastore
        .delete_bucket("test-bucket-lifecycle")
        .await
        .expect("Failed to delete bucket");
    
    // Verify bucket is gone
    let result = datastore.get_bucket("test-bucket-lifecycle").await;
    assert!(result.is_err());
}

#[tokio::test]
#[ignore] // Requires PostgreSQL instance
async fn test_event_operations() {
    let config = create_test_config();
    
    let datastore = Datastore::new_with_config(config, false)
        .await
        .expect("Failed to initialize datastore");
    
    // Create bucket
    let bucket = create_test_bucket("test-bucket-events");
    datastore
        .create_bucket(&bucket)
        .await
        .expect("Failed to create bucket");
    
    // Insert event
    let event = create_test_event(serde_json::json!({"app": "Chrome", "title": "Test"}));
    datastore
        .insert_events("test-bucket-events", vec![event.clone()])
        .await
        .expect("Failed to insert event");
    
    // Retrieve events
    let events = datastore
        .get_events("test-bucket-events", None, None, None)
        .await
        .expect("Failed to get events");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data.get("app"), Some(&serde_json::json!("Chrome")));
    
    // Cleanup
    datastore
        .delete_bucket("test-bucket-events")
        .await
        .expect("Failed to delete bucket");
}

#[tokio::test]
#[ignore] // Requires PostgreSQL instance
async fn test_heartbeat_merge() {
    let config = create_test_config();
    
    let datastore = Datastore::new_with_config(config, false)
        .await
        .expect("Failed to initialize datastore");
    
    // Create bucket
    let bucket = create_test_bucket("test-bucket-heartbeat");
    datastore
        .create_bucket(&bucket)
        .await
        .expect("Failed to create bucket");
    
    // Insert first heartbeat
    let event1 = create_test_event(serde_json::json!({"app": "Chrome"}));
    datastore
        .heartbeat("test-bucket-heartbeat", event1, 10.0)
        .await
        .expect("Failed to insert first heartbeat");
    
    // Insert second heartbeat (should merge with first)
    let event2 = create_test_event(serde_json::json!({"app": "Chrome"}));
    datastore
        .heartbeat("test-bucket-heartbeat", event2, 10.0)
        .await
        .expect("Failed to insert second heartbeat");
    
    // Should still have only 1 event (merged)
    let events = datastore
        .get_events("test-bucket-heartbeat", None, None, None)
        .await
        .expect("Failed to get events");
    assert_eq!(events.len(), 1);
    
    // Cleanup
    datastore
        .delete_bucket("test-bucket-heartbeat")
        .await
        .expect("Failed to delete bucket");
}
