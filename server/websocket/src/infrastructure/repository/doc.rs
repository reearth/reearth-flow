use crate::domain::repository::{doc::DocumentStorageRepository, kv::KVStore};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Infrastructure implementation of DocumentStorageRepository using GCS
pub struct DocumentStorageRepositoryImpl<K>
where
    K: KVStore + 'static + Send + Sync,
{
    store: Arc<K>,
}

impl<K> DocumentStorageRepositoryImpl<K>
where
    K: KVStore + 'static + Send + Sync,
{
    pub fn new(store: Arc<K>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl<K> DocumentStorageRepository for DocumentStorageRepositoryImpl<K>
where
    K: KVStore + 'static + Send + Sync,
{
    async fn save_snapshot(&self, document_id: &str, data: &[u8]) -> Result<()> {
        // TODO: Implement
        Ok(())
    }

    async fn load_document(&self, document_id: &str) -> Result<Option<Vec<u8>>> {
        // TODO: Implement
        Ok(None)
    }

    async fn flush_updates(&self, document_id: &str, updates: Vec<Vec<u8>>) -> Result<()> {
        // TODO: Implement
        Ok(())
    }
}
