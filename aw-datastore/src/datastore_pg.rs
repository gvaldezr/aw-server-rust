/// PostgreSQL implementations for DatastoreInstance
/// 
/// This module contains async PostgreSQL implementations of all datastore operations.
/// Called by worker.rs which uses deadpool-postgres connection pool.

use std::collections::HashMap;
use chrono::{DateTime, Duration, Utc};

use deadpool_postgres::Client;
use tokio_postgres::Row;

use aw_models::Bucket;
use aw_models::BucketMetadata;
use aw_models::Event;
use serde_json::Map;

use crate::DatastoreError;
use crate::DatastoreInstance;

impl DatastoreInstance {
    /// Create a new bucket (PostgreSQL)
    pub async fn create_bucket_pg(
        client: &Client,
        bucket: &Bucket,
    ) -> Result<(), DatastoreError> {
        use chrono::Utc;
        
        let data_json = serde_json::to_value(&bucket.data)
            .map_err(|e| DatastoreError::InternalError(format!("Failed to serialize bucket data: {}", e)))?;
        
        let created = bucket.created.unwrap_or_else(|| Utc::now());

        client
            .execute(
                "INSERT INTO buckets (name, type, client, hostname, created, data)
                 VALUES ($1, $2, $3, $4, $5, $6)",
                &[
                    &bucket.id,
                    &bucket._type,
                    &bucket.client,
                    &bucket.hostname,
                    &created,
                    &data_json,
                ],
            )
            .await
            .map_err(|e| {
                if e.to_string().contains("duplicate key") || e.to_string().contains("unique constraint") {
                    DatastoreError::BucketAlreadyExists(bucket.id.clone())
                } else {
                    DatastoreError::InternalError(format!("Failed to create bucket: {}", e))
                }
            })?;

        Ok(())
    }

    /// Delete a bucket and all its events (PostgreSQL)
    pub async fn delete_bucket_pg(
        client: &Client,
        bucket_id: &str,
    ) -> Result<(), DatastoreError> {
        let rows_affected = client
            .execute("DELETE FROM buckets WHERE name = $1", &[&bucket_id])
            .await
            .map_err(|e| DatastoreError::InternalError(format!("Failed to delete bucket: {}", e)))?;

        if rows_affected == 0 {
            return Err(DatastoreError::NoSuchBucket(bucket_id.to_string()));
        }

        Ok(())
    }

    /// Get a bucket by ID (PostgreSQL)
    pub async fn get_bucket_pg(
        client: &Client,
        bucket_id: &str,
    ) -> Result<Bucket, DatastoreError> {
        let row = client
            .query_opt(
                "SELECT id, name, type, client, hostname, created, data,
                        (SELECT MIN(starttime) FROM events WHERE bucketrow = buckets.id) as min_start,
                        (SELECT MAX(endtime) FROM events WHERE bucketrow = buckets.id) as max_end
                 FROM buckets WHERE name = $1",
                &[&bucket_id],
            )
            .await
            .map_err(|e| DatastoreError::InternalError(format!("Failed to query bucket: {}", e)))?
            .ok_or_else(|| DatastoreError::NoSuchBucket(bucket_id.to_string()))?;

        parse_bucket_row(&row)
    }

    /// Get all buckets (PostgreSQL)
    pub async fn get_buckets_pg(client: &Client) -> Result<HashMap<String, Bucket>, DatastoreError> {
        let rows = client
            .query(
                "SELECT buckets.id, buckets.name, buckets.type, buckets.client,
                        buckets.hostname, buckets.created, buckets.data,
                        MIN(events.starttime) as min_start,
                        MAX(events.endtime) as max_end
                 FROM buckets
                 LEFT OUTER JOIN events ON buckets.id = events.bucketrow
                 GROUP BY buckets.id, buckets.name, buckets.type, buckets.client,
                          buckets.hostname, buckets.created, buckets.data",
                &[],
            )
            .await
            .map_err(|e| DatastoreError::InternalError(format!("Failed to query buckets: {}", e)))?;

        let mut buckets = HashMap::new();
        for row in rows {
            let bucket = parse_bucket_row(&row)?;
            buckets.insert(bucket.id.clone(), bucket);
        }

        Ok(buckets)
    }

    /// Insert multiple events into a bucket (PostgreSQL)
    pub async fn insert_events_pg(
        client: &Client,
        bucket_id: &str,
        events: Vec<Event>,
    ) -> Result<Vec<Event>, DatastoreError> {
        // Get bucket ID
        let bucket_row: i32 = client
            .query_one("SELECT id FROM buckets WHERE name = $1", &[&bucket_id])
            .await
            .map_err(|_| DatastoreError::NoSuchBucket(bucket_id.to_string()))?
            .get(0);

        // Insert events (PostgreSQL handles concurrent inserts efficiently)
        let mut inserted_events = Vec::new();

        for event in events {
            let data_json = serde_json::to_value(&event.data)
                .map_err(|e| DatastoreError::InternalError(format!("Failed to serialize event data: {}", e)))?;

            let endtime = event.timestamp + event.duration;

            let row = client
                .query_one(
                    "INSERT INTO events (bucketrow, starttime, endtime, data)
                     VALUES ($1, $2, $3, $4)
                     RETURNING id, bucketrow, starttime, endtime, data",
                    &[
                        &bucket_row,
                        &event.timestamp,
                        &endtime,
                        &data_json,
                    ],
                )
                .await
                .map_err(|e| DatastoreError::InternalError(format!("Failed to insert event: {}", e)))?;

            inserted_events.push(parse_event_row(&row)?);
        }

        Ok(inserted_events)
    }

    /// Process heartbeat - merge with last event or create new (PostgreSQL)
    pub async fn heartbeat_pg(
        client: &Client,
        bucket_id: &str,
        event: Event,
        pulsetime: f64,
    ) -> Result<Event, DatastoreError> {
        // Get bucket ID
        let bucket_row: i32 = client
            .query_one("SELECT id FROM buckets WHERE name = $1", &[&bucket_id])
            .await
            .map_err(|_| DatastoreError::NoSuchBucket(bucket_id.to_string()))?
            .get(0);

        let endtime = event.timestamp + event.duration;
        let pulsetime_duration = Duration::milliseconds((pulsetime * 1000.0) as i64);

        // Try to find last event within pulsetime window with matching data
        let data_json = serde_json::to_value(&event.data)
            .map_err(|e| DatastoreError::InternalError(format!("Failed to serialize event data: {}", e)))?;

        let last_event_opt = client
            .query_opt(
                "SELECT id, bucketrow, starttime, endtime, data
                 FROM events
                 WHERE bucketrow = $1
                   AND endtime >= $2
                   AND data = $3
                 ORDER BY endtime DESC
                 LIMIT 1",
                &[
                    &bucket_row,
                    &(event.timestamp - pulsetime_duration),
                    &data_json,
                ],
            )
            .await
            .map_err(|e| DatastoreError::InternalError(format!("Failed to query last event: {}", e)))?;

        if let Some(last_row) = last_event_opt {
            // Merge with existing event
            let event_id: i64 = last_row.get(0);
            let last_endtime: DateTime<Utc> = last_row.get(3);
            
            let new_endtime = if endtime > last_endtime {
                endtime
            } else {
                last_endtime
            };

            let updated_row = client
                .query_one(
                    "UPDATE events SET endtime = $1 WHERE id = $2
                     RETURNING id, bucketrow, starttime, endtime, data",
                    &[&new_endtime, &event_id],
                )
                .await
                .map_err(|e| DatastoreError::InternalError(format!("Failed to update event: {}", e)))?;

            parse_event_row(&updated_row)
        } else {
            // Insert new event
            let row = client
                .query_one(
                    "INSERT INTO events (bucketrow, starttime, endtime, data)
                     VALUES ($1, $2, $3, $4)
                     RETURNING id, bucketrow, starttime, endtime, data",
                    &[&bucket_row, &event.timestamp, &endtime, &data_json],
                )
                .await
                .map_err(|e| DatastoreError::InternalError(format!("Failed to insert heartbeat event: {}", e)))?;

            parse_event_row(&row)
        }
    }

    /// Get a single event by ID (PostgreSQL)
    pub async fn get_event_pg(
        client: &Client,
        bucket_id: &str,
        event_id: i64,
    ) -> Result<Event, DatastoreError> {
        let row = client
            .query_opt(
                "SELECT e.id, e.bucketrow, e.starttime, e.endtime, e.data
                 FROM events e
                 JOIN buckets b ON e.bucketrow = b.id
                 WHERE b.name = $1 AND e.id = $2",
                &[&bucket_id, &event_id],
            )
            .await
            .map_err(|e| DatastoreError::InternalError(format!("Failed to query event: {}", e)))?
            .ok_or_else(|| DatastoreError::InternalError(format!("Event {} not found", event_id)))?;

        parse_event_row(&row)
    }

    /// Get events with optional time filtering (PostgreSQL)
    pub async fn get_events_pg(
        client: &Client,
        bucket_id: &str,
        starttime: Option<DateTime<Utc>>,
        endtime: Option<DateTime<Utc>>,
        limit: Option<u64>,
        unclipped: bool,
    ) -> Result<Vec<Event>, DatastoreError> {
        // Get bucket ID
        let bucket_row: i32 = client
            .query_one("SELECT id FROM buckets WHERE name = $1", &[&bucket_id])
            .await
            .map_err(|_| DatastoreError::NoSuchBucket(bucket_id.to_string()))?
            .get(0);

        // Build query dynamically based on filters
        let mut query = String::from(
            "SELECT id, bucketrow, starttime, endtime, data
             FROM events
             WHERE bucketrow = $1"
        );
        
        let has_starttime = starttime.is_some();
        let has_endtime = endtime.is_some();
        let has_limit = limit.is_some();
        
        let mut param_count = 1;
        
        if has_starttime {
            param_count += 1;
            query.push_str(&format!(" AND endtime >= ${}", param_count));
        }

        if has_endtime {
            param_count += 1;
            query.push_str(&format!(" AND starttime <= ${}", param_count));
        }

        query.push_str(" ORDER BY starttime DESC");

        if has_limit {
            param_count += 1;
            query.push_str(&format!(" LIMIT ${}", param_count));
        }

        // Execute query with appropriate parameters
        let rows = match (starttime, endtime, limit) {
            (None, None, None) => {
                client.query(&query, &[&bucket_row]).await
            }
            (Some(ref start), None, None) => {
                client.query(&query, &[&bucket_row, start]).await
            }
            (None, Some(ref end), None) => {
                client.query(&query, &[&bucket_row, end]).await
            }
            (Some(ref start), Some(ref end), None) => {
                client.query(&query, &[&bucket_row, start, end]).await
            }
            (None, None, Some(lim)) => {
                let lim_i64 = lim as i64;
                client.query(&query, &[&bucket_row, &lim_i64]).await
            }
            (Some(ref start), None, Some(lim)) => {
                let lim_i64 = lim as i64;
                client.query(&query, &[&bucket_row, start, &lim_i64]).await
            }
            (None, Some(ref end), Some(lim)) => {
                let lim_i64 = lim as i64;
                client.query(&query, &[&bucket_row, end, &lim_i64]).await
            }
            (Some(ref start), Some(ref end), Some(lim)) => {
                let lim_i64 = lim as i64;
                client.query(&query, &[&bucket_row, start, end, &lim_i64]).await
            }
        }
        .map_err(|e| DatastoreError::InternalError(format!("Failed to query events: {}", e)))?;

        let mut events = Vec::new();
        for row in rows {
            let mut event = parse_event_row(&row)?;
            
            // Apply time clipping if requested
            if !unclipped {
                if let Some(start) = starttime {
                    if event.timestamp < start {
                        let duration_lost = start - event.timestamp;
                        event.duration = event.duration - duration_lost;
                        event.timestamp = start;
                    }
                }
                
                if let Some(end) = endtime {
                    let event_end = event.timestamp + event.duration;
                    if event_end > end {
                        let duration_lost = event_end - end;
                        event.duration = event.duration - duration_lost;
                    }
                }
            }
            
            events.push(event);
        }

        Ok(events)
    }

    /// Get count of events in time range (PostgreSQL)
    pub async fn get_event_count_pg(
        client: &Client,
        bucket_id: &str,
        starttime: Option<DateTime<Utc>>,
        endtime: Option<DateTime<Utc>>,
    ) -> Result<i64, DatastoreError> {
        // Get bucket ID
        let bucket_row: i32 = client
            .query_one("SELECT id FROM buckets WHERE name = $1", &[&bucket_id])
            .await
            .map_err(|_| DatastoreError::NoSuchBucket(bucket_id.to_string()))?
            .get(0);

        // Build query dynamically based on filters
        let mut query = String::from(
            "SELECT COUNT(*) FROM events WHERE bucketrow = $1"
        );
        
        let has_starttime = starttime.is_some();
        let has_endtime = endtime.is_some();
        
        let mut param_count = 1;

        if has_starttime {
            param_count += 1;
            query.push_str(&format!(" AND endtime >= ${}", param_count));
        }

        if has_endtime {
            param_count += 1;
            query.push_str(&format!(" AND starttime <= ${}", param_count));
        }

        // Execute query with appropriate parameters
        let row = match (starttime, endtime) {
            (None, None) => {
                client.query_one(&query, &[&bucket_row]).await
            }
            (Some(ref start), None) => {
                client.query_one(&query, &[&bucket_row, start]).await
            }
            (None, Some(ref end)) => {
                client.query_one(&query, &[&bucket_row, end]).await
            }
            (Some(ref start), Some(ref end)) => {
                client.query_one(&query, &[&bucket_row, start, end]).await
            }
        }
        .map_err(|e| DatastoreError::InternalError(format!("Failed to count events: {}", e)))?;

        Ok(row.get(0))
    }

    /// Delete events by their IDs (PostgreSQL)
    pub async fn delete_events_by_id_pg(
        client: &Client,
        bucket_id: &str,
        event_ids: Vec<i64>,
    ) -> Result<(), DatastoreError> {
        if event_ids.is_empty() {
            return Ok(());
        }

        // Get bucket ID for validation
        let bucket_row: i32 = client
            .query_one("SELECT id FROM buckets WHERE name = $1", &[&bucket_id])
            .await
            .map_err(|_| DatastoreError::NoSuchBucket(bucket_id.to_string()))?
            .get(0);

        client
            .execute(
                "DELETE FROM events WHERE bucketrow = $1 AND id = ANY($2)",
                &[&bucket_row, &event_ids],
            )
            .await
            .map_err(|e| DatastoreError::InternalError(format!("Failed to delete events: {}", e)))?;

        Ok(())
    }

    /// Get key-value pairs matching a pattern (PostgreSQL)
    pub async fn get_key_values_pg(
        client: &Client,
        pattern: &str,
    ) -> Result<HashMap<String, String>, DatastoreError> {
        let rows = client
            .query(
                "SELECT key, value FROM key_value WHERE key LIKE $1",
                &[&pattern],
            )
            .await
            .map_err(|e| DatastoreError::InternalError(format!("Failed to query key-values: {}", e)))?;

        let mut result = HashMap::new();
        for row in rows {
            let key: String = row.get(0);
            let value: String = row.get(1);
            result.insert(key, value);
        }

        Ok(result)
    }

    /// Get a single key-value (PostgreSQL)
    pub async fn get_key_value_pg(client: &Client, key: &str) -> Result<String, DatastoreError> {
        let row = client
            .query_opt("SELECT value FROM key_value WHERE key = $1", &[&key])
            .await
            .map_err(|e| DatastoreError::InternalError(format!("Failed to query key-value: {}", e)))?
            .ok_or_else(|| DatastoreError::NoSuchKey(key.to_string()))?;

        Ok(row.get(0))
    }

    /// Set a key-value pair (PostgreSQL)
    pub async fn set_key_value_pg(
        client: &Client,
        key: &str,
        value: &str,
    ) -> Result<(), DatastoreError> {
        client
            .execute(
                "INSERT INTO key_value (key, value)
                 VALUES ($1, $2)
                 ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value",
                &[&key, &value],
            )
            .await
            .map_err(|e| DatastoreError::InternalError(format!("Failed to set key-value: {}", e)))?;

        Ok(())
    }

    /// Delete a key-value pair (PostgreSQL)
    pub async fn delete_key_value_pg(client: &Client, key: &str) -> Result<(), DatastoreError> {
        let rows_affected = client
            .execute("DELETE FROM key_value WHERE key = $1", &[&key])
            .await
            .map_err(|e| DatastoreError::InternalError(format!("Failed to delete key-value: {}", e)))?;

        if rows_affected == 0 {
            return Err(DatastoreError::NoSuchKey(key.to_string()));
        }

        Ok(())
    }
}

/// Helper: Parse bucket from PostgreSQL row
fn parse_bucket_row(row: &Row) -> Result<Bucket, DatastoreError> {
    let data_json: serde_json::Value = row.get(6);
    
    let data_map: Map<String, serde_json::Value> = match data_json {
        serde_json::Value::Object(map) => map,
        _ => Map::new(),  // Default to empty map if not an object
    };

    let opt_start: Option<DateTime<Utc>> = row.get(7);
    let opt_end: Option<DateTime<Utc>> = row.get(8);

    Ok(Bucket {
        bid: Some(row.get::<_, i32>(0) as i64),  // SERIAL is i32, convert to i64
        id: row.get(1),
        _type: row.get(2),
        client: row.get(3),
        hostname: row.get(4),
        created: Some(row.get(5)),  // Database has NOT NULL, so this is always Some
        data: data_map,
        metadata: BucketMetadata {
            start: opt_start,
            end: opt_end,
        },
        events: None,
        last_updated: None,
    })
}

/// Helper: Parse event from PostgreSQL row
fn parse_event_row(row: &Row) -> Result<Event, DatastoreError> {
    let starttime: DateTime<Utc> = row.get(2);
    let endtime: DateTime<Utc> = row.get(3);
    let duration = endtime - starttime;

    let data_json: serde_json::Value = row.get(4);
    let data_map: Map<String, serde_json::Value> = match data_json {
        serde_json::Value::Object(map) => map,
        _ => return Err(DatastoreError::InternalError("Event data is not a JSON object".to_string())),
    };

    Ok(Event {
        id: Some(row.get(0)),
        timestamp: starttime,
        duration,
        data: data_map,
    })
}
