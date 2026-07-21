use std::sync::Arc;

use reearth_flow_action_log::{
    action_critical_log, action_error_log, action_log, action_warn_log, factory::LoggerFactory,
    ActionLogger,
};
use reearth_flow_diagnostics::Severity;
use tracing::{debug_span, error_span, info_span, trace_span, warn_span};

use crate::node_info_tls;

pub(crate) struct LogEventHandler {
    #[allow(dead_code)]
    pub(crate) workflow_id: uuid::Uuid,
    pub(crate) job_id: uuid::Uuid,
    pub(crate) logger: Arc<ActionLogger>,
}

impl LogEventHandler {
    pub(crate) fn new(
        workflow_id: uuid::Uuid,
        job_id: uuid::Uuid,
        logger_factory: Arc<LoggerFactory>,
    ) -> Self {
        let logger = logger_factory.action_logger(&job_id.to_string());
        Self {
            workflow_id,
            job_id,
            logger: Arc::new(logger),
        }
    }
}

#[async_trait::async_trait]
impl reearth_flow_runtime::event::EventHandler for LogEventHandler {
    async fn on_event(&self, event: &reearth_flow_runtime::event::Event) {
        match event {
            reearth_flow_runtime::event::Event::Log {
                span,
                level,
                node_handle,
                node_name,
                message,
            } => {
                let node_id = node_handle
                    .clone()
                    .map(|h| h.id.to_string())
                    .unwrap_or_else(|| "".to_string());
                match *level {
                    tracing::Level::ERROR => {
                        let span = span.clone().unwrap_or_else(|| error_span!(""));

                        if !node_id.is_empty() {
                            node_info_tls::set_node_info(node_id.clone(), node_name.clone());
                        }

                        action_error_log!(parent: span, self.logger, "{:?}", message);

                        node_info_tls::clear_node_info();
                    }
                    tracing::Level::WARN => {
                        let span = span.clone().unwrap_or_else(|| warn_span!(""));

                        if !node_id.is_empty() {
                            node_info_tls::set_node_info(node_id.clone(), node_name.clone());
                        }

                        action_warn_log!(parent: span, self.logger, "{:?}", message);

                        node_info_tls::clear_node_info();
                    }
                    tracing::Level::INFO => {
                        let span = span.clone().unwrap_or_else(|| info_span!(""));

                        if !node_id.is_empty() {
                            node_info_tls::set_node_info(node_id.clone(), node_name.clone());
                        }

                        action_log!(parent: span, self.logger, "{:?}", message);

                        node_info_tls::clear_node_info();
                    }
                    tracing::Level::DEBUG => {
                        let span = span.clone().unwrap_or_else(|| debug_span!(""));
                        tracing::event!(parent: span, tracing::Level::DEBUG, "job_id"=self.job_id.to_string(), "node_id"=node_id, "{:?}", message);
                    }
                    tracing::Level::TRACE => {
                        let span = span.clone().unwrap_or_else(|| trace_span!(""));
                        tracing::event!(parent: span, tracing::Level::TRACE, "job_id"=self.job_id.to_string(), "node_id"=node_id, "{:?}", message);
                    }
                }
            }
            reearth_flow_runtime::event::Event::Diagnostic(d) => {
                let node_id = d.node_id.clone().unwrap_or_else(|| "".to_string());
                let node_name = d.action_type.clone();
                let message = &d.message;
                match d.severity {
                    Severity::Fatal => {
                        let span = error_span!("");

                        if !node_id.is_empty() {
                            node_info_tls::set_node_info(node_id.clone(), node_name.clone());
                        }

                        action_critical_log!(parent: span, self.logger, "{:?}", message);

                        node_info_tls::clear_node_info();
                    }
                    Severity::Error => {
                        let span = error_span!("");

                        if !node_id.is_empty() {
                            node_info_tls::set_node_info(node_id.clone(), node_name.clone());
                        }

                        action_error_log!(parent: span, self.logger, "{:?}", message);

                        node_info_tls::clear_node_info();
                    }
                    Severity::Warn => {
                        let span = warn_span!("");

                        if !node_id.is_empty() {
                            node_info_tls::set_node_info(node_id.clone(), node_name.clone());
                        }

                        action_warn_log!(parent: span, self.logger, "{:?}", message);

                        node_info_tls::clear_node_info();
                    }
                    Severity::Info | Severity::Debug | Severity::Trace => {
                        let span = info_span!("");

                        if !node_id.is_empty() {
                            node_info_tls::set_node_info(node_id.clone(), node_name.clone());
                        }

                        action_log!(parent: span, self.logger, "{:?}", message);

                        node_info_tls::clear_node_info();
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use reearth_flow_diagnostics::{Diagnostic, DiagnosticDraft, ErrorCode};
    use reearth_flow_runtime::event::{Event, EventHandler};
    use slog::{Drain, Level, Logger, Never, OwnedKVList, Record};

    use super::*;

    /// A synchronous slog drain that records each log call's level and
    /// rendered message into a shared buffer.
    ///
    /// The production per-job logger (`LoggerFactory::action_logger`) wraps a
    /// `slog_async::Async` around a `sloggers` file logger that is *itself*
    /// asynchronous, so observing a rendered record through it means waiting
    /// on two independent, unjoined writer threads and reading back a temp
    /// file — racy enough that the record could still be in flight when the
    /// test reads (it was, on CI: the file never appeared). These tests only
    /// need to prove that a diagnostic's `Severity` drives the slog level the
    /// handler emits at, so we bypass the file entirely and capture the
    /// record synchronously, in-process. `LogEventHandler`'s fields are
    /// `pub(crate)`, so the test builds one directly around this drain
    /// instead of going through `new()` and the async factory.
    struct CaptureDrain {
        records: Arc<Mutex<Vec<(Level, String)>>>,
    }

    impl Drain for CaptureDrain {
        type Ok = ();
        type Err = Never;

        fn log(&self, record: &Record, _values: &OwnedKVList) -> Result<(), Never> {
            self.records
                .lock()
                .unwrap()
                .push((record.level(), record.msg().to_string()));
            Ok(())
        }
    }

    /// Sends a single `Event::Diagnostic` through `LogEventHandler::on_event`
    /// and returns the one slog record it emitted (level + rendered message).
    fn render_diagnostic(diagnostic: Diagnostic) -> (Level, String) {
        let records = Arc::new(Mutex::new(Vec::new()));
        let drain = CaptureDrain {
            records: records.clone(),
        };
        let logger = Logger::root(drain.fuse(), slog::o!());

        let handler = LogEventHandler {
            workflow_id: uuid::Uuid::new_v4(),
            job_id: uuid::Uuid::new_v4(),
            logger: Arc::new(logger),
        };

        let event = Event::Diagnostic(Arc::new(diagnostic));
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(handler.on_event(&event));

        let records = records.lock().unwrap();
        assert_eq!(
            records.len(),
            1,
            "expected exactly one slog record, got {}: {records:?}",
            records.len()
        );
        records[0].clone()
    }

    /// Proof of the CRITICAL path at the handler level (2a Task 12): a
    /// Fatal-disposition code (`InternalInvariantViolation`, whose registry
    /// default disposition is `Fatal`) defaults to `Severity::Fatal` per
    /// `Diagnostic::from_draft` (2a Task 2), and `LogEventHandler::on_event`
    /// renders `Severity::Fatal` through `action_critical_log!`, which emits
    /// at slog's `Critical` level (the production `Json` drain serializes
    /// that as `"level":"CRITICAL"`).
    #[test]
    fn fatal_diagnostic_renders_as_a_single_critical_line() {
        let d = Diagnostic::from_draft(
            DiagnosticDraft::new(ErrorCode::InternalInvariantViolation),
            Some("node-x".into()),
            Some("TestAction".into()),
            None,
        );
        assert_eq!(d.severity, reearth_flow_diagnostics::Severity::Fatal);
        let message = d.message.clone();

        let (level, msg) = render_diagnostic(d);

        assert_eq!(level, Level::Critical);
        assert!(
            msg.contains(&message),
            "expected msg to contain {message:?}, got {msg:?}"
        );
    }

    /// Companion case: a Warn-severity diagnostic renders at slog's `Warning`
    /// level through the exact same handler path, proving `Severity` — not
    /// the call site — drives the emitted log level.
    #[test]
    fn warn_diagnostic_renders_as_a_single_warning_line() {
        let d = Diagnostic::from_draft(
            DiagnosticDraft::new(ErrorCode::GltfZeroFaceSolid),
            Some("node-y".into()),
            Some("TestAction".into()),
            None,
        );
        assert_eq!(d.severity, reearth_flow_diagnostics::Severity::Warn);
        let message = d.message.clone();

        let (level, msg) = render_diagnostic(d);

        assert_eq!(level, Level::Warning);
        assert!(
            msg.contains(&message),
            "expected msg to contain {message:?}, got {msg:?}"
        );
    }
}
