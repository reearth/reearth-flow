mod action;
pub mod factory;
pub(crate) mod json;
mod split;

pub use slog::crit as slog_crit;
pub use slog::error as slog_error;
pub use slog::info as slog_info;
pub use slog::warn as slog_warn;
pub use tracing::error as tracing_error;
pub use tracing::info as tracing_info;
pub use tracing::warn as tracing_warn;

pub use slog::{o, Discard, Drain, Logger as ActionLogger};

#[macro_export]
macro_rules! action_log {
    (parent: $parent:expr, $logger:expr, $($args:tt)*) => {{
        $crate::slog_info!($logger, $($args)*);
        let parent_clone = $parent.clone();
        $crate::tracing_info!(parent: parent_clone, $($args)*); // Use the cloned parent context
    }};
}

#[macro_export]
macro_rules! action_error_log {
    (parent: $parent:expr, $logger:expr, $($args:tt)*) => {{
        $crate::slog_error!($logger, $($args)*);
        let parent_clone = $parent.clone();
        $crate::tracing_error!(parent: parent_clone, $($args)*); // Use the cloned parent context
    }};
}

#[macro_export]
macro_rules! action_warn_log {
    (parent: $parent:expr, $logger:expr, $($args:tt)*) => {{
        $crate::slog_warn!($logger, $($args)*);
        let parent_clone = $parent.clone();
        $crate::tracing_warn!(parent: parent_clone, $($args)*);
    }};
}

#[macro_export]
macro_rules! action_critical_log {
    (parent: $parent:expr, $logger:expr, $($args:tt)*) => {{
        $crate::slog_crit!($logger, $($args)*);
        // tracing has no Critical level; ERROR is the closest fan-out.
        let parent_clone = $parent.clone();
        $crate::tracing_error!(parent: parent_clone, $($args)*);
    }};
}

#[macro_export]
macro_rules! slow_action_log {
    (parent: $parent:expr, $logger:expr, $($args:tt)*) => {{
        $crate::slog_info!($logger, $($args)*);
        let parent_clone = $parent.clone();
        $crate::tracing_info!(parent: parent_clone, $($args)*); // Use the cloned parent context
    }};
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use slog::{Drain, Logger};

    #[derive(Clone, Default)]
    struct CaptureDrain {
        records: Arc<Mutex<Vec<(slog::Level, String)>>>,
    }

    impl Drain for CaptureDrain {
        type Ok = ();
        type Err = slog::Never;
        fn log(
            &self,
            record: &slog::Record,
            _values: &slog::OwnedKVList,
        ) -> Result<Self::Ok, Self::Err> {
            self.records
                .lock()
                .unwrap()
                .push((record.level(), format!("{}", record.msg())));
            Ok(())
        }
    }

    #[test]
    fn action_warn_log_writes_a_warning_record_to_slog() {
        let drain = CaptureDrain::default();
        let logger = Logger::root(drain.clone().fuse(), slog::o!());
        let span = tracing::warn_span!("test");
        crate::action_warn_log!(parent: span, logger, "dropped {} features", 3);
        let records = drain.records.lock().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].0, slog::Level::Warning);
        assert_eq!(records[0].1, "dropped 3 features");
    }

    #[test]
    fn action_critical_log_writes_a_critical_record_to_slog() {
        let drain = CaptureDrain::default();
        let logger = Logger::root(drain.clone().fuse(), slog::o!());
        let span = tracing::error_span!("test");
        crate::action_critical_log!(parent: span, logger, "workflow aborted: {}", "invariant violated");
        let records = drain.records.lock().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].0, slog::Level::Critical);
        assert_eq!(records[0].1, "workflow aborted: invariant violated");
    }
}
