use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use deadpool_postgres::Pool;

/// Health check status
#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy(String),
    Degraded(String),
}

impl HealthStatus {
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }

    pub fn is_unhealthy(&self) -> bool {
        matches!(self, HealthStatus::Unhealthy(_))
    }
}

/// Health checker component with timeout
pub struct HealthChecker {
    pool: Arc<Pool>,
    timeout_duration: Duration,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(pool: Arc<Pool>, timeout_duration: Duration) -> Self {
        HealthChecker {
            pool,
            timeout_duration,
        }
    }

    /// Execute comprehensive health check with timeout
    pub async fn check(&self) -> HealthStatus {
        match timeout(self.timeout_duration, self.check_internal()).await {
            Ok(Ok(status)) => status,
            Ok(Err(e)) => HealthStatus::Unhealthy(format!("Health check failed: {}", e)),
            Err(_) => HealthStatus::Unhealthy(format!(
                "Health check timed out after {:?}",
                self.timeout_duration
            )),
        }
    }

    /// Internal health check logic
    async fn check_internal(&self) -> Result<HealthStatus, Box<dyn std::error::Error>> {
        // Check 1: Get connection from pool
        let client = match self.pool.get().await {
            Ok(client) => client,
            Err(e) => {
                return Ok(HealthStatus::Unhealthy(format!(
                    "Cannot get connection from pool: {}",
                    e
                )));
            }
        };

        // Check 2: Execute simple query to verify connectivity
        match client.query_one("SELECT 1 as health_check", &[]).await {
            Ok(_) => {}
            Err(e) => {
                return Ok(HealthStatus::Unhealthy(format!(
                    "Database query failed: {}",
                    e
                )));
            }
        }

        // Check 3: Verify schema version table exists
        match self.check_schema_version(&client).await {
            Ok(version) => {
                if version > 0 {
                    Ok(HealthStatus::Healthy)
                } else {
                    Ok(HealthStatus::Degraded(
                        "Schema version is 0, database may not be initialized".to_string(),
                    ))
                }
            }
            Err(e) => Ok(HealthStatus::Degraded(format!(
                "Cannot verify schema version: {}",
                e
            ))),
        }
    }

    /// Check database schema version
    async fn check_schema_version(
        &self,
        client: &deadpool_postgres::Client,
    ) -> Result<i32, Box<dyn std::error::Error>> {
        // Check if schema_version table exists
        let table_exists = client
            .query_opt(
                "SELECT 1 FROM information_schema.tables 
                 WHERE table_schema = 'public' AND table_name = 'schema_version'",
                &[],
            )
            .await?;

        if table_exists.is_none() {
            return Ok(0); // Schema not initialized yet
        }

        // Get current schema version
        let row = client
            .query_one("SELECT MAX(version) as version FROM schema_version", &[])
            .await?;

        let version: Option<i32> = row.get("version");
        Ok(version.unwrap_or(0))
    }

    /// Quick liveness check (just connectivity, no schema validation)
    pub async fn liveness_check(&self) -> HealthStatus {
        match timeout(Duration::from_secs(2), self.liveness_check_internal()).await {
            Ok(Ok(_)) => HealthStatus::Healthy,
            Ok(Err(e)) => HealthStatus::Unhealthy(format!("Liveness check failed: {}", e)),
            Err(_) => HealthStatus::Unhealthy("Liveness check timed out".to_string()),
        }
    }

    async fn liveness_check_internal(&self) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.pool.get().await?;
        client.query_one("SELECT 1", &[]).await?;
        Ok(())
    }

    /// Readiness check (comprehensive, for load balancer health checks)
    pub async fn readiness_check(&self) -> HealthStatus {
        self.check().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests would require a running PostgreSQL instance
    // In a real implementation, use testcontainers for integration tests

    #[test]
    fn test_health_status_is_healthy() {
        assert!(HealthStatus::Healthy.is_healthy());
        assert!(!HealthStatus::Unhealthy("error".to_string()).is_healthy());
        assert!(!HealthStatus::Degraded("warning".to_string()).is_healthy());
    }

    #[test]
    fn test_health_status_is_unhealthy() {
        assert!(!HealthStatus::Healthy.is_unhealthy());
        assert!(HealthStatus::Unhealthy("error".to_string()).is_unhealthy());
        assert!(!HealthStatus::Degraded("warning".to_string()).is_unhealthy());
    }
}
