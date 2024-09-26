use std::path::PathBuf;

use slog::{o, Logger};

use crate::action::action_logger;
use crate::split::split_logger;

#[derive(Debug, Clone)]
pub struct LoggerFactory {
    parent: Logger,
    root_path: PathBuf,
}

impl LoggerFactory {
    pub fn new(logger: Logger, root_path: PathBuf) -> Self {
        Self {
            parent: logger,
            root_path,
        }
    }

    /// Creates a action-specific logger.
    pub fn action_logger(&self, action: &str) -> Logger {
        let term_logger = self.parent.new(o!("action" => action.to_string()));
        split_logger(
            term_logger.clone(),
            action_logger(self.root_path.clone(), action),
        )
    }
}

pub fn create_root_logger(root_path: PathBuf) -> Logger {
    action_logger(root_path, "all")
}
