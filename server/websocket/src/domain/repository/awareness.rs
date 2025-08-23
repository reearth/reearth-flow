use crate::domain::value_objects::document_name::DocumentName;
use anyhow::Result;
use async_trait::async_trait;
use yrs::{sync::Awareness, Doc};

/// Repository interface for Y.js awareness operations
#[async_trait]
pub trait AwarenessRepository: Send + Sync {
    /// Load Y.js document and create awareness
    async fn load_awareness(&self, document_name: &DocumentName, doc: &Doc) -> Result<()>;

    /// Save awareness state
    async fn save_awareness_state<D>(
        &self,
        document_name: &DocumentName,
        awareness: &Awareness,
        redis: &D,
    ) -> Result<()>;

    /// Get awareness update for broadcasting
    async fn get_awareness_update(&self, document_name: &DocumentName) -> Result<Option<Vec<u8>>>;
}
