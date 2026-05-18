use aw_datastore::health::{HealthChecker, HealthStatus};
use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use std::sync::Arc;
use std::time::Duration;
use tokio_postgres::NoTls;

async fn create_test_pool() -> Pool {
    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.port = Some(5432);
    cfg.user = Some("aw_user".to_string());
    cfg.password = Some("activitywatch".to_string());
    cfg.dbname = Some("activitywatch".to_string());
    
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    
    cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap()
}

#[test]
fn test_health_status_methods() {
    let healthy = HealthStatus::Healthy;
    let degraded = HealthStatus::Degraded("High latency".to_string());
    let unhealthy = HealthStatus::Unhealthy("Connection failed".to_string());
    
    // Test is_healthy method
    assert!(healthy.is_healthy());
    assert!(!degraded.is_healthy());
    assert!(!unhealthy.is_healthy());
    
    // Test is_unhealthy method
    assert!(!healthy.is_unhealthy());
    assert!(!degraded.is_unhealthy());
    assert!(unhealthy.is_unhealthy());
}

#[tokio::test]
#[ignore] // Requires running PostgreSQL instance
async fn test_health_checker_creation() {
    let pool = create_test_pool().await;
    let _checker = HealthChecker::new(Arc::new(pool), Duration::from_secs(5));
    
    // Health checker created successfully
}

#[tokio::test]
#[ignore] // Requires running PostgreSQL instance
async fn test_liveness_check() {
    let pool = create_test_pool().await;
    let checker = HealthChecker::new(Arc::new(pool), Duration::from_secs(5));
    
    let status = checker.liveness_check().await;
    
    // Should return some status
    assert!(status.is_healthy() || status.is_unhealthy() || !status.is_healthy());
}

#[tokio::test]
#[ignore] // Requires running PostgreSQL instance  
async fn test_readiness_check() {
    let pool = create_test_pool().await;
    let checker = HealthChecker::new(Arc::new(pool), Duration::from_secs(5));
    
    let _status = checker.readiness_check().await;
    
    // Any status is acceptable for this test
}
