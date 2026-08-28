use std::sync::Arc;

use parking_lot::Mutex;
use reearth_flow_runtime::node::NodeHandle;
use reearth_flow_runtime::node::NodeStatus;
use uuid::Uuid;

use reearth_flow_worker::pubsub::Publisher;
use reearth_flow_worker::types::diagnostic_event::{DiagnosticEvent, ENABLE_DIAGNOSTICS};
use reearth_flow_worker::types::log_stream_event::LogStreamEvent;
use reearth_flow_worker::types::node_status_event::{
    NodeMetrics as PublishNodeMetrics, NodeStatus as PublishNodeStatus, NodeStatusEvent,
};

/// The identity a failure event carried: always a node id, plus the node's
/// name when the event had one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FailedNode {
    pub(crate) id: String,
    pub(crate) name: Option<String>,
}

#[derive(Debug)]
pub(crate) struct NodeFailureHandler {
    pub(crate) failed_processor_nodes: Arc<Mutex<Vec<NodeHandle>>>,
    pub(crate) failed_sinks: Arc<Mutex<Vec<String>>>,
    /// Superset of the two above plus status-Failed transitions (how source
    /// failures surface). Feeds the synthesized failure diagnostics only —
    /// never all_success(), which would change job-result semantics.
    pub(crate) failed_details: Arc<Mutex<Vec<FailedNode>>>,
}

impl NodeFailureHandler {
    pub(crate) fn new() -> Self {
        Self {
            failed_processor_nodes: Arc::new(Mutex::new(Vec::new())),
            failed_sinks: Arc::new(Mutex::new(Vec::new())),
            failed_details: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn all_success(&self) -> bool {
        self.failed_processor_nodes.lock().is_empty() && self.failed_sinks.lock().is_empty()
    }

    pub(crate) fn failed_nodes(&self) -> Vec<String> {
        let mut failed_nodes = self
            .failed_processor_nodes
            .lock()
            .iter()
            .map(|n| n.id.to_string())
            .collect::<Vec<String>>();
        failed_nodes.extend(self.failed_sinks.lock().iter().cloned());
        failed_nodes
    }

    /// Deduped by node id, keeping the first named entry for each.
    pub(crate) fn failure_details(&self) -> Vec<FailedNode> {
        let mut out: Vec<FailedNode> = Vec::new();
        for detail in self.failed_details.lock().iter() {
            match out.iter_mut().find(|d| d.id == detail.id) {
                Some(existing) => {
                    if existing.name.is_none() {
                        existing.name = detail.name.clone();
                    }
                }
                None => out.push(detail.clone()),
            }
        }
        out
    }
}

#[async_trait::async_trait]
impl reearth_flow_runtime::event::EventHandler for NodeFailureHandler {
    async fn on_event(&self, event: &reearth_flow_runtime::event::Event) {
        match event {
            reearth_flow_runtime::event::Event::ProcessorFailed { node, name } => {
                self.failed_processor_nodes.lock().push(node.clone());
                self.failed_details.lock().push(FailedNode {
                    id: node.id.to_string(),
                    name: Some(name.clone()),
                });
            }
            reearth_flow_runtime::event::Event::SinkFinishFailed { name } => {
                self.failed_sinks.lock().push(name.clone());
                self.failed_details.lock().push(FailedNode {
                    id: name.clone(),
                    name: Some(name.clone()),
                });
            }
            reearth_flow_runtime::event::Event::NodeStatusChanged {
                node_handle,
                status: NodeStatus::Failed,
                ..
            } => {
                self.failed_details.lock().push(FailedNode {
                    id: node_handle.id.to_string(),
                    name: None,
                });
            }
            _ => {}
        }
    }
}
pub(crate) struct EventHandler<P: Publisher> {
    pub(crate) workflow_id: Uuid,
    pub(crate) job_id: Uuid,
    pub(crate) publisher: P,
}

impl<P: Publisher> EventHandler<P> {
    pub(crate) fn new(workflow_id: Uuid, job_id: Uuid, publisher: P) -> Self {
        Self {
            workflow_id,
            job_id,
            publisher,
        }
    }

    /// Takes the gate as an explicit `bool` rather than reading process-global `ENABLE_DIAGNOSTICS` directly, so tests can exercise both branches without racing each other.
    async fn handle_diagnostic(
        &self,
        enabled: bool,
        diagnostic: &reearth_flow_diagnostics::Diagnostic,
    ) {
        if !enabled {
            return;
        }
        let diagnostic_event = DiagnosticEvent::new(self.workflow_id, self.job_id, diagnostic);
        if let Err(e) = self.publisher.publish(diagnostic_event).await {
            tracing::error!("Failed to publish diagnostic event: {}", e);
        }
    }
}

#[async_trait::async_trait]
impl<P: Publisher + 'static> reearth_flow_runtime::event::EventHandler for EventHandler<P> {
    async fn on_event(&self, event: &reearth_flow_runtime::event::Event) {
        match event {
            reearth_flow_runtime::event::Event::Log {
                level,
                span: _,
                node_handle,
                node_name: _,
                message,
            } => {
                let log_stream_event = LogStreamEvent::new(
                    self.workflow_id,
                    self.job_id,
                    level.to_string(),
                    node_handle.clone().map(|h| h.id.to_string()),
                    message.clone(),
                );
                if let Err(e) = self.publisher.publish(log_stream_event).await {
                    tracing::error!("Failed to publish log stream event: {}", e);
                }
            }
            reearth_flow_runtime::event::Event::NodeStatusChanged {
                node_handle,
                status,
                feature_id,
                metrics,
            } => {
                tracing::info!(
                    "SENDING NODE STATUS EVENT: node_id={}, status={:?}, feature_id={:?}",
                    node_handle.id,
                    status,
                    feature_id
                );

                let publish_status = match status {
                    NodeStatus::Starting => PublishNodeStatus::Starting,
                    NodeStatus::Processing => PublishNodeStatus::Processing,
                    NodeStatus::Completed => {
                        tracing::info!("Node completed: {}", node_handle.id);
                        PublishNodeStatus::Completed
                    }
                    NodeStatus::Failed => {
                        tracing::warn!("Node failed: {}", node_handle.id);
                        PublishNodeStatus::Failed
                    }
                };

                let publish_metrics = metrics.map(|m| PublishNodeMetrics {
                    features_processed: m.features_processed,
                    features_written: m.features_written,
                    finish_feature_count: m.finish_feature_count,
                });

                let node_status_event = NodeStatusEvent::new(
                    self.workflow_id,
                    self.job_id,
                    node_handle.id.to_string(),
                    publish_status,
                    *feature_id,
                    publish_metrics,
                );

                match self.publisher.publish(node_status_event).await {
                    Ok(_) => {
                        tracing::info!(
                            "Successfully published node status: node_id={}, status={:?}",
                            node_handle.id,
                            status
                        );
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to publish node status event for node_id={}, status={:?}: {}",
                            node_handle.id,
                            status,
                            e
                        );
                    }
                }
            }
            reearth_flow_runtime::event::Event::Diagnostic(diagnostic) => {
                self.handle_diagnostic(*ENABLE_DIAGNOSTICS, diagnostic)
                    .await;
            }
            _ => {}
        }
    }

    async fn on_shutdown(&self) {
        tracing::info!("EventHandler shutting down. Closing publisher...");
        self.publisher.shutdown().await;
        tracing::info!("Publisher shutdown complete");
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use reearth_flow_diagnostics::{Diagnostic, DiagnosticDraft, ErrorCode};
    use reearth_flow_runtime::event::EventHandler as RuntimeEventHandler;
    use reearth_flow_worker::pubsub::EncodableMessage;

    use super::*;

    /// Status-Failed transitions are how source failures surface; they must
    /// reach failure_details without flipping all_success, which still drives
    /// the job result.
    #[tokio::test]
    async fn status_failed_feeds_details_but_not_all_success() {
        let handler = NodeFailureHandler::new();

        handler
            .on_event(&reearth_flow_runtime::event::Event::NodeStatusChanged {
                node_handle: NodeHandle::for_test("node-a"),
                status: NodeStatus::Failed,
                feature_id: None,
                metrics: None,
            })
            .await;
        handler
            .on_event(&reearth_flow_runtime::event::Event::ProcessorFailed {
                node: NodeHandle::for_test("node-a"),
                name: "CityGML Reader".to_string(),
            })
            .await;

        assert!(!handler.all_success());
        let details = handler.failure_details();
        assert_eq!(details.len(), 1);
        assert_eq!(details[0].name.as_deref(), Some("CityGML Reader"));

        let status_only = NodeFailureHandler::new();
        status_only
            .on_event(&reearth_flow_runtime::event::Event::NodeStatusChanged {
                node_handle: NodeHandle::for_test("node-b"),
                status: NodeStatus::Failed,
                feature_id: None,
                metrics: None,
            })
            .await;
        assert!(status_only.all_success());
        assert_eq!(status_only.failure_details().len(), 1);
    }

    #[derive(Debug, Default, Clone)]
    struct CountingPublisher {
        count: Arc<AtomicUsize>,
    }

    impl CountingPublisher {
        fn count(&self) -> usize {
            self.count.load(Ordering::SeqCst)
        }
    }

    #[derive(Debug, thiserror::Error)]
    #[error("counting publisher error")]
    struct CountingPublisherError;

    #[async_trait::async_trait]
    impl Publisher for CountingPublisher {
        type Error = CountingPublisherError;

        async fn publish<M: EncodableMessage>(&self, _message: M) -> Result<(), Self::Error> {
            self.count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn shutdown(&self) {}
    }

    fn sample_diagnostic() -> Diagnostic {
        Diagnostic::from_draft(
            DiagnosticDraft::new(ErrorCode::Cesium3dtilesEmptyGeometry),
            Some("node-1".to_string()),
            Some("Cesium 3D Tiles Writer".to_string()),
            None,
        )
    }

    #[tokio::test]
    async fn handle_diagnostic_publishes_nothing_when_disabled() {
        let publisher = CountingPublisher::default();
        let handler = EventHandler::new(Uuid::new_v4(), Uuid::new_v4(), publisher.clone());

        handler.handle_diagnostic(false, &sample_diagnostic()).await;

        assert_eq!(publisher.count(), 0);
    }

    #[tokio::test]
    async fn handle_diagnostic_publishes_once_when_enabled() {
        let publisher = CountingPublisher::default();
        let handler = EventHandler::new(Uuid::new_v4(), Uuid::new_v4(), publisher.clone());

        handler.handle_diagnostic(true, &sample_diagnostic()).await;

        assert_eq!(publisher.count(), 1);
    }

    /// `FLOW_WORKER_ENABLE_DIAGNOSTICS` is unset in this process, so `ENABLE_DIAGNOSTICS` reads `false`.
    #[tokio::test]
    async fn on_event_diagnostic_publishes_nothing_with_flag_unset_by_default() {
        let publisher = CountingPublisher::default();
        let handler = EventHandler::new(Uuid::new_v4(), Uuid::new_v4(), publisher.clone());
        let event = reearth_flow_runtime::event::Event::Diagnostic(Arc::new(sample_diagnostic()));

        handler.on_event(&event).await;

        assert_eq!(publisher.count(), 0);
        assert!(!*ENABLE_DIAGNOSTICS, "flag must default to false");
    }

    #[tokio::test]
    async fn on_event_non_diagnostic_events_are_unaffected_by_the_new_arm() {
        let publisher = CountingPublisher::default();
        let handler = EventHandler::new(Uuid::new_v4(), Uuid::new_v4(), publisher.clone());
        let event = reearth_flow_runtime::event::Event::SourceFlushed;

        handler.on_event(&event).await;

        assert_eq!(publisher.count(), 0);
    }
}
