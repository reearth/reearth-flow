use std::collections::HashSet;
use std::io::Write;
use std::sync::Arc;
use std::time::Duration;

use backon::{ExponentialBuilder, Retryable};
use chrono::Utc;
use parking_lot::RwLock;
use reearth_flow_runner::node_info_tls;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;
use uuid::Uuid;

use crate::action_log_parser::{LogParser, LogPattern};
use crate::pubsub::backend::PubSubBackend;
use crate::pubsub::publisher::Publisher;
use crate::types::user_facing_log_event::{UserFacingLogEvent, UserFacingLogLevel};

#[derive(Clone, Debug)]
pub struct WorkflowExecutionInfo {
    pub workflow_name: String,
}

pub struct UserFacingLogHandler {
    workflow_id: Uuid,
    job_id: Uuid,
    publisher: PubSubBackend,
    tokio_handle: tokio::runtime::Handle,
    workflow_info: Arc<RwLock<Option<WorkflowExecutionInfo>>>,
    // Track workflow errors
    workflow_error_occurred: Arc<RwLock<bool>>,
    // Track which specific nodes failed
    failed_nodes: Arc<RwLock<HashSet<String>>>,
    // Workflow started flag
    workflow_started: Arc<RwLock<bool>>,
    // Log parser
    log_parser: LogParser,
    // Count of active node instances for workflow completion detection
    active_node_count: Arc<RwLock<usize>>,
    // Store pending workflow completion message
    pending_workflow_completion: Arc<RwLock<Option<UserFacingLogEvent>>>,
    // Track if workflow failed message was already emitted
    workflow_failed_emitted: Arc<RwLock<bool>>,
}

impl UserFacingLogHandler {
    pub fn new(
        workflow_id: Uuid,
        job_id: Uuid,
        publisher: PubSubBackend,
        tokio_handle: tokio::runtime::Handle,
    ) -> Self {
        Self {
            workflow_id,
            job_id,
            publisher,
            tokio_handle,
            workflow_info: Arc::new(RwLock::new(None)),
            workflow_error_occurred: Arc::new(RwLock::new(false)),
            failed_nodes: Arc::new(RwLock::new(HashSet::new())),
            workflow_started: Arc::new(RwLock::new(false)),
            log_parser: LogParser::new(),
            active_node_count: Arc::new(RwLock::new(0)),
            pending_workflow_completion: Arc::new(RwLock::new(None)),
            workflow_failed_emitted: Arc::new(RwLock::new(false)),
        }
    }

    pub fn set_workflow_name(&self, workflow_name: String) {
        let mut workflow_info = self.workflow_info.write();
        *workflow_info = Some(WorkflowExecutionInfo { workflow_name });
    }

    pub fn send_workflow_definition_error(&self, error: &dyn std::error::Error) {
        let display_message = Self::format_workflow_definition_error(error);

        let event = UserFacingLogEvent {
            workflow_id: self.workflow_id,
            job_id: self.job_id,
            timestamp: Utc::now(),
            level: UserFacingLogLevel::Error,
            node_name: None,
            node_id: None,
            display_message,
        };

        self.publish_event(event);
    }

    fn format_workflow_definition_error(error: &dyn std::error::Error) -> String {
        format!("Workflow definition error: {error}")
    }

    fn publish_event(&self, event: UserFacingLogEvent) {
        self.write_to_file(&event);

        let publisher = self.publisher.clone();
        let event_clone = event.clone();
        let handle = self.tokio_handle.clone();

        handle.spawn(async move {
            // Retry logic with exponential backoff
            let publish_operation = || async {
                let timeout_duration = Duration::from_secs(5);
                tokio::time::timeout(timeout_duration, async {
                    match &publisher {
                        PubSubBackend::Google(p) => p
                            .publish(event_clone.clone())
                            .await
                            .map_err(|e| e.to_string()),
                        PubSubBackend::Noop(p) => p
                            .publish(event_clone.clone())
                            .await
                            .map_err(|e| format!("{e:?}")),
                    }
                })
                .await
                .map_err(|_| "Timeout while publishing user-facing log event".to_string())
                .and_then(|result| result)
            };

            let result = publish_operation
                .retry(
                    ExponentialBuilder::default()
                        .with_max_times(3)
                        .with_min_delay(Duration::from_millis(100))
                        .with_max_delay(Duration::from_secs(5))
                        .with_jitter(),
                )
                .await;

            if let Err(e) = result {
                tracing::error!(
                    "Failed to publish user-facing log event after 3 retries: {}",
                    e
                );
            }
        });
    }

    fn process_log_pattern(
        &self,
        pattern: LogPattern,
        event_node_id: Option<String>,
        event_node_name: Option<String>,
    ) -> Option<UserFacingLogEvent> {
        match pattern {
            LogPattern::WorkflowStart => {
                let workflow_started = *self.workflow_started.read();
                if !workflow_started {
                    *self.workflow_started.write() = true;
                    let workflow_info = self.workflow_info.read();
                    let workflow_name = workflow_info
                        .as_ref()
                        .map(|info| info.workflow_name.clone())
                        .unwrap_or_else(|| "Unknown".to_string());
                    drop(workflow_info);

                    Some(UserFacingLogEvent {
                        workflow_id: self.workflow_id,
                        job_id: self.job_id,
                        timestamp: Utc::now(),
                        level: UserFacingLogLevel::Info,
                        node_name: None,
                        node_id: None,
                        display_message: format!("{workflow_name} Workflow - Started..."),
                    })
                } else {
                    None
                }
            }

            LogPattern::NodeStart { node_name, node_id } => {
                *self.active_node_count.write() += 1;

                let final_node_name = event_node_name.clone().unwrap_or(node_name.clone());
                let final_node_id = event_node_id.clone().or(node_id.clone());

                tracing::debug!(
                    "NodeStart: parsed_name={}, parsed_id={:?}, event_name={:?}, event_id={:?}, final_name={}, final_id={:?}",
                    node_name, node_id, event_node_name, event_node_id, final_node_name, final_node_id
                );

                Some(UserFacingLogEvent {
                    workflow_id: self.workflow_id,
                    job_id: self.job_id,
                    timestamp: Utc::now(),
                    level: UserFacingLogLevel::Info,
                    node_name: Some(final_node_name.clone()),
                    node_id: final_node_id,
                    display_message: format!("{final_node_name} - Running..."),
                })
            }

            LogPattern::NodeFinish {
                node_name,
                node_id,
                elapsed,
            } => {
                let final_node_name = event_node_name.clone().unwrap_or(node_name.clone());
                let final_node_id = event_node_id.clone().or(node_id.clone());

                let mut active_count = self.active_node_count.write();
                if *active_count > 0 {
                    *active_count -= 1;
                }
                drop(active_count);

                let is_failed = self.failed_nodes.read().contains(&final_node_name);

                let result = if !is_failed {
                    Some(UserFacingLogEvent {
                        workflow_id: self.workflow_id,
                        job_id: self.job_id,
                        timestamp: Utc::now(),
                        level: UserFacingLogLevel::Success,
                        node_name: Some(final_node_name.clone()),
                        node_id: final_node_id,
                        display_message: format!(
                            "{final_node_name} - Finished in {:.2}s",
                            elapsed.as_secs_f64()
                        ),
                    })
                } else {
                    None
                };

                self.check_and_complete_workflow();

                result
            }

            LogPattern::NodeError {
                node_name,
                node_id,
                error,
            } => {
                let final_node_name = event_node_name.clone().unwrap_or(node_name.clone());
                let final_node_id = event_node_id.clone().or(node_id.clone());

                self.failed_nodes.write().insert(final_node_name.clone());
                *self.workflow_error_occurred.write() = true;

                let mut active_count = self.active_node_count.write();
                if *active_count > 0 {
                    *active_count -= 1;
                }
                drop(active_count);

                let simple_error = if error.contains("Failed to process attributes") {
                    "Failed to process attributes".to_string()
                } else {
                    error
                };

                let result = Some(UserFacingLogEvent {
                    workflow_id: self.workflow_id,
                    job_id: self.job_id,
                    timestamp: Utc::now(),
                    level: UserFacingLogLevel::Error,
                    node_name: Some(final_node_name.clone()),
                    node_id: final_node_id,
                    display_message: format!("{final_node_name} - Failed: {simple_error}"),
                });

                // Check if all nodes are completed after processing this error event
                self.check_and_complete_workflow();

                result
            }

            LogPattern::WorkflowCompleted => {
                let error_occurred = *self.workflow_error_occurred.read();
                let workflow_failed_emitted = *self.workflow_failed_emitted.read();

                // If we already emitted a workflow failed message, skip the completion message
                if error_occurred && workflow_failed_emitted {
                    None
                } else {
                    let completion_event = if !error_occurred {
                        UserFacingLogEvent {
                            workflow_id: self.workflow_id,
                            job_id: self.job_id,
                            timestamp: Utc::now(),
                            level: UserFacingLogLevel::Success,
                            node_name: None,
                            node_id: None,
                            display_message: "Workflow finished successfully.".to_string(),
                        }
                    } else {
                        UserFacingLogEvent {
                            workflow_id: self.workflow_id,
                            job_id: self.job_id,
                            timestamp: Utc::now(),
                            level: UserFacingLogLevel::Error,
                            node_name: None,
                            node_id: None,
                            display_message: "Workflow execution failed.".to_string(),
                        }
                    };

                    // Check if all nodes are already completed
                    let active_count = *self.active_node_count.read();
                    tracing::debug!(
                        "WorkflowCompleted: active_count={}, error_occurred={}",
                        active_count,
                        error_occurred
                    );
                    if active_count == 0 {
                        // All nodes completed, emit immediately
                        Some(completion_event)
                    } else {
                        // Store for later emission when all nodes complete
                        *self.pending_workflow_completion.write() = Some(completion_event);

                        // Check if we should emit immediately due to error
                        self.check_and_complete_workflow();

                        None
                    }
                }
            }

            LogPattern::WorkflowFailed(error_message) => {
                *self.workflow_error_occurred.write() = true;

                // Check if we should suppress this message to avoid duplicates
                let mut workflow_failed_emitted = self.workflow_failed_emitted.write();
                if *workflow_failed_emitted {
                    // Already emitted a workflow failed message, skip this one
                    None
                } else {
                    *workflow_failed_emitted = true;

                    // Create the specific error message event
                    let error_event = UserFacingLogEvent {
                        workflow_id: self.workflow_id,
                        job_id: self.job_id,
                        timestamp: Utc::now(),
                        level: UserFacingLogLevel::Error,
                        node_name: None,
                        node_id: None,
                        display_message: error_message.clone(),
                    };

                    // For factory errors, return both messages combined
                    if error_message.contains("Invalid configuration") {
                        // Combine both messages to ensure proper ordering
                        self.publish_event(error_event.clone());

                        let failed_event = UserFacingLogEvent {
                            workflow_id: self.workflow_id,
                            job_id: self.job_id,
                            timestamp: Utc::now(),
                            level: UserFacingLogLevel::Error,
                            node_name: None,
                            node_id: None,
                            display_message: "Workflow execution failed.".to_string(),
                        };
                        Some(failed_event)
                    } else {
                        Some(error_event)
                    }
                }
            }

            LogPattern::NodeTerminate => {
                // For terminate events, decrement active count but don't emit user-facing log
                let mut active_count = self.active_node_count.write();
                if *active_count > 0 {
                    *active_count -= 1;
                }
                drop(active_count);

                // Check if all nodes are completed after processing this terminate event
                self.check_and_complete_workflow();

                // Return None to suppress the terminate log in user-facing output
                None
            }
        }
    }

    fn check_and_complete_workflow(&self) {
        let active_count = *self.active_node_count.read();
        let error_occurred = *self.workflow_error_occurred.read();

        // If workflow has error, we should complete even if there are pending nodes
        // (e.g., sink nodes that won't receive data due to source error)
        if active_count == 0 || error_occurred {
            let mut pending = self.pending_workflow_completion.write();
            if let Some(completion_event) = pending.take() {
                tracing::debug!(
                    "check_and_complete_workflow: publishing completion event, active_count={}, error_occurred={}",
                    active_count,
                    error_occurred
                );
                self.publish_event(completion_event);
            }
        }
    }

    fn write_to_file(&self, event: &UserFacingLogEvent) {
        if let Ok(json_line) = serde_json::to_string(event) {
            let writer = crate::logger::DynamicUserFacingLogFileWriter;
            let mut file_writer = writer.make_writer();
            if let Err(e) = writeln!(file_writer, "{json_line}") {
                tracing::error!("Failed to write user-facing log to file: {}", e);
            }
        }
    }

    fn should_process_event(&self, _event: &Event<'_>, target: &str, level: &Level) -> bool {
        match *level {
            Level::INFO => {
                if target.contains("reearth_flow_runner::runner") {
                    return true;
                }
                if target.contains("reearth_flow_runtime::executor") {
                    return true;
                }
                if target.contains("reearth_flow_runner::log_event_handler") {
                    return true;
                }
            }
            Level::ERROR | Level::WARN => {
                if target.contains("reearth_flow") {
                    return true;
                }
                if target.contains("action-")
                    || target.contains("processor")
                    || target.contains("sink")
                {
                    return true;
                }
            }
            _ => {}
        }
        false
    }
}

#[derive(Clone, Debug)]
struct SpanFields {
    node_id: Option<String>,
    node_name: Option<String>,
    workflow_name: Option<String>,
}

#[derive(Clone)]
pub struct UserFacingLogLayer {
    handler: Arc<UserFacingLogHandler>,
}

impl UserFacingLogLayer {
    pub fn new(handler: Arc<UserFacingLogHandler>) -> Self {
        Self { handler }
    }
}

impl<S> Layer<S> for UserFacingLogLayer
where
    S: Subscriber + for<'a> LookupSpan<'a> + Send + Sync,
{
    fn on_new_span(
        &self,
        attrs: &tracing::span::Attributes<'_>,
        id: &tracing::span::Id,
        ctx: Context<'_, S>,
    ) {
        let span = ctx.span(id).unwrap();
        let mut fields = SpanFields {
            node_id: None,
            node_name: None,
            workflow_name: None,
        };

        let mut visitor = FieldVisitor::new(&mut fields);
        attrs.record(&mut visitor);

        let mut extensions = span.extensions_mut();
        extensions.insert(fields.clone());

        if span.name() == "workflow_execution" {
            if let Some(workflow_name) = fields.workflow_name {
                let mut workflow_info = self.handler.workflow_info.write();
                *workflow_info = Some(WorkflowExecutionInfo {
                    workflow_name: workflow_name.clone(),
                });
            }
        }
    }

    fn on_record(
        &self,
        id: &tracing::span::Id,
        values: &tracing::span::Record<'_>,
        ctx: Context<'_, S>,
    ) {
        if let Some(span) = ctx.span(id) {
            let mut extensions = span.extensions_mut();
            if let Some(fields) = extensions.get_mut::<SpanFields>() {
                let mut visitor = FieldVisitor::new(fields);
                values.record(&mut visitor);
            }
        }
    }

    fn on_close(&self, _id: tracing::span::Id, _ctx: Context<'_, S>) {}

    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let meta = event.metadata();
        let level = meta.level();
        let target = meta.target();

        if self.handler.should_process_event(event, target, level) {
            let mut fields_extractor = EventFieldsExtractor::default();
            event.record(&mut fields_extractor);
            let message = fields_extractor.message.unwrap_or_else(|| "".to_string());
            let mut event_node_id = fields_extractor.node_id;
            let mut event_node_name = fields_extractor.node_name;

            if target.contains("reearth_flow_runner::log_event_handler")
                && (event_node_id.is_none() || event_node_name.is_none())
            {
                if let Some((tls_node_id, tls_node_name)) = node_info_tls::get_node_info() {
                    if event_node_id.is_none() {
                        event_node_id = Some(tls_node_id);
                    }
                    if event_node_name.is_none() {
                        event_node_name = tls_node_name;
                    }
                }
            }

            const INTERNAL_LOG_FILTERS: &[(&str, &str)] = &[
                ("reearth_flow_worker::event_handler", "Node failed:"),
                ("reearth_flow_worker::command", "Failed nodes:"),
            ];

            if INTERNAL_LOG_FILTERS
                .iter()
                .any(|(t, msg)| target.contains(t) && message.starts_with(msg))
            {
                return;
            }

            // Debug log for runner messages
            if target.contains("reearth_flow_runner::runner")
                && (message.contains("Start workflow") || message.contains("Finish workflow"))
            {
                tracing::debug!("Processing runner message: {}", message);
            }

            if let Some(pattern) = self.handler.log_parser.parse(&message) {
                tracing::debug!("Parsed pattern: {:?} from message: {}", pattern, message);

                if let Some(user_event) =
                    self.handler
                        .process_log_pattern(pattern, event_node_id, event_node_name)
                {
                    self.handler.publish_event(user_event);
                }
            } else if target.contains("reearth_flow_runner::runner") {
                tracing::debug!("No pattern matched for runner message: {}", message);
            }
        }
    }
}

struct FieldVisitor<'a> {
    fields: &'a mut SpanFields,
}

impl<'a> FieldVisitor<'a> {
    fn new(fields: &'a mut SpanFields) -> Self {
        Self { fields }
    }
}

impl tracing::field::Visit for FieldVisitor<'_> {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        match field.name() {
            "node_id" => self.fields.node_id = Some(value.to_string()),
            "node.id" => self.fields.node_id = Some(value.to_string()),
            "node_name" => self.fields.node_name = Some(value.to_string()),
            "node.name" => self.fields.node_name = Some(value.to_string()),
            "workflow.name" => self.fields.workflow_name = Some(value.to_string()),
            _ => {}
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        match field.name() {
            "node_id" => self.fields.node_id = Some(format!("{value:?}")),
            "node.id" => self.fields.node_id = Some(format!("{value:?}")),
            "node_name" => self.fields.node_name = Some(format!("{value:?}")),
            "node.name" => self.fields.node_name = Some(format!("{value:?}")),
            "workflow.name" => self.fields.workflow_name = Some(format!("{value:?}")),
            _ => {}
        }
    }
}

#[derive(Default)]
struct EventFieldsExtractor {
    message: Option<String>,
    node_id: Option<String>,
    node_name: Option<String>,
}

impl tracing::field::Visit for EventFieldsExtractor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        match field.name() {
            "message" => self.message = Some(value.to_string()),
            "node_id" => self.node_id = Some(value.to_string()),
            "node_name" => self.node_name = Some(value.to_string()),
            _ => {}
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        match field.name() {
            "message" if self.message.is_none() => {
                self.message = Some(format!("{value:?}"));
            }
            "node_id" if self.node_id.is_none() => {
                self.node_id = Some(format!("{value:?}"));
            }
            "node_name" if self.node_name.is_none() => {
                self.node_name = Some(format!("{value:?}"));
            }
            _ => {}
        }
    }
}
