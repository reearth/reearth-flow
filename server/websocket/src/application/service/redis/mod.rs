use crate::domain::repository::redis::RedisRepository;
use anyhow::Result;
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Redis application service that provides high-level Redis operations
/// using the domain repository interface.
pub struct RedisService<R: RedisRepository> {
    repository: Arc<R>,
}

impl<R: RedisRepository> RedisService<R> {
    /// Create a new Redis service with the given repository implementation.
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// Get the underlying repository for direct access if needed.
    pub fn repository(&self) -> &Arc<R> {
        &self.repository
    }

    // Document management operations
    
    /// Register a new document instance and acquire its lock.
    /// Returns true if both registration and lock acquisition succeeded.
    pub async fn register_and_lock_document(
        &self,
        doc_id: &str,
        instance_id: &str,
        ttl_seconds: u64,
    ) -> Result<bool, R::Error> {
        // First register the document instance
        let registered = self.repository
            .register_doc_instance(doc_id, instance_id, ttl_seconds)
            .await?;
        
        if registered {
            // If registration succeeded, try to acquire the document lock
            let locked = self.repository
                .acquire_doc_lock(doc_id, instance_id)
                .await?;
            Ok(locked)
        } else {
            Ok(false)
        }
    }
    
    /// Unregister a document instance and release its lock.
    pub async fn unregister_and_unlock_document(
        &self,
        doc_id: &str,
        instance_id: &str,
    ) -> Result<(), R::Error> {
        // Release the document lock first
        let _ = self.repository
            .release_doc_lock(doc_id, instance_id)
            .await?;
        
        // Remove the instance heartbeat
        let _ = self.repository
            .remove_instance_heartbeat(doc_id, instance_id)
            .await?;
        
        Ok(())
    }
    
    /// Check if a document has active instances.
    pub async fn has_active_instances(
        &self,
        doc_id: &str,
        timeout_secs: u64,
    ) -> Result<bool, R::Error> {
        let count = self.repository
            .get_active_instances(doc_id, timeout_secs)
            .await?;
        Ok(count > 0)
    }
    
    /// Update heartbeat for a document instance.
    pub async fn heartbeat(
        &self,
        doc_id: &str,
        instance_id: &str,
    ) -> Result<(), R::Error> {
        self.repository
            .update_instance_heartbeat(doc_id, instance_id)
            .await
    }

    // Stream operations
    
    /// Publish an update to a document stream.
    pub async fn publish_document_update(
        &self,
        doc_id: &str,
        update: &[u8],
        instance_id: &str,
    ) -> Result<(), R::Error> {
        let stream_key = format!("yjs:stream:{}", doc_id);
        self.repository
            .publish_update(&stream_key, update, instance_id)
            .await
    }
    
    /// Publish an update to a document stream with TTL.
    pub async fn publish_document_update_with_ttl(
        &self,
        doc_id: &str,
        update: &[u8],
        instance_id: &str,
        ttl: u64,
    ) -> Result<(), R::Error> {
        let stream_key = format!("yjs:stream:{}", doc_id);
        self.repository
            .publish_update_with_ttl(&stream_key, update, instance_id, ttl)
            .await
    }
    
    /// Read and filter updates from a document stream.
    pub async fn read_document_updates(
        &self,
        doc_id: &str,
        count: usize,
        instance_id: &str,
        last_read_id: &Arc<Mutex<String>>,
    ) -> Result<Vec<Bytes>, R::Error> {
        let stream_key = format!("yjs:stream:{}", doc_id);
        self.repository
            .read_and_filter(&stream_key, count, instance_id, last_read_id)
            .await
    }
    
    /// Delete a document stream.
    pub async fn delete_document_stream(
        &self,
        doc_id: &str,
    ) -> Result<(), R::Error> {
        self.repository
            .delete_stream(doc_id)
            .await
    }

    // Generic key-value operations
    
    /// Set a key-value pair with optional TTL.
    pub async fn set_with_ttl(
        &self,
        key: &str,
        value: &str,
        ttl_seconds: Option<u64>,
    ) -> Result<(), R::Error> {
        self.repository.set(key, value).await?;
        
        if let Some(ttl) = ttl_seconds {
            self.repository.expire(key, ttl).await?;
        }
        
        Ok(())
    }
    
    /// Get a value by key.
    pub async fn get(&self, key: &str) -> Result<Option<String>, R::Error> {
        self.repository.get(key).await
    }
    
    /// Delete a key.
    pub async fn delete(&self, key: &str) -> Result<(), R::Error> {
        self.repository.del(key).await
    }
    
    /// Check if a key exists.
    pub async fn exists(&self, key: &str) -> Result<bool, R::Error> {
        self.repository.exists(key).await
    }

    // Lock operations
    
    /// Acquire a distributed lock with automatic release after TTL.
    pub async fn acquire_lock_with_ttl(
        &self,
        lock_key: &str,
        lock_value: &str,
        ttl_seconds: u64,
    ) -> Result<bool, R::Error> {
        self.repository
            .acquire_lock(lock_key, lock_value, ttl_seconds)
            .await
    }
    
    /// Release a distributed lock.
    pub async fn release_lock(
        &self,
        lock_key: &str,
        lock_value: &str,
    ) -> Result<(), R::Error> {
        self.repository
            .release_lock(lock_key, lock_value)
            .await
    }
}

/// Convenience type alias for Redis service using the concrete RedisStore implementation.
pub type ConcreteRedisService = RedisService<crate::infrastructure::redis::RedisStore>;

impl ConcreteRedisService {
    /// Create a new Redis service from a RedisStore.
    pub fn from_store(store: crate::infrastructure::redis::RedisStore) -> Self {
        Self::new(Arc::new(store))
    }
}
