//! Coordinates concurrent execution of accumulating processor `finish()` calls.
//!
//! Accumulating processors (e.g., aggregators, sorters) buffer data during processing
//! and emit all results during their `finish()` phase. Running multiple `finish()` calls
//! simultaneously can cause high peak memory usage.
//!
//! This module provides a global permit system to limit concurrency. Configure via:
//! - `FLOW_RUNTIME_ACCUMULATING_FINISH_CONCURRENCY` - Maximum concurrent finish calls (default: 1)
//!
//! Setting to 1 serializes finish operations, reducing memory pressure at the cost of throughput.

use std::sync::{Condvar, LazyLock, Mutex};

struct AccumulatingFinishLimiter {
    current: Mutex<usize>,
    condvar: Condvar,
    max_concurrent: usize,
}

impl AccumulatingFinishLimiter {
    fn new(max_concurrent: usize) -> Self {
        Self {
            current: Mutex::new(0),
            condvar: Condvar::new(),
            max_concurrent,
        }
    }

    fn acquire(&self) -> AccumulatingFinishGuard<'_> {
        let mut count = self.current.lock().unwrap();
        while *count >= self.max_concurrent {
            count = self.condvar.wait(count).unwrap();
        }
        *count += 1;
        AccumulatingFinishGuard { limiter: self }
    }
}

pub struct AccumulatingFinishGuard<'a> {
    limiter: &'a AccumulatingFinishLimiter,
}

impl Drop for AccumulatingFinishGuard<'_> {
    fn drop(&mut self) {
        let mut count = self.limiter.current.lock().unwrap();
        *count -= 1;
        self.limiter.condvar.notify_one();
    }
}

static LIMITER: LazyLock<AccumulatingFinishLimiter> = LazyLock::new(|| {
    let limit = std::env::var("FLOW_RUNTIME_ACCUMULATING_FINISH_CONCURRENCY")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(1);
    AccumulatingFinishLimiter::new(limit)
});

pub fn acquire_permit() -> AccumulatingFinishGuard<'static> {
    LIMITER.acquire()
}
