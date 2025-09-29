use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use time::OffsetDateTime;
use yrs::{Doc, ReadTxn, StateVector, Transact};

use crate::application::kv::DocOps;
use crate::application::services::broadcast_pool::BroadcastPool;
use crate::domain::entity::doc::Document;
use crate::domain::repository::document::DocumentRepository;
use crate::domain::value_objects::http::HistoryItem;
use crate::infrastructure::gcs::{GcsStore, UpdateInfo};

pub struct DocumentRepositoryImpl {
    pool: Arc<BroadcastPool>,
}

impl DocumentRepositoryImpl {
    pub fn new(pool: Arc<BroadcastPool>) -> Self {
        Self { pool }
    }

    fn store(&self) -> Arc<GcsStore> {
        self.pool.get_store()
    }

    fn to_document(doc_id: &str, doc: Doc, version: u64, timestamp: DateTime<Utc>) -> Document {
        let read_txn = doc.transact();
        let state = read_txn.encode_state_as_update_v1(&StateVector::default());
        drop(read_txn);

        Document::new(doc_id.to_string(), state, version, timestamp)
    }

    fn metadata_to_timestamp(metadata: Option<(u32, OffsetDateTime)>) -> (u64, DateTime<Utc>) {
        match metadata {
            Some((clock, ts)) => {
                let timestamp = chrono::DateTime::from_timestamp(ts.unix_timestamp(), 0)
                    .unwrap_or_else(Utc::now);
                (clock as u64, timestamp)
            }
            None => (0, Utc::now()),
        }
    }

    fn updates_to_history_items(updates: Vec<UpdateInfo>) -> Vec<HistoryItem> {
        updates.into_iter().map(HistoryItem::from).collect()
    }

    fn metadata_list_to_history(metadata: Vec<(u32, OffsetDateTime)>) -> Vec<(u32, DateTime<Utc>)> {
        metadata
            .into_iter()
            .map(|(clock, ts)| {
                let timestamp = chrono::DateTime::from_timestamp(ts.unix_timestamp(), 0)
                    .unwrap_or_else(Utc::now);
                (clock, timestamp)
            })
            .collect()
    }
}

#[async_trait]
impl DocumentRepository for DocumentRepositoryImpl {
    async fn create_snapshot(&self, doc_id: &str, version: u64) -> Result<Option<Document>> {
        let store = self.store();
        let snapshot = store.create_snapshot_from_version(doc_id, version).await?;

        snapshot.map_or(Ok(None), |doc| {
            let document = Self::to_document(doc_id, doc, version, Utc::now());
            Ok(Some(document))
        })
    }

    async fn fetch_latest(&self, doc_id: &str) -> Result<Option<Document>> {
        let store = self.store();

        let doc = Doc::new();
        let mut txn = doc.transact_mut();

        match store.load_doc_v2(doc_id, &mut txn).await {
            Ok(()) => {
                let state = txn.encode_diff_v1(&StateVector::default());
                drop(txn);

                let metadata = store.get_latest_update_metadata(doc_id).await?;
                let (version, timestamp) = Self::metadata_to_timestamp(metadata);

                Ok(Some(Document::new(
                    doc_id.to_string(),
                    state,
                    version,
                    timestamp,
                )))
            }
            Err(_) => {
                drop(txn);

                let fallback_doc = Doc::new();
                let mut fallback_txn = fallback_doc.transact_mut();
                match store.load_doc(doc_id, &mut fallback_txn).await {
                    Ok(true) => {
                        drop(fallback_txn);
                        let read_txn = fallback_doc.transact();
                        let state = read_txn.encode_diff_v1(&StateVector::default());
                        drop(read_txn);

                        let metadata = store.get_latest_update_metadata(doc_id).await?;
                        let (version, timestamp) = Self::metadata_to_timestamp(metadata);

                        Ok(Some(Document::new(
                            doc_id.to_string(),
                            state,
                            version,
                            timestamp,
                        )))
                    }
                    Ok(false) => Ok(None),
                    Err(err) => Err(err.into()),
                }
            }
        }
    }

    async fn fetch_history(&self, doc_id: &str) -> Result<Vec<HistoryItem>> {
        let store = self.store();
        let updates = store.get_updates(doc_id).await?;
        Ok(Self::updates_to_history_items(updates))
    }

    async fn fetch_history_metadata(&self, doc_id: &str) -> Result<Vec<(u32, DateTime<Utc>)>> {
        let store = self.store();
        let metadata = store.get_updates_metadata(doc_id).await?;
        Ok(Self::metadata_list_to_history(metadata))
    }

    async fn fetch_history_version(
        &self,
        doc_id: &str,
        version: u64,
    ) -> Result<Option<HistoryItem>> {
        let store = self.store();
        let result = store.get_updates_by_version(doc_id, version as u32).await?;
        Ok(result.map(HistoryItem::from))
    }

    async fn rollback(&self, doc_id: &str, version: u64) -> Result<Document> {
        let store = self.store();
        let doc = store.rollback_to(doc_id, version as u32).await?;
        let document = Self::to_document(doc_id, doc, version, Utc::now());
        Ok(document)
    }

    async fn flush_to_gcs(&self, doc_id: &str) -> Result<()> {
        self.pool.flush_to_gcs(doc_id).await
    }

    async fn save_snapshot(&self, doc_id: &str) -> Result<()> {
        self.pool.save_snapshot(doc_id).await
    }

    async fn copy_document(&self, doc_id: &str, source: &str) -> Result<()> {
        let store = self.store();
        store.copy_document(doc_id, source).await?;
        Ok(())
    }

    async fn import_document(&self, doc_id: &str, data: &[u8]) -> Result<()> {
        let store = self.store();
        store.import_document(doc_id, data).await?;
        Ok(())
    }
}
