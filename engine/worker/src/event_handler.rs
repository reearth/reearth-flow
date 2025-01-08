use std::sync::Arc;

use parking_lot::Mutex;
use reearth_flow_runtime::node::NodeHandle;
use uuid::Uuid;

use crate::{
    pubsub::publisher::Publisher,
    types::{
        edge_pass_through_event::{self, EdgePassThroughEvent, EventStatus},
        log_stream_event::LogStreamEvent,
    },
};

use self::edge_pass_through_event::UpdatedEdge;

#[derive(Debug)]
pub(crate) struct ProcessFailedHandler {
    pub(crate) failed_nodes: Arc<Mutex<Vec<NodeHandle>>>,
}

impl ProcessFailedHandler {
    pub(crate) fn new() -> Self {
        let failed_nodes = Arc::new(Mutex::new(Vec::new()));
        Self { failed_nodes }
    }
}

#[async_trait::async_trait]
impl reearth_flow_runtime::event::EventHandler for ProcessFailedHandler {
    async fn on_event(&self, event: &reearth_flow_runtime::event::Event) {
        if let reearth_flow_runtime::event::Event::ProcessorFailed { node, .. } = event {
            self.failed_nodes.lock().push(node.clone());
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
}

#[async_trait::async_trait]
impl<P: Publisher + 'static> reearth_flow_runtime::event::EventHandler for EventHandler<P> {
    async fn on_event(&self, event: &reearth_flow_runtime::event::Event) {
        match event {
            reearth_flow_runtime::event::Event::Log {
                level,
                span: _,
                node_handle,
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
            reearth_flow_runtime::event::Event::EdgePassThrough {
                edge_id,
                feature_id,
            } => {
                let edge_pass_through_event = EdgePassThroughEvent {
                    workflow_id: self.workflow_id,
                    job_id: self.job_id,
                    status: EventStatus::InProgress,
                    timestamp: chrono::Utc::now(),
                    updated_edges: vec![UpdatedEdge {
                        id: edge_id.to_string(),
                        status: EventStatus::InProgress,
                        feature_id: Some(*feature_id),
                    }],
                };
                if let Err(e) = self.publisher.publish(edge_pass_through_event).await {
                    tracing::error!("Failed to publish edge pass through event: {}", e);
                }
            }
            _ => {}
        }
    }

    async fn on_shutdown(&self) {
        self.publisher.shutdown().await;
    }
}
