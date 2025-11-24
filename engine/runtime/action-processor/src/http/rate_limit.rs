use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use super::params::{RateLimitConfig, TimingStrategy};

pub(crate) struct RateLimiter {
    config: RateLimitConfig,
    state: Arc<Mutex<RateLimitState>>,
}

struct RateLimitState {
    requests_made: u32,
    interval_start: Instant,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(RateLimitState {
                requests_made: 0,
                interval_start: Instant::now(),
            })),
        }
    }

    pub fn acquire(&self) {
        let mut state = self.state.lock().unwrap();

        let elapsed = state.interval_start.elapsed();
        let interval = Duration::from_millis(self.config.interval_ms);

        if elapsed >= interval {
            state.requests_made = 0;
            state.interval_start = Instant::now();
        }

        if state.requests_made >= self.config.requests {
            let remaining = interval.saturating_sub(elapsed);
            drop(state);
            thread::sleep(remaining);

            state = self.state.lock().unwrap();
            state.requests_made = 0;
            state.interval_start = Instant::now();
        }

        match self.config.timing {
            TimingStrategy::Burst => {
                state.requests_made += 1;
            }
            TimingStrategy::Distributed => {
                let delay_per_request =
                    Duration::from_millis(self.config.interval_ms / self.config.requests as u64);
                let expected_time = delay_per_request * state.requests_made;
                let actual_time = state.interval_start.elapsed();

                if actual_time < expected_time {
                    let wait_time = expected_time - actual_time;
                    drop(state);
                    thread::sleep(wait_time);
                    let mut state = self.state.lock().unwrap();
                    state.requests_made += 1;
                } else {
                    state.requests_made += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_burst() {
        let config = RateLimitConfig {
            requests: 5,
            interval_ms: 1000,
            timing: TimingStrategy::Burst,
        };

        let limiter = RateLimiter::new(config);

        let start = Instant::now();

        // First 5 requests should be immediate
        for _ in 0..5 {
            limiter.acquire();
        }

        let elapsed = start.elapsed();
        assert!(elapsed < Duration::from_millis(100)); // Should be very fast

        // 6th request should wait
        let start_6th = Instant::now();
        limiter.acquire();
        let elapsed_6th = start_6th.elapsed();
        assert!(elapsed_6th >= Duration::from_millis(900)); // Should wait ~1 second
    }

    #[test]
    fn test_rate_limiter_distributed() {
        let config = RateLimitConfig {
            requests: 4,
            interval_ms: 400, // 100ms per request
            timing: TimingStrategy::Distributed,
        };

        let limiter = RateLimiter::new(config);
        let start = Instant::now();

        // Each request should be ~100ms apart
        for i in 0..4 {
            limiter.acquire();
            let elapsed = start.elapsed();
            let expected_min = Duration::from_millis(i * 100);
            assert!(elapsed >= expected_min);
        }
    }
}
