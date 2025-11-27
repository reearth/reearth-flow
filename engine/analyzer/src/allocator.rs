use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, Ordering};

thread_local! {
    /// Current memory allocated by this thread (cumulative, persists across process() calls)
    static CURRENT_ALLOC: Cell<usize> = const { Cell::new(0) };
    /// Peak memory allocated during the current tracking session (reset on start_tracking)
    static SESSION_PEAK_ALLOC: Cell<usize> = const { Cell::new(0) };
    /// Memory allocated at the start of tracking session (to compute peak delta)
    static SESSION_START_ALLOC: Cell<usize> = const { Cell::new(0) };
    /// Whether tracking is enabled for this thread
    static TRACKING_ENABLED: Cell<bool> = const { Cell::new(false) };
}

static ANALYZER_ENABLED: AtomicBool = AtomicBool::new(false);

/// A custom allocator that wraps the system allocator and tracks
/// memory allocations per thread when the analyzer is enabled.
///
/// To use this allocator, add the following to your main.rs or lib.rs:
/// ```ignore
/// use reearth_flow_analyzer::AnalyzerAllocator;
/// #[global_allocator]
/// static GLOBAL: AnalyzerAllocator = AnalyzerAllocator::new(std::alloc::System);
/// ```
pub struct AnalyzerAllocator<A: GlobalAlloc = System> {
    inner: A,
}

impl<A: GlobalAlloc> AnalyzerAllocator<A> {
    pub const fn new(inner: A) -> Self {
        Self { inner }
    }
}

impl Default for AnalyzerAllocator<System> {
    fn default() -> Self {
        Self::new(System)
    }
}

unsafe impl<A: GlobalAlloc> GlobalAlloc for AnalyzerAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);
        if !ptr.is_null() && ANALYZER_ENABLED.load(Ordering::Relaxed) {
            CURRENT_ALLOC.with(|current| {
                let new_val = current.get() + layout.size();
                current.set(new_val);

                // Update session peak if tracking is enabled
                TRACKING_ENABLED.with(|enabled| {
                    if enabled.get() {
                        SESSION_PEAK_ALLOC.with(|peak| {
                            let start = SESSION_START_ALLOC.with(|s| s.get());
                            let delta = new_val.saturating_sub(start);
                            if delta > peak.get() {
                                peak.set(delta);
                            }
                        });
                    }
                });
            });
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if ANALYZER_ENABLED.load(Ordering::Relaxed) {
            CURRENT_ALLOC.with(|current| {
                current.set(current.get().saturating_sub(layout.size()));
            });
        }
        self.inner.dealloc(ptr, layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = self.inner.realloc(ptr, layout, new_size);
        if !new_ptr.is_null() && ANALYZER_ENABLED.load(Ordering::Relaxed) {
            CURRENT_ALLOC.with(|current| {
                let old_size = layout.size();
                let diff = new_size as isize - old_size as isize;
                let new_val = (current.get() as isize + diff).max(0) as usize;
                current.set(new_val);

                // Update session peak if tracking is enabled and memory increased
                if diff > 0 {
                    TRACKING_ENABLED.with(|enabled| {
                        if enabled.get() {
                            SESSION_PEAK_ALLOC.with(|peak| {
                                let start = SESSION_START_ALLOC.with(|s| s.get());
                                let delta = new_val.saturating_sub(start);
                                if delta > peak.get() {
                                    peak.set(delta);
                                }
                            });
                        }
                    });
                }
            });
        }
        new_ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc_zeroed(layout);
        if !ptr.is_null() && ANALYZER_ENABLED.load(Ordering::Relaxed) {
            CURRENT_ALLOC.with(|current| {
                let new_val = current.get() + layout.size();
                current.set(new_val);

                // Update session peak if tracking is enabled
                TRACKING_ENABLED.with(|enabled| {
                    if enabled.get() {
                        SESSION_PEAK_ALLOC.with(|peak| {
                            let start = SESSION_START_ALLOC.with(|s| s.get());
                            let delta = new_val.saturating_sub(start);
                            if delta > peak.get() {
                                peak.set(delta);
                            }
                        });
                    }
                });
            });
        }
        ptr
    }
}

/// Enable the analyzer globally. This must be called before any tracking
/// will occur.
pub fn enable_analyzer() {
    ANALYZER_ENABLED.store(true, Ordering::SeqCst);
}

/// Disable the analyzer globally.
pub fn disable_analyzer() {
    ANALYZER_ENABLED.store(false, Ordering::SeqCst);
}

/// Check if the analyzer is enabled globally.
pub fn is_analyzer_enabled() -> bool {
    ANALYZER_ENABLED.load(Ordering::Relaxed)
}

/// Start tracking memory for a single process() call.
/// This resets the session peak counter but NOT the current allocation counter.
/// - Peak will track the maximum NEW allocations during this session
/// - Current memory persists across sessions (cumulative thread allocations)
pub fn start_tracking() {
    let current = CURRENT_ALLOC.with(|c| c.get());
    SESSION_START_ALLOC.with(|s| s.set(current));
    SESSION_PEAK_ALLOC.with(|p| p.set(0)); // Reset peak delta for this session
    TRACKING_ENABLED.with(|enabled| enabled.set(true));
}

/// Stop tracking memory and return the stats.
/// Returns `(current_memory_bytes, peak_delta_bytes)`.
/// - current_memory_bytes: Total memory currently allocated by this thread (cumulative)
/// - peak_delta_bytes: Peak NEW memory allocated during this tracking session
pub fn stop_tracking() -> (usize, usize) {
    TRACKING_ENABLED.with(|enabled| enabled.set(false));
    let current = CURRENT_ALLOC.with(|c| c.get());
    let peak_delta = SESSION_PEAK_ALLOC.with(|p| p.get());
    (current, peak_delta)
}

/// Get current memory statistics without stopping tracking.
/// Returns `(current_memory_bytes, peak_delta_bytes)`.
pub fn get_current_stats() -> (usize, usize) {
    let current = CURRENT_ALLOC.with(|c| c.get());
    let peak_delta = SESSION_PEAK_ALLOC.with(|p| p.get());
    (current, peak_delta)
}

/// Check if tracking is enabled for the current thread.
pub fn is_tracking() -> bool {
    TRACKING_ENABLED.with(|enabled| enabled.get())
}

/// Get the current cumulative memory allocated by this thread.
pub fn get_current_memory() -> usize {
    CURRENT_ALLOC.with(|c| c.get())
}

/// A guard that automatically starts tracking when created and stops when dropped.
/// Returns the memory stats when dropped.
pub struct TrackingGuard {
    _private: (),
}

impl TrackingGuard {
    /// Create a new tracking guard that starts tracking immediately.
    pub fn new() -> Self {
        start_tracking();
        Self { _private: () }
    }

    /// Get current stats without stopping tracking.
    pub fn current_stats(&self) -> (usize, usize) {
        get_current_stats()
    }

    /// Stop tracking and return the final stats.
    pub fn finish(self) -> (usize, usize) {
        let stats = stop_tracking();
        std::mem::forget(self); // Don't call drop
        stats
    }
}

impl Default for TrackingGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TrackingGuard {
    fn drop(&mut self) {
        stop_tracking();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracking_disabled_by_default() {
        assert!(!is_tracking());
    }

    #[test]
    fn test_enable_disable_analyzer() {
        disable_analyzer();
        assert!(!is_analyzer_enabled());
        enable_analyzer();
        assert!(is_analyzer_enabled());
        disable_analyzer();
        assert!(!is_analyzer_enabled());
    }

    #[test]
    fn test_start_stop_tracking() {
        enable_analyzer();

        start_tracking();
        assert!(is_tracking());

        // Allocate some memory
        let _v: Vec<u8> = vec![0u8; 1024];

        let (_current, _peak) = stop_tracking();
        assert!(!is_tracking());

        // Note: Actual memory tracking only works when AnalyzerAllocator is set as
        // #[global_allocator]. In unit tests, we cannot verify actual memory values.
        // The allocator tracking is tested implicitly when running the actual CLI with
        // the analyzer feature enabled.

        disable_analyzer();
    }

    #[test]
    fn test_tracking_guard() {
        enable_analyzer();

        let (_current, _peak) = {
            let guard = TrackingGuard::new();
            let _v: Vec<u8> = vec![0u8; 2048];
            guard.finish()
        };

        assert!(!is_tracking());

        // Note: Actual memory tracking only works when AnalyzerAllocator is set as
        // #[global_allocator]. In unit tests, we cannot verify actual memory values.

        disable_analyzer();
    }

    #[test]
    fn test_thread_local_counters() {
        // Test that thread-local counters work correctly when manually set
        // (simulating what happens when AnalyzerAllocator is active)
        enable_analyzer();

        // Manually set CURRENT_ALLOC to simulate an allocation
        CURRENT_ALLOC.with(|c| c.set(1000));

        // Start tracking - should record the baseline
        start_tracking();
        assert!(is_tracking());

        // Simulate more allocations
        CURRENT_ALLOC.with(|c| c.set(3000));
        SESSION_PEAK_ALLOC.with(|p| p.set(2000)); // Peak delta of 2000 from baseline

        let (current, peak) = stop_tracking();
        assert!(!is_tracking());

        // Current should be 3000 (cumulative)
        assert_eq!(current, 3000);
        // Peak should be 2000 (peak delta during session)
        assert_eq!(peak, 2000);

        // Current memory should persist after stopping tracking
        assert_eq!(get_current_memory(), 3000);

        disable_analyzer();
    }

    #[test]
    fn test_session_peak_tracking() {
        // Test that peak tracking correctly computes delta from session start
        enable_analyzer();

        // Set initial memory state
        CURRENT_ALLOC.with(|c| c.set(5000));

        // Start tracking - session start should be 5000
        start_tracking();

        // Verify session start was recorded
        let session_start = SESSION_START_ALLOC.with(|s| s.get());
        assert_eq!(session_start, 5000);

        // Simulate allocation that increases memory
        CURRENT_ALLOC.with(|c| c.set(8000));
        // Manually update peak (normally done by allocator)
        let delta = 8000 - session_start;
        SESSION_PEAK_ALLOC.with(|p| {
            if delta > p.get() {
                p.set(delta);
            }
        });

        let (current, peak) = stop_tracking();

        assert_eq!(current, 8000); // Total cumulative memory
        assert_eq!(peak, 3000); // Peak NEW allocations during session (8000 - 5000)

        disable_analyzer();
    }
}
