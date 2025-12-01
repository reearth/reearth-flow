//! Analyzer integration for the runtime.
//!
//! This module provides integration between the runtime and the analyzer crate
//! when the `analyzer` feature is enabled.

pub use reearth_flow_analyzer::{
    create_channel, default_reports_dir, disable_analyzer, enable_analyzer, estimate_size,
    generate_report_filename, get_current_memory, get_current_stats, is_analyzer_enabled,
    is_tracking, start_tracking, stop_tracking, AnalyzerAllocator, AnalyzerEvent,
    AnalyzerEventReceiver, AnalyzerEventSender, AnalyzerReport, AnalyzerSink, AnalyzerSinkBuilder,
    EdgeId as AnalyzerEdgeId, NodeHandle as AnalyzerNodeHandle, TrackingGuard,
    DEFAULT_CHANNEL_CAPACITY,
};

use std::sync::Arc;
use tokio::sync::Notify;

/// Context for the analyzer, containing the event sender and configuration.
#[cfg(feature = "analyzer")]
#[derive(Clone)]
pub struct AnalyzerContext {
    /// Sender for analyzer events.
    pub sender: AnalyzerEventSender,
    /// Whether memory tracking is enabled.
    pub track_memory: bool,
    /// Whether feature size tracking is enabled.
    pub track_features: bool,
}

#[cfg(feature = "analyzer")]
impl AnalyzerContext {
    /// Create a new analyzer context.
    pub fn new(sender: AnalyzerEventSender) -> Self {
        Self {
            sender,
            track_memory: true,
            track_features: true,
        }
    }

    /// Create a new analyzer context with custom settings.
    pub fn with_settings(
        sender: AnalyzerEventSender,
        track_memory: bool,
        track_features: bool,
    ) -> Self {
        Self {
            sender,
            track_memory,
            track_features,
        }
    }

    /// Send an event to the analyzer.
    pub fn send(&self, event: AnalyzerEvent) {
        let _ = self.sender.send(event);
    }

    /// Send a workflow start event.
    pub fn workflow_start(&self, workflow_id: uuid::Uuid, workflow_name: String) {
        self.send(AnalyzerEvent::WorkflowStart {
            timestamp_ms: AnalyzerEvent::now_ms(),
            workflow_id,
            workflow_name,
        });
    }

    /// Send a workflow end event.
    pub fn workflow_end(&self, workflow_id: uuid::Uuid, success: bool) {
        tracing::info!(
            "AnalyzerContext::workflow_end called: workflow_id={}, success={}",
            workflow_id,
            success
        );
        let event = AnalyzerEvent::WorkflowEnd {
            timestamp_ms: AnalyzerEvent::now_ms(),
            workflow_id,
            success,
        };
        match self.sender.send(event) {
            Ok(_) => tracing::info!("WorkflowEnd event sent successfully"),
            Err(e) => tracing::error!("Failed to send WorkflowEnd event: {:?}", e),
        }
    }

    /// Send an action memory event.
    pub fn action_memory(
        &self,
        node_id: uuid::Uuid,
        node_name: String,
        current_memory_bytes: usize,
        peak_memory_bytes: usize,
        processing_time_ms: u64,
        start_timestamp_ms: u64,
    ) {
        if !self.track_memory {
            return;
        }
        self.send(AnalyzerEvent::ActionMemory {
            timestamp_ms: AnalyzerEvent::now_ms(),
            node_id,
            node_name,
            thread_name: std::thread::current()
                .name()
                .unwrap_or("unknown")
                .to_string(),
            current_memory_bytes,
            peak_memory_bytes,
            processing_time_ms,
            start_timestamp_ms,
        });
    }

    /// Send an edge feature event.
    pub fn edge_feature(
        &self,
        edge_id: String,
        feature_id: uuid::Uuid,
        feature_size_bytes: usize,
        source_node_id: uuid::Uuid,
    ) {
        if !self.track_features {
            return;
        }
        self.send(AnalyzerEvent::EdgeFeature {
            timestamp_ms: AnalyzerEvent::now_ms(),
            edge_id: AnalyzerEdgeId::new(edge_id),
            feature_id,
            feature_size_bytes,
            source_node_id,
        });
    }

    /// Send a node processing state event.
    pub fn node_processing_state(
        &self,
        node_id: uuid::Uuid,
        features_waiting: u64,
        features_processing: u64,
    ) {
        self.send(AnalyzerEvent::NodeProcessingState {
            timestamp_ms: AnalyzerEvent::now_ms(),
            node_id,
            features_waiting,
            features_processing,
        });
    }
}

/// Create an analyzer context and sink for a workflow execution.
pub fn create_analyzer(
    channel_capacity: usize,
) -> (AnalyzerContext, AnalyzerEventReceiver, Arc<Notify>) {
    enable_analyzer();
    let (sender, receiver) = create_channel(channel_capacity);
    let shutdown = Arc::new(Notify::new());
    let context = AnalyzerContext::new(sender);
    (context, receiver, shutdown)
}

/// Helper macro to conditionally compile analyzer code.
#[macro_export]
macro_rules! with_analyzer {
    ($ctx:expr, $code:expr) => {{
        if let Some(analyzer) = $ctx {
            $code
        }
    }};
}

/// Helper struct for tracking memory during a process() call.
#[cfg(feature = "analyzer")]
pub struct ProcessMemoryTracker {
    node_id: uuid::Uuid,
    node_name: String,
    start_time: std::time::Instant,
    start_timestamp_ms: u64,
    context: Option<AnalyzerContext>,
}

#[cfg(feature = "analyzer")]
impl ProcessMemoryTracker {
    /// Start tracking memory for a process() call.
    pub fn start(node_id: uuid::Uuid, node_name: String, context: Option<AnalyzerContext>) -> Self {
        let start_timestamp_ms = AnalyzerEvent::now_ms();
        if context.as_ref().map(|c| c.track_memory).unwrap_or(false) {
            start_tracking();
        }
        Self {
            node_id,
            node_name,
            start_time: std::time::Instant::now(),
            start_timestamp_ms,
            context,
        }
    }

    /// Finish tracking and send the memory event.
    pub fn finish(self) {
        let elapsed = self.start_time.elapsed();
        if let Some(context) = &self.context {
            if context.track_memory {
                let (current, peak) = stop_tracking();
                context.action_memory(
                    self.node_id,
                    self.node_name.clone(),
                    current,
                    peak,
                    elapsed.as_millis() as u64,
                    self.start_timestamp_ms,
                );
            }
        }
    }
}

#[cfg(not(feature = "analyzer"))]
pub struct ProcessMemoryTracker;

#[cfg(not(feature = "analyzer"))]
impl ProcessMemoryTracker {
    pub fn start(_node_id: uuid::Uuid, _node_name: String, _context: Option<()>) -> Self {
        Self
    }

    pub fn finish(self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_analyzer() {
        let (context, _receiver, _shutdown) = create_analyzer(100);
        assert!(context.track_memory);
        assert!(context.track_features);
    }

    #[test]
    fn test_send_events() {
        let (context, receiver, _shutdown) = create_analyzer(100);

        let workflow_id = uuid::Uuid::new_v4();
        context.workflow_start(workflow_id, "test".to_string());

        let event = receiver.recv_timeout(std::time::Duration::from_millis(100));
        assert!(event.is_ok());
    }
}
