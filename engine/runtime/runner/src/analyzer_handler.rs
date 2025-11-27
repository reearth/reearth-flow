//! Analyzer event handler for capturing runtime events and forwarding them to the analyzer.

#[cfg(feature = "analyzer")]
use reearth_flow_runtime::analyzer::{
    AnalyzerContext, AnalyzerEdgeId, AnalyzerEvent, AnalyzerEventSender,
};
#[cfg(feature = "analyzer")]
use reearth_flow_runtime::event::{Event, EventHandler};

/// Event handler that forwards runtime events to the analyzer.
#[cfg(feature = "analyzer")]
pub struct AnalyzerEventHandler {
    context: AnalyzerContext,
    workflow_id: uuid::Uuid,
    workflow_name: String,
}

#[cfg(feature = "analyzer")]
impl AnalyzerEventHandler {
    /// Create a new analyzer event handler.
    pub fn new(
        sender: AnalyzerEventSender,
        workflow_id: uuid::Uuid,
        workflow_name: String,
    ) -> Self {
        let context = AnalyzerContext::new(sender);
        // Send workflow start event
        context.workflow_start(workflow_id, workflow_name.clone());
        Self {
            context,
            workflow_id,
            workflow_name,
        }
    }

    /// Get a clone of the analyzer context for passing to executors.
    #[allow(dead_code)]
    pub fn context(&self) -> AnalyzerContext {
        self.context.clone()
    }
}

#[cfg(feature = "analyzer")]
#[async_trait::async_trait]
impl EventHandler for AnalyzerEventHandler {
    async fn on_event(&self, event: &Event) {
        match event {
            Event::ActionMemory {
                node_id,
                node_name,
                thread_name,
                current_memory_bytes,
                peak_memory_bytes,
                processing_time_ms,
            } => {
                // Forward memory event to analyzer
                self.context.send(AnalyzerEvent::ActionMemory {
                    timestamp_ms: AnalyzerEvent::now_ms(),
                    node_id: *node_id,
                    node_name: node_name.clone(),
                    thread_name: thread_name.clone(),
                    current_memory_bytes: *current_memory_bytes,
                    peak_memory_bytes: *peak_memory_bytes,
                    processing_time_ms: *processing_time_ms,
                });
            }
            Event::EdgeFeature {
                edge_id,
                feature_id,
                feature_size_bytes,
                source_node_id,
            } => {
                // Forward edge feature event to analyzer
                self.context.send(AnalyzerEvent::EdgeFeature {
                    timestamp_ms: AnalyzerEvent::now_ms(),
                    edge_id: AnalyzerEdgeId::new(edge_id.clone()),
                    feature_id: *feature_id,
                    feature_size_bytes: *feature_size_bytes,
                    source_node_id: *source_node_id,
                });
            }
            Event::NodeQueueDepth {
                node_id,
                node_name: _,
                features_waiting,
                features_processing,
                bytes_waiting: _,
            } => {
                // Forward queue depth event to analyzer
                self.context.send(AnalyzerEvent::NodeProcessingState {
                    timestamp_ms: AnalyzerEvent::now_ms(),
                    node_id: *node_id,
                    features_waiting: *features_waiting,
                    features_processing: *features_processing,
                });
            }
            Event::EdgePassThrough {
                feature_id,
                edge_id,
            } => {
                tracing::trace!(
                    "Analyzer: Feature {} passed through edge {}",
                    feature_id,
                    edge_id
                );
            }
            Event::NodeStatusChanged {
                node_handle,
                status,
                feature_id: _,
            } => {
                use reearth_flow_runtime::node::NodeStatus;
                match status {
                    NodeStatus::Processing => {
                        tracing::trace!("Analyzer: Node {} is processing", node_handle.id);
                    }
                    NodeStatus::Completed => {
                        tracing::trace!("Analyzer: Node {} completed", node_handle.id);
                    }
                    NodeStatus::Failed => {
                        tracing::trace!("Analyzer: Node {} failed", node_handle.id);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    async fn on_shutdown(&self) {
        // Workflow end event is sent explicitly in the runner based on result
        // This handler is just for cleanup
        tracing::info!(
            "Analyzer: Shutting down for workflow {} ({})",
            self.workflow_name,
            self.workflow_id
        );
    }
}

/// Placeholder module for when analyzer feature is disabled.
#[cfg(not(feature = "analyzer"))]
pub struct AnalyzerEventHandler;

#[cfg(not(feature = "analyzer"))]
impl AnalyzerEventHandler {
    pub fn new(_sender: (), _workflow_id: uuid::Uuid, _workflow_name: String) -> Self {
        Self
    }
}
