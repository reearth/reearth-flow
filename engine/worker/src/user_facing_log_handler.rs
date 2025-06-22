use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use parking_lot::RwLock;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::Layer;
use uuid::Uuid;

use crate::pubsub::backend::PubSubBackend;
use crate::pubsub::publisher::Publisher;
use crate::types::user_facing_log_event::{UserFacingLogEvent, UserFacingLogLevel};

// Simple workflow structures for dependency analysis
#[derive(Debug, Clone, serde::Deserialize)]
struct SimpleWorkflow {
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

#[derive(Clone, Debug)]
pub struct NodeExecutionInfo {
    #[allow(dead_code)]
    pub node_id: String,
    pub node_name: String,
    pub step_number: usize,
    pub start_time: Instant,
    pub running_logged: bool,
    pub finished_logged: bool,
    pub node_type: String,
}

#[derive(Clone, Debug)]
pub struct WorkflowExecutionInfo {
    #[allow(dead_code)]
    pub workflow_name: String,
    #[allow(dead_code)]
    pub start_time: Instant,
}

pub struct UserFacingLogHandler {
    workflow_id: Uuid,
    job_id: Uuid,
    publisher: PubSubBackend,
    tokio_handle: tokio::runtime::Handle,
    // State management
    node_execution_map: Arc<RwLock<HashMap<String, NodeExecutionInfo>>>,
    workflow_info: Arc<RwLock<Option<WorkflowExecutionInfo>>>,
    // Step number mapping from topological order
    node_step_mapping: Arc<RwLock<HashMap<String, usize>>>,
    // Store node type information for runtime use
    node_type_mapping: Arc<RwLock<HashMap<String, String>>>,
    // Store node name information for runtime use
    node_name_mapping: Arc<RwLock<HashMap<String, String>>>,
    // Track workflow errors
    workflow_error_occurred: Arc<RwLock<bool>>,
    // Track which specific nodes failed
    failed_nodes: Arc<RwLock<HashSet<String>>>,
    // Total number of steps in the workflow
    total_steps: Arc<RwLock<usize>>,
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
            node_execution_map: Arc::new(RwLock::new(HashMap::new())),
            workflow_info: Arc::new(RwLock::new(None)),
            node_step_mapping: Arc::new(RwLock::new(HashMap::new())),
            node_type_mapping: Arc::new(RwLock::new(HashMap::new())),
            node_name_mapping: Arc::new(RwLock::new(HashMap::new())),
            workflow_error_occurred: Arc::new(RwLock::new(false)),
            failed_nodes: Arc::new(RwLock::new(HashSet::new())),
            total_steps: Arc::new(RwLock::new(0)),
        }
    }

    pub fn set_node_step_mapping(&self, mapping: HashMap<String, usize>) {
        let mut step_mapping = self.node_step_mapping.write();
        *step_mapping = mapping;
    }

    pub fn analyze_workflow_and_set_step_mapping(&self, workflow_yaml: &str) -> Result<(), Box<dyn std::error::Error>> {
        let workflow: SimpleWorkflow = serde_yaml::from_str(workflow_yaml)?;
        
        // Find the main graph (assuming first graph for simplicity)
        if let Some(graph) = workflow.graphs.first() {
            let (step_mapping, type_mapping, name_mapping) = self.calculate_step_mapping(graph)?;
            
            // Calculate and store total steps
            let total_steps = step_mapping.values().max().copied().unwrap_or(0);
            *self.total_steps.write() = total_steps;
            
            self.set_node_step_mapping(step_mapping);
            
            // Store node type mapping for runtime use
            let mut node_type_mapping = self.node_type_mapping.write();
            *node_type_mapping = type_mapping;
            
            // Store node name mapping for runtime use
            let mut node_name_mapping = self.node_name_mapping.write();
            *node_name_mapping = name_mapping;
            
            tracing::debug!("Successfully calculated step mapping from workflow");
        } else {
            tracing::warn!("No graphs found in workflow");
        }
        
        Ok(())
    }

    fn calculate_step_mapping(&self, graph: &SimpleGraph) -> Result<(HashMap<String, usize>, HashMap<String, String>, HashMap<String, String>), Box<dyn std::error::Error>> {
        // Build adjacency list and in-degree count
        let mut adj_list: HashMap<String, Vec<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut all_nodes: HashSet<String> = HashSet::new();

        // Initialize nodes
        for node in &graph.nodes {
            all_nodes.insert(node.id.clone());
            adj_list.insert(node.id.clone(), Vec::new());
            in_degree.insert(node.id.clone(), 0);
        }

        // Build graph from edges
        for edge in &graph.edges {
            adj_list.entry(edge.from.clone()).or_default().push(edge.to.clone());
            *in_degree.entry(edge.to.clone()).or_insert(0) += 1;
        }

        // Kahn's algorithm for topological sorting
        let mut queue: VecDeque<String> = VecDeque::new();
        let mut result: Vec<String> = Vec::new();

        // Find all nodes with in-degree 0
        for (node_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(node_id.clone());
            }
        }

        while let Some(current) = queue.pop_front() {
            result.push(current.clone());

            // For each neighbor of current node
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

        // Check for cycles
        if result.len() != all_nodes.len() {
            return Err("Cyclic dependency detected in workflow".into());
        }

        // Create step mapping (only for action nodes, excluding sources)
        let mut step_mapping = HashMap::new();
        let mut type_mapping = HashMap::new();
        let mut name_mapping = HashMap::new();
        let mut step_counter = 0;

        for node_id in result {
            if let Some(node) = graph.nodes.iter().find(|n| n.id == node_id) {
                // Only assign step numbers to action nodes (processors and sinks)
                if node.node_type == "action" {
                    step_counter += 1;
                    step_mapping.insert(node_id.clone(), step_counter);
                    name_mapping.insert(node_id.clone(), node.name.clone());
                    tracing::debug!("Step {}: {} ({})", step_counter, node.name, node_id);
                    
                    // Store the actual action type for runtime use
                    // The actual categorization (source/processor/sink) will be determined at runtime
                    // based on the node's behavior and connections
                    type_mapping.insert(node_id.clone(), "action".to_string());
                } else {
                    // For non-action nodes, preserve original type
                    type_mapping.insert(node_id.clone(), node.node_type.clone());
                    name_mapping.insert(node_id.clone(), node.name.clone());
                }
            }
        }

        Ok((step_mapping, type_mapping, name_mapping))
    }

    fn publish_event(&self, event: UserFacingLogEvent) {
        // Write to file
        self.write_to_file(&event);

        // Publish to pub/sub
        let publisher = self.publisher.clone();
        self.tokio_handle.spawn(async move {
            let result = match publisher {
                PubSubBackend::Google(p) => p.publish(event).await.map_err(|e| e.to_string()),
                PubSubBackend::Noop(p) => p.publish(event).await.map_err(|e| format!("{:?}", e)),
            };
            if let Err(e) = result {
                tracing::error!("Failed to publish user-facing log event: {}", e);
            }
        });
    }

    pub fn handle_runtime_event(&self, event: &reearth_flow_runtime::event::Event) {
        match event {
            reearth_flow_runtime::event::Event::Log { level: _, span: _, node_handle: _, message: _ } => {
                // Simplified approach: No longer handle action-log messages for Running status
                // All nodes now emit Running at NodeStatus::Starting timing
            }
            reearth_flow_runtime::event::Event::ProcessorFailed { node, name: _ } => {
                // Mark this node as failed
                self.failed_nodes.write().insert(node.id.to_string());
                *self.workflow_error_occurred.write() = true;
                
                let node_map = self.node_execution_map.read();
                if let Some(node_info) = node_map.get(&node.id.to_string()) {
                    let event = UserFacingLogEvent {
                        workflow_id: self.workflow_id,
                        job_id: self.job_id,
                        timestamp: Utc::now(),
                        level: UserFacingLogLevel::Error,
                        node_id: Some(node.id.to_string()),
                        node_name: Some(node_info.node_name.clone()),
                        display_message: format!(
                            "Step {}: {} - Failed during execution",
                            node_info.step_number, node_info.node_name
                        ),
                    };
                    self.publish_event(event);
                }
            }
            reearth_flow_runtime::event::Event::SinkFinishFailed { name } => {
                // Mark workflow as failed
                *self.workflow_error_occurred.write() = true;
                
                // For sink failures, we might not have node_id, so use the name
                let event = UserFacingLogEvent {
                    workflow_id: self.workflow_id,
                    job_id: self.job_id,
                    timestamp: Utc::now(),
                    level: UserFacingLogLevel::Error,
                    node_id: None,
                    node_name: Some(name.clone()),
                    display_message: format!("Sink {} - Failed to finish", name),
                };
                self.publish_event(event);
            }
            reearth_flow_runtime::event::Event::NodeStatusChanged { node_handle, status, feature_id: _ } => {
                self.handle_node_status_changed(node_handle, status);
            }
            _ => {
                // Other events are handled by the tracing layer
            }
        }
    }

    fn handle_node_status_changed(&self, node_handle: &reearth_flow_runtime::node::NodeHandle, status: &reearth_flow_runtime::node::NodeStatus) {
        let node_id = node_handle.id.to_string();
        
        match status {
            reearth_flow_runtime::node::NodeStatus::Starting => {
                // Create node execution info from workflow analysis
                let step_mapping = self.node_step_mapping.read();
                let step_number = step_mapping.get(&node_id).copied().unwrap_or(0);
                drop(step_mapping);

                let type_mapping = self.node_type_mapping.read();
                let node_type = type_mapping.get(&node_id).cloned().unwrap_or_default();
                drop(type_mapping);

                // Get node name from mapping or use node type as fallback
                let name_mapping = self.node_name_mapping.read();
                let node_name = name_mapping.get(&node_id).cloned()
                    .unwrap_or_else(|| format!("{} Node", node_type));

                let node_info = NodeExecutionInfo {
                    node_id: node_id.clone(),
                    node_name: node_name.clone(),
                    step_number,
                    start_time: Instant::now(),
                    running_logged: false,
                    finished_logged: false,
                    node_type: node_type.clone(),
                };

                let mut node_map = self.node_execution_map.write();
                node_map.insert(node_id.clone(), node_info.clone());
                drop(node_map);

                // Emit workflow start on first node
                if step_number == 1 {
                    let event = UserFacingLogEvent {
                        workflow_id: self.workflow_id,
                        job_id: self.job_id,
                        timestamp: Utc::now(),
                        level: UserFacingLogLevel::Info,
                        node_id: None,
                        node_name: None,
                        display_message: "Workflow Example - Started...".to_string(),
                    };
                    self.publish_event(event);
                }

                // Emit Running for all nodes using NodeStatus::Starting (simplified approach)
                if step_number > 0 {
                    let mut node_map = self.node_execution_map.write();
                    if let Some(node_info) = node_map.get_mut(&node_id) {
                        node_info.running_logged = true;
                        let event = UserFacingLogEvent {
                            workflow_id: self.workflow_id,
                            job_id: self.job_id,
                            timestamp: Utc::now(),
                            level: UserFacingLogLevel::Info,
                            node_id: Some(node_id),
                            node_name: Some(node_name),
                            display_message: format!("Step {}: {} - Running...", step_number, node_info.node_name),
                        };
                        self.publish_event(event);
                    }
                }
            },
            reearth_flow_runtime::node::NodeStatus::Completed => {
                let mut node_map = self.node_execution_map.write();
                if let Some(node_info) = node_map.get_mut(&node_id) {
                    // Only emit finished log once per node
                    if !node_info.finished_logged {
                        node_info.finished_logged = true;
                        let elapsed = node_info.start_time.elapsed();
                        
                        // Check if this specific node failed
                        let is_node_failed = self.failed_nodes.read().contains(&node_id);
                        let (level, status_text) = if is_node_failed {
                            (UserFacingLogLevel::Error, "Failed")
                        } else {
                            (UserFacingLogLevel::Success, "Finished")
                        };
                        
                        let event = UserFacingLogEvent {
                            workflow_id: self.workflow_id,
                            job_id: self.job_id,
                            timestamp: Utc::now(),
                            level,
                            node_id: Some(node_id.clone()),
                            node_name: Some(node_info.node_name.clone()),
                            display_message: format!(
                                "Step {}: {} - {} in {:.2}s",
                                node_info.step_number,
                                node_info.node_name,
                                status_text,
                                elapsed.as_secs_f64()
                            ),
                        };
                        self.publish_event(event);

                        // Emit workflow completion on last node
                        let total_steps = *self.total_steps.read();
                        if node_info.step_number == total_steps {
                            let error_occurred = *self.workflow_error_occurred.read();
                            let (level, message) = if error_occurred {
                                (UserFacingLogLevel::Error, "Workflow failed.".to_string())
                            } else {
                                (UserFacingLogLevel::Success, "Workflow finished successfully.".to_string())
                            };
                            
                            let event = UserFacingLogEvent {
                                workflow_id: self.workflow_id,
                                job_id: self.job_id,
                                timestamp: Utc::now(),
                                level,
                                node_id: None,
                                node_name: None,
                                display_message: message,
                            };
                            self.publish_event(event);
                        }
                    }
                }
            },
            _ => {
                // Handle other statuses if needed
            }
        }
    }

    fn write_to_file(&self, event: &UserFacingLogEvent) {
        if let Ok(json_line) = serde_json::to_string(event) {
            // Use the global file writer, similar to stdout-log
            let writer = crate::logger::DynamicUserFacingLogFileWriter;
            let mut file_writer = writer.make_writer();
            if let Err(e) = writeln!(file_writer, "{}", json_line) {
                tracing::error!("Failed to write user-facing log to file: {}", e);
            }
        }
    }

    fn extract_span_fields<S>(ctx: &Context<'_, S>) -> Option<(Option<String>, Option<String>, Option<String>)>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        let current_span = ctx.lookup_current()?;
        let mut node_id = None;
        let mut node_name = None;
        let mut workflow_name = None;

        // Walk up the span hierarchy to find node or workflow information
        for span in current_span.scope() {
            let extensions = span.extensions();
            
            // Check if this is a node execution span
            if span.name() == "node_execution" {
                if let Some(fields) = extensions.get::<SpanFields>() {
                    node_id = fields.node_id.clone();
                    node_name = fields.node_name.clone();
                }
            }
            
            // Check if this is a workflow execution span
            if span.name() == "workflow_execution" {
                if let Some(fields) = extensions.get::<SpanFields>() {
                    workflow_name = fields.workflow_name.clone();
                }
            }
        }

        Some((node_id, node_name, workflow_name))
    }

    fn should_process_event(&self, _event: &Event<'_>, target: &str, level: &Level) -> bool {
        // Filter rules for user-facing logs
        match *level {
            Level::INFO => {
                // Workflow level events
                if target.contains("reearth_flow_runner::runner") {
                    return true;
                }
                // Node execution events
                if target.contains("reearth_flow_runtime::executor") {
                    return true;
                }
                // Action log events
                if target.contains("reearth_flow_runner::log_event_handler") {
                    return true;
                }
            }
            Level::ERROR | Level::WARN => {
                // All errors and warnings from the flow engine should be user-facing
                if target.contains("reearth_flow") {
                    return true;
                }
                // Also catch errors from dependencies that might be relevant
                if target.contains("action-") || target.contains("processor") || target.contains("sink") {
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    #[allow(dead_code)]
    fn format_display_message(
        &self,
        level: &UserFacingLogLevel,
        node_info: Option<&NodeExecutionInfo>,
        workflow_info: Option<&WorkflowExecutionInfo>,
        message: &str,
        elapsed: Option<Duration>,
    ) -> String {
        match (level, node_info, workflow_info) {
            // Node-level messages
            (UserFacingLogLevel::Info, Some(info), _) => {
                format!("Step {}: {} - Running...", info.step_number, info.node_name)
            }
            (UserFacingLogLevel::Success, Some(info), _) => {
                if let Some(duration) = elapsed {
                    format!(
                        "Step {}: {} - Finished in {:.2}s",
                        info.step_number,
                        info.node_name,
                        duration.as_secs_f64()
                    )
                } else {
                    format!("Step {}: {} - Finished", info.step_number, info.node_name)
                }
            }
            (UserFacingLogLevel::Error, Some(info), _) => {
                format!(
                    "Step {}: {} - Failed with an error.",
                    info.step_number, info.node_name
                )
            }
            // Workflow-level messages
            (UserFacingLogLevel::Info, None, Some(wf_info)) => {
                format!("Workflow {} - Started...", wf_info.workflow_name)
            }
            (UserFacingLogLevel::Success, None, _) => "Workflow finished successfully.".to_string(),
            (UserFacingLogLevel::Error, None, _) => "Workflow failed.".to_string(),
            // Fallback
            _ => message.to_string(),
        }
    }
}

// Helper struct to store span fields
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
        Self {
            handler,
        }
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

        // Extract fields from span attributes
        let mut visitor = FieldVisitor::new(&mut fields);
        attrs.record(&mut visitor);

        // Store fields in span extensions
        let mut extensions = span.extensions_mut();
        extensions.insert(fields.clone());

        // Handle workflow start
        if span.name() == "workflow_execution" {
            tracing::debug!("Processing workflow_execution span: {:?}", fields);
            if let Some(workflow_name) = fields.workflow_name {
                let mut workflow_info = self.handler.workflow_info.write();
                *workflow_info = Some(WorkflowExecutionInfo {
                    workflow_name: workflow_name.clone(),
                    start_time: Instant::now(),
                });

                // Emit workflow start event
                let event = UserFacingLogEvent {
                    workflow_id: self.handler.workflow_id,
                    job_id: self.handler.job_id,
                    timestamp: Utc::now(),
                    level: UserFacingLogLevel::Info,
                    node_id: None,
                    node_name: None,
                    display_message: format!("Workflow {} - Started...", workflow_name),
                };
                self.handler.publish_event(event);
            }
        }

        // Handle node start
        if span.name() == "node_execution" {
            if let (Some(node_id), Some(node_name)) = (fields.node_id, fields.node_name) {
                // Get step number from pre-computed mapping
                let step_mapping = self.handler.node_step_mapping.read();
                let step_number = step_mapping.get(&node_id).copied().unwrap_or(0);
                drop(step_mapping);

                // Get node type
                let type_mapping = self.handler.node_type_mapping.read();
                let node_type = type_mapping.get(&node_id).cloned().unwrap_or_default();
                drop(type_mapping);

                let node_info = NodeExecutionInfo {
                    node_id: node_id.clone(),
                    node_name: node_name.clone(),
                    step_number,
                    start_time: Instant::now(),
                    running_logged: false,
                    finished_logged: false,
                    node_type: node_type.clone(),
                };

                let mut node_map = self.handler.node_execution_map.write();
                node_map.insert(node_id.clone(), node_info.clone());
                drop(node_map);

                // Simplified approach: emit "Running" for all nodes when span starts
                // This provides uniform timing for both source and action nodes
                if step_number > 0 {
                    let mut node_map = self.handler.node_execution_map.write();
                    if let Some(node_info) = node_map.get_mut(&node_id) {
                        node_info.running_logged = true;
                        let event = UserFacingLogEvent {
                            workflow_id: self.handler.workflow_id,
                            job_id: self.handler.job_id,
                            timestamp: Utc::now(),
                            level: UserFacingLogLevel::Info,
                            node_id: Some(node_id),
                            node_name: Some(node_name),
                            display_message: format!("Step {}: {} - Running...", step_number, node_info.node_name),
                        };
                        self.handler.publish_event(event);
                    }
                }
            }
        }
    }

    fn on_close(&self, id: tracing::span::Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(&id) {
            let extensions = span.extensions();
            if let Some(fields) = extensions.get::<SpanFields>() {
                // Handle node completion
                if span.name() == "node_execution" {
                    if let Some(node_id) = &fields.node_id {
                        let node_map = self.handler.node_execution_map.read();
                        if let Some(node_info) = node_map.get(node_id) {
                            let elapsed = node_info.start_time.elapsed();
                            
                            // Check if this specific node failed
                            let is_node_failed = self.handler.failed_nodes.read().contains(node_id);
                            let (level, status_text) = if is_node_failed {
                                (UserFacingLogLevel::Error, "Failed")
                            } else {
                                (UserFacingLogLevel::Success, "Finished")
                            };
                            
                            let event = UserFacingLogEvent {
                                workflow_id: self.handler.workflow_id,
                                job_id: self.handler.job_id,
                                timestamp: Utc::now(),
                                level,
                                node_id: Some(node_id.clone()),
                                node_name: Some(node_info.node_name.clone()),
                                display_message: format!(
                                    "Step {}: {} - {} in {:.2}s",
                                    node_info.step_number,
                                    node_info.node_name,
                                    status_text,
                                    elapsed.as_secs_f64()
                                ),
                            };
                            self.handler.publish_event(event);
                        }
                    }
                }

                // Handle workflow completion
                if span.name() == "workflow_execution" {
                    tracing::debug!("Processing workflow_execution span close");
                    let error_occurred = *self.handler.workflow_error_occurred.read();
                    let (level, message) = if error_occurred {
                        (UserFacingLogLevel::Error, "Workflow failed.".to_string())
                    } else {
                        (UserFacingLogLevel::Success, "Workflow finished successfully.".to_string())
                    };
                    
                    let event = UserFacingLogEvent {
                        workflow_id: self.handler.workflow_id,
                        job_id: self.handler.job_id,
                        timestamp: Utc::now(),
                        level,
                        node_id: None,
                        node_name: None,
                        display_message: message,
                    };
                    self.handler.publish_event(event);
                }
            }
        }
    }

    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        let meta = event.metadata();
        let level = meta.level();
        let target = meta.target();

        if self.handler.should_process_event(event, target, level) {
            // Extract message from event
            let mut message_extractor = MessageExtractor::default();
            event.record(&mut message_extractor);
            let message = message_extractor.0.unwrap_or_default();
            
            // Check for error conditions
            if *level == Level::ERROR || *level == Level::WARN || 
               message.contains("Failed") || message.contains("failed") || 
               message.contains("Error") || message.contains("error") ||
               message.contains("panic") || message.contains("Panic") ||
               message.contains("JoinError") {
                
                // Mark workflow as failed
                *self.handler.workflow_error_occurred.write() = true;
                
                // Create user-friendly error message
                let user_friendly_message = if message.contains("JoinError") && message.contains("Panic") {
                    "A critical error occurred during workflow execution."
                } else if message.contains("panic") || message.contains("Panic") {
                    "A panic occurred during execution."
                } else if message.contains("Failed to workflow") {
                    "Workflow execution failed."
                } else {
                    &message
                };
                
                let display_message = if let Some((_node_id, node_name, _)) = UserFacingLogHandler::extract_span_fields(&ctx) {
                    if let Some(node_name) = &node_name {
                        format!("{} - {}", node_name, user_friendly_message)
                    } else {
                        user_friendly_message.to_string()
                    }
                } else {
                    user_friendly_message.to_string()
                };

                if let Some((node_id, node_name, _)) = UserFacingLogHandler::extract_span_fields(&ctx) {
                    // Mark this specific node as failed
                    if let Some(ref nid) = node_id {
                        self.handler.failed_nodes.write().insert(nid.clone());
                    }
                    
                    let event = UserFacingLogEvent {
                        workflow_id: self.handler.workflow_id,
                        job_id: self.handler.job_id,
                        timestamp: Utc::now(),
                        level: UserFacingLogLevel::Error,
                        node_id,
                        node_name,
                        display_message,
                    };
                    self.handler.publish_event(event);
                } else {
                    // Even without node context, show workflow-level errors
                    let event = UserFacingLogEvent {
                        workflow_id: self.handler.workflow_id,
                        job_id: self.handler.job_id,
                        timestamp: Utc::now(),
                        level: UserFacingLogLevel::Error,
                        node_id: None,
                        node_name: None,
                        display_message,
                    };
                    self.handler.publish_event(event);
                }
            }
        }
    }
}

// Field visitor to extract span fields
struct FieldVisitor<'a> {
    fields: &'a mut SpanFields,
}

impl<'a> FieldVisitor<'a> {
    fn new(fields: &'a mut SpanFields) -> Self {
        Self { fields }
    }
}

impl<'a> tracing::field::Visit for FieldVisitor<'a> {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        match field.name() {
            "node.id" => self.fields.node_id = Some(value.to_string()),
            "node.name" => self.fields.node_name = Some(value.to_string()),
            "workflow.name" => self.fields.workflow_name = Some(value.to_string()),
            _ => {}
        }
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        match field.name() {
            "node.id" => self.fields.node_id = Some(format!("{:?}", value)),
            "node.name" => self.fields.node_name = Some(format!("{:?}", value)),
            "workflow.name" => self.fields.workflow_name = Some(format!("{:?}", value)),
            _ => {}
        }
    }
}

// Message extractor (reused from logger.rs pattern)
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
            self.0 = Some(format!("{:?}", value));
        }
    }
}