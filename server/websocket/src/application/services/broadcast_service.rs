use crate::domain::entity::{BroadcastMessage, DocumentId};
use crate::domain::repositories::BroadcastRepository;
use anyhow::Result;
use bytes::Bytes;
use std::sync::Arc;

pub struct BroadcastService {
    repository: Arc<dyn BroadcastRepository>,
}

impl BroadcastService {
    pub fn new(repository: Arc<dyn BroadcastRepository>) -> Self {
        Self { repository }
    }

    pub async fn broadcast_to_document(
        &self,
        doc_id: &DocumentId,
        message: BroadcastMessage,
    ) -> Result<()> {
        let channel = format!("doc:{}", doc_id.as_str());
        self.repository.publish(&channel, message.data).await?;
        Ok(())
    }

    pub async fn broadcast_to_instance(
        &self,
        instance_id: &str,
        doc_id: &DocumentId,
        data: Bytes,
    ) -> Result<()> {
        let channel = format!("instance:{}:doc:{}", instance_id, doc_id.as_str());
        self.repository.publish(&channel, data).await?;
        Ok(())
    }

    pub async fn get_subscriber_count(&self, doc_id: &DocumentId) -> Result<usize> {
        let channel = format!("doc:{}", doc_id.as_str());
        self.repository.subscriber_count(&channel).await
    }
}
