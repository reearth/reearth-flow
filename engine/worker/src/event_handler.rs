use uuid::Uuid;

use crate::{
    pubsub::publisher::Publisher,
    types::edge_pass_through_event::{self, EdgePassThroughEvent, EventStatus},
};

use self::edge_pass_through_event::UpdatedEdge;

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
        if let reearth_flow_runtime::event::Event::EdgePassThrough {
            edge_id,
            feature_id,
        } = event
        {
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
    }

    async fn on_shutdown(&self) {
        self.publisher.shutdown().await;
    }
}
