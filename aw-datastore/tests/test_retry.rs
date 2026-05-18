/// Integration tests for RetryPolicy module
/// 
/// Tests validate exponential backoff retry logic, transient error classification,
/// max attempts enforcement, and jitter randomization.

use aw_datastore::RetryPolicy;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[tokio::test]
async fn test_retry_transient_error_succeeds() {
    // Test that retry succeeds after transient errors
    let policy = RetryPolicy {
        max_attempts: 5,
        initial_delay_ms: 10,
        max_delay_ms: 100,
    };

    let attempt_counter = Arc::new(AtomicU32::new(0));
    let counter_clone = Arc::clone(&attempt_counter);

    let result = policy
        .execute(|| async {
            let attempt = counter_clone.fetch_add(1, Ordering::SeqCst) + 1;
            
            // Fail first 3 attempts with transient errors
            if attempt < 4 {
                Err("connection timeout: database unavailable")
            } else {
                Ok("success")
            }
        })
        .await;

    assert_eq!(result, Ok("success"));
    assert_eq!(attempt_counter.load(Ordering::SeqCst), 4);
}

#[tokio::test]
async fn test_retry_permanent_error_fails_immediately() {
    // Test that permanent errors fail immediately without retry
    let policy = RetryPolicy {
        max_attempts: 5,
        initial_delay_ms: 10,
        max_delay_ms: 100,
    };

    let attempt_counter = Arc::new(AtomicU32::new(0));
    let counter_clone = Arc::clone(&attempt_counter);

    let result = policy
        .execute(|| async {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Err::<(), _>("authentication failed: invalid credentials")
        })
        .await;

    // Should fail immediately (1 attempt only)
    assert!(result.is_err());
    assert_eq!(attempt_counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn test_retry_max_attempts_exceeded() {
    // Test that retry stops after max_attempts
    let policy = RetryPolicy {
        max_attempts: 3,
        initial_delay_ms: 10,
        max_delay_ms: 100,
    };

    let attempt_counter = Arc::new(AtomicU32::new(0));
    let counter_clone = Arc::clone(&attempt_counter);

    let result = policy
        .execute(|| async {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Err::<(), _>("connection timeout: database unavailable")
        })
        .await;

    // Should fail after exactly max_attempts
    assert!(result.is_err());
    assert_eq!(attempt_counter.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_exponential_backoff() {
    // Test that retry delays increase exponentially
    let policy = RetryPolicy {
        max_attempts: 4,
        initial_delay_ms: 50,
        max_delay_ms: 5000,
    };

    let attempt_counter = Arc::new(AtomicU32::new(0));
    let counter_clone = Arc::clone(&attempt_counter);
    
    let start = Instant::now();

    let result = policy
        .execute(|| async {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Err::<(), _>("connection timeout: database unavailable")
        })
        .await;

    let elapsed = start.elapsed();

    // Expected delays (with jitter, delays vary):
    // Attempt 1: immediate
    // Attempt 2: ~50ms (100ms * 2^0 = 50ms base, with ±25% jitter = 37-62ms)
    // Attempt 3: ~100ms (100ms * 2^1 = 100ms base, with ±25% jitter = 75-125ms)
    // Attempt 4: ~200ms (100ms * 2^2 = 200ms base, with ±25% jitter = 150-250ms)
    // Total: ~350ms minimum (with jitter variance)

    assert!(result.is_err());
    assert_eq!(attempt_counter.load(Ordering::SeqCst), 4);
    
    // Verify total elapsed time is in expected range (accounting for jitter)
    // Lower bound: 37 + 75 + 150 = 262ms
    // Upper bound: 62 + 125 + 250 = 437ms
    assert!(elapsed >= Duration::from_millis(200), "Elapsed: {:?}", elapsed);
    assert!(elapsed <= Duration::from_millis(500), "Elapsed: {:?}", elapsed);
}

#[tokio::test]
async fn test_jitter_variation() {
    // Test that jitter prevents thundering herd
    let policy = RetryPolicy {
        max_attempts: 2,
        initial_delay_ms: 100,
        max_delay_ms: 500,
    };

    // Run multiple retry sequences and verify delays vary
    let mut delays = Vec::new();

    for _ in 0..5 {
        let start = Instant::now();
        
        let _ = policy
            .execute(|| async {
                Err::<(), _>("connection timeout: database unavailable")
            })
            .await;

        delays.push(start.elapsed());
    }

    // Verify that not all delays are identical (jitter is working)
    let first_delay = delays[0];
    let all_same = delays.iter().all(|d| *d == first_delay);
    
    assert!(
        !all_same,
        "All delays should not be identical due to jitter. Delays: {:?}",
        delays
    );

    // Verify delays are within expected range
    // With 100ms base delay and ±25% jitter:
    // Expected range: 75-125ms
    for delay in &delays {
        assert!(
            *delay >= Duration::from_millis(50) && *delay <= Duration::from_millis(150),
            "Delay {:?} outside expected range",
            delay
        );
    }
}

#[tokio::test]
async fn test_max_delay_capping() {
    // Test that delays are capped at max_delay_ms
    let policy = RetryPolicy {
        max_attempts: 10,
        initial_delay_ms: 100,
        max_delay_ms: 200, // Cap at 200ms
    };

    let attempt_counter = Arc::new(AtomicU32::new(0));
    let counter_clone = Arc::clone(&attempt_counter);
    
    let start = Instant::now();

    let _ = policy
        .execute(|| async {
            let attempt = counter_clone.fetch_add(1, Ordering::SeqCst) + 1;
            
            // Fail all attempts
            Err::<(), _>(format!("connection timeout on attempt {}", attempt))
        })
        .await;

    let elapsed = start.elapsed();

    // With max_delay_ms = 200ms, delays should be:
    // Attempt 1: immediate
    // Attempt 2: 100ms * 2^0 = 100ms (capped: 100ms)
    // Attempt 3: 100ms * 2^1 = 200ms (capped: 200ms)
    // Attempt 4: 100ms * 2^2 = 400ms → capped to 200ms
    // Attempt 5: 100ms * 2^3 = 800ms → capped to 200ms
    // ... remaining attempts: capped to 200ms
    
    // Expected minimum total: 100 + 200 + 200 + 200 + 200 + 200 + 200 + 200 + 200 = 1600ms
    // With jitter (±25%), expect ~1200-2000ms range
    
    assert_eq!(attempt_counter.load(Ordering::SeqCst), 10);
    assert!(elapsed >= Duration::from_millis(1000), "Elapsed: {:?}", elapsed);
    assert!(elapsed <= Duration::from_millis(2500), "Elapsed: {:?}", elapsed);
}

#[tokio::test]
async fn test_deadlock_error_is_transient() {
    // Test that deadlock errors are classified as transient
    let policy = RetryPolicy {
        max_attempts: 3,
        initial_delay_ms: 10,
        max_delay_ms: 100,
    };

    let attempt_counter = Arc::new(AtomicU32::new(0));
    let counter_clone = Arc::clone(&attempt_counter);

    let result = policy
        .execute(|| async {
            let attempt = counter_clone.fetch_add(1, Ordering::SeqCst) + 1;
            
            if attempt < 3 {
                Err("deadlock detected: retry transaction")
            } else {
                Ok(())
            }
        })
        .await;

    // Deadlock should be retried
    assert!(result.is_ok());
    assert_eq!(attempt_counter.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_serialization_error_is_transient() {
    // Test that serialization errors are classified as transient
    let policy = RetryPolicy {
        max_attempts: 3,
        initial_delay_ms: 10,
        max_delay_ms: 100,
    };

    let attempt_counter = Arc::new(AtomicU32::new(0));
    let counter_clone = Arc::clone(&attempt_counter);

    let result = policy
        .execute(|| async {
            let attempt = counter_clone.fetch_add(1, Ordering::SeqCst) + 1;
            
            if attempt < 3 {
                Err("serialization failure: concurrent update")
            } else {
                Ok(())
            }
        })
        .await;

    // Serialization error should be retried
    assert!(result.is_ok());
    assert_eq!(attempt_counter.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_pool_timeout_is_transient() {
    // Test that pool timeout errors are classified as transient
    let policy = RetryPolicy {
        max_attempts: 3,
        initial_delay_ms: 10,
        max_delay_ms: 100,
    };

    let attempt_counter = Arc::new(AtomicU32::new(0));
    let counter_clone = Arc::clone(&attempt_counter);

    let result = policy
        .execute(|| async {
            let attempt = counter_clone.fetch_add(1, Ordering::SeqCst) + 1;
            
            if attempt < 3 {
                Err("PoolTimeout: failed to get connection from pool")
            } else {
                Ok(())
            }
        })
        .await;

    // Pool timeout should be retried
    assert!(result.is_ok());
    assert_eq!(attempt_counter.load(Ordering::SeqCst), 3);
}
