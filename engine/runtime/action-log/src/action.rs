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
        let mut builder = FileLoggerBuilder::new(root_path.join(format!("{}.log", action)));
        builder.level(Severity::Info);
        builder
            .build_with_custom_format(|decorator| Ok(Json::new(decorator)))
            .unwrap()
    }
}
