//! @ai:module:intent Rate limiting for API requests
//! @ai:module:layer infrastructure
//! @ai:module:public_api RateLimiter
//! @ai:module:stateless false

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// @ai:intent Trait for rate limiting functionality
pub trait RateLimiterTrait: Send + Sync {
    /// @ai:intent Wait until a request is allowed
    fn wait(&self) -> impl std::future::Future<Output = ()> + Send;
}

/// @ai:intent Token bucket rate limiter for API requests
pub struct RateLimiter {
    state: Arc<Mutex<RateLimiterState>>,
    requests_per_minute: u32,
}

struct RateLimiterState {
    tokens: f64,
    last_update: Instant,
}

impl RateLimiter {
    /// @ai:intent Create a new rate limiter
    /// @ai:pre requests_per_minute > 0
    /// @ai:effects pure
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            state: Arc::new(Mutex::new(RateLimiterState {
                tokens: requests_per_minute as f64,
                last_update: Instant::now(),
            })),
            requests_per_minute,
        }
    }

    /// @ai:intent Refill tokens based on elapsed time
    /// @ai:effects state:write
    fn refill_tokens(state: &mut RateLimiterState, rpm: u32) {
        let now = Instant::now();
        let elapsed = now.duration_since(state.last_update);
        let tokens_to_add = elapsed.as_secs_f64() * (rpm as f64 / 60.0);
        state.tokens = (state.tokens + tokens_to_add).min(rpm as f64);
        state.last_update = now;
    }
}

impl RateLimiterTrait for RateLimiter {
    /// @ai:intent Wait until a request is allowed
    /// @ai:effects state:write, time
    async fn wait(&self) {
        loop {
            let sleep_duration = {
                let mut state = self.state.lock().await;
                Self::refill_tokens(&mut state, self.requests_per_minute);

                if state.tokens >= 1.0 {
                    state.tokens -= 1.0;
                    return;
                }

                let tokens_needed = 1.0 - state.tokens;
                let seconds_to_wait = tokens_needed / (self.requests_per_minute as f64 / 60.0);
                Duration::from_secs_f64(seconds_to_wait)
            };

            tokio::time::sleep(sleep_duration).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_initial_requests() {
        let limiter = RateLimiter::new(60);

        let start = Instant::now();
        limiter.wait().await;
        let elapsed = start.elapsed();

        assert!(elapsed < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_rate_limiter_throttles_excess_requests() {
        let limiter = RateLimiter::new(60);

        for _ in 0..60 {
            limiter.wait().await;
        }

        let start = Instant::now();
        limiter.wait().await;
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(900));
    }
}
