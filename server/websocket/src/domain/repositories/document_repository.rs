use crate::domain::entity::{Document, DocumentId};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait DocumentRepository: Send + Sync {
    async fn get(&self, id: &DocumentId) -> Result<Option<Document>>;

    async fn save(&self, document: &Document) -> Result<()>;

    async fn delete(&self, id: &DocumentId) -> Result<()>;

    async fn exists(&self, id: &DocumentId) -> Result<bool>;
}
