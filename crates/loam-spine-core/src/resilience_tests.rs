// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[test]
fn circuit_breaker_closed_initially() {
    let cb = CircuitBreaker::new(CircuitBreakerConfig::default());
    assert_eq!(cb.state(), CircuitState::Closed);
    assert!(cb.can_execute());
}

#[test]
fn circuit_breaker_opens_after_threshold() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        recovery_timeout_secs: 60,
        success_threshold: 2,
    };
    let cb = CircuitBreaker::new(config);

    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Closed);
    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Closed);
    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Open);
    assert!(!cb.can_execute());
}

#[test]
fn circuit_breaker_half_open_closes_after_success_threshold() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        recovery_timeout_secs: 0,
        success_threshold: 2,
    };
    let cb = CircuitBreaker::new(config);
    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Open);
    assert!(cb.can_execute());
    assert_eq!(cb.state(), CircuitState::HalfOpen);

    cb.record_success();
    assert_eq!(cb.state(), CircuitState::HalfOpen);
    cb.record_success();
    assert_eq!(cb.state(), CircuitState::Closed);
}

#[test]
fn circuit_breaker_success_resets_failures_in_closed() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        recovery_timeout_secs: 60,
        success_threshold: 2,
    };
    let cb = CircuitBreaker::new(config);

    cb.record_failure();
    cb.record_failure();
    cb.record_success();
    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Closed);
}

#[test]
fn retry_policy_exponential_backoff() {
    let policy = RetryPolicy::default_policy();
    let d0 = policy.exponential_backoff(0);
    let d1 = policy.exponential_backoff(1);
    let d2 = policy.exponential_backoff(2);

    assert!(d0 >= Duration::from_millis(1));
    assert!(d0 <= Duration::from_millis(5_000));
    assert!(d1 >= d0 || d1 <= Duration::from_millis(5_000));
    assert!(d2 <= Duration::from_millis(5_000));
}

#[test]
fn retry_policy_respects_max_delay() {
    let config = RetryPolicyConfig {
        max_retries: 5,
        base_delay_ms: 1000,
        max_delay_ms: 500,
    };
    let policy = RetryPolicy::new(config);
    let d = policy.exponential_backoff(10);
    assert!(d <= Duration::from_millis(500));
}

#[test]
fn hash_u32_deterministic() {
    assert_eq!(hash_u32(0), hash_u32(0));
    assert_eq!(hash_u32(42), hash_u32(42));
}

#[tokio::test]
async fn resilient_adapter_success_no_retry() {
    let cb = std::sync::Arc::new(CircuitBreaker::new(CircuitBreakerConfig::default()));
    let adapter = ResilientAdapter::new(std::sync::Arc::clone(&cb), RetryPolicy::default_policy());

    let result = adapter.execute(|| async { Ok::<_, String>(42) }).await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(cb.state(), CircuitState::Closed);
}

#[tokio::test]
async fn resilient_adapter_retries_then_succeeds() {
    let cb = std::sync::Arc::new(CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 10,
        recovery_timeout_secs: 60,
        success_threshold: 2,
    }));
    let policy = RetryPolicy::new(RetryPolicyConfig {
        max_retries: 3,
        base_delay_ms: 1,
        max_delay_ms: 10,
    });
    let adapter = ResilientAdapter::new(cb, policy);

    let attempts = std::sync::Arc::new(AtomicU32::new(0));
    let attempts_clone = std::sync::Arc::clone(&attempts);
    let result = adapter
        .execute(|| {
            let a = std::sync::Arc::clone(&attempts_clone);
            async move {
                let n = a.fetch_add(1, Ordering::SeqCst);
                if n < 2 {
                    Err::<i32, _>("transient")
                } else {
                    Ok(99)
                }
            }
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 99);
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn resilient_adapter_circuit_open_rejects_immediately() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        recovery_timeout_secs: 3600,
        success_threshold: 2,
    };
    let cb = std::sync::Arc::new(CircuitBreaker::new(config));
    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Open);

    let adapter = ResilientAdapter::new(cb, RetryPolicy::default_policy());
    let mut called = false;
    let result = adapter
        .execute(|| {
            called = true;
            async { Ok::<i32, String>(1) }
        })
        .await;

    assert!(result.is_err());
    assert!(!called);
    assert!(result.unwrap_err().to_string().contains("circuit breaker"));
}

#[tokio::test]
async fn resilient_adapter_exhausted_retries_returns_last_error() {
    let cb = std::sync::Arc::new(CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 10,
        recovery_timeout_secs: 60,
        success_threshold: 2,
    }));
    let policy = RetryPolicy::new(RetryPolicyConfig {
        max_retries: 2,
        base_delay_ms: 1,
        max_delay_ms: 5,
    });
    let adapter = ResilientAdapter::new(cb, policy);

    let result = adapter
        .execute(|| async { Err::<i32, _>("always fails") })
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("always fails"));
}

#[test]
fn circuit_breaker_half_open_failure_reopens() {
    let config = CircuitBreakerConfig {
        failure_threshold: 1,
        recovery_timeout_secs: 60,
        success_threshold: 2,
    };
    let cb = CircuitBreaker::new(config);
    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Open);
    cb.state
        .store(STATE_HALF_OPEN, std::sync::atomic::Ordering::Release);
    assert_eq!(cb.state(), CircuitState::HalfOpen);

    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Open);
    assert!(!cb.can_execute());
}

#[test]
fn circuit_breaker_record_success_in_closed_resets_failures() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        recovery_timeout_secs: 60,
        success_threshold: 2,
    };
    let cb = CircuitBreaker::new(config);
    cb.record_failure();
    cb.record_failure();
    cb.record_success();
    assert_eq!(cb.state(), CircuitState::Closed);
    cb.record_failure();
    assert_eq!(cb.state(), CircuitState::Closed);
}

#[test]
fn retry_policy_new_and_max_retries_value() {
    let config = RetryPolicyConfig {
        max_retries: 5,
        base_delay_ms: 50,
        max_delay_ms: 1000,
    };
    let policy = RetryPolicy::new(config);
    assert_eq!(policy.max_retries_value(), 5);
}

#[test]
fn retry_policy_exponential_backoff_bounds() {
    let config = RetryPolicyConfig {
        max_retries: 10,
        base_delay_ms: 10,
        max_delay_ms: 100,
    };
    let policy = RetryPolicy::new(config);
    for attempt in 0..20 {
        let d = policy.exponential_backoff(attempt);
        assert!(d.as_millis() <= 100, "attempt {attempt} exceeded max");
    }
}

#[test]
fn hash_u32_varied_inputs() {
    assert_ne!(hash_u32(0), hash_u32(1));
    assert_ne!(hash_u32(1), hash_u32(2));
    assert_eq!(hash_u32(100), hash_u32(100));
}

#[tokio::test]
async fn resilient_adapter_single_retry_succeeds() {
    let cb = std::sync::Arc::new(CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 10,
        recovery_timeout_secs: 60,
        success_threshold: 2,
    }));
    let policy = RetryPolicy::new(RetryPolicyConfig {
        max_retries: 2,
        base_delay_ms: 1,
        max_delay_ms: 5,
    });
    let adapter = ResilientAdapter::new(cb, policy);

    let attempts = std::sync::Arc::new(AtomicU32::new(0));
    let attempts_clone = std::sync::Arc::clone(&attempts);
    let result = adapter
        .execute(|| {
            let a = std::sync::Arc::clone(&attempts_clone);
            async move {
                let n = a.fetch_add(1, Ordering::SeqCst);
                if n < 1 {
                    Err::<i32, _>("first attempt fails")
                } else {
                    Ok(123)
                }
            }
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 123);
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn execute_classified_permanent_error_fails_fast() {
    let cb = std::sync::Arc::new(CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 10,
        recovery_timeout_secs: 60,
        success_threshold: 2,
    }));
    let policy = RetryPolicy::new(RetryPolicyConfig {
        max_retries: 5,
        base_delay_ms: 1,
        max_delay_ms: 5,
    });
    let adapter = ResilientAdapter::new(cb, policy);

    let attempts = std::sync::Arc::new(AtomicU32::new(0));
    let attempts_clone = std::sync::Arc::clone(&attempts);
    let result = adapter
        .execute_classified(
            || {
                let a = std::sync::Arc::clone(&attempts_clone);
                async move {
                    a.fetch_add(1, Ordering::SeqCst);
                    Err::<i32, _>("permanent: auth denied")
                }
            },
            |e: &&str| !e.starts_with("permanent"),
        )
        .await;

    assert!(result.is_err());
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        1,
        "permanent errors should only be attempted once"
    );
}

#[tokio::test]
async fn execute_classified_transient_errors_still_retry() {
    let cb = std::sync::Arc::new(CircuitBreaker::new(CircuitBreakerConfig {
        failure_threshold: 10,
        recovery_timeout_secs: 60,
        success_threshold: 2,
    }));
    let policy = RetryPolicy::new(RetryPolicyConfig {
        max_retries: 3,
        base_delay_ms: 1,
        max_delay_ms: 5,
    });
    let adapter = ResilientAdapter::new(cb, policy);

    let attempts = std::sync::Arc::new(AtomicU32::new(0));
    let attempts_clone = std::sync::Arc::clone(&attempts);
    let result = adapter
        .execute_classified(
            || {
                let a = std::sync::Arc::clone(&attempts_clone);
                async move {
                    let n = a.fetch_add(1, Ordering::SeqCst);
                    if n < 2 {
                        Err::<i32, _>("transient: timeout")
                    } else {
                        Ok(42)
                    }
                }
            },
            |e: &&str| e.starts_with("transient"),
        )
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(attempts.load(Ordering::SeqCst), 3);
}
