use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::models::{Document, DocumentId};
use crate::domain::repositories::DocumentRepository;
use crate::infrastructure::storage::kv::get_oid;
use crate::infrastructure::BroadcastPool;

pub struct DocumentRepositoryImpl {
    pool: Arc<BroadcastPool>,
    cache: Arc<RwLock<HashMap<DocumentId, Document>>>,
}

impl DocumentRepositoryImpl {
    pub fn new(pool: Arc<BroadcastPool>) -> Self {
        Self {
            pool,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl DocumentRepository for DocumentRepositoryImpl {
    async fn get(&self, id: &DocumentId) -> Result<Option<Document>> {
        let cache = self.cache.read().await;
        if let Some(doc) = cache.get(id) {
            return Ok(Some(doc.clone()));
        }
        drop(cache);

        match self.pool.get_group(id.as_str()).await {
            Ok(group) => {
                let doc = Document {
                    id: id.clone(),
                    awareness: group.awareness().clone(),
                };

                let mut cache = self.cache.write().await;
                cache.insert(id.clone(), doc.clone());

                Ok(Some(doc))
            }
            Err(_) => Ok(None),
        }
    }

    async fn save(&self, document: &Document) -> Result<()> {
        let mut cache = self.cache.write().await;
        cache.insert(document.id.clone(), document.clone());

        self.pool.flush_to_gcs(document.id.as_str()).await?;

        Ok(())
    }

    async fn delete(&self, id: &DocumentId) -> Result<()> {
        let mut cache = self.cache.write().await;
        cache.remove(id);

        Ok(())
    }

    async fn exists(&self, id: &DocumentId) -> Result<bool> {
        let cache = self.cache.read().await;
        if cache.contains_key(id) {
            return Ok(true);
        }
        drop(cache);

        let store = self.pool.get_store();
        let oid = get_oid(&*store, id.as_str().as_bytes()).await?;
        Ok(oid.is_some())
    }
}
