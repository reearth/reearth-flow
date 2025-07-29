use crate::domain::repository::RedisStreamRepository;
use crate::domain::value_object::DocumentName;
use crate::storage::redis::RedisStore;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Infrastructure implementation of RedisStreamRepository using Redis
pub struct RedisStreamRepositoryImpl {
    redis_store: Arc<RedisStore>,
}

impl RedisStreamRepositoryImpl {
    pub fn new(redis_store: Arc<RedisStore>) -> Self {
        Self { redis_store }
    }

    fn stream_key(&self, document_name: &DocumentName) -> String {
        format!("doc_updates:{}", document_name.as_str())
    }

    fn last_id_key(&self, document_name: &DocumentName) -> String {
        format!("doc_last_id:{}", document_name.as_str())
    }
}

#[async_trait]
impl RedisStreamRepository for RedisStreamRepositoryImpl {
    async fn add_update(
        &self,
        document_name: &DocumentName,
        update: &[u8],
    ) -> Result<String> {
        let stream_key = self.stream_key(document_name);
        
        // Add update to Redis stream
        // This is a simplified implementation - you may need to adjust based on your RedisStore API
        let id = self.redis_store.xadd(&stream_key, "*", "data", update).await?;
        Ok(id)
    }

    async fn read_updates(
        &self,
        document_name: &DocumentName,
        last_id: &str,
    ) -> Result<Vec<(String, Vec<u8>)>> {
        let stream_key = self.stream_key(document_name);
        
        // Read from Redis stream starting from last_id
        let entries = self.redis_store.xread(&stream_key, last_id, None).await?;
        
        let mut updates = Vec::new();
        for (id, fields) in entries {
            if let Some(data) = fields.get("data") {
                updates.push((id, data.clone()));
            }
        }
        
        Ok(updates)
    }

    async fn get_last_read_id(
        &self,
        document_name: &DocumentName,
    ) -> Result<Option<String>> {
        let key = self.last_id_key(document_name);
        
        match self.redis_store.get(&key).await {
            Ok(id) => Ok(Some(String::from_utf8(id)?)),
            Err(_) => Ok(None), // Key doesn't exist
        }
    }

    async fn set_last_read_id(
        &self,
        document_name: &DocumentName,
        id: &str,
    ) -> Result<()> {
        let key = self.last_id_key(document_name);
        self.redis_store.set(&key, id.as_bytes()).await
    }
}
