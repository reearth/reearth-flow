use crate::domain::repository::DocumentStorageRepository;
use crate::domain::value_object::DocumentName;
use crate::storage::gcs::GcsStore;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Infrastructure implementation of DocumentStorageRepository using GCS
pub struct DocumentStorageRepositoryImpl {
    gcs_store: Arc<GcsStore>,
}

impl DocumentStorageRepositoryImpl {
    pub fn new(gcs_store: Arc<GcsStore>) -> Self {
        Self { gcs_store }
    }
}

#[async_trait]
impl DocumentStorageRepository for DocumentStorageRepositoryImpl {
    async fn save_snapshot(
        &self,
        document_name: &DocumentName,
        data: &[u8],
    ) -> Result<()> {
        let key = format!("documents/{}/snapshot", document_name.as_str());
        self.gcs_store.put(&key, data).await
    }

    async fn load_document(
        &self,
        document_name: &DocumentName,
    ) -> Result<Option<Vec<u8>>> {
        let key = format!("documents/{}/snapshot", document_name.as_str());
        match self.gcs_store.get(&key).await {
            Ok(data) => Ok(Some(data)),
            Err(e) => {
                // Check if it's a "not found" error
                if e.to_string().contains("not found") || e.to_string().contains("404") {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    async fn flush_updates(
        &self,
        document_name: &DocumentName,
        updates: Vec<Vec<u8>>,
    ) -> Result<()> {
        for (index, update) in updates.iter().enumerate() {
            let key = format!("documents/{}/updates/{:06}", document_name.as_str(), index);
            self.gcs_store.put(&key, update).await?;
        }
        Ok(())
    }
}
