use anyhow::Result;
use std::sync::Arc;
use time::OffsetDateTime;
use yrs::{Doc, Transact};

use crate::infrastructure::storage::gcs::UpdateInfo;
use crate::domain::entity::DocumentId;
use crate::infrastructure::storage::kv::DocOps;
use crate::infrastructure::{GcsStore, RedisStore};

/// 文档存储服务 - DDD 应用层服务
/// 负责文档的存储、加载和快照管理
pub struct DocumentStorageService {
    gcs_store: Arc<GcsStore>,
    redis_store: Arc<RedisStore>,
}

impl DocumentStorageService {
    pub fn new(gcs_store: Arc<GcsStore>, redis_store: Arc<RedisStore>) -> Self {
        Self {
            gcs_store,
            redis_store,
        }
    }

    /// 获取存储引用
    pub fn get_store(&self) -> Arc<GcsStore> {
        self.gcs_store.clone()
    }

    /// 保存快照
    pub async fn save_snapshot(&self, doc_id: &str) -> Result<()> {
        // 通过 GCS 存储保存快照
        // 这里需要实现具体的快照保存逻辑
        tracing::info!("Saving snapshot for document: {}", doc_id);
        // TODO: 实现快照保存逻辑
        Ok(())
    }

    /// 刷新到 GCS
    pub async fn flush_to_gcs(&self, doc_id: &str) -> Result<()> {
        // 将数据刷新到 GCS 存储
        tracing::info!("Flushing document to GCS: {}", doc_id);
        // TODO: 实现 GCS 刷新逻辑
        Ok(())
    }

    /// 加载文档
    pub async fn load_document(&self, doc_id: &str) -> Result<Option<Doc>> {
        let storage = &self.gcs_store;

        match storage.load_doc_v2(doc_id).await {
            Ok(doc) => Ok(Some(doc)),
            Err(_) => {
                let doc = Doc::new();
                let mut txn = doc.transact_mut();
                let loaded = storage.load_doc(doc_id, &mut txn).await?;

                if loaded {
                    drop(txn);
                    Ok(Some(doc))
                } else {
                    Ok(None)
                }
            }
        }
    }

    /// 获取最新更新元数据
    pub async fn get_latest_update_metadata(
        &self,
        doc_id: &str,
    ) -> Result<Option<(u32, OffsetDateTime)>> {
        let storage = &self.gcs_store;
        storage.get_latest_update_metadata(doc_id).await
    }

    /// 获取更新列表
    pub async fn get_updates(&self, doc_id: &str) -> Result<Vec<UpdateInfo>> {
        let storage = &self.gcs_store;
        storage.get_updates(doc_id).await
    }

    /// 获取更新元数据列表
    pub async fn get_updates_metadata(&self, doc_id: &str) -> Result<Vec<(u32, OffsetDateTime)>> {
        let storage = &self.gcs_store;
        storage.get_updates_metadata(doc_id).await
    }

    /// 根据版本获取更新
    pub async fn get_updates_by_version(
        &self,
        doc_id: &str,
        version: u32,
    ) -> Result<Option<UpdateInfo>> {
        let storage = &self.gcs_store;
        storage.get_updates_by_version(doc_id, version).await
    }
}
