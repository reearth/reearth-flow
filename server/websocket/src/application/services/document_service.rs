use std::sync::Arc;

use crate::application::kv::DocOps;
use chrono::{DateTime, Utc};
use thiserror::Error;
use tracing::error;
use yrs::{Doc, ReadTxn, StateVector, Transact};

use crate::application::services::broadcast_pool::BroadcastPool;
use crate::domain::entity::doc::Document;
use crate::domain::value_objects::http::HistoryItem;

#[derive(Debug, Error)]
pub enum DocumentServiceError {
    #[error("document not found: {document_id}")]
    NotFound { document_id: String },
    #[error("invalid request: {message}")]
    InvalidRequest { message: String },
    #[error("{message}")]
    Unexpected {
        message: String,
        #[source]
        source: anyhow::Error,
    },
}

#[derive(Clone, Debug)]
pub struct DocumentService {
    pool: Arc<BroadcastPool>,
}

impl DocumentService {
    pub fn new(pool: Arc<BroadcastPool>) -> Self {
        Self { pool }
    }

    pub async fn create_snapshot(
        &self,
        doc_id: &str,
        version: u64,
    ) -> Result<Document, DocumentServiceError> {
        let storage = self.pool.get_store();

        let doc = storage
            .create_snapshot_from_version(doc_id, version)
            .await
            .map_err(|err| DocumentServiceError::Unexpected {
                message: format!(
                    "failed to create snapshot for document '{}' at version {}",
                    doc_id, version
                ),
                source: err.into(),
            })?
            .ok_or_else(|| DocumentServiceError::Unexpected {
                message: format!("snapshot not found for document '{}'", doc_id),
                source: anyhow::anyhow!("snapshot not found"),
            })?;

        let read_txn = doc.transact();
        let state = read_txn.encode_state_as_update_v1(&StateVector::default());
        drop(read_txn);

        let document = Document::new(doc_id.to_string(), state, version, Utc::now());
        Ok(document)
    }

    pub async fn get_latest_document(
        &self,
        doc_id: &str,
    ) -> Result<Document, DocumentServiceError> {
        if let Err(err) = self.pool.flush_to_gcs(doc_id).await {
            error!(
                "failed to flush websocket changes for '{}' before fetching latest: {}",
                doc_id, err
            );
        }

        let storage = self.pool.get_store();
        let doc = Doc::new();
        let mut txn = doc.transact_mut();

        let result = match storage.load_doc_v2(doc_id, &mut txn).await {
            Ok(()) => {
                let state = txn.encode_diff_v1(&StateVector::default());
                drop(txn);

                let metadata = storage
                    .get_latest_update_metadata(doc_id)
                    .await
                    .map_err(|err| DocumentServiceError::Unexpected {
                        message: format!("failed to get latest metadata for document '{}'", doc_id),
                        source: err.into(),
                    })?;

                let (version, timestamp) = metadata
                    .map(|(clock, ts)| {
                        let ts = chrono::DateTime::from_timestamp(ts.unix_timestamp(), 0)
                            .unwrap_or_else(Utc::now);
                        (clock as u64, ts)
                    })
                    .unwrap_or_else(|| (0, Utc::now()));

                Ok(Document::new(doc_id.to_string(), state, version, timestamp))
            }
            Err(_) => {
                drop(txn);

                let fallback_doc = Doc::new();
                let mut fallback_txn = fallback_doc.transact_mut();
                match storage.load_doc(doc_id, &mut fallback_txn).await {
                    Ok(true) => {
                        drop(fallback_txn);
                        let read_txn = fallback_doc.transact();
                        let state = read_txn.encode_diff_v1(&StateVector::default());
                        drop(read_txn);

                        let metadata =
                            storage
                                .get_latest_update_metadata(doc_id)
                                .await
                                .map_err(|err| DocumentServiceError::Unexpected {
                                    message: format!(
                                        "failed to get latest metadata for document '{}'",
                                        doc_id
                                    ),
                                    source: err.into(),
                                })?;

                        let (version, timestamp) = metadata
                            .map(|(clock, ts)| {
                                let ts = chrono::DateTime::from_timestamp(ts.unix_timestamp(), 0)
                                    .unwrap_or_else(Utc::now);
                                (clock as u64, ts)
                            })
                            .unwrap_or_else(|| (0, Utc::now()));

                        Ok(Document::new(doc_id.to_string(), state, version, timestamp))
                    }
                    Ok(false) => Err(DocumentServiceError::NotFound {
                        document_id: doc_id.to_string(),
                    }),
                    Err(err) => Err(DocumentServiceError::Unexpected {
                        message: format!("failed to load document '{}'", doc_id),
                        source: err.into(),
                    }),
                }
            }
        };

        result
    }

    pub async fn get_history(
        &self,
        doc_id: &str,
    ) -> Result<Vec<HistoryItem>, DocumentServiceError> {
        let storage = self.pool.get_store();
        let updates =
            storage
                .get_updates(doc_id)
                .await
                .map_err(|err| DocumentServiceError::Unexpected {
                    message: format!("failed to load history for document '{}'", doc_id),
                    source: err.into(),
                })?;

        let history = updates.into_iter().map(HistoryItem::from).collect();
        Ok(history)
    }

    pub async fn rollback(
        &self,
        doc_id: &str,
        version: u64,
    ) -> Result<Document, DocumentServiceError> {
        let storage = self.pool.get_store();
        let doc = storage
            .rollback_to(doc_id, version as u32)
            .await
            .map_err(|err| {
                let message = err.to_string();
                if message.contains("not found") {
                    DocumentServiceError::NotFound {
                        document_id: doc_id.to_string(),
                    }
                } else if message.contains("version") {
                    DocumentServiceError::InvalidRequest { message }
                } else {
                    DocumentServiceError::Unexpected {
                        message: format!(
                            "failed to rollback document '{}' to version {}",
                            doc_id, version
                        ),
                        source: err.into(),
                    }
                }
            })?;

        let read_txn = doc.transact();
        let state = read_txn.encode_state_as_update_v1(&StateVector::default());
        drop(read_txn);

        let document = Document::new(doc_id.to_string(), state, version, Utc::now());
        Ok(document)
    }

    pub async fn get_history_metadata(
        &self,
        doc_id: &str,
    ) -> Result<Vec<(u32, DateTime<Utc>)>, DocumentServiceError> {
        let storage = self.pool.get_store();
        let metadata = storage.get_updates_metadata(doc_id).await.map_err(|err| {
            DocumentServiceError::Unexpected {
                message: format!("failed to load history metadata for '{}'", doc_id),
                source: err.into(),
            }
        })?;

        let history = metadata
            .into_iter()
            .map(|(clock, ts)| {
                let timestamp = chrono::DateTime::from_timestamp(ts.unix_timestamp(), 0)
                    .unwrap_or_else(Utc::now);
                (clock, timestamp)
            })
            .collect();

        Ok(history)
    }

    pub async fn get_history_by_version(
        &self,
        doc_id: &str,
        version: u64,
    ) -> Result<HistoryItem, DocumentServiceError> {
        let storage = self.pool.get_store();
        let version_u32 = version as u32;

        let update_info = storage
            .get_updates_by_version(doc_id, version_u32)
            .await
            .map_err(|err| DocumentServiceError::Unexpected {
                message: format!(
                    "failed to load history version {} for document '{}'",
                    version, doc_id
                ),
                source: err.into(),
            })?;

        match update_info {
            Some(info) => Ok(HistoryItem::from(info)),
            None => Err(DocumentServiceError::NotFound {
                document_id: format!("{}@{}", doc_id, version),
            }),
        }
    }

    pub async fn flush_to_gcs(&self, doc_id: &str) -> Result<(), DocumentServiceError> {
        self.pool
            .flush_to_gcs(doc_id)
            .await
            .map_err(|err| DocumentServiceError::Unexpected {
                message: format!("failed to flush updates for document '{}'", doc_id),
                source: err,
            })
    }

    pub async fn save_snapshot(&self, doc_id: &str) -> Result<(), DocumentServiceError> {
        self.pool
            .save_snapshot(doc_id)
            .await
            .map_err(|err| DocumentServiceError::Unexpected {
                message: format!("failed to save snapshot for document '{}'", doc_id),
                source: err,
            })
    }

    pub async fn copy_document(
        &self,
        doc_id: &str,
        source: &str,
    ) -> Result<(), DocumentServiceError> {
        let storage = self.pool.get_store();
        storage.copy_document(doc_id, source).await.map_err(|err| {
            DocumentServiceError::Unexpected {
                message: format!("failed to copy document '{}' from '{}'", doc_id, source),
                source: err.into(),
            }
        })
    }

    pub async fn import_document(
        &self,
        doc_id: &str,
        data: &[u8],
    ) -> Result<(), DocumentServiceError> {
        let storage = self.pool.get_store();
        storage.import_document(doc_id, data).await.map_err(|err| {
            DocumentServiceError::Unexpected {
                message: format!("failed to import document '{}'", doc_id),
                source: err.into(),
            }
        })
    }
}
