use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use std::sync::Arc;
use tracing::error;
use yrs::{Doc, ReadTxn, StateVector, Transact};

use crate::domain::value_objects::http::{
    CreateSnapshotRequest, DocumentResponse, HistoryMetadataResponse, HistoryResponse,
    RollbackRequest, SnapshotResponse,
};
use crate::domain::value_objects::http::{Document, HistoryItem};
use crate::storage::kv::DocOps;
use crate::AppState;

pub struct DocumentHandler;

impl DocumentHandler {
    pub async fn create_snapshot(
        State(state): State<Arc<AppState>>,
        Json(request): Json<CreateSnapshotRequest>,
    ) -> Response {
        let storage = state.pool.get_store();
        let doc_id = request.doc_id;
        let version = request.version;

        let result = async {
            let doc_result = storage
                .create_snapshot_from_version(&doc_id, version)
                .await?;

            let doc = match doc_result {
                Some(doc) => doc,
                None => return Err(anyhow::anyhow!("Failed to create snapshot")),
            };

            let read_txn = doc.transact();
            let state = read_txn.encode_state_as_update_v1(&StateVector::default());
            drop(read_txn);

            let timestamp = Utc::now();
            let document = Document {
                id: doc_id.clone(),
                version,
                timestamp,
                updates: state,
            };

            Ok::<_, anyhow::Error>(document)
        }
        .await;

        match result {
            Ok(doc) => Json(SnapshotResponse {
                id: doc.id,
                updates: doc.updates,
                version: doc.version,
                timestamp: doc.timestamp.to_rfc3339(),
                name: request.name,
            })
            .into_response(),
            Err(err) => {
                error!("Failed to create snapshot for document {}: {}", doc_id, err);
                let status_code = if err.to_string().contains("not found") {
                    StatusCode::NOT_FOUND
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                };

                (status_code, format!("Error: {err}")).into_response()
            }
        }
    }

    pub async fn get_latest(
        Path(doc_id): Path<String>,
        State(state): State<Arc<AppState>>,
    ) -> Response {
        if let Err(e) = state.pool.flush_to_gcs(&doc_id).await {
            error!("Failed to flush websocket changes for '{}': {}", doc_id, e);
        }

        let storage = state.pool.get_store();

        let result = async {
            match storage.load_doc_v2(&doc_id).await {
                Ok(doc) => {
                    let read_txn = doc.transact();
                    let state = read_txn.encode_diff_v1(&StateVector::default());
                    drop(read_txn);

                    let metadata = storage.get_latest_update_metadata(&doc_id).await?;

                    let latest_clock = metadata.map(|(clock, _)| clock).unwrap_or(0);
                    let timestamp = if let Some((_, ts)) = metadata {
                        chrono::DateTime::from_timestamp(ts.unix_timestamp(), 0)
                            .unwrap_or(Utc::now())
                    } else {
                        Utc::now()
                    };

                    let document = Document {
                        id: doc_id.clone(),
                        version: latest_clock as u64,
                        timestamp,
                        updates: state,
                    };

                    Ok::<_, anyhow::Error>(document)
                }
                Err(_) => {
                    let doc = Doc::new();
                    let mut txn = doc.transact_mut();
                    let load_result = storage.load_doc(&doc_id, &mut txn).await;

                    match load_result {
                        Ok(true) => {
                            drop(txn);
                            let read_txn = doc.transact();
                            let state = read_txn.encode_diff_v1(&StateVector::default());
                            drop(read_txn);

                            let metadata = storage.get_latest_update_metadata(&doc_id).await?;

                            let latest_clock = metadata.map(|(clock, _)| clock).unwrap_or(0);
                            let timestamp = if let Some((_, ts)) = metadata {
                                chrono::DateTime::from_timestamp(ts.unix_timestamp(), 0)
                                    .unwrap_or(Utc::now())
                            } else {
                                Utc::now()
                            };

                            let document = Document {
                                id: doc_id.clone(),
                                version: latest_clock as u64,
                                timestamp,
                                updates: state,
                            };

                            Ok::<_, anyhow::Error>(document)
                        }
                        Ok(false) => Err(anyhow::anyhow!("Document not found: {}", doc_id)),
                        Err(e) => Err(anyhow::anyhow!("Failed to load document: {}", e)),
                    }
                }
            }
        }
        .await;

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

                (status_code, format!("Error: {err}")).into_response()
            }
        }
    }

    pub async fn get_history(
        Path(doc_id): Path<String>,
        State(state): State<Arc<AppState>>,
    ) -> Response {
        let storage = state.pool.get_store();

        let result = async {
            let updates = storage.get_updates(&doc_id).await?;

            let history_items: Vec<HistoryItem> =
                updates.into_iter().map(HistoryItem::from).collect();

            Ok::<_, anyhow::Error>(history_items)
        }
        .await;

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

                (status_code, format!("Error: {err}")).into_response()
            }
        }
    }

    pub async fn rollback(
        Path(doc_id): Path<String>,
        State(state): State<Arc<AppState>>,
        Json(request): Json<RollbackRequest>,
    ) -> Response {
        let storage = state.pool.get_store();
        let version = request.version;

        let rollback_result = async {
            let doc = storage.rollback_to(&doc_id, version as u32).await?;

            let read_txn = doc.transact();
            let state = read_txn.encode_state_as_update_v1(&StateVector::default());

            Ok::<_, anyhow::Error>(Document {
                id: doc_id.clone(),
                version,
                timestamp: Utc::now(),
                updates: state,
            })
        }
        .await;

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

                (status_code, format!("Error: {err}")).into_response()
            }
        }
    }

    pub async fn get_history_metadata(
        Path(doc_id): Path<String>,
        State(state): State<Arc<AppState>>,
    ) -> Response {
        let storage = state.pool.get_store();

        let result = async {
            let metadata = storage.get_updates_metadata(&doc_id).await?;
            Ok::<_, anyhow::Error>(metadata)
        }
        .await;

        match result {
            Ok(metadata) => {
                let history: Vec<HistoryMetadataResponse> = metadata
                    .into_iter()
                    .map(|(clock, timestamp)| HistoryMetadataResponse {
                        version: clock as u64,
                        timestamp: chrono::DateTime::from_timestamp(timestamp.unix_timestamp(), 0)
                            .unwrap_or(Utc::now())
                            .to_rfc3339(),
                    })
                    .collect();

                Json(history).into_response()
            }
            Err(err) => {
                error!(
                    "Failed to get history metadata for document {}: {}",
                    doc_id, err
                );

                let status_code = if err.to_string().contains("not found") {
                    StatusCode::NOT_FOUND
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                };

                (status_code, format!("Error: {err}")).into_response()
            }
        }
    }

    pub async fn get_history_by_version(
        Path((doc_id, version)): Path<(String, u64)>,
        State(state): State<Arc<AppState>>,
    ) -> Response {
        let storage = state.pool.get_store();
        let version_u32 = version as u32;

        let result = async {
            let update_info = storage.get_updates_by_version(&doc_id, version_u32).await?;

            match update_info {
                Some(info) => {
                    let item = HistoryItem::from(info);
                    Ok::<_, anyhow::Error>(item)
                }
                None => Err(anyhow::anyhow!("History version not found")),
            }
        }
        .await;

        match result {
            Ok(item) => {
                let response = HistoryResponse {
                    updates: item.updates,
                    version: item.version,
                    timestamp: item.timestamp.to_rfc3339(),
                };

                Json(response).into_response()
            }
            Err(err) => {
                error!(
                    "Failed to get history for document {} version {}: {}",
                    doc_id, version, err
                );

                let status_code = if err.to_string().contains("not found") {
                    StatusCode::NOT_FOUND
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                };

                (status_code, format!("Error: {err}")).into_response()
            }
        }
    }

    pub async fn flush_to_gcs(
        Path(doc_id): Path<String>,
        State(state): State<Arc<AppState>>,
    ) -> Response {
        match state.pool.save_snapshot(&doc_id).await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(err) => {
                error!("Failed to flush document {} to GCS: {}", doc_id, err);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
