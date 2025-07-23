use anyhow::Result;
use std::sync::Arc;
use time::OffsetDateTime;
use yrs::{Doc, Transact};

use crate::infrastructure::storage::gcs::UpdateInfo;
use crate::infrastructure::{GcsStore, RedisStore};

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

    pub async fn save_snapshot(&self, doc_id: &str) -> Result<()> {
        tracing::info!("Saving snapshot for document: {}", doc_id);

        match self.gcs_store.trim_updates_logarithmic(doc_id, 4).await {
            Ok(Some(_doc)) => {
                tracing::info!("Successfully created snapshot for document: {}", doc_id);
                Ok(())
            }
            Ok(None) => {
                tracing::warn!(
                    "No updates found to create snapshot for document: {}",
                    doc_id
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to create snapshot for document {}: {:?}", doc_id, e);
                Err(e)
            }
        }
    }

    pub async fn flush_to_gcs(&self, doc_id: &str) -> Result<()> {
        tracing::info!("Flushing document to GCS: {}", doc_id);

        if !self.redis_store.check_stream_exists(doc_id).await? {
            tracing::debug!("No stream data found for document: {}", doc_id);
            return Ok(());
        }

        let stream_data = self.redis_store.read_all_stream_data(doc_id).await?;

        if stream_data.is_empty() {
            tracing::debug!("No stream data to flush for document: {}", doc_id);
            return Ok(());
        }

        tracing::info!(
            "Flushing {} updates to GCS for document: {}",
            stream_data.len(),
            doc_id
        );

        for update_bytes in stream_data {
            match self
                .gcs_store
                .push_update(doc_id, &update_bytes, &self.redis_store)
                .await
            {
                Ok(version) => {
                    tracing::debug!(
                        "Successfully pushed update version {} for document: {}",
                        version,
                        doc_id
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to push update to GCS for document {}: {:?}",
                        doc_id,
                        e
                    );
                    return Err(e);
                }
            }
        }

        tracing::info!(
            "Successfully flushed all updates to GCS for document: {}",
            doc_id
        );
        Ok(())
    }

    pub async fn load_document(&self, doc_id: &str) -> Result<Option<Doc>> {
        tracing::info!("Loading document: {}", doc_id);

        let updates = match self.gcs_store.get_updates(doc_id).await {
            Ok(updates) => updates,
            Err(e) => {
                tracing::error!("Failed to get updates for document {}: {:?}", doc_id, e);
                return Err(e);
            }
        };

        if updates.is_empty() {
            tracing::debug!("No updates found for document: {}", doc_id);
            return Ok(None);
        }

        let doc = Doc::new();

        {
            let mut txn = doc.transact_mut();
            for update_info in updates {
                if let Err(e) = txn.apply_update(update_info.update) {
                    tracing::error!("Failed to apply update for document {}: {:?}", doc_id, e);
                    return Err(anyhow::anyhow!("Failed to apply update: {:?}", e));
                }
            }
        }

        tracing::info!("Successfully loaded document: {}", doc_id);
        Ok(Some(doc))
    }

    pub async fn get_latest_update_metadata(
        &self,
        doc_id: &str,
    ) -> Result<Option<(u32, OffsetDateTime)>> {
        let storage = &self.gcs_store;
        storage.get_latest_update_metadata(doc_id).await
    }

    pub async fn get_updates(&self, doc_id: &str) -> Result<Vec<UpdateInfo>> {
        let storage = &self.gcs_store;
        storage.get_updates(doc_id).await
    }

    pub async fn get_updates_metadata(&self, doc_id: &str) -> Result<Vec<(u32, OffsetDateTime)>> {
        let storage = &self.gcs_store;
        storage.get_updates_metadata(doc_id).await
    }

    pub async fn get_updates_by_version(
        &self,
        doc_id: &str,
        version: u32,
    ) -> Result<Option<UpdateInfo>> {
        let storage = &self.gcs_store;
        storage.get_updates_by_version(doc_id, version).await
    }
}
