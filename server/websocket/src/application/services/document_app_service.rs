use anyhow::Result;
use std::sync::Arc;
use time::OffsetDateTime;
use yrs::{Doc, ReadTxn, StateVector, Transact};

use crate::domain::{DocumentId, DocumentService};
use crate::infrastructure::storage::gcs::UpdateInfo;
use crate::infrastructure::storage::kv::DocOps;
use crate::infrastructure::BroadcastPool;

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

    pub async fn get_document_state(&self, doc_id: &str) -> Result<Vec<u8>> {
        let doc_id = DocumentId::from(doc_id);

        let document = self.document_service.get_or_create(doc_id).await?;

        let awareness = document.awareness.read().await;
        let doc = awareness.doc();
        let state = doc
            .transact()
            .encode_state_as_update_v1(&StateVector::default());

        Ok(state)
    }

    pub async fn rollback_document(&self, doc_id: &str, target_clock: u32) -> Result<Doc> {
        let storage = self.broadcast_pool.get_store();
        storage.rollback_to(doc_id, target_clock).await
    }

    pub async fn create_snapshot(&self, doc_id: &str, version: u64) -> Result<Option<Doc>> {
        let storage = self.broadcast_pool.get_store();
        storage.create_snapshot_from_version(doc_id, version).await
    }

    pub async fn save_snapshot(&self, doc_id: &str) -> Result<()> {
        self.broadcast_pool.save_snapshot(doc_id).await
    }

    pub async fn flush_to_gcs(&self, doc_id: &str) -> Result<()> {
        self.broadcast_pool.flush_to_gcs(doc_id).await
    }

    pub async fn load_document(&self, doc_id: &str) -> Result<Option<Doc>> {
        let storage = self.broadcast_pool.get_store();

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

    pub async fn get_latest_update_metadata(
        &self,
        doc_id: &str,
    ) -> Result<Option<(u32, OffsetDateTime)>> {
        let storage = self.broadcast_pool.get_store();
        storage.get_latest_update_metadata(doc_id).await
    }

    pub async fn get_updates(&self, doc_id: &str) -> Result<Vec<UpdateInfo>> {
        let storage = self.broadcast_pool.get_store();
        storage.get_updates(doc_id).await
    }

    pub async fn get_updates_metadata(&self, doc_id: &str) -> Result<Vec<(u32, OffsetDateTime)>> {
        let storage = self.broadcast_pool.get_store();
        storage.get_updates_metadata(doc_id).await
    }

    pub async fn get_updates_by_version(
        &self,
        doc_id: &str,
        version: u32,
    ) -> Result<Option<UpdateInfo>> {
        let storage = self.broadcast_pool.get_store();
        storage.get_updates_by_version(doc_id, version).await
    }
}
