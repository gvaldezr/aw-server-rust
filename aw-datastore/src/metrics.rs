use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Prometheus-style metrics for database operations
#[derive(Clone)]
pub struct DbMetrics {
    inner: Arc<DbMetricsInner>,
}

struct DbMetricsInner {
    // Counters
    queries_total: AtomicU64,
    errors_total: AtomicU64,

    // Gauges
    pool_connections_active: AtomicU64,
    pool_connections_idle: AtomicU64,

    // Histogram buckets for query duration (simplified)
    query_duration_p50_ms: AtomicU64,
    query_duration_p95_ms: AtomicU64,
    query_duration_p99_ms: AtomicU64,
}

impl DbMetrics {
    pub fn new() -> Self {
        DbMetrics {
            inner: Arc::new(DbMetricsInner {
                queries_total: AtomicU64::new(0),
                errors_total: AtomicU64::new(0),
                pool_connections_active: AtomicU64::new(0),
                pool_connections_idle: AtomicU64::new(0),
                query_duration_p50_ms: AtomicU64::new(0),
                query_duration_p95_ms: AtomicU64::new(0),
                query_duration_p99_ms: AtomicU64::new(0),
            }),
        }
    }

    /// Record a database query execution
    pub fn record_query(&self, operation: &str, duration: Duration) {
        self.inner.queries_total.fetch_add(1, Ordering::Relaxed);

        let duration_ms = duration.as_millis() as u64;

        // Update histogram approximations (simplified percentile tracking)
        // In production, use a proper histogram library like prometheus crate
        log::trace!("Query '{}' completed in {}ms", operation, duration_ms);

        // Simple running average approach (not true percentiles, but demonstrates concept)
        let current_p50 = self.inner.query_duration_p50_ms.load(Ordering::Relaxed);
        let new_p50 = (current_p50 + duration_ms) / 2;
        self.inner
            .query_duration_p50_ms
            .store(new_p50, Ordering::Relaxed);

        if duration_ms > current_p50 {
            let current_p95 = self.inner.query_duration_p95_ms.load(Ordering::Relaxed);
            let new_p95 = (current_p95 + duration_ms) / 2;
            self.inner
                .query_duration_p95_ms
                .store(new_p95, Ordering::Relaxed);
        }

        if duration_ms > self.inner.query_duration_p95_ms.load(Ordering::Relaxed) {
            let current_p99 = self.inner.query_duration_p99_ms.load(Ordering::Relaxed);
            let new_p99 = (current_p99 + duration_ms) / 2;
            self.inner
                .query_duration_p99_ms.store(new_p99, Ordering::Relaxed);
        }
    }

    /// Update connection pool statistics
    pub fn update_pool_stats(&self, active: usize, idle: usize) {
        self.inner
            .pool_connections_active
            .store(active as u64, Ordering::Relaxed);
        self.inner
            .pool_connections_idle
            .store(idle as u64, Ordering::Relaxed);
    }

    /// Record a database error
    pub fn record_error(&self, error_type: &str) {
        self.inner.errors_total.fetch_add(1, Ordering::Relaxed);
        log::warn!("Database error recorded: {}", error_type);
    }

    /// Get total query count
    pub fn get_queries_total(&self) -> u64 {
        self.inner.queries_total.load(Ordering::Relaxed)
    }

    /// Get total error count
    pub fn get_errors_total(&self) -> u64 {
        self.inner.errors_total.load(Ordering::Relaxed)
    }

    /// Get active connection count
    pub fn get_pool_connections_active(&self) -> u64 {
        self.inner
            .pool_connections_active
            .load(Ordering::Relaxed)
    }

    /// Get idle connection count
    pub fn get_pool_connections_idle(&self) -> u64 {
        self.inner.pool_connections_idle.load(Ordering::Relaxed)
    }

    /// Get query duration percentiles (approximations)
    pub fn get_query_duration_percentiles(&self) -> (u64, u64, u64) {
        (
            self.inner.query_duration_p50_ms.load(Ordering::Relaxed),
            self.inner.query_duration_p95_ms.load(Ordering::Relaxed),
            self.inner.query_duration_p99_ms.load(Ordering::Relaxed),
        )
    }

    /// Format metrics as Prometheus exposition format
    pub fn to_prometheus_format(&self) -> String {
        format!(
            "# HELP db_queries_total Total number of database queries executed\n\
             # TYPE db_queries_total counter\n\
             db_queries_total {}\n\
             \n\
             # HELP db_errors_total Total number of database errors\n\
             # TYPE db_errors_total counter\n\
             db_errors_total {}\n\
             \n\
             # HELP db_pool_connections_active Number of active pool connections\n\
             # TYPE db_pool_connections_active gauge\n\
             db_pool_connections_active {}\n\
             \n\
             # HELP db_pool_connections_idle Number of idle pool connections\n\
             # TYPE db_pool_connections_idle gauge\n\
             db_pool_connections_idle {}\n\
             \n\
             # HELP db_query_duration_ms Query duration percentiles in milliseconds\n\
             # TYPE db_query_duration_ms gauge\n\
             db_query_duration_ms{{quantile=\"0.5\"}} {}\n\
             db_query_duration_ms{{quantile=\"0.95\"}} {}\n\
             db_query_duration_ms{{quantile=\"0.99\"}} {}\n",
            self.get_queries_total(),
            self.get_errors_total(),
            self.get_pool_connections_active(),
            self.get_pool_connections_idle(),
            self.inner.query_duration_p50_ms.load(Ordering::Relaxed),
            self.inner.query_duration_p95_ms.load(Ordering::Relaxed),
            self.inner.query_duration_p99_ms.load(Ordering::Relaxed),
        )
    }
}

impl Default for DbMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_counter_increments() {
        let metrics = DbMetrics::new();
        assert_eq!(metrics.get_queries_total(), 0);

        metrics.record_query("SELECT", Duration::from_millis(10));
        assert_eq!(metrics.get_queries_total(), 1);

        metrics.record_query("INSERT", Duration::from_millis(20));
        assert_eq!(metrics.get_queries_total(), 2);
    }

    #[test]
    fn test_error_counter_increments() {
        let metrics = DbMetrics::new();
        assert_eq!(metrics.get_errors_total(), 0);

        metrics.record_error("connection_error");
        assert_eq!(metrics.get_errors_total(), 1);

        metrics.record_error("query_error");
        assert_eq!(metrics.get_errors_total(), 2);
    }

    #[test]
    fn test_pool_stats_updated() {
        let metrics = DbMetrics::new();
        assert_eq!(metrics.get_pool_connections_active(), 0);
        assert_eq!(metrics.get_pool_connections_idle(), 0);

        metrics.update_pool_stats(5, 10);
        assert_eq!(metrics.get_pool_connections_active(), 5);
        assert_eq!(metrics.get_pool_connections_idle(), 10);
    }

    #[test]
    fn test_prometheus_format() {
        let metrics = DbMetrics::new();
        metrics.record_query("SELECT", Duration::from_millis(10));
        metrics.update_pool_stats(3, 7);

        let output = metrics.to_prometheus_format();
        assert!(output.contains("db_queries_total 1"));
        assert!(output.contains("db_pool_connections_active 3"));
        assert!(output.contains("db_pool_connections_idle 7"));
    }
}
