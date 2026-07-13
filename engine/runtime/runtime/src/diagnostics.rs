use std::sync::Arc;

use reearth_flow_diagnostics::{DiagnosticKind, ErrorCode, NodeDiagnostics};

use crate::event::{Event, EventHub};
use crate::node::NodeHandle;

/// Runtime-side wrapper pairing the crate-agnostic aggregator with the
/// runtime identities needed to emit `Event::Log`s.
#[derive(Debug)]
pub struct NodeDiagnosticsHandle {
    pub node_handle: NodeHandle,
    pub node_name: String,
    pub inner: Arc<NodeDiagnostics>,
}

pub type SharedNodeDiagnostics = Arc<NodeDiagnosticsHandle>;

impl NodeDiagnosticsHandle {
    pub fn new(
        node_handle: NodeHandle,
        node_name: String,
        action_type: String,
        warn_once: reearth_flow_diagnostics::WarnOnceSet,
    ) -> Self {
        let node_id = node_handle.id.to_string();
        Self {
            node_handle,
            node_name,
            inner: Arc::new(NodeDiagnostics::new(node_id, action_type, warn_once)),
        }
    }

    /// finish()-time drop reporting for code without an ExecutorContext.
    pub fn report_drop(&self, code: ErrorCode, feature_id: Option<uuid::Uuid>) {
        self.inner
            .record(DiagnosticKind::WarnDrop, code, feature_id);
    }
}

/// Drain the node's buckets and emit one WARN `Event::Log` per (code, kind)
/// summary. Called once per node after `finish()`.
pub fn emit_summaries(event_hub: &EventHub, handle: &NodeDiagnosticsHandle) {
    for summary in handle.inner.drain_summaries() {
        event_hub.send(Event::Log {
            level: tracing::Level::WARN,
            span: None,
            node_handle: Some(handle.node_handle.clone()),
            node_name: Some(handle.node_name.clone()),
            message: summary.message.clone(),
        });
    }
}
