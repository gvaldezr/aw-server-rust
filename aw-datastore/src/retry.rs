use std::time::Duration;
use std::fmt::Debug;
use tokio::time::sleep;
use rand::Rng;

/// Exponential backoff retry handler for transient database errors
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        RetryPolicy {
            max_attempts: 5,
            initial_delay_ms: 100,
            max_delay_ms: 5000,
        }
    }
}

impl RetryPolicy {
    /// Execute an async operation with exponential backoff retry on transient errors
    pub async fn execute<F, Fut, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: Debug,
    {
        let mut attempt = 0;
        let mut rng = rand::thread_rng();

        loop {
            attempt += 1;

            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if attempt >= self.max_attempts {
                        log::error!("Operation failed after {} attempts: {:?}", attempt, error);
                        return Err(error);
                    }

                    if !Self::is_transient_error(&error) {
                        log::warn!("Non-transient error, not retrying: {:?}", error);
                        return Err(error);
                    }

                    // Calculate delay with exponential backoff
                    let base_delay = self.initial_delay_ms * (2_u64.pow(attempt - 1));
                    let delay = std::cmp::min(base_delay, self.max_delay_ms);

                    // Add jitter (±25% random variation)
                    let jitter_range = delay / 4;
                    let jitter = rng.gen_range(0..=jitter_range * 2);
                    let final_delay = delay.saturating_sub(jitter_range) + jitter;

                    log::warn!(
                        "Transient error on attempt {}/{}: {:?}. Retrying in {}ms",
                        attempt,
                        self.max_attempts,
                        error,
                        final_delay
                    );

                    sleep(Duration::from_millis(final_delay)).await;
                }
            }
        }
    }

    /// Classify PostgreSQL errors as transient or permanent
    fn is_transient_error<E: Debug>(error: &E) -> bool {
        let error_str = format!("{:?}", error);

        // Check for common PostgreSQL transient error patterns
        // Connection errors
        if error_str.contains("connection") && error_str.contains("refused") {
            return true;
        }
        if error_str.contains("connection") && error_str.contains("timeout") {
            return true;
        }
        if error_str.contains("broken pipe") {
            return true;
        }

        // PostgreSQL-specific error codes (would need tokio-postgres Error type for precise matching)
        // 40001 - serialization_failure
        // 40P01 - deadlock_detected
        // 08006 - connection_failure
        // 08003 - connection_does_not_exist
        // 08000 - connection_exception
        // 53300 - too_many_connections
        // 57P03 - cannot_connect_now

        if error_str.contains("serialization") {
            return true;
        }
        if error_str.contains("deadlock") {
            return true;
        }
        if error_str.contains("too many connections") {
            return true;
        }

        // Pool-specific errors
        if error_str.contains("pool timeout") || error_str.contains("PoolTimeout") {
            return true;
        }

        // Default to non-transient (safer to not retry by default)
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_succeeds_after_failures() {
        let policy = RetryPolicy {
            max_attempts: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
        };

        let attempt_count = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&attempt_count);
        
        let result = policy
            .execute(|| async {
                let attempt = counter_clone.fetch_add(1, Ordering::SeqCst) + 1;
                if attempt < 3 {
                    Err("connection timeout")
                } else {
                    Ok(42)
                }
            })
            .await;

        assert_eq!(result, Ok(42));
        assert_eq!(attempt_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_fails_on_max_attempts() {
        let policy = RetryPolicy {
            max_attempts: 2,
            initial_delay_ms: 10,
            max_delay_ms: 100,
        };

        let attempt_count = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&attempt_count);
        
        let result = policy
            .execute(|| async {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>("connection timeout")
            })
            .await;

        assert!(result.is_err());
        assert_eq!(attempt_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_non_transient_error_no_retry() {
        let policy = RetryPolicy::default();

        let attempt_count = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&attempt_count);
        
        let result = policy
            .execute(|| async {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>("syntax error")
            })
            .await;

        assert!(result.is_err());
        assert_eq!(attempt_count.load(Ordering::SeqCst), 1); // No retry
    }
}
