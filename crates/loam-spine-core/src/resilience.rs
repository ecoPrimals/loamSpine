// SPDX-License-Identifier: AGPL-3.0-or-later

//! Resilience patterns for PrimalAdapter and inter-primal calls.
//!
//! Provides circuit-breaker and retry with exponential backoff for
//! transient failures when communicating with discovered primals.

use std::sync::atomic::{AtomicU8, AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::error::{LoamSpineError, LoamSpineResult};

/// Circuit breaker state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CircuitState {
    /// Normal operation; calls are allowed.
    Closed,
    /// Failing; calls are rejected until recovery timeout.
    Open,
    /// Testing recovery; one call allowed to probe.
    HalfOpen,
}

/// Default failure count before circuit opens.
/// Tolerates transient bursts (network blips, GC pauses) without premature tripping.
pub const CIRCUIT_FAILURE_THRESHOLD: u32 = 5;

/// Seconds before an open circuit transitions to half-open for recovery probe.
/// Allows downstream services 30s to recover from transient overload.
pub const CIRCUIT_RECOVERY_TIMEOUT_SECS: u64 = 30;

/// Successful probes in half-open required to close the circuit.
/// Requires two consecutive successes to confirm genuine recovery.
pub const CIRCUIT_SUCCESS_THRESHOLD: u32 = 2;

/// Configuration for the circuit breaker.
#[derive(Clone, Debug)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit.
    pub failure_threshold: u32,
    /// Seconds to wait before transitioning from Open to HalfOpen.
    pub recovery_timeout_secs: u64,
    /// Successes in HalfOpen required to close the circuit.
    pub success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: CIRCUIT_FAILURE_THRESHOLD,
            recovery_timeout_secs: CIRCUIT_RECOVERY_TIMEOUT_SECS,
            success_threshold: CIRCUIT_SUCCESS_THRESHOLD,
        }
    }
}

const STATE_CLOSED: u8 = 0;
const STATE_OPEN: u8 = 1;
const STATE_HALF_OPEN: u8 = 2;

/// Circuit breaker for protecting against cascading failures.
///
/// Uses atomic operations for lock-free state transitions.
/// States: Closed (normal) → Open (failing) → HalfOpen (testing) → Closed.
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: AtomicU8,
    failure_count: AtomicU32,
    success_count: AtomicU32,
    /// Unix timestamp (secs) when circuit opened.
    opened_at_secs: AtomicU64,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with the given config.
    #[must_use]
    pub const fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: AtomicU8::new(STATE_CLOSED),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            opened_at_secs: AtomicU64::new(0),
        }
    }

    /// Check if a call is allowed (circuit not open, or recovery probe allowed).
    ///
    /// When Open, transitions to HalfOpen if recovery timeout has elapsed.
    #[must_use]
    pub fn can_execute(&self) -> bool {
        let s = self.state.load(Ordering::Acquire);
        match s {
            STATE_CLOSED | STATE_HALF_OPEN => true,
            STATE_OPEN => self.try_transition_open_to_half_open(),
            _ => false,
        }
    }

    fn try_transition_open_to_half_open(&self) -> bool {
        let opened = self.opened_at_secs.load(Ordering::Acquire);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if now.saturating_sub(opened) >= self.config.recovery_timeout_secs
            && self
                .state
                .compare_exchange(
                    STATE_OPEN,
                    STATE_HALF_OPEN,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
        {
            self.success_count.store(0, Ordering::Release);
            tracing::info!("circuit breaker: Open → HalfOpen (recovery timeout elapsed)");
            return true;
        }
        false
    }

    /// Record a successful call.
    ///
    /// In Closed: resets failure count. In HalfOpen: increments success count
    /// and transitions to Closed when threshold reached.
    pub fn record_success(&self) {
        let s = self.state.load(Ordering::Acquire);
        match s {
            STATE_CLOSED => {
                self.failure_count.store(0, Ordering::Release);
            }
            STATE_HALF_OPEN => {
                let prev = self.success_count.fetch_add(1, Ordering::AcqRel);
                if prev + 1 >= self.config.success_threshold
                    && self
                        .state
                        .compare_exchange(
                            STATE_HALF_OPEN,
                            STATE_CLOSED,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        )
                        .is_ok()
                {
                    self.failure_count.store(0, Ordering::Release);
                    self.success_count.store(0, Ordering::Release);
                    tracing::info!("circuit breaker: HalfOpen → Closed (recovered)");
                }
            }
            _ => {}
        }
    }

    /// Record a failed call.
    ///
    /// In Closed: increments failure count and opens when threshold reached.
    /// In HalfOpen: immediately opens.
    pub fn record_failure(&self) {
        let s = self.state.load(Ordering::Acquire);
        match s {
            STATE_CLOSED => {
                let prev = self.failure_count.fetch_add(1, Ordering::AcqRel);
                if prev + 1 >= self.config.failure_threshold {
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    if self
                        .state
                        .compare_exchange(
                            STATE_CLOSED,
                            STATE_OPEN,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        )
                        .is_ok()
                    {
                        self.opened_at_secs.store(now, Ordering::Release);
                        tracing::warn!(
                            "circuit breaker: Closed → Open ({} failures)",
                            self.config.failure_threshold
                        );
                    }
                }
            }
            STATE_HALF_OPEN => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                if self
                    .state
                    .compare_exchange(
                        STATE_HALF_OPEN,
                        STATE_OPEN,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    )
                    .is_ok()
                {
                    self.opened_at_secs.store(now, Ordering::Release);
                    tracing::warn!("circuit breaker: HalfOpen → Open (probe failed)");
                }
            }
            _ => {}
        }
    }

    /// Get the current circuit state.
    #[must_use]
    pub fn state(&self) -> CircuitState {
        match self.state.load(Ordering::Acquire) {
            STATE_OPEN => CircuitState::Open,
            STATE_HALF_OPEN => CircuitState::HalfOpen,
            _ => CircuitState::Closed,
        }
    }
}

/// Base delay in milliseconds for exponential backoff.
/// First retry after 100ms provides fast recovery for transient errors.
pub const RETRY_BASE_DELAY_MS: u64 = 100;

/// Maximum delay cap in milliseconds for exponential backoff.
/// Caps at 10s to prevent unbounded waits while allowing meaningful cooldown.
pub const RETRY_MAX_DELAY_MS: u64 = 10_000;

/// Maximum number of retry attempts before giving up.
/// Three retries with exponential backoff covers ~1.5s of transient failures.
pub const RETRY_MAX_ATTEMPTS: u32 = 3;

/// Configuration for retry with exponential backoff.
#[derive(Clone, Debug)]
pub struct RetryPolicyConfig {
    /// Maximum number of retry attempts (excluding the initial attempt).
    pub max_retries: u32,
    /// Base delay in milliseconds for the first retry.
    pub base_delay_ms: u64,
    /// Maximum delay in milliseconds (caps exponential growth).
    pub max_delay_ms: u64,
}

impl Default for RetryPolicyConfig {
    fn default() -> Self {
        Self {
            max_retries: RETRY_MAX_ATTEMPTS,
            base_delay_ms: RETRY_BASE_DELAY_MS,
            max_delay_ms: RETRY_MAX_DELAY_MS,
        }
    }
}

/// Retry policy with exponential backoff and jitter.
#[derive(Clone, Debug)]
pub struct RetryPolicy {
    config: RetryPolicyConfig,
}

impl RetryPolicy {
    /// Create a new retry policy with the given config.
    #[must_use]
    pub const fn new(config: RetryPolicyConfig) -> Self {
        Self { config }
    }

    /// Create with default config (3 retries, 100ms base, 5000ms max).
    #[must_use]
    pub fn default_policy() -> Self {
        Self::new(RetryPolicyConfig::default())
    }

    /// Compute delay for the given attempt (0 = first retry).
    ///
    /// Uses exponential backoff with ±25% jitter to avoid thundering herd.
    #[must_use]
    pub fn exponential_backoff(&self, attempt: u32) -> Duration {
        let base = self.config.base_delay_ms;
        let max = self.config.max_delay_ms;
        let delay_ms = base.saturating_mul(2_u64.saturating_pow(attempt)).min(max);
        let low = (delay_ms * 3) / 4;
        let range = (delay_ms / 2).max(1);
        let jittered = low + (u64::from(hash_u32(attempt)) % range);
        Duration::from_millis(jittered.min(max))
    }
}

impl RetryPolicy {
    /// Get max_retries from config.
    #[must_use]
    pub const fn max_retries_value(&self) -> u32 {
        self.config.max_retries
    }
}

/// Simple hash for jitter (no external deps).
const fn hash_u32(x: u32) -> u32 {
    let mut h = x.wrapping_mul(0x9E37_79B9);
    h ^= h >> 16;
    h = h.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 13;
    h
}

/// Wrapper that executes async operations with retry and circuit-breaker.
#[derive(Debug)]
pub struct ResilientAdapter {
    circuit_breaker: std::sync::Arc<CircuitBreaker>,
    retry_policy: RetryPolicy,
}

impl ResilientAdapter {
    /// Create a new resilient adapter with the given circuit breaker and retry policy.
    #[must_use]
    pub const fn new(
        circuit_breaker: std::sync::Arc<CircuitBreaker>,
        retry_policy: RetryPolicy,
    ) -> Self {
        Self {
            circuit_breaker,
            retry_policy,
        }
    }

    /// Execute an async operation with retry and circuit-breaker protection.
    ///
    /// All errors are treated as transient and retried. Use
    /// [`execute_classified`](Self::execute_classified) when you need to
    /// distinguish transient from permanent errors.
    ///
    /// # Errors
    ///
    /// Returns `LoamSpineError::CapabilityUnavailable` when circuit is open.
    /// Propagates the last error after all retries exhausted.
    pub async fn execute<F, Fut, T, E>(&self, operation: F) -> LoamSpineResult<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display + Send,
    {
        self.execute_classified(operation, |_| true).await
    }

    /// Execute an async operation with retry, circuit-breaker, and error
    /// classification.
    ///
    /// Only errors for which `is_transient` returns `true` are retried;
    /// permanent errors fail fast without consuming remaining retries.
    ///
    /// # Errors
    ///
    /// Returns `LoamSpineError::CapabilityUnavailable` when circuit is open.
    /// Propagates the last error after all retries exhausted or on permanent failure.
    pub async fn execute_classified<F, Fut, T, E, C>(
        &self,
        mut operation: F,
        is_transient: C,
    ) -> LoamSpineResult<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display + Send,
        C: Fn(&E) -> bool,
    {
        if !self.circuit_breaker.can_execute() {
            return Err(LoamSpineError::CapabilityUnavailable(
                "circuit breaker open".to_string(),
            ));
        }

        let mut last_err: Option<LoamSpineError> = None;
        let max_retries = self.retry_policy.max_retries_value();

        for attempt in 0..=max_retries {
            match operation().await {
                Ok(value) => {
                    self.circuit_breaker.record_success();
                    return Ok(value);
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    let transient = is_transient(&e);
                    self.circuit_breaker.record_failure();

                    let spine_err = LoamSpineError::Network(err_msg.clone());
                    last_err = Some(spine_err);

                    if !transient {
                        tracing::debug!("permanent failure (no retry): {err_msg}");
                        break;
                    }

                    if attempt < max_retries {
                        let delay = self.retry_policy.exponential_backoff(attempt);
                        tracing::debug!(
                            attempt = attempt + 1,
                            max = max_retries + 1,
                            delay_ms = delay.as_millis(),
                            "retrying after transient failure: {err_msg}"
                        );
                        tokio::time::sleep(delay).await;
                    } else {
                        break;
                    }
                }
            }
        }

        Err(last_err.unwrap_or_else(|| {
            LoamSpineError::Internal("resilient adapter: no error captured".to_string())
        }))
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests use unwrap for conciseness")]
#[path = "resilience_tests.rs"]
mod tests;
