use std::env;
use std::path::PathBuf;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use slog::Logger;
use sloggers::file::FileLoggerBuilder;
use sloggers::null::NullLoggerBuilder;
use sloggers::types::Severity;
use sloggers::{Build, BuildWithCustomFormat};

use reearth_flow_common::str::{is_boolean, parse_boolean};

use crate::json::Json;

static ACTION_LOG_DISABLE: Lazy<Mutex<Option<String>>> =
    Lazy::new(|| Mutex::new(env::var("ACTION_LOG_DISABLE").ok()));

pub(crate) fn action_logger(root_path: PathBuf, action: &str) -> Logger {
    let disable = ACTION_LOG_DISABLE.lock().unwrap().clone();
    match disable {
        Some(disable) if is_boolean(&disable) && parse_boolean(&disable) => {
            NullLoggerBuilder.build().unwrap()
        }
        _ => {
            let mut builder = FileLoggerBuilder::new(root_path.join(format!("{}.log", action)));
            builder.level(Severity::Info);
            builder
                .build_with_custom_format(|decorator| Ok(Json::new(decorator)))
                .unwrap()
        }
    }
}
