use aw_datastore::metrics::DbMetrics;
use std::time::Duration;

#[test]
fn test_metrics_initialization() {
    let metrics = DbMetrics::new();
    let prometheus = metrics.to_prometheus_format();
    
    // Check that counters start at 0
    assert!(prometheus.contains("aw_db_queries_total 0"));
    assert!(prometheus.contains("aw_db_errors_total 0"));
}

#[test]
fn test_record_query() {
    let metrics = DbMetrics::new();
    
    // Record some queries
    metrics.record_query("get_bucket", Duration::from_millis(10));
    metrics.record_query("get_bucket", Duration::from_millis(20));
    metrics.record_query("insert_events", Duration::from_millis(50));
    
    let prometheus = metrics.to_prometheus_format();
    
    // Check query count increased
    assert!(prometheus.contains("aw_db_queries_total 3"));
}

#[test]
fn test_record_error() {
    let metrics = DbMetrics::new();
    
    // Record some errors
    metrics.record_error("create_bucket");
    metrics.record_error("insert_events");
    metrics.record_error("insert_events");
    
    let prometheus = metrics.to_prometheus_format();
    
    // Check error count increased
    assert!(prometheus.contains("aw_db_errors_total 3"));
}

#[test]
fn test_update_pool_stats() {
    let metrics = DbMetrics::new();
    
    // Simulate pool usage
    metrics.update_pool_stats(5, 15); // 5 active, 15 idle
    
    let prometheus = metrics.to_prometheus_format();
    
    // Check pool stats are recorded
    assert!(prometheus.contains("aw_db_pool_connections_active 5"));
    assert!(prometheus.contains("aw_db_pool_connections_idle 15"));
}

#[test]
fn test_query_duration_percentiles() {
    let metrics = DbMetrics::new();
    
    // Record queries with varying durations
    for i in 1..=100 {
        metrics.record_query("test_query", Duration::from_millis(i));
    }
    
    let prometheus = metrics.to_prometheus_format();
    
    // P50 should be around 50ms
    let p50_line = prometheus
        .lines()
        .find(|l| l.starts_with("aw_db_query_duration_p50_ms"))
        .expect("P50 metric not found");
    
    let p50_value: f64 = p50_line
        .split_whitespace()
        .last()
        .unwrap()
        .parse()
        .unwrap();
    
    assert!(p50_value >= 45.0 && p50_value <= 55.0, "P50 = {}", p50_value);
    
    // P95 should be around 95ms
    let p95_line = prometheus
        .lines()
        .find(|l| l.starts_with("aw_db_query_duration_p95_ms"))
        .expect("P95 metric not found");
    
    let p95_value: f64 = p95_line
        .split_whitespace()
        .last()
        .unwrap()
        .parse()
        .unwrap();
    
    assert!(p95_value >= 90.0 && p95_value <= 100.0, "P95 = {}", p95_value);
}

#[test]
fn test_concurrent_metrics_updates() {
    use std::sync::Arc;
    use std::thread;
    
    let metrics = Arc::new(DbMetrics::new());
    let mut handles = vec![];
    
    // Spawn 10 threads, each recording 100 queries
    for _ in 0..10 {
        let metrics_clone = Arc::clone(&metrics);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                metrics_clone.record_query("concurrent_test", Duration::from_millis(10));
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    let prometheus = metrics.to_prometheus_format();
    
    // Total should be 10 * 100 = 1000
    assert!(prometheus.contains("aw_db_queries_total 1000"));
}

#[test]
fn test_prometheus_format_structure() {
    let metrics = DbMetrics::new();
    
    metrics.record_query("test", Duration::from_millis(5));
    metrics.record_error("test_error");
    metrics.update_pool_stats(3, 7);
    
    let prometheus = metrics.to_prometheus_format();
    
    // Check all expected metrics are present
    assert!(prometheus.contains("# HELP aw_db_queries_total"));
    assert!(prometheus.contains("# TYPE aw_db_queries_total counter"));
    assert!(prometheus.contains("# HELP aw_db_errors_total"));
    assert!(prometheus.contains("# TYPE aw_db_errors_total counter"));
    assert!(prometheus.contains("# HELP aw_db_pool_connections_active"));
    assert!(prometheus.contains("# TYPE aw_db_pool_connections_active gauge"));
    assert!(prometheus.contains("# HELP aw_db_query_duration_p50_ms"));
    assert!(prometheus.contains("# TYPE aw_db_query_duration_p50_ms gauge"));
}

#[test]
fn test_metrics_reset_behavior() {
    let metrics = DbMetrics::new();
    
    // Record some data
    metrics.record_query("test", Duration::from_millis(10));
    metrics.record_error("test_error");
    
    let first_prometheus = metrics.to_prometheus_format();
    assert!(first_prometheus.contains("aw_db_queries_total 1"));
    assert!(first_prometheus.contains("aw_db_errors_total 1"));
    
    // Record more data (metrics should accumulate, not reset)
    metrics.record_query("test", Duration::from_millis(20));
    
    let second_prometheus = metrics.to_prometheus_format();
    assert!(second_prometheus.contains("aw_db_queries_total 2"));
}
