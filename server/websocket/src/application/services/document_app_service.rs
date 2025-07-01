use anyhow::Result;
use std::sync::Arc;
use yrs::{Doc, ReadTxn, StateVector, Transact};

use crate::domain::{DocumentId, DocumentService};
use crate::infrastructure::BroadcastPool;

/// 文档应用服务
pub struct DocumentAppService {
    document_service: Arc<DocumentService>,
    broadcast_pool: Arc<BroadcastPool>,
}

impl DocumentAppService {
    pub fn new(document_service: Arc<DocumentService>, broadcast_pool: Arc<BroadcastPool>) -> Self {
        Self {
            document_service,
            broadcast_pool,
        }
    }

    /// 获取文档的当前状态
    pub async fn get_document_state(&self, doc_id: &str) -> Result<Vec<u8>> {
        let doc_id = DocumentId::from(doc_id);

        // 确保文档存在
        let document = self.document_service.get_or_create(doc_id).await?;

        // 获取文档状态
        let awareness = document.awareness.read().await;
        let doc = awareness.doc();
        let state = doc
            .transact()
            .encode_state_as_update_v1(&StateVector::default());

        Ok(state)
    }

    /// 回滚文档到指定版本
    pub async fn rollback_document(&self, doc_id: &str, target_clock: u32) -> Result<Doc> {
        let storage = self.broadcast_pool.get_store();
        storage.rollback_to(doc_id, target_clock).await
    }

    /// 创建文档快照
    pub async fn create_snapshot(&self, doc_id: &str, version: u64) -> Result<Option<Doc>> {
        let storage = self.broadcast_pool.get_store();
        storage.create_snapshot_from_version(doc_id, version).await
    }

    /// 保存文档快照到存储
    pub async fn save_snapshot(&self, doc_id: &str) -> Result<()> {
        self.broadcast_pool.save_snapshot(doc_id).await
    }

    /// 刷新文档到GCS
    pub async fn flush_to_gcs(&self, doc_id: &str) -> Result<()> {
        self.broadcast_pool.flush_to_gcs(doc_id).await
    }
}
