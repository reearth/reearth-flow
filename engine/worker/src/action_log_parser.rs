use regex::Regex;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum LogPattern {
    WorkflowStart,
    WorkflowCompleted,
    WorkflowFailed(#[allow(dead_code)] String),
    NodeStart(String), // node_name
    NodeFinish {
        node_name: String,
        elapsed: Duration,
    },
    NodeTerminate,
    NodeError {
        node_name: String,
        error: String,
    },
}

pub struct LogParser {
    workflow_start: Regex,
    node_start: Regex,
    node_starting: Regex,
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
            node_start: Regex::new(r#"([^(]+) \(([^)]+)\) (process start|sink start|source start)"#).unwrap(),
            node_starting: Regex::new(r"Starting").unwrap(),
            node_finish: Regex::new(r#"([^(]+) \(([^)]+)\) (process finish|sink finish|source finish)\. elapsed = ([\d\.]+)(m?s|µs)"#).unwrap(),
            node_terminate: Regex::new(r#"([^(]+) \(([^)]+)\) (process terminate|sink terminate|source terminate)\. elapsed = ([\d\.]+)(m?s|µs)"#).unwrap(),
            processor_error: Regex::new(r#"Error operation, processor node name = ([^(]+) \(([^)]+)\), node_id = .*, error = (.+)"#).unwrap(),
            source_error: Regex::new(r#"([^(]+) \(([^)]+)\) source error: (.+)"#).unwrap(),
            sink_error: Regex::new(r#"([^(]+) \(([^)]+)\) sink error: (.+)"#).unwrap(),
            workflow_failed: Regex::new(r"Failed nodes:").unwrap(),
            workflow_completed: Regex::new(r"Finish workflow = .* \((success|failed)\)").unwrap(),
            factory_error: Regex::new(r"Failed to workflow: ExecutionError\(Factory\(([^(]+)").unwrap(),
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
            let factory_name = caps.get(1).unwrap().as_str();
            let action_name = if factory_name.ends_with("Factory") {
                factory_name.strip_suffix("Factory").unwrap_or(factory_name)
            } else {
                factory_name
            };
            // Factory errors occur before node execution, so treat as workflow-level error
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
            return Some(LogPattern::WorkflowFailed(error_detail));
        }

        if let Some(caps) = self.source_error.captures(log_line) {
            let node_name = caps.get(2).unwrap().as_str().to_string();
            let error = caps.get(3).unwrap().as_str().to_string();
            return Some(LogPattern::NodeError { node_name, error });
        }

        if self.workflow_failed.is_match(log_line) {
            let error = "Workflow execution failed".to_string();
            return Some(LogPattern::WorkflowFailed(error));
        }

        if let Some(caps) = self.node_starting.captures(log_line) {
            let node_name = caps.get(1).unwrap().as_str().to_string();
            return Some(LogPattern::NodeStart(node_name));
        }

        if let Some(caps) = self.node_start.captures(log_line) {
            let node_name = caps.get(2).unwrap().as_str().to_string();
            return Some(LogPattern::NodeStart(node_name));
        }

        if let Some(caps) = self.node_finish.captures(log_line) {
            let node_name = caps.get(2).unwrap().as_str().to_string();
            let elapsed =
                self.parse_duration(caps.get(4).unwrap().as_str(), caps.get(5).unwrap().as_str());
            return Some(LogPattern::NodeFinish { node_name, elapsed });
        }

        if let Some(_caps) = self.node_terminate.captures(log_line) {
            return Some(LogPattern::NodeTerminate);
        }

        if let Some(caps) = self.processor_error.captures(log_line) {
            let node_name = caps.get(2).unwrap().as_str().to_string();
            let mut error = caps.get(3).unwrap().as_str().to_string();

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
            return Some(LogPattern::NodeError { node_name, error });
        }

        if let Some(caps) = self.sink_error.captures(log_line) {
            let node_name = caps.get(2).unwrap().as_str().to_string();
            let mut error = caps.get(3).unwrap().as_str().to_string();

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

            return Some(LogPattern::NodeError { node_name, error });
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
