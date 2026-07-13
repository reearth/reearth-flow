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
    use std::fs;

    use reearth_flow_action_log::factory;
    use reearth_flow_diagnostics::{Diagnostic, DiagnosticDraft, ErrorCode};
    use reearth_flow_runtime::event::{Event, EventHandler};

    use super::*;

    /// Sends a single `Event::Diagnostic(Arc::new(diagnostic))` through
    /// `LogEventHandler::on_event`, then flushes and returns the resulting
    /// per-job `{job_id}.log` contents.
    ///
    /// `LoggerFactory::action_logger(&job_id.to_string())` (called from
    /// `LogEventHandler::new`) names the per-job log file after the action
    /// string it's given — `{job_id}.log` — and wraps its drain in
    /// `slog_async::Async` (see `action-log/src/split.rs`): writes are
    /// enqueued to a background thread, not applied synchronously.
    /// `slog-async`'s `AsyncCore::drop` only sends the flush/terminate
    /// message and joins that thread once the *last* reference to the
    /// drain's `Arc` is dropped. `LogEventHandler` is the sole owner of its
    /// `Arc<ActionLogger>` here (never cloned), so dropping `handler` on
    /// this thread — never the background writer thread itself —
    /// deterministically blocks until every record enqueued on the per-job
    /// drain has been written to `{job_id}.log`, with no sleep/poll needed
    /// before reading.
    ///
    /// We deliberately do NOT read `all.log` here: it's produced by
    /// `factory::create_root_logger`, whose async drain is the shared
    /// parent of every per-job logger and stays alive on `logger_factory`
    /// after `handler` is dropped. Its background thread is never joined
    /// by this test, so a read of `all.log` would depend on scheduling
    /// luck rather than the deterministic drop-flush guaranteed above.
    fn render_diagnostic_to_action_log(diagnostic: Diagnostic) -> String {
        let tempdir = tempfile::tempdir().unwrap();
        let action_log_dir = tempdir.path().join("action-log");
        fs::create_dir_all(&action_log_dir).unwrap();

        let root_logger = factory::create_root_logger(action_log_dir.clone());
        let logger_factory = Arc::new(LoggerFactory::new(root_logger, action_log_dir.clone()));

        let job_id = uuid::Uuid::new_v4();
        let handler = LogEventHandler::new(uuid::Uuid::new_v4(), job_id, logger_factory);

        let event = Event::Diagnostic(Arc::new(diagnostic));
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(handler.on_event(&event));

        // Flush: see the doc comment above for why this drop must happen
        // here, before the read below.
        drop(handler);

        let job_log_path = action_log_dir.join(format!("{job_id}.log"));
        fs::read_to_string(&job_log_path).unwrap_or_else(|e| {
            let listing = fs::read_dir(&action_log_dir)
                .map(|entries| {
                    entries
                        .filter_map(|entry| {
                            entry
                                .ok()
                                .map(|e| e.file_name().to_string_lossy().into_owned())
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            panic!(
                "failed to read {}: {e}; action-log dir contains: {listing:?}",
                job_log_path.display()
            )
        })
    }

    fn single_json_line(content: &str) -> serde_json::Value {
        let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
        assert_eq!(
            lines.len(),
            1,
            "expected exactly one action-log line, got {}: {content:?}",
            lines.len()
        );
        serde_json::from_str(lines[0]).expect("action-log line must be valid JSON")
    }

    /// End-to-end proof of the CRITICAL path at the handler level (2a Task
    /// 12): a Fatal-disposition code (`InternalInvariantViolation`, whose
    /// registry default disposition is `Fatal`) defaults to
    /// `Severity::Fatal` per `Diagnostic::from_draft` (2a Task 2), and
    /// `LogEventHandler::on_event` renders `Severity::Fatal` through
    /// `action_critical_log!`, which the `Json` drain serializes as
    /// `"level":"CRITICAL"` (see `slog::Level::Critical::as_str()`).
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

        let content = render_diagnostic_to_action_log(d);
        let parsed = single_json_line(&content);

        assert_eq!(parsed["level"], "CRITICAL");
        assert!(
            parsed["msg"]
                .as_str()
                .expect("msg must be a string")
                .contains(&message),
            "expected msg to contain {message:?}, got {parsed}"
        );
    }

    /// Companion case: a Warn-severity diagnostic renders at `"WARNING"`
    /// through the exact same handler path, proving `Severity` — not the
    /// call site — drives the rendered log level.
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

        let content = render_diagnostic_to_action_log(d);
        let parsed = single_json_line(&content);

        assert_eq!(parsed["level"], "WARNING");
        assert!(
            parsed["msg"]
                .as_str()
                .expect("msg must be a string")
                .contains(&message),
            "expected msg to contain {message:?}, got {parsed}"
        );
    }
}
