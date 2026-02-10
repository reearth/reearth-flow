/// Get current process RSS in MB (requires `memory-stats` feature).
pub fn current_rss_mb() -> f64 {
    #[cfg(feature = "memory-stats")]
    {
        use sysinfo::{Pid, ProcessesToUpdate, System};
        let pid = Pid::from_u32(std::process::id());
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
        sys.process(pid)
            .map(|p| p.memory() as f64 / 1024.0 / 1024.0)
            .unwrap_or(0.0)
    }
    #[cfg(not(feature = "memory-stats"))]
    {
        0.0
    }
}
