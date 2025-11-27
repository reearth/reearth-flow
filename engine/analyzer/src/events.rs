use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a node in the workflow graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeHandle {
    pub id: Uuid,
}

impl NodeHandle {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }
}

impl std::fmt::Display for NodeHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// Unique identifier for an edge in the workflow graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(pub String);

impl EdgeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for EdgeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Events emitted by the analyzer during workflow execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalyzerEvent {
    /// Memory usage report from an action's process() call.
    ActionMemory {
        /// Unix timestamp in milliseconds.
        timestamp_ms: u64,
        /// The node that processed the feature.
        node_id: Uuid,
        /// Human-readable node name.
        node_name: String,
        /// Name of the thread that processed the feature.
        thread_name: String,
        /// Current memory usage in bytes at end of process().
        current_memory_bytes: usize,
        /// Peak memory usage in bytes during process().
        peak_memory_bytes: usize,
        /// Time spent in process() in milliseconds.
        processing_time_ms: u64,
    },

    /// Feature passing through an edge.
    EdgeFeature {
        /// Unix timestamp in milliseconds.
        timestamp_ms: u64,
        /// The edge the feature is passing through.
        edge_id: EdgeId,
        /// The feature's unique identifier.
        feature_id: Uuid,
        /// Estimated size of the feature in bytes.
        feature_size_bytes: usize,
        /// The source node that emitted the feature.
        source_node_id: Uuid,
    },

    /// Node processing state change (for queue tracking).
    NodeProcessingState {
        /// Unix timestamp in milliseconds.
        timestamp_ms: u64,
        /// The node whose state changed.
        node_id: Uuid,
        /// Number of features waiting in the input queue.
        features_waiting: u64,
        /// Number of features currently being processed.
        features_processing: u64,
    },

    /// Workflow execution started.
    WorkflowStart {
        /// Unix timestamp in milliseconds.
        timestamp_ms: u64,
        /// The workflow's unique identifier.
        workflow_id: Uuid,
        /// Human-readable workflow name.
        workflow_name: String,
    },

    /// Workflow execution ended.
    WorkflowEnd {
        /// Unix timestamp in milliseconds.
        timestamp_ms: u64,
        /// The workflow's unique identifier.
        workflow_id: Uuid,
        /// Whether the workflow completed successfully.
        success: bool,
    },
}

impl AnalyzerEvent {
    /// Get the timestamp of this event in milliseconds.
    pub fn timestamp_ms(&self) -> u64 {
        match self {
            AnalyzerEvent::ActionMemory { timestamp_ms, .. } => *timestamp_ms,
            AnalyzerEvent::EdgeFeature { timestamp_ms, .. } => *timestamp_ms,
            AnalyzerEvent::NodeProcessingState { timestamp_ms, .. } => *timestamp_ms,
            AnalyzerEvent::WorkflowStart { timestamp_ms, .. } => *timestamp_ms,
            AnalyzerEvent::WorkflowEnd { timestamp_ms, .. } => *timestamp_ms,
        }
    }

    /// Get the current Unix timestamp in milliseconds.
    pub fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

/// Sender for analyzer events.
pub type AnalyzerEventSender = crossbeam_channel::Sender<AnalyzerEvent>;

/// Receiver for analyzer events.
pub type AnalyzerEventReceiver = crossbeam_channel::Receiver<AnalyzerEvent>;

/// Create a new bounded channel for analyzer events.
pub fn create_channel(capacity: usize) -> (AnalyzerEventSender, AnalyzerEventReceiver) {
    crossbeam_channel::bounded(capacity)
}

/// Default channel capacity for analyzer events.
pub const DEFAULT_CHANNEL_CAPACITY: usize = 10000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_serialization() {
        let event = AnalyzerEvent::ActionMemory {
            timestamp_ms: 1234567890,
            node_id: Uuid::new_v4(),
            node_name: "TestNode".to_string(),
            thread_name: "main".to_string(),
            current_memory_bytes: 1024,
            peak_memory_bytes: 2048,
            processing_time_ms: 100,
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: AnalyzerEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.timestamp_ms(), deserialized.timestamp_ms());
    }

    #[test]
    fn test_channel_creation() {
        let (sender, receiver) = create_channel(100);

        let event = AnalyzerEvent::WorkflowStart {
            timestamp_ms: AnalyzerEvent::now_ms(),
            workflow_id: Uuid::new_v4(),
            workflow_name: "test".to_string(),
        };

        sender.send(event.clone()).unwrap();
        let received = receiver.recv().unwrap();

        assert_eq!(event.timestamp_ms(), received.timestamp_ms());
    }
}
