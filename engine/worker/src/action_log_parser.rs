use regex::Regex;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum LogPattern {
    WorkflowStart,
    WorkflowCompleted,
    WorkflowFailed(#[allow(dead_code)] String),
    NodeStart {
        node_name: String,
        node_id: Option<String>,
    },
    NodeFinish {
        node_name: String,
        node_id: Option<String>,
        elapsed: Duration,
    },
    NodeTerminate,
    NodeError {
        node_name: String,
        node_id: Option<String>,
        error: String,
    },
}

pub struct LogParser {
    workflow_start: Regex,
    node_start: Regex,
    node_finish: Regex,
    node_terminate: Regex,
    processor_error: Regex,
    source_error: Regex,
    sink_error: Regex,
    workflow_failed: Regex,
    workflow_completed: Regex,
    factory_error: Regex,
}

impl LogParser {
    pub fn new() -> Self {
        Self {
            workflow_start: Regex::new(r"Start workflow =").unwrap(),
            node_start: Regex::new(r#"([\w][\w ]*) (process start|sink start|source start)\.\.\."#).unwrap(),
            node_finish: Regex::new(r#"([\w][\w ]*) (process finish|sink finish|source finish)\. elapsed = ([\d\.]+)(m?s|µs)"#).unwrap(),
            node_terminate: Regex::new(r#"([\w][\w ]*) (process terminate|sink terminate|source terminate)\. elapsed = ([\d\.]+)(m?s|µs)"#).unwrap(),
            processor_error: Regex::new(r#"Error operation, processor node name = ([\w][\w ]*) \(([^)]+)\), node_id = ([a-f0-9-]+), .+, error = (.+)"#).unwrap(),
            source_error: Regex::new(r#"([\w][\w ]*) source error: (.+)"#).unwrap(),
            sink_error: Regex::new(r#"([\w][\w ]*) sink error: (.+)"#).unwrap(),
            workflow_failed: Regex::new(r"Failed nodes:").unwrap(),
            workflow_completed: Regex::new(r"Finish workflow = .* \((success|failed)\)").unwrap(),
            factory_error: Regex::new(r#"Failed to workflow: ExecutionError\(Factory \{ node_id: "([^"]+)", node_name: "([^"]+)", error: ([^(]+)\("#).unwrap(),
        }
    }

    pub fn parse(&self, log_line: &str) -> Option<LogPattern> {
        // Workflow patterns
        if self.workflow_start.is_match(log_line) {
            return Some(LogPattern::WorkflowStart);
        }

        if self.workflow_completed.is_match(log_line) {
            return Some(LogPattern::WorkflowCompleted);
        }

        // Check specific error patterns before generic workflow error
        if let Some(caps) = self.factory_error.captures(log_line) {
            let node_id = caps.get(1).unwrap().as_str();
            let node_name = caps.get(2).unwrap().as_str();
            let factory_name = caps.get(3).unwrap().as_str();
            let action_name = if factory_name.ends_with("Factory") {
                factory_name.strip_suffix("Factory").unwrap_or(factory_name)
            } else {
                factory_name
            };
            // Extract the actual error message from the log
            let error_detail =
                if let Some(error_match) = log_line.split(&format!("{factory_name}(\"")).nth(1) {
                    if let Some(msg_end) = error_match.find("\")") {
                        let msg = &error_match[..msg_end];
                        format!("Invalid configuration for {action_name}: {msg}")
                    } else {
                        format!("Invalid configuration for {action_name}")
                    }
                } else {
                    format!("Invalid configuration for {action_name}")
                };
            // For now, still return as WorkflowFailed but include node info in the message
            return Some(LogPattern::NodeError {
                node_name: node_name.to_string(),
                node_id: Some(node_id.to_string()),
                error: error_detail,
            });
        }

        if let Some(caps) = self.source_error.captures(log_line) {
            let node_name = caps.get(1).unwrap().as_str().to_string();
            let error = caps.get(2).unwrap().as_str().to_string();
            return Some(LogPattern::NodeError {
                node_name,
                node_id: None,
                error,
            });
        }

        if self.workflow_failed.is_match(log_line) {
            let error = "Workflow execution failed".to_string();
            return Some(LogPattern::WorkflowFailed(error));
        }

        if let Some(caps) = self.node_start.captures(log_line) {
            let node_name = caps.get(1).unwrap().as_str().to_string();
            return Some(LogPattern::NodeStart {
                node_name,
                node_id: None,
            });
        }

        if let Some(caps) = self.node_finish.captures(log_line) {
            let node_name = caps.get(1).unwrap().as_str().to_string();
            let elapsed =
                self.parse_duration(caps.get(3).unwrap().as_str(), caps.get(4).unwrap().as_str());
            return Some(LogPattern::NodeFinish {
                node_name,
                node_id: None,
                elapsed,
            });
        }

        if let Some(_caps) = self.node_terminate.captures(log_line) {
            return Some(LogPattern::NodeTerminate);
        }

        if let Some(caps) = self.processor_error.captures(log_line) {
            let node_name = caps.get(2).unwrap().as_str().to_string();
            let node_id = Some(caps.get(3).unwrap().as_str().to_string());
            let mut error = caps.get(4).unwrap().as_str().to_string();

            // Handle error format like: BulkRenamer(\"error message\")
            if let Some(start_idx) = error.find("(\\\"") {
                if let Some(end_idx) = error.rfind("\\\")") {
                    let prefix_len = "(\\\"".len();
                    error = error[start_idx + prefix_len..end_idx].to_string();
                }
            }
            // Handle error format like: "error message"
            else if error.starts_with('"') && error.ends_with('"') {
                error = error[1..error.len() - 1].to_string();
            }

            if error.contains("Failed to process attributes") {
                error = "Failed to process attributes".to_string();
            }
            return Some(LogPattern::NodeError {
                node_name,
                node_id,
                error,
            });
        }

        if let Some(caps) = self.sink_error.captures(log_line) {
            let node_name = caps.get(1).unwrap().as_str().to_string();
            let mut error = caps.get(2).unwrap().as_str().to_string();

            // Handle CannotReceiveFromChannel wrapper
            const CHANNEL_ERROR_PREFIX: &str = "CannotReceiveFromChannel(\"";
            const CHANNEL_ERROR_SUFFIX: &str = "\")";

            if error.starts_with(CHANNEL_ERROR_PREFIX) && error.ends_with(CHANNEL_ERROR_SUFFIX) {
                error = error[CHANNEL_ERROR_PREFIX.len()..error.len() - CHANNEL_ERROR_SUFFIX.len()]
                    .to_string();

                // Handle nested error patterns like SinkType("error message")
                if let Some(paren_start) = error.find('(') {
                    if let Some(quote_start) = error[paren_start..].find('"') {
                        let quote_start = paren_start + quote_start + 1;
                        if let Some(quote_end) = error[quote_start..].find('"') {
                            let quote_end = quote_start + quote_end;
                            error = error[quote_start..quote_end].to_string();
                        }
                    }
                }
            }

            return Some(LogPattern::NodeError {
                node_name,
                node_id: None,
                error,
            });
        }

        None
    }

    fn parse_duration(&self, time_str: &str, unit: &str) -> Duration {
        let value: f64 = time_str.parse().unwrap_or(0.0);

        let value = value.clamp(0.0, 86_400_000.0);

        match unit {
            "ms" => Duration::from_secs_f64(value / 1000.0),
            "µs" => Duration::from_secs_f64(value / 1_000_000.0),
            _ => Duration::from_secs_f64(value),
        }
    }
}

#[cfg(test)]
mod sink_and_processor_error_shape_tests {
    //! Phase 2a Task 8 rewired `SinkNode`/`ProcessorNode` finish()/process()
    //! failures to preserve structured diagnostics: the wrapping
    //! `ExecutionError` variant changed from `CannotReceiveFromChannel`
    //! (Display: "Cannot receive from channel: {0}") to `Sink`/`Processor`
    //! (Display: "Sink error: {0}"/"Processor error: {0}"). These tests lock
    //! in that `sink_error` still parses both the legacy Debug-wrapper shape
    //! (kept matching for the deploy gap, per the migration posture) and the
    //! new structured shape, and that `processor_error` — which matches an
    //! unrelated per-feature `process()` log line, not the finish() path
    //! Task 8 touched — is unaffected either way.
    use super::*;

    fn node_error(pattern: LogPattern) -> (String, Option<String>, String) {
        match pattern {
            LogPattern::NodeError {
                node_name,
                node_id,
                error,
            } => (node_name, node_id, error),
            other => panic!("expected NodeError, got {other:?}"),
        }
    }

    /// Legacy shape, from before Task 8: the sink log line's `{}` formatted
    /// an `ExecutionError::CannotReceiveFromChannel(format!("{e:?}"))`,
    /// whose Display is "Cannot receive from channel: {0}" — but the
    /// unwrap's `CHANNEL_ERROR_PREFIX` check targets the *Debug* tuple-variant
    /// syntax, `CannotReceiveFromChannel("...")`, matching an older log
    /// format. Keeping this parseable maintains compatibility with any
    /// still-running old-shape worker/engine build pairing during a
    /// rolling deploy.
    #[test]
    fn sink_error_strips_legacy_debug_wrapper_when_present() {
        let parser = LogParser::new();
        let msg = r#"JSON Writer sink error: CannotReceiveFromChannel("boom")"#;

        let pattern = parser.parse(msg).expect("sink_error must match");
        let (node_name, node_id, error) = node_error(pattern);

        assert_eq!(node_name, "JSON Writer");
        assert_eq!(node_id, None);
        assert_eq!(error, "boom");
    }

    /// New shape, post-Task-8: the sink log line's `{}` now formats
    /// `ExecutionError::Sink(e)` directly, Display "Sink error: {0}". This
    /// doesn't start with `CannotReceiveFromChannel("` so the legacy unwrap
    /// is a no-op and the full structured text passes through untouched —
    /// which is the desired, MORE informative user-facing message (no
    /// longer mislabeled as a channel error).
    #[test]
    fn sink_error_passes_through_new_structured_shape_untouched() {
        let parser = LogParser::new();
        let msg = "JSON Writer sink error: Sink error: boom";

        let pattern = parser.parse(msg).expect("sink_error must match");
        let (node_name, node_id, error) = node_error(pattern);

        assert_eq!(node_name, "JSON Writer");
        assert_eq!(node_id, None);
        assert_eq!(error, "Sink error: boom");
    }

    /// `processor_error` matches the per-feature `process()` failure log
    /// line ("Error operation, processor node name = ..."), which is
    /// produced independently of the `ExecutionError` enum Task 8 changed
    /// (it Debug-formats the action's raw error directly). Confirms Task 8
    /// left this pattern's behavior untouched.
    #[test]
    fn processor_error_shape_is_unaffected_by_the_executionerror_display_change() {
        let parser = LogParser::new();
        let msg = "Error operation, processor node name = Attribute Aggregator (ErrorProcessor), node_id = b1fa0a3e-61d3-48e2-a328-e7226c2ad1ae, feature id = None, error = \"Attribute not found: nonexistentAttribute\"";

        let pattern = parser.parse(msg).expect("processor_error must match");
        let (node_name, node_id, error) = node_error(pattern);

        assert_eq!(node_name, "ErrorProcessor");
        assert_eq!(
            node_id.as_deref(),
            Some("b1fa0a3e-61d3-48e2-a328-e7226c2ad1ae")
        );
        assert_eq!(error, "Attribute not found: nonexistentAttribute");
    }

    /// Real production shape from pre-enrichment legacy logs: the sink log
    /// line's `{}` formatted an `ExecutionError::CannotReceiveFromChannel`
    /// whose Display is "Cannot receive from channel: {0}". This captures
    /// the actual production Display output when an action's error is wrapped
    /// with `format!("{e:?}")` inside the variant.
    #[test]
    fn sink_error_parses_real_legacy_channel_display_text() {
        let parser = LogParser::new();
        let msg = r#"MyWriter sink error: Cannot receive from channel: SinkError("boom")"#;

        let pattern = parser.parse(msg).expect("sink_error must match");
        let (node_name, node_id, error) = node_error(pattern);

        assert_eq!(node_name, "MyWriter");
        assert_eq!(node_id, None);
        assert_eq!(error, r#"Cannot receive from channel: SinkError("boom")"#);
    }
}
