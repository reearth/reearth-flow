use std::{env, str::FromStr, sync::LazyLock, time::Duration};

// Helper function to get config value from CLI args or env var
fn get_config<T: FromStr>(cli_flag: &str, env_var: &str, default: T) -> T {
    // 1) Check CLI argument
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == format!("--{cli_flag}") {
            if let Some(v) = args.next() {
                if let Ok(parsed) = v.parse::<T>() {
                    return parsed;
                }
            }
        } else if let Some(v) = arg.strip_prefix(&format!("--{cli_flag}=")) {
            if let Ok(parsed) = v.parse::<T>() {
                return parsed;
            }
        }
    }

    // 2) Check environment variable
    env::var(env_var)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

// Lazy static variables for runtime configuration
pub static WORKING_DIRECTORY: LazyLock<Option<String>> = LazyLock::new(|| {
    let v = get_config(
        "working-dir",
        "FLOW_RUNTIME_WORKING_DIRECTORY",
        String::new(),
    );
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
});

pub static ACTION_LOG_DISABLE: LazyLock<bool> = LazyLock::new(|| {
    get_config(
        "action-log-disable",
        "FLOW_RUNTIME_ACTION_LOG_DISABLE",
        false,
    )
});

pub static CHANNEL_BUFFER_SIZE: LazyLock<usize> = LazyLock::new(|| {
    get_config(
        "channel-buffer-size",
        "FLOW_RUNTIME_CHANNEL_BUFFER_SIZE",
        256,
    )
});

pub static EVENT_HUB_CAPACITY: LazyLock<usize> = LazyLock::new(|| {
    get_config(
        "event-hub-capacity",
        "FLOW_RUNTIME_EVENT_HUB_CAPACITY",
        8192,
    )
});

pub static THREAD_POOL_SIZE: LazyLock<usize> =
    LazyLock::new(|| get_config("thread-pool-size", "FLOW_RUNTIME_THREAD_POOL_SIZE", 30));

pub static FEATURE_FLUSH_THRESHOLD: LazyLock<usize> = LazyLock::new(|| {
    get_config(
        "feature-flush-threshold",
        "FLOW_RUNTIME_FEATURE_FLUSH_THRESHOLD",
        512,
    )
});

pub static ASYNC_WORKER_NUM: LazyLock<usize> = LazyLock::new(|| {
    get_config(
        "async-worker-num",
        "FLOW_RUNTIME_ASYNC_WORKER_NUM",
        num_cpus::get(),
    )
});

pub static FEATURE_WRITER_DISABLE: LazyLock<bool> = LazyLock::new(|| {
    get_config(
        "feature-writer-disable",
        "FLOW_RUNTIME_FEATURE_WRITER_DISABLE",
        false,
    )
});

pub static SLOW_ACTION_THRESHOLD: LazyLock<Duration> = LazyLock::new(|| {
    Duration::from_millis(get_config(
        "slow-action-threshold",
        "FLOW_RUNTIME_SLOW_ACTION_THRESHOLD_MS",
        300,
    ))
});

pub static NODE_STATUS_PROPAGATION_DELAY: LazyLock<Duration> = LazyLock::new(|| {
    Duration::from_millis(get_config(
        "node-status-propagation-delay-ms",
        "FLOW_RUNTIME_NODE_STATUS_PROPAGATION_DELAY_MS",
        500,
    ))
});

/// Runtime configuration for the Flow engine.
/// CLI arguments take precedence over environment variables, which take precedence over defaults.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Whether to disable the Action log (default: false)
    pub action_log_disable: bool,
    /// Buffer size for channels exchanged by worker threads (default: 256)
    pub channel_buffer_size: usize,
    /// Capacity size of event hub channels (default: 8192)
    pub event_hub_capacity: usize,
    /// Worker thread pool size (default: 30)
    pub thread_pool_size: usize,
    /// Sink node flush threshold size (default: 512)
    pub feature_flush_threshold: usize,
    /// Tokio Worker number (default: number of CPUs)
    pub async_worker_num: usize,
    /// Whether to disable the ability to export data to the feature store (default: false)
    pub feature_writer_disable: bool,
    /// Threshold for writing slow action logs in milliseconds (default: 300)
    pub slow_action_threshold: u64,
    /// Delay in milliseconds to ensure node status events propagate (default: 500)
    pub node_status_propagation_delay_ms: u64,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            action_log_disable: false,
            channel_buffer_size: 256,
            event_hub_capacity: 8192,
            thread_pool_size: 30,
            feature_flush_threshold: 512,
            async_worker_num: num_cpus::get(),
            feature_writer_disable: false,
            slow_action_threshold: 300,
            node_status_propagation_delay_ms: 500,
        }
    }
}
