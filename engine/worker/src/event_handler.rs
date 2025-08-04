use std::sync::Arc;

use parking_lot::Mutex;
use reearth_flow_runtime::node::NodeHandle;
use reearth_flow_runtime::node::NodeStatus;
use uuid::Uuid;

use crate::types::job_status_event::{JobStatus as PublishJobStatus, JobStatusEvent};
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

#[derive(Debug)]
pub(crate) struct JobStatusHandler<P: Publisher> {
    pub(crate) workflow_id: Uuid,
    pub(crate) job_id: Uuid,
    pub(crate) publisher: P,
    pub(crate) job_started: Arc<Mutex<bool>>,
    pub(crate) total_nodes: Arc<Mutex<Option<usize>>>,
    pub(crate) completed_nodes: Arc<Mutex<usize>>,
    pub(crate) failed_nodes: Arc<Mutex<Vec<String>>>,
}

impl<P: Publisher> JobStatusHandler<P> {
    pub(crate) fn new(workflow_id: Uuid, job_id: Uuid, publisher: P) -> Self {
        Self {
            workflow_id,
            job_id,
            publisher,
            job_started: Arc::new(Mutex::new(false)),
            total_nodes: Arc::new(Mutex::new(None)),
            completed_nodes: Arc::new(Mutex::new(0)),
            failed_nodes: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn send_job_status(&self, status: PublishJobStatus, message: Option<String>) {
        let failed_nodes = if status == PublishJobStatus::Failed {
            let failed = self.failed_nodes.lock().clone();
            if failed.is_empty() {
                None
            } else {
                Some(failed)
            }
        } else {
            None
        };

        let job_status_event = JobStatusEvent::new(
            self.workflow_id,
            self.job_id,
            status.clone(),
            message,
            failed_nodes,
        );

        if let Err(e) = self.publisher.publish(job_status_event).await {
            tracing::error!(
                "Failed to publish job status event: {:?}, error: {}",
                status,
                e
            );
        } else {
            tracing::info!("Published job status event: {:?}", status);
        }
    }
}

#[async_trait::async_trait]
impl<P: Publisher + 'static> reearth_flow_runtime::event::EventHandler for JobStatusHandler<P> {
    async fn on_event(&self, event: &reearth_flow_runtime::event::Event) {
        match event {
            reearth_flow_runtime::event::Event::SourceFlushed => {
                let mut started = self.job_started.lock();
                if !*started {
                    *started = true;
                    drop(started);
                    self.send_job_status(PublishJobStatus::Running, None).await;
                }
            }
            reearth_flow_runtime::event::Event::ProcessorFinished { .. } => {
                let mut completed = self.completed_nodes.lock();
                *completed += 1;
                let completed_count = *completed;
                drop(completed);

                // Check if all nodes are completed
                if let Some(total) = *self.total_nodes.lock() {
                    if completed_count >= total && self.failed_nodes.lock().is_empty() {
                        self.send_job_status(PublishJobStatus::Completed, None)
                            .await;
                    }
                }
            }
            reearth_flow_runtime::event::Event::ProcessorFailed { node, .. } => {
                let mut failed = self.failed_nodes.lock();
                failed.push(node.id.to_string());
                drop(failed);

                self.send_job_status(
                    PublishJobStatus::Failed,
                    Some(format!("Node {} failed", node.id)),
                )
                .await;
            }
            reearth_flow_runtime::event::Event::SinkFinished { .. } => {
                let mut completed = self.completed_nodes.lock();
                *completed += 1;
                let completed_count = *completed;
                drop(completed);

                // Check if all nodes are completed
                if let Some(total) = *self.total_nodes.lock() {
                    if completed_count >= total && self.failed_nodes.lock().is_empty() {
                        self.send_job_status(PublishJobStatus::Completed, None)
                            .await;
                    }
                }
            }
            reearth_flow_runtime::event::Event::SinkFinishFailed { name } => {
                let mut failed = self.failed_nodes.lock();
                failed.push(name.clone());
                drop(failed);

                self.send_job_status(
                    PublishJobStatus::Failed,
                    Some(format!("Sink {} failed", name)),
                )
                .await;
            }
            _ => {}
        }
    }

    async fn on_shutdown(&self) {
        tracing::info!("JobStatusHandler shutting down. Closing publisher...");
        self.publisher.shutdown().await;
        tracing::info!("JobStatusHandler publisher shutdown complete");
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
