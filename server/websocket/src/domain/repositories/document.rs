use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::entities::doc::{Document, HistoryItem};

#[async_trait]
pub trait DocumentRepository: Send + Sync {
    async fn create_snapshot(&self, doc_id: &str, version: u64) -> Result<Option<Document>>;
    async fn fetch_latest(&self, doc_id: &str) -> Result<Option<Document>>;
    async fn fetch_history(&self, doc_id: &str) -> Result<Vec<HistoryItem>>;
    async fn fetch_history_metadata(&self, doc_id: &str) -> Result<Vec<(u32, DateTime<Utc>)>>;
    async fn fetch_history_version(
        &self,
        doc_id: &str,
        version: u64,
    ) -> Result<Option<HistoryItem>>;
    async fn rollback(&self, doc_id: &str, version: u64) -> Result<Document>;
    async fn flush_to_gcs(&self, doc_id: &str) -> Result<()>;
    async fn save_snapshot(&self, doc_id: &str) -> Result<()>;
    async fn copy_document(&self, doc_id: &str, source: &str) -> Result<()>;
    async fn import_document(&self, doc_id: &str, data: &[u8]) -> Result<()>;
}
