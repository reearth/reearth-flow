use chrono::Utc;
use std::sync::Arc;
use thrift::server::TProcessor;
use thrift::{ApplicationError, ApplicationErrorKind};
use tracing::{debug, error, info};
use yrs::updates::encoder::Encode;
use yrs::{Doc, ReadTxn, StateVector, Transact};

use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::thrift::document::{
    Document as ThriftDocument, DocumentServiceSyncHandler, DocumentServiceSyncProcessor,
    GetHistoryRequest, GetHistoryResponse, GetLatestRequest, GetLatestResponse, History,
    RollbackRequest, RollbackResponse,
};

#[derive(Debug, Clone)]
struct Document {
    pub id: String,
    pub updates: Vec<u8>,
    pub version: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
struct HistoryItem {
    pub version: u64,
    pub updates: Vec<u8>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub struct DocumentHandler {
    storage: Arc<GcsStore>,
}

impl DocumentHandler {
    pub fn new(storage: Arc<GcsStore>) -> Self {
        Self { storage }
    }

    pub fn processor(self) -> impl TProcessor {
        DocumentServiceSyncProcessor::new(self)
    }
}

impl DocumentServiceSyncHandler for DocumentHandler {
    fn handle_get_latest(&self, request: GetLatestRequest) -> thrift::Result<GetLatestResponse> {
        let doc_id_display = request.doc_id.as_deref().unwrap_or("");
        debug!(
            "Handling GetLatest request for document: {}",
            doc_id_display
        );

        let doc_id = match &request.doc_id {
            Some(id) => id.clone(),
            None => {
                return Err(thrift::Error::Application(ApplicationError::new(
                    ApplicationErrorKind::Unknown,
                    "Document ID is required".to_string(),
                )))
            }
        };

        let storage = self.storage.clone();
        let doc_id_clone = doc_id.clone();

        let doc = Doc::new();

        let result = tokio::task::block_in_place(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                let mut txn = doc.transact_mut();
                let load_result = storage.load_doc(&doc_id_clone, &mut txn).await;

                match load_result {
                    Ok(true) => {
                        drop(txn);
                        let read_txn = doc.transact();
                        let state = read_txn.encode_diff_v1(&StateVector::default());
                        drop(read_txn);

                        let updates = storage.get_updates(&doc_id_clone).await?;

                        let latest_clock = if !updates.is_empty() {
                            updates.last().unwrap().clock as i32
                        } else {
                            0
                        };

                        let timestamp = if !updates.is_empty() {
                            let last_update = updates.last().unwrap();
                            chrono::DateTime::from_timestamp(
                                last_update.timestamp.unix_timestamp(),
                                0,
                            )
                            .unwrap_or(Utc::now())
                        } else {
                            Utc::now()
                        };

                        let document = Document {
                            id: doc_id_clone,
                            version: latest_clock as u64,
                            timestamp,
                            updates: state,
                        };

                        Ok::<_, anyhow::Error>(document)
                    }
                    Ok(false) => Err(anyhow::anyhow!("Document not found: {}", doc_id_clone)),
                    Err(e) => Err(anyhow::anyhow!("Failed to load document: {}", e)),
                }
            })
        });

        match result {
            Ok(doc) => {
                let updates_as_i32: Vec<i32> = doc.updates.iter().map(|&b| b as i32).collect();

                let document = ThriftDocument {
                    id: Some(doc.id),
                    updates: Some(updates_as_i32),
                    version: Some(doc.version as i32),
                    timestamp: Some(doc.timestamp.to_rfc3339()),
                };

                Ok(GetLatestResponse {
                    document: Some(document),
                })
            }
            Err(err) => {
                error!("Failed to get document {}: {}", doc_id, err);
                Err(thrift::Error::Application(ApplicationError::new(
                    ApplicationErrorKind::Unknown,
                    format!("Failed to get document: {}", err),
                )))
            }
        }
    }

    fn handle_get_history(&self, request: GetHistoryRequest) -> thrift::Result<GetHistoryResponse> {
        let doc_id_display = request.doc_id.as_deref().unwrap_or("");
        debug!(
            "Handling GetHistory request for document: {}",
            doc_id_display
        );

        let doc_id = match &request.doc_id {
            Some(id) => id.clone(),
            None => {
                return Err(thrift::Error::Application(ApplicationError::new(
                    ApplicationErrorKind::Unknown,
                    "Document ID is required".to_string(),
                )))
            }
        };

        let storage = self.storage.clone();
        let doc_id_clone = doc_id.clone();

        let result = tokio::task::block_in_place(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                let updates = storage.get_updates(&doc_id_clone).await?;

                let history_items: Vec<HistoryItem> = updates
                    .iter()
                    .map(|update_info| HistoryItem {
                        version: update_info.clock as u64,
                        updates: update_info.update.encode_v1(),
                        timestamp: chrono::DateTime::from_timestamp(
                            update_info.timestamp.unix_timestamp(),
                            0,
                        )
                        .unwrap_or(Utc::now()),
                    })
                    .collect();

                Ok::<_, anyhow::Error>(history_items)
            })
        });

        match result {
            Ok(history_items) => {
                let history = history_items
                    .into_iter()
                    .map(|item| {
                        let updates_as_i32: Vec<i32> =
                            item.updates.iter().map(|&b| b as i32).collect();

                        History {
                            version: Some(item.version as i32),
                            updates: Some(updates_as_i32),
                            timestamp: Some(item.timestamp.to_rfc3339()),
                        }
                    })
                    .collect();

                Ok(GetHistoryResponse {
                    history: Some(history),
                })
            }
            Err(err) => {
                error!("Failed to get history for document {}: {}", doc_id, err);
                Err(thrift::Error::Application(ApplicationError::new(
                    ApplicationErrorKind::Unknown,
                    format!("Failed to get document history: {}", err),
                )))
            }
        }
    }

    fn handle_rollback(&self, request: RollbackRequest) -> thrift::Result<RollbackResponse> {
        let doc_id = match &request.doc_id {
            Some(id) => id.clone(),
            None => {
                return Err(thrift::Error::Application(ApplicationError::new(
                    ApplicationErrorKind::Unknown,
                    "Document ID is required".to_string(),
                )))
            }
        };

        let version = match request.version {
            Some(v) => v as u64,
            None => {
                return Err(thrift::Error::Application(ApplicationError::new(
                    ApplicationErrorKind::Unknown,
                    "Version is required".to_string(),
                )))
            }
        };

        info!("Rolling back document {} to version {}", doc_id, version);

        let storage = self.storage.clone();
        let doc_id_clone = doc_id.clone();

        let rollback_result = tokio::task::block_in_place(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                let doc = Doc::new();
                let mut txn = doc.transact_mut();

                let loaded = storage.load_doc(&doc_id_clone, &mut txn).await?;
                if !loaded {
                    return Err(anyhow::anyhow!("Document not found: {}", doc_id_clone));
                }

                drop(txn);
                let read_txn = doc.transact();
                let state = read_txn.encode_diff_v1(&StateVector::default());
                drop(read_txn);

                Ok::<_, anyhow::Error>(Document {
                    id: doc_id_clone,
                    version,
                    timestamp: Utc::now(),
                    updates: state,
                })
            })
        });

        match rollback_result {
            Ok(doc) => {
                let updates_as_i32: Vec<i32> = doc.updates.iter().map(|&b| b as i32).collect();

                let document = ThriftDocument {
                    id: Some(doc.id),
                    updates: Some(updates_as_i32),
                    version: Some(doc.version as i32),
                    timestamp: Some(doc.timestamp.to_rfc3339()),
                };

                Ok(RollbackResponse {
                    document: Some(document),
                })
            }
            Err(err) => {
                error!(
                    "Failed to rollback document {} to version {}: {}",
                    doc_id, version, err
                );
                Err(thrift::Error::Application(ApplicationError::new(
                    ApplicationErrorKind::Unknown,
                    format!("Failed to rollback document: {}", err),
                )))
            }
        }
    }
}
