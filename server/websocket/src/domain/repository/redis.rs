use anyhow;
use async_trait::async_trait;
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Redis repository interface for domain layer operations.
/// This trait abstracts Redis-specific operations and provides a clean interface
/// for the application layer to interact with Redis functionality.
#[async_trait]
pub trait RedisRepository: Send + Sync {
    /// Error type returned from the implementation.
    type Error: Send + Sync + 'static;

    // Basic key-value operations

    /// Set a key-value pair in Redis.
    async fn set(&self, key: &str, value: &str) -> Result<(), Self::Error>;

    /// Get a value by key from Redis.
    async fn get(&self, key: &str) -> Result<Option<String>, Self::Error>;

    /// Delete a key from Redis.
    async fn del(&self, key: &str) -> Result<(), Self::Error>;

    /// Check if a key exists in Redis.
    async fn exists(&self, key: &str) -> Result<bool, Self::Error>;

    /// Set a key-value pair only if the key does not exist (SET NX).
    async fn set_nx(&self, key: &str, value: &str) -> Result<bool, Self::Error>;

    /// Set expiration time for a key in seconds.
    async fn expire(&self, key: &str, ttl_seconds: u64) -> Result<(), Self::Error>;

    // Lock operations

    /// Acquire a distributed lock with TTL.
    /// Returns true if lock was acquired, false if already held by another process.
    async fn acquire_lock(
        &self,
        lock_key: &str,
        lock_value: &str,
        ttl_seconds: u64,
    ) -> Result<bool, Self::Error>;

    async fn acquire_oid_lock(&self, ttl_seconds: u64) -> Result<String, Self::Error>;

    async fn release_lock(&self, lock_key: &str, lock_value: &str) -> Result<(), Self::Error>;

    async fn release_oid_lock(&self, lock_value: &str) -> Result<(), Self::Error>;

    // Document-specific operations

    /// Register a document instance with TTL.
    /// Returns true if registration was successful (instance didn't exist).
    async fn register_doc_instance(
        &self,
        doc_id: &str,
        instance_id: &str,
        ttl_seconds: u64,
    ) -> Result<bool, Self::Error>;

    /// Get the active instance ID for a document.
    async fn get_doc_instance(&self, doc_id: &str) -> Result<Option<String>, Self::Error>;

    /// Acquire a document-specific lock.
    async fn acquire_doc_lock(&self, doc_id: &str, instance_id: &str) -> Result<bool, Self::Error>;

    /// Release a document-specific lock.
    /// Returns true if the lock was successfully released.
    async fn release_doc_lock(&self, doc_id: &str, instance_id: &str) -> Result<bool, Self::Error>;

    // Stream operations

    /// Publish an update to a Redis stream.
    async fn publish_update(
        &self,
        stream_key: &str,
        update: &[u8],
        instance_id: &str,
    ) -> Result<(), Self::Error>;

    /// Publish an update to a Redis stream with TTL.
    async fn publish_update_with_ttl(
        &self,
        stream_key: &str,
        update: &[u8],
        instance_id: &str,
        ttl: u64,
    ) -> Result<(), Self::Error>;

    /// Read and filter updates from a Redis stream.
    /// Returns filtered updates excluding those from the specified instance.
    async fn read_and_filter(
        &self,
        stream_key: &str,
        count: usize,
        instance_id: &str,
        last_read_id: &Arc<Mutex<String>>,
    ) -> Result<Vec<Bytes>, Self::Error>;

    /// Delete a Redis stream.
    async fn delete_stream(&self, doc_id: &str) -> Result<(), Self::Error>;

    // Instance heartbeat operations

    /// Update heartbeat for a document instance.
    /// Records the current timestamp for the instance.
    async fn update_instance_heartbeat(
        &self,
        doc_id: &str,
        instance_id: &str,
    ) -> Result<(), Self::Error>;

    /// Get count of active instances for a document.
    /// Instances are considered active if their last heartbeat was within timeout_secs.
    async fn get_active_instances(
        &self,
        doc_id: &str,
        timeout_secs: u64,
    ) -> Result<i64, Self::Error>;

    /// Remove heartbeat for a document instance.
    /// Returns true if this was the last instance (document instances hash was deleted).
    async fn remove_instance_heartbeat(
        &self,
        doc_id: &str,
        instance_id: &str,
    ) -> Result<bool, Self::Error>;

    // Connection management

    /// Create a dedicated Redis connection for stream operations.
    /// This is useful for long-running operations that need their own connection.
    async fn create_dedicated_connection(
        &self,
    ) -> Result<Box<dyn RedisConnection<Error = anyhow::Error>>, Self::Error>;
}

/// Trait for Redis connections used in stream operations.
/// This allows for abstraction over different types of Redis connections.
#[async_trait]
pub trait RedisConnection: Send + Sync {
    /// Error type for connection operations.
    type Error: Send + Sync + 'static;

    /// Publish an update using this dedicated connection.
    async fn publish_update(
        &mut self,
        stream_key: &str,
        update: &[u8],
        instance_id: &str,
    ) -> Result<(), Self::Error>;

    /// Publish an update with TTL using this dedicated connection.
    async fn publish_update_with_ttl(
        &mut self,
        stream_key: &str,
        update: &[u8],
        instance_id: &str,
        ttl: u64,
    ) -> Result<(), Self::Error>;

    /// Read and filter updates using this dedicated connection.
    async fn read_and_filter(
        &mut self,
        stream_key: &str,
        count: usize,
        instance_id: &str,
        last_read_id: &Arc<Mutex<String>>,
    ) -> Result<Vec<Bytes>, Self::Error>;
}
