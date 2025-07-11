use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Write;
use std::sync::Arc;

use chrono::Utc;
use parking_lot::RwLock;
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

// Simple workflow structures for dependency analysis
#[derive(Debug, Clone, serde::Deserialize)]
struct SimpleWorkflow {
    name: String,
    graphs: Vec<SimpleGraph>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct SimpleGraph {
    nodes: Vec<SimpleNode>,
    edges: Vec<SimpleEdge>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct SimpleNode {
    id: String,
    name: String,
    #[serde(rename = "type")]
    node_type: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct SimpleEdge {
    from: String,
    to: String,
}

// Type alias for calculate_step_mapping return type
type StepMappingResult = (HashMap<String, usize>, HashMap<String, String>);

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
    // Step number mapping from topological order
    node_step_mapping: Arc<RwLock<HashMap<String, usize>>>,
    // Store node name information for runtime use
    node_name_mapping: Arc<RwLock<HashMap<String, String>>>,
    // Map node name to step number
    node_name_to_step: Arc<RwLock<HashMap<String, usize>>>,
    // Track workflow errors
    workflow_error_occurred: Arc<RwLock<bool>>,
    // Track which specific nodes failed
    failed_nodes: Arc<RwLock<HashSet<String>>>,
    // Total number of steps in the workflow
    total_steps: Arc<RwLock<usize>>,
    // Workflow started flag
    workflow_started: Arc<RwLock<bool>>,
    // Log parser
    log_parser: LogParser,
    // Track occurrences of node names for disambiguation
    node_name_seen_count: Arc<RwLock<HashMap<String, usize>>>,
    // Store total count of each node name
    node_name_total_count: Arc<RwLock<HashMap<String, usize>>>,
    // Track currently active node instances to maintain consistent numbering
    active_node_instances: Arc<RwLock<HashMap<String, Vec<usize>>>>,
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
            node_step_mapping: Arc::new(RwLock::new(HashMap::new())),
            node_name_mapping: Arc::new(RwLock::new(HashMap::new())),
            node_name_to_step: Arc::new(RwLock::new(HashMap::new())),
            workflow_error_occurred: Arc::new(RwLock::new(false)),
            failed_nodes: Arc::new(RwLock::new(HashSet::new())),
            total_steps: Arc::new(RwLock::new(0)),
            workflow_started: Arc::new(RwLock::new(false)),
            log_parser: LogParser::new(),
            node_name_seen_count: Arc::new(RwLock::new(HashMap::new())),
            node_name_total_count: Arc::new(RwLock::new(HashMap::new())),
            active_node_instances: Arc::new(RwLock::new(HashMap::new())),
            active_node_count: Arc::new(RwLock::new(0)),
            pending_workflow_completion: Arc::new(RwLock::new(None)),
            workflow_failed_emitted: Arc::new(RwLock::new(false)),
        }
    }

    pub fn set_node_step_mapping(&self, mapping: HashMap<String, usize>) {
        let mut step_mapping = self.node_step_mapping.write();
        *step_mapping = mapping;
    }

    pub fn analyze_workflow_and_set_step_mapping(
        &self,
        workflow_yaml: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let workflow: SimpleWorkflow = serde_yaml::from_str(workflow_yaml)?;

        let mut workflow_info = self.workflow_info.write();
        *workflow_info = Some(WorkflowExecutionInfo {
            workflow_name: workflow.name.clone(),
        });
        drop(workflow_info);

        if let Some(graph) = workflow.graphs.first() {
            let (step_mapping, name_mapping) = self.calculate_step_mapping(graph)?;

            let total_steps = step_mapping.values().max().copied().unwrap_or(0);
            *self.total_steps.write() = total_steps;

            self.set_node_step_mapping(step_mapping.clone());

            let mut node_name_mapping = self.node_name_mapping.write();
            *node_name_mapping = name_mapping.clone();
            drop(node_name_mapping);

            let mut name_to_step = HashMap::new();
            let mut sorted_steps: Vec<_> = step_mapping.iter().collect();
            sorted_steps.sort_by_key(|(_, &step)| step);

            // Create mapping from node_name to list of steps for duplicate names
            let mut node_name_steps: HashMap<String, Vec<usize>> = HashMap::new();
            for (node_id, &step_num) in sorted_steps {
                if let Some(node_name) = name_mapping.get(node_id) {
                    node_name_steps
                        .entry(node_name.clone())
                        .or_default()
                        .push(step_num);
                    tracing::debug!(
                        "Mapping node {} ({}) to step {}",
                        node_name,
                        node_id,
                        step_num
                    );
                }
            }

            // For each node name, create entries for each step
            for (node_name, mut steps) in node_name_steps {
                steps.sort();
                for (index, &step_num) in steps.iter().enumerate() {
                    let key = if steps.len() > 1 {
                        format!("{}#{}", node_name, index + 1)
                    } else {
                        node_name.clone()
                    };
                    name_to_step.insert(key, step_num);
                }
                // Also add the base name mapping to first step
                if let Some(&first_step) = steps.first() {
                    name_to_step.insert(node_name, first_step);
                }
            }

            let mut node_name_to_step = self.node_name_to_step.write();
            *node_name_to_step = name_to_step;

            let mut total_count = HashMap::new();
            for node in &graph.nodes {
                if node.node_type == "action" {
                    *total_count.entry(node.name.clone()).or_insert(0) += 1;
                }
            }
            let mut node_name_total_count = self.node_name_total_count.write();
            *node_name_total_count = total_count;

            tracing::debug!("Successfully calculated step mapping from workflow");
        } else {
            tracing::warn!("No graphs found in workflow");
        }

        Ok(())
    }

    fn calculate_step_mapping(
        &self,
        graph: &SimpleGraph,
    ) -> Result<StepMappingResult, Box<dyn std::error::Error>> {
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut all_nodes: HashSet<String> = HashSet::new();

        for node in &graph.nodes {
            all_nodes.insert(node.id.clone());
            adj_list.insert(node.id.clone(), Vec::new());
            in_degree.insert(node.id.clone(), 0);
        }

        for edge in &graph.edges {
            adj_list
                .entry(edge.from.clone())
                .or_default()
                .push(edge.to.clone());
            *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
        }

        let mut queue: VecDeque<String> = VecDeque::new();
        let mut result: Vec<String> = Vec::new();

        for (node_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node_id.clone());
            }
        }

        while let Some(current) = queue.pop_front() {
            result.push(current.clone());

            if let Some(neighbors) = adj_list.get(&current) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(neighbor.clone());
                        }
                    }
                }
            }
        }

        if result.len() != all_nodes.len() {
            return Err("Cyclic dependency detected in workflow".into());
        }

        let mut step_mapping = HashMap::new();
        let mut name_mapping = HashMap::new();
        let mut step_counter = 0;

        for node_id in result {
            if let Some(node) = graph.nodes.iter().find(|n| n.id == node_id) {
                if node.node_type == "action" {
                    step_counter += 1;
                    step_mapping.insert(node_id.clone(), step_counter);

                    name_mapping.insert(node_id.clone(), node.name.clone());
                    tracing::debug!("Step {}: {} ({})", step_counter, node.name, node_id);
                } else {
                    name_mapping.insert(node_id.clone(), node.name.clone());
                }
            }
        }

        Ok((step_mapping, name_mapping))
    }

    fn publish_event(&self, event: UserFacingLogEvent) {
        self.write_to_file(&event);

        let publisher = self.publisher.clone();
        self.tokio_handle.spawn(async move {
            let result = match publisher {
                PubSubBackend::Google(p) => p.publish(event).await.map_err(|e| e.to_string()),
                PubSubBackend::Noop(p) => p.publish(event).await.map_err(|e| format!("{e:?}")),
            };
            if let Err(e) = result {
                tracing::error!("Failed to publish user-facing log event: {}", e);
            }
        });
    }

    fn process_log_pattern(&self, pattern: LogPattern) -> Option<UserFacingLogEvent> {
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
                        display_message: format!("Workflow {workflow_name} - Started..."),
                    })
                } else {
                    None
                }
            }

            LogPattern::NodeStart(node_name) => {
                let total_count = self.node_name_total_count.read();
                let count = total_count.get(&node_name).copied().unwrap_or(1);
                drop(total_count);

                let (display_name, step_number) = if count > 1 {
                    let mut seen_count = self.node_name_seen_count.write();
                    let seen = seen_count.entry(node_name.clone()).or_insert(0);
                    *seen += 1;
                    let instance_num = *seen;

                    let mut active_instances = self.active_node_instances.write();
                    active_instances
                        .entry(node_name.clone())
                        .or_default()
                        .push(instance_num);
                    drop(active_instances);
                    drop(seen_count);

                    *self.active_node_count.write() += 1;
                    tracing::debug!(
                        "NodeStart (duplicate): incremented active_count for {}",
                        node_name
                    );

                    // Look up step number using the indexed key
                    let name_to_step = self.node_name_to_step.read();
                    let lookup_key = format!("{node_name}#{instance_num}");
                    let step_number = name_to_step.get(&lookup_key).copied().unwrap_or(0);
                    drop(name_to_step);

                    (node_name.clone(), step_number)
                } else {
                    *self.active_node_count.write() += 1;
                    tracing::debug!("NodeStart: incremented active_count for {}", node_name);

                    let name_to_step = self.node_name_to_step.read();
                    let step_number = name_to_step.get(&node_name).copied().unwrap_or(0);
                    drop(name_to_step);

                    (node_name.clone(), step_number)
                };

                if step_number > 0 {
                    Some(UserFacingLogEvent {
                        workflow_id: self.workflow_id,
                        job_id: self.job_id,
                        timestamp: Utc::now(),
                        level: UserFacingLogLevel::Info,
                        node_name: Some(display_name.clone()),
                        display_message: format!("Step {step_number}: {display_name} - Running..."),
                    })
                } else {
                    None
                }
            }

            LogPattern::NodeFinish { node_name, elapsed } => {
                let total_count = self.node_name_total_count.read();
                let count = total_count.get(&node_name).copied().unwrap_or(1);
                drop(total_count);

                let (display_name, step_number) = if count > 1 {
                    let mut active_instances = self.active_node_instances.write();
                    if let Some(instances) = active_instances.get_mut(&node_name) {
                        if let Some(instance_num) = instances.first().copied() {
                            instances.remove(0);

                            let name_to_step = self.node_name_to_step.read();
                            let lookup_key = format!("{node_name}#{instance_num}");
                            let step_number = name_to_step.get(&lookup_key).copied().unwrap_or(0);
                            drop(name_to_step);

                            (node_name.clone(), step_number)
                        } else {
                            (node_name.clone(), 0)
                        }
                    } else {
                        (node_name.clone(), 0)
                    }
                } else {
                    let name_to_step = self.node_name_to_step.read();
                    let step_number = name_to_step.get(&node_name).copied().unwrap_or(0);
                    drop(name_to_step);

                    (node_name.clone(), step_number)
                };

                let mut active_count = self.active_node_count.write();
                if *active_count > 0 {
                    *active_count -= 1;
                }
                drop(active_count);

                let result = if step_number > 0 {
                    let is_failed = self.failed_nodes.read().contains(&display_name);

                    if !is_failed {
                        Some(UserFacingLogEvent {
                            workflow_id: self.workflow_id,
                            job_id: self.job_id,
                            timestamp: Utc::now(),
                            level: UserFacingLogLevel::Success,
                            node_name: Some(display_name.clone()),
                            display_message: format!(
                                "Step {step_number}: {display_name} - Finished in {:.2}s",
                                elapsed.as_secs_f64()
                            ),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                };

                self.check_and_complete_workflow();

                result
            }

            LogPattern::NodeError { node_name, error } => {
                // Check if this node name has duplicates
                let total_count = self.node_name_total_count.read();
                let count = total_count.get(&node_name).copied().unwrap_or(1);
                drop(total_count);

                // For error events, get the first active instance and remove it
                let (display_name, step_number) = if count > 1 {
                    let mut active_instances = self.active_node_instances.write();
                    if let Some(instances) = active_instances.get_mut(&node_name) {
                        if let Some(instance_num) = instances.first().copied() {
                            instances.remove(0);

                            let name_to_step = self.node_name_to_step.read();
                            let lookup_key = format!("{node_name}#{instance_num}");
                            let step_number = name_to_step.get(&lookup_key).copied().unwrap_or(0);
                            drop(name_to_step);

                            (node_name.clone(), step_number)
                        } else {
                            (node_name.clone(), 0)
                        }
                    } else {
                        (node_name.clone(), 0)
                    }
                } else {
                    let name_to_step = self.node_name_to_step.read();
                    let step_number = name_to_step.get(&node_name).copied().unwrap_or(0);
                    drop(name_to_step);

                    (node_name.clone(), step_number)
                };

                self.failed_nodes.write().insert(display_name.clone());
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

                let result = if step_number > 0 {
                    Some(UserFacingLogEvent {
                        workflow_id: self.workflow_id,
                        job_id: self.job_id,
                        timestamp: Utc::now(),
                        level: UserFacingLogLevel::Error,
                        node_name: Some(display_name.clone()),
                        display_message: format!(
                            "Step {step_number}: {display_name} - Failed: {simple_error}"
                        ),
                    })
                } else {
                    Some(UserFacingLogEvent {
                        workflow_id: self.workflow_id,
                        job_id: self.job_id,
                        timestamp: Utc::now(),
                        level: UserFacingLogLevel::Error,
                        node_name: Some(display_name.clone()),
                        display_message: format!("{display_name} - Failed: {simple_error}"),
                    })
                };

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
                            display_message: "Workflow finished successfully.".to_string(),
                        }
                    } else {
                        UserFacingLogEvent {
                            workflow_id: self.workflow_id,
                            job_id: self.job_id,
                            timestamp: Utc::now(),
                            level: UserFacingLogLevel::Error,
                            node_name: None,
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
                            display_message: "Workflow execution failed.".to_string(),
                        };
                        Some(failed_event)
                    } else {
                        Some(error_event)
                    }
                }
            }

            LogPattern::NodeTerminate {
                node_name,
                elapsed: _,
            } => {
                // For terminate events, decrement active count but don't emit user-facing log
                let total_count = self.node_name_total_count.read();
                let count = total_count.get(&node_name).copied().unwrap_or(1);
                drop(total_count);

                // Remove from active instances if it's a duplicate node
                if count > 1 {
                    let mut active_instances = self.active_node_instances.write();
                    if let Some(instances) = active_instances.get_mut(&node_name) {
                        if !instances.is_empty() {
                            instances.remove(0);
                        }
                    }
                }

                // Decrement active count
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

    fn on_close(&self, _id: tracing::span::Id, _ctx: Context<'_, S>) {}

    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let meta = event.metadata();
        let level = meta.level();
        let target = meta.target();

        if self.handler.should_process_event(event, target, level) {
            // Extract message from the event (same approach as StdoutLogPublishLayer)
            let mut message_extractor = MessageExtractor::default();
            event.record(&mut message_extractor);
            let message = message_extractor.0.unwrap_or_else(|| "".to_string());

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
                if let Some(user_event) = self.handler.process_log_pattern(pattern) {
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
            "node.name" => self.fields.node_name = Some(value.to_string()),
            "workflow.name" => self.fields.workflow_name = Some(value.to_string()),
            _ => {}
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        match field.name() {
            "node.name" => self.fields.node_name = Some(format!("{value:?}")),
            "workflow.name" => self.fields.workflow_name = Some(format!("{value:?}")),
            _ => {}
        }
    }
}

#[derive(Default)]
struct MessageExtractor(Option<String>);

impl tracing::field::Visit for MessageExtractor {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0 = Some(value.to_string());
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" && self.0.is_none() {
            self.0 = Some(format!("{value:?}"));
        }
    }
}
