use std::env;
use std::path::PathBuf;

use once_cell::sync::Lazy;
use slog::Logger;
use sloggers::file::FileLoggerBuilder;
use sloggers::null::NullLoggerBuilder;
use sloggers::types::Severity;
use sloggers::{Build, BuildWithCustomFormat};

use crate::json::Json;

static ACTION_LOG_DISABLE: Lazy<bool> = Lazy::new(|| {
    env::var("FLOW_RUNTIME_ACTION_LOG_DISABLE")
        .ok()
        .map(|s| s.to_lowercase() == "true")
        .unwrap_or(false)
});

pub(crate) fn action_logger(root_path: PathBuf, action: &str) -> Logger {
    if *ACTION_LOG_DISABLE {
        NullLoggerBuilder.build().unwrap()
    } else {
        let mut builder = FileLoggerBuilder::new(root_path.join(format!("{action}.log")));
        builder.level(Severity::Trace);
        builder
            .build_with_custom_format(|decorator| Ok(Json::new(decorator)))
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_action_logger_with_disable_flag() {
        env::set_var("FLOW_RUNTIME_ACTION_LOG_DISABLE", "true");
        let temp_dir = env::temp_dir();
        let logger = action_logger(temp_dir, "test_action");
        
        slog::info!(logger, "test message");
        
        env::remove_var("FLOW_RUNTIME_ACTION_LOG_DISABLE");
    }

    #[test]
    fn test_action_logger_path_creation() {
        let temp_dir = env::temp_dir();
        let action_name = "plateau_processor";
        
        env::set_var("FLOW_RUNTIME_ACTION_LOG_DISABLE", "true");
        let logger = action_logger(temp_dir.clone(), action_name);
        slog::debug!(logger, "debug log");
        env::remove_var("FLOW_RUNTIME_ACTION_LOG_DISABLE");
    }

    #[test]
    fn test_action_logger_with_japanese_action_name() {
        env::set_var("FLOW_RUNTIME_ACTION_LOG_DISABLE", "true");
        let temp_dir = env::temp_dir();
        let logger = action_logger(temp_dir, "建物処理");
        
        slog::info!(logger, "Japanese action name test");
        
        env::remove_var("FLOW_RUNTIME_ACTION_LOG_DISABLE");
    }
}

