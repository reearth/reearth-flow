use crate::domain::models::{BroadcastMessage, DocumentId};
use crate::domain::repositories::BroadcastRepository;
use anyhow::Result;
use bytes::Bytes;
use std::sync::Arc;

/// 广播领域服务
pub struct BroadcastService {
    repository: Arc<dyn BroadcastRepository>,
}

impl BroadcastService {
    pub fn new(repository: Arc<dyn BroadcastRepository>) -> Self {
        Self { repository }
    }

    /// 广播消息到文档频道
    pub async fn broadcast_to_document(
        &self,
        doc_id: &DocumentId,
        message: BroadcastMessage,
    ) -> Result<()> {
        let channel = format!("doc:{}", doc_id.as_str());
        self.repository.publish(&channel, message.data).await?;
        Ok(())
    }

    /// 广播消息到实例频道
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

    /// 获取文档的订阅者数量
    pub async fn get_subscriber_count(&self, doc_id: &DocumentId) -> Result<usize> {
        let channel = format!("doc:{}", doc_id.as_str());
        self.repository.subscriber_count(&channel).await
    }
}
