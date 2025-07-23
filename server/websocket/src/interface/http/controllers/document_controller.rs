use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use std::sync::Arc;
use tracing::error;
use yrs::{ReadTxn, StateVector, Transact};

use crate::application::dto::AppState;
use crate::interface::http::dto::{
    CreateSnapshotRequest, DocumentDto, DocumentResponse, HistoryItemDto, HistoryMetadataResponse,
    HistoryResponse, RollbackRequest, SnapshotResponse,
};

/// Document Controller - Interface Layer
/// Handles HTTP requests and delegates to Application Services
pub struct DocumentController;

impl DocumentController {
    /// Create a snapshot of a document
    pub async fn create_snapshot(
        State(state): State<Arc<AppState>>,
        Json(request): Json<CreateSnapshotRequest>,
    ) -> Response {
        let doc_id = request.doc_id;
        let version = request.version;

        let result = async {
            let doc_result = state
                .document_service
                .create_snapshot(&doc_id, version)
                .await?;

            let doc = match doc_result {
                Some(doc) => doc,
                None => return Err(anyhow::anyhow!("Failed to create snapshot")),
            };

            let read_txn = doc.transact();
            let state = read_txn.encode_state_as_update_v1(&StateVector::default());
            drop(read_txn);

            let timestamp = Utc::now();
            let document = DocumentDto {
                id: doc_id.clone(),
                version,
                timestamp,
                updates: state,
            };

            let response = SnapshotResponse {
                success: true,
                doc_id: document.id,
                version: document.version,
                timestamp: document.timestamp.to_rfc3339(),
            };

            Ok::<SnapshotResponse, anyhow::Error>(response)
        }
        .await;

        match result {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(e) => {
                error!("Failed to create snapshot: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }

    /// Get the latest version of a document
    pub async fn get_latest(
        Path(doc_id): Path<String>,
        State(state): State<Arc<AppState>>,
    ) -> Response {
        let result = async {
            let doc_result = state.document_service.load_document(&doc_id).await?;

            let doc = match doc_result {
                Some(doc) => doc,
                None => return Err(anyhow::anyhow!("Document not found")),
            };

            let read_txn = doc.transact();
            let updates = read_txn.encode_state_as_update_v1(&StateVector::default());
            drop(read_txn);

            // Get version from latest update metadata
            let version = match state
                .document_service
                .get_latest_update_metadata(&doc_id)
                .await?
            {
                Some((clock, _)) => clock as u64,
                None => 1,
            };

            let timestamp = Utc::now();
            let response = DocumentResponse {
                id: doc_id,
                updates,
                version,
                timestamp: timestamp.to_rfc3339(),
            };

            Ok::<DocumentResponse, anyhow::Error>(response)
        }
        .await;

        match result {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(e) => {
                error!("Failed to get latest document: {}", e);
                StatusCode::NOT_FOUND.into_response()
            }
        }
    }

    /// Get document history
    pub async fn get_history(
        Path(doc_id): Path<String>,
        State(state): State<Arc<AppState>>,
    ) -> Response {
        let result = async {
            let history = state.document_service.get_updates(&doc_id).await?;

            let history_items: Vec<HistoryItemDto> = history.into_iter().map(Into::into).collect();

            let response = HistoryResponse {
                doc_id,
                history: history_items,
            };

            Ok::<HistoryResponse, anyhow::Error>(response)
        }
        .await;

        match result {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(e) => {
                error!("Failed to get document history: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }

    /// Rollback document to a specific version
    pub async fn rollback(
        Path(doc_id): Path<String>,
        State(state): State<Arc<AppState>>,
        Json(request): Json<RollbackRequest>,
    ) -> Response {
        let result = async {
            let doc = state
                .document_service
                .rollback_document(&doc_id, request.version as u32)
                .await?;

            let read_txn = doc.transact();
            let updates = read_txn.encode_state_as_update_v1(&StateVector::default());
            drop(read_txn);

            // Get version from latest update metadata after rollback
            let version = match state
                .document_service
                .get_latest_update_metadata(&doc_id)
                .await?
            {
                Some((clock, _)) => clock as u64,
                None => request.version,
            };

            let timestamp = Utc::now();
            let response = DocumentResponse {
                id: doc_id,
                updates,
                version,
                timestamp: timestamp.to_rfc3339(),
            };

            Ok::<DocumentResponse, anyhow::Error>(response)
        }
        .await;

        match result {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(e) => {
                error!("Failed to rollback document: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }

    /// Get history metadata (available versions)
    pub async fn get_history_metadata(
        Path(doc_id): Path<String>,
        State(state): State<Arc<AppState>>,
    ) -> Response {
        let result = async {
            let history = state.document_service.get_updates(&doc_id).await?;

            let versions: Vec<u64> = history.iter().map(|item| item.clock as u64).collect();

            let response = HistoryMetadataResponse { versions };

            Ok::<HistoryMetadataResponse, anyhow::Error>(response)
        }
        .await;

        match result {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(e) => {
                error!("Failed to get history metadata: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }

    /// Get document history by specific version
    pub async fn get_history_by_version(
        Path((doc_id, version)): Path<(String, u64)>,
        State(state): State<Arc<AppState>>,
    ) -> Response {
        let result = async {
            let history = state.document_service.get_updates(&doc_id).await?;

            let history_item = history
                .into_iter()
                .find(|item| item.clock as u64 == version)
                .map(Into::into);

            match history_item {
                Some(item) => Ok::<HistoryItemDto, anyhow::Error>(item),
                None => Err(anyhow::anyhow!("Version not found")),
            }
        }
        .await;

        match result {
            Ok(response) => (StatusCode::OK, Json(response)).into_response(),
            Err(e) => {
                error!("Failed to get history by version: {}", e);
                StatusCode::NOT_FOUND.into_response()
            }
        }
    }

    /// Flush document to GCS
    pub async fn flush_to_gcs(
        Path(doc_id): Path<String>,
        State(state): State<Arc<AppState>>,
    ) -> Response {
        let result = state.document_service.flush_to_gcs(&doc_id).await;

        match result {
            Ok(_) => StatusCode::OK.into_response(),
            Err(e) => {
                error!("Failed to flush to GCS: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}
