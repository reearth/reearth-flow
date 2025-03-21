use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use std::sync::Arc;
use tracing::{debug, error, info};
use yrs::updates::encoder::Encode;
use yrs::{Doc, ReadTxn, StateVector, Transact};

use crate::doc::types::{Document, HistoryItem};
use crate::doc::types::{DocumentResponse, HistoryResponse, RollbackRequest};
use crate::storage::kv::DocOps;
use crate::AppState;

pub struct DocumentHandler;

impl DocumentHandler {
    pub async fn get_latest(
        Path(doc_id): Path<String>,
        State(state): State<Arc<AppState>>,
    ) -> Response {
        debug!("Handling GetLatest request for document: {}", doc_id);

        let storage = state.pool.get_store();
        let doc = Doc::new();
        let doc_id_clone = doc_id.clone();

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

                        let metadata = storage.get_latest_update_metadata(&doc_id_clone).await?;

                        let latest_clock = metadata.map(|(clock, _)| clock).unwrap_or(0);
                        let timestamp = if let Some((_, ts)) = metadata {
                            chrono::DateTime::from_timestamp(ts.unix_timestamp(), 0)
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
            Ok(doc) => Json(DocumentResponse {
                id: doc.id,
                updates: doc.updates,
                version: doc.version,
                timestamp: doc.timestamp.to_rfc3339(),
            })
            .into_response(),
            Err(err) => {
                error!("Failed to get document {}: {}", doc_id, err);

                let status_code = if err.to_string().contains("not found") {
                    StatusCode::NOT_FOUND
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                };

                (status_code, format!("Error: {}", err)).into_response()
            }
        }
    }

    pub async fn get_history(
        Path(doc_id): Path<String>,
        State(state): State<Arc<AppState>>,
    ) -> Response {
        debug!("Handling GetHistory request for document: {}", doc_id);

        let storage = state.pool.get_store();
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
                let history: Vec<HistoryResponse> = history_items
                    .into_iter()
                    .map(|item| HistoryResponse {
                        updates: item.updates,
                        version: item.version,
                        timestamp: item.timestamp.to_rfc3339(),
                    })
                    .collect();

                Json(history).into_response()
            }
            Err(err) => {
                error!("Failed to get history for document {}: {}", doc_id, err);

                let status_code = if err.to_string().contains("not found") {
                    StatusCode::NOT_FOUND
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                };

                (status_code, format!("Error: {}", err)).into_response()
            }
        }
    }

    pub async fn rollback(
        Path(doc_id): Path<String>,
        State(state): State<Arc<AppState>>,
        Json(request): Json<RollbackRequest>,
    ) -> Response {
        info!(
            "Rolling back document {} to version {}",
            doc_id, request.version
        );

        let storage = state.pool.get_store();
        let doc_id_clone = doc_id.clone();
        let version = request.version;

        let rollback_result = tokio::task::block_in_place(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                let doc = storage.rollback_to(&doc_id_clone, version as u32).await?;

                let read_txn = doc.transact();
                let state = read_txn.encode_diff_v1(&StateVector::default());

                Ok::<_, anyhow::Error>(Document {
                    id: doc_id_clone,
                    version,
                    timestamp: Utc::now(),
                    updates: state,
                })
            })
        });

        match rollback_result {
            Ok(doc) => Json(DocumentResponse {
                id: doc.id,
                updates: doc.updates,
                version: doc.version,
                timestamp: doc.timestamp.to_rfc3339(),
            })
            .into_response(),
            Err(err) => {
                error!(
                    "Failed to rollback document {} to version {}: {}",
                    doc_id, version, err
                );

                let status_code = if err.to_string().contains("not found") {
                    StatusCode::NOT_FOUND
                } else if err.to_string().contains("version") {
                    StatusCode::BAD_REQUEST
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                };

                (status_code, format!("Error: {}", err)).into_response()
            }
        }
    }
}
