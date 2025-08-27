use std::sync::Arc;

use parking_lot::Mutex;
use reearth_flow_runtime::node::NodeHandle;
use reearth_flow_runtime::node::NodeStatus;
use uuid::Uuid;

use crate::types::node_status_event::{NodeStatus as PublishNodeStatus, NodeStatusEvent};
use crate::{
    pubsub::publisher::Publisher,
    types::{
        edge_pass_through_event::{self, EdgePassThroughEvent, EventStatus},
        log_stream_event::LogStreamEvent,
    },
};

use self::edge_pass_through_event::UpdatedEdge;

#[derive(Debug)]
pub(crate) struct NodeFailureHandler {
    pub(crate) failed_processor_nodes: Arc<Mutex<Vec<NodeHandle>>>,
    pub(crate) failed_sinks: Arc<Mutex<Vec<String>>>,
}

impl NodeFailureHandler {
    pub(crate) fn new() -> Self {
        let failed_processor_nodes = Arc::new(Mutex::new(Vec::new()));
        let failed_sinks = Arc::new(Mutex::new(Vec::new()));
        Self {
            failed_processor_nodes,
            failed_sinks,
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
}

#[async_trait::async_trait]
impl reearth_flow_runtime::event::EventHandler for NodeFailureHandler {
    async fn on_event(&self, event: &reearth_flow_runtime::event::Event) {
        match event {
            reearth_flow_runtime::event::Event::ProcessorFailed { node, .. } => {
                self.failed_processor_nodes.lock().push(node.clone());
            }
            reearth_flow_runtime::event::Event::SinkFinishFailed { name } => {
                self.failed_sinks.lock().push(name.clone());
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
            reearth_flow_runtime::event::Event::EdgeCompleted {
                edge_id,
                feature_id,
            } => {
                let edge_completed_event = EdgePassThroughEvent {
                    workflow_id: self.workflow_id,
                    job_id: self.job_id,
                    status: EventStatus::Completed,
                    timestamp: chrono::Utc::now(),
                    updated_edges: vec![UpdatedEdge {
                        id: edge_id.to_string(),
                        status: EventStatus::Completed,
                        feature_id: Some(*feature_id),
                    }],
                };
                if let Err(e) = self.publisher.publish(edge_completed_event).await {
                    tracing::error!("Failed to publish edge completed event: {}", e);
                }
            }
            reearth_flow_runtime::event::Event::NodeStatusChanged {
                node_handle,
                status,
                feature_id,
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

                let node_status_event = NodeStatusEvent::new(
                    self.workflow_id,
                    self.job_id,
                    node_handle.id.to_string(),
                    publish_status,
                    *feature_id,
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
            _ => {}
        }
    }

    async fn on_shutdown(&self) {
        tracing::info!("EventHandler shutting down. Closing publisher...");
        self.publisher.shutdown().await;
        tracing::info!("Publisher shutdown complete");
    }
}
