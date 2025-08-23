use crate::domain::repository::RedisRepository;
use crate::domain::value_objects::document_name::DocumentName;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use yrs::{sync::Awareness, Doc};

/// Repository interface for Y.js awareness operations
#[async_trait]
pub trait AwarenessRepository: Send + Sync {
    /// Load Y.js document and create awareness
    async fn load_awareness(&self, document_name: &DocumentName) -> Result<Arc<RwLock<Awareness>>>;

    /// Save awareness state
    async fn save_awareness_state(
        &self,
        document_name: &DocumentName,
        awareness: &Awareness,
        redis: &dyn RedisRepository<Error = anyhow::Error>,
    ) -> Result<()>;

    /// Get awareness update for broadcasting
    async fn get_awareness_update(&self, document_name: &DocumentName) -> Result<Option<Vec<u8>>>;
}
