use std::path::PathBuf;

use reearth_flow_common::runtime_config::ACTION_LOG_DISABLE;
use slog::Logger;
use sloggers::file::FileLoggerBuilder;
use sloggers::null::NullLoggerBuilder;
use sloggers::types::Severity;
use sloggers::{Build, BuildWithCustomFormat};

use crate::json::Json;

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
