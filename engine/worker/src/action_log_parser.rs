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
    NodeError {
        node_name: String,
        error: String,
    },
    SourceEvaluationError {
        node_name: String,
        error: String,
    },
}

pub struct LogParser {
    workflow_start: Regex,
    node_start: Regex,
    node_starting: Regex,
    node_finish: Regex,
    processor_error: Regex,
    source_error: Regex,
    workflow_failed: Regex,
    workflow_completed: Regex,
    factory_error: Regex,
    workflow_error: Regex,
}

impl LogParser {
    pub fn new() -> Self {
        Self {
            workflow_start: Regex::new(r"Start workflow =").unwrap(),
            node_start: Regex::new(r#"([^(]+) \(([^)]+)\) (process start|sink start|source start)"#).unwrap(),
            node_starting: Regex::new(r"Starting").unwrap(),
            node_finish: Regex::new(r#"([^(]+) \(([^)]+)\) (process finish|sink finish|finish source complete)\. elapsed = ([\d\.]+)(m?s|µs)"#).unwrap(),
            processor_error: Regex::new(r#"Error operation, processor node name = ([^(]+) \(([^)]+)\), node_id = .*, error = "([^"]+)""#).unwrap(),
            source_error: Regex::new(r"Failed to workflow: ExecutionError\(Source\(([^(]+)").unwrap(),
            workflow_failed: Regex::new(r"Failed nodes:").unwrap(),
            workflow_completed: Regex::new(r"Finish workflow =").unwrap(),
            factory_error: Regex::new(r"Failed to workflow: ExecutionError\(Factory\(([^(]+)").unwrap(),
            workflow_error: Regex::new(r"Failed to workflow").unwrap(),
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
            let error_detail = if let Some(error_match) = log_line.split(&format!("{}(\"", factory_name)).nth(1) {
                if let Some(msg_end) = error_match.find("\")") {
                    let msg = &error_match[..msg_end];
                    format!("Invalid configuration for {}: {}", action_name, msg)
                } else {
                    format!("Invalid configuration for {}", action_name)
                }
            } else {
                format!("Invalid configuration for {}", action_name)
            };
            return Some(LogPattern::WorkflowFailed(error_detail));
        }

        if let Some(caps) = self.source_error.captures(log_line) {
            let source_name = caps.get(1).unwrap().as_str();
            let node_name = source_name.to_string();
            let error = "Invalid expression syntax".to_string();
            return Some(LogPattern::SourceEvaluationError { node_name, error });
        }

        if self.workflow_failed.is_match(log_line) {
            let error = "Workflow execution failed".to_string();
            return Some(LogPattern::WorkflowFailed(error));
        }

        // Skip generic workflow errors - they're handled by specific error patterns above
        // (Factory errors, Source errors, etc.)

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

        if let Some(caps) = self.processor_error.captures(log_line) {
            let node_name = caps.get(2).unwrap().as_str().to_string();
            let mut error = caps.get(3).unwrap().as_str().to_string();
            if error.contains("Failed to process attributes") {
                error = "Failed to process attributes".to_string();
            }
            return Some(LogPattern::NodeError { node_name, error });
        }

        None
    }

    fn parse_duration(&self, time_str: &str, unit: &str) -> Duration {
        let value: f64 = time_str.parse().unwrap_or(0.0);
        match unit {
            "ms" => Duration::from_secs_f64(value / 1000.0),
            "µs" => Duration::from_secs_f64(value / 1_000_000.0),
            _ => Duration::from_secs_f64(value),
        }
    }
}
