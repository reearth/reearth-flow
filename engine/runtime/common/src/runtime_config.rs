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
