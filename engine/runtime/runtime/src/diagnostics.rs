use std::sync::Arc;

use reearth_flow_diagnostics::{DiagnosticKind, ErrorCode, NodeDiagnostics};

use crate::event::EventHub;
use crate::node::NodeHandle;

/// Runtime-side wrapper pairing the crate-agnostic aggregator with the
/// runtime identities needed to emit `Event::Diagnostic`s.
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

/// Drain the node's buckets once and, for each (code, kind) summary, emit a
/// structured `Event::Diagnostic`. `LogEventHandler` renders it through the
/// action log (see `runner::log_event_handler`), so no twin `Event::Log` is
/// sent here. Called once per node after `finish()`. Returns the drained
/// summaries for callers that need them (e.g. RunSummary, Task 5).
pub fn emit_summaries(
    event_hub: &EventHub,
    handle: &NodeDiagnosticsHandle,
) -> Vec<reearth_flow_diagnostics::Diagnostic> {
    let summaries = handle.inner.drain_summaries();
    for summary in &summaries {
        event_hub.diagnostic(summary.clone());
    }
    summaries
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Event;
    use crate::node::{NodeHandle, NodeId};
    use reearth_flow_diagnostics::{DiagnosticKind, ErrorCode};

    fn handle() -> NodeDiagnosticsHandle {
        NodeDiagnosticsHandle::new(
            NodeHandle::new(NodeId::new("node-1".to_string())),
            "writer-1".to_string(),
            "Cesium 3D Tiles Writer".to_string(),
            Arc::default(),
        )
    }

    #[test]
    fn emit_summaries_sends_exactly_one_diagnostic_and_zero_log_per_summary_and_drains_once() {
        let handle = handle();
        handle.inner.record(
            DiagnosticKind::WarnDrop,
            ErrorCode::Cesium3dtilesEmptyGeometry,
            None,
        );
        handle.inner.record(
            DiagnosticKind::WarnDrop,
            ErrorCode::Cesium3dtilesNonCitygmlGeometry,
            None,
        );
        let event_hub = EventHub::new(30);
        let mut receiver = event_hub.sender.subscribe();

        let summaries = emit_summaries(&event_hub, &handle);
        assert_eq!(summaries.len(), 2);

        let mut log_count = 0;
        let mut diagnostic_count = 0;
        for _ in 0..2 {
            match receiver.try_recv().expect("expected event") {
                Event::Log { .. } => log_count += 1,
                Event::Diagnostic(d) => {
                    diagnostic_count += 1;
                    assert_eq!(d.aggregated.as_ref().unwrap().count, 1);
                }
                other => panic!("unexpected event: {other:?}"),
            }
        }
        assert_eq!(log_count, 0);
        assert_eq!(diagnostic_count, 2);
        assert!(receiver.try_recv().is_err());

        // second call: buckets already drained, no events, empty Vec
        let mut receiver2 = event_hub.sender.subscribe();
        let summaries2 = emit_summaries(&event_hub, &handle);
        assert!(summaries2.is_empty());
        assert!(receiver2.try_recv().is_err());
    }
}
