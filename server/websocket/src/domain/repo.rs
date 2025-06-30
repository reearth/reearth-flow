use super::value_objects::*;
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait SnapshotRepository: Send + Sync {
    async fn save_snapshot(
        &self,
        document_id: &DocumentId,
        version: &UpdateVersion,
        data: Vec<u8>,
        metadata: HashMap<String, String>,
    ) -> Result<()>;

    async fn get_latest_snapshot(
        &self,
        document_id: &DocumentId,
    ) -> Result<Option<(UpdateVersion, Vec<u8>)>>;

    async fn get_snapshot_by_version(
        &self,
        document_id: &DocumentId,
        version: &UpdateVersion,
    ) -> Result<Option<Vec<u8>>>;

    async fn list_snapshots(
        &self,
        document_id: &DocumentId,
    ) -> Result<Vec<(UpdateVersion, chrono::DateTime<chrono::Utc>)>>;

    async fn delete_snapshots_before_version(
        &self,
        document_id: &DocumentId,
        version: &UpdateVersion,
    ) -> Result<()>;
}
