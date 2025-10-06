use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use tracing::{error, warn};

use crate::{
    presentation::http::dto::{
        CreateSnapshotRequest, DocumentResponse, HistoryMetadataResponse, HistoryResponse,
        ImportDocumentRequest, RollbackRequest, SnapshotResponse,
    },
    AppState, DocumentUseCaseError,
};

pub struct DocumentHandler;

impl DocumentHandler {
    pub async fn create_snapshot(
        State(state): State<Arc<AppState>>,
        Json(request): Json<CreateSnapshotRequest>,
    ) -> Response {
        let CreateSnapshotRequest {
            doc_id,
            version,
            name,
        } = request;

        match state
            .document_usecase
            .create_snapshot(&doc_id, version)
            .await
        {
            Ok(document) => Json(SnapshotResponse {
                id: document.id.value().to_string(),
                updates: document.updates,
                version: document.version.value(),
                timestamp: document.timestamp.to_rfc3339(),
                name,
            })
            .into_response(),
            Err(err) => handle_service_error(&format!("create_snapshot [{}]", doc_id), err),
        }
    }

    pub async fn get_latest(
        Path(doc_id): Path<String>,
        State(state): State<Arc<AppState>>,
    ) -> Response {
        match state.document_usecase.get_latest_document(&doc_id).await {
            Ok(document) => Json(DocumentResponse {
                id: document.id.value().to_string(),
                updates: document.updates,
                version: document.version.value(),
                timestamp: document.timestamp.to_rfc3339(),
            })
            .into_response(),
            Err(err) => handle_service_error(&format!("get_latest [{}]", doc_id), err),
        }
    }

    pub async fn get_history(
        Path(doc_id): Path<String>,
        State(state): State<Arc<AppState>>,
    ) -> Response {
        match state.document_usecase.get_history(&doc_id).await {
            Ok(history) => {
                let response: Vec<HistoryResponse> = history
                    .into_iter()
                    .map(|item| HistoryResponse {
                        updates: item.updates,
                        version: item.version.value(),
                        timestamp: item.timestamp.to_rfc3339(),
                    })
                    .collect();

                Json(response).into_response()
            }
            Err(err) => handle_service_error(&format!("get_history [{}]", doc_id), err),
        }
    }

    pub async fn rollback(
        Path(doc_id): Path<String>,
        State(state): State<Arc<AppState>>,
        Json(request): Json<RollbackRequest>,
    ) -> Response {
        match state
            .document_usecase
            .rollback(&doc_id, request.version)
            .await
        {
            Ok(document) => Json(DocumentResponse {
                id: document.id.value().to_string(),
                updates: document.updates,
                version: document.version.value(),
                timestamp: document.timestamp.to_rfc3339(),
            })
            .into_response(),
            Err(err) => handle_service_error(&format!("rollback [{}]", doc_id), err),
        }
    }

    pub async fn get_history_metadata(
        Path(doc_id): Path<String>,
        State(state): State<Arc<AppState>>,
    ) -> Response {
        match state.document_usecase.get_history_metadata(&doc_id).await {
            Ok(metadata) => {
                let response: Vec<HistoryMetadataResponse> = metadata
                    .into_iter()
                    .map(|(clock, timestamp)| HistoryMetadataResponse {
                        version: clock as u64,
                        timestamp: timestamp.to_rfc3339(),
                    })
                    .collect();

                Json(response).into_response()
            }
            Err(err) => handle_service_error(&format!("get_history_metadata [{}]", doc_id), err),
        }
    }

    pub async fn get_history_by_version(
        Path((doc_id, version)): Path<(String, u64)>,
        State(state): State<Arc<AppState>>,
    ) -> Response {
        match state
            .document_usecase
            .get_history_by_version(&doc_id, version)
            .await
        {
            Ok(item) => {
                let response = HistoryResponse {
                    updates: item.updates,
                    version: item.version.value(),
                    timestamp: item.timestamp.to_rfc3339(),
                };

                Json(response).into_response()
            }
            Err(err) => handle_service_error(
                &format!("get_history_by_version [{}@{}]", doc_id, version),
                err,
            ),
        }
    }

    pub async fn flush_to_gcs(
        Path(doc_id): Path<String>,
        State(state): State<Arc<AppState>>,
    ) -> Response {
        match state.document_usecase.save_snapshot(&doc_id).await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(err) => handle_service_error(&format!("flush_to_gcs [{}]", doc_id), err),
        }
    }

    pub async fn copy_document(
        Path((doc_id, source)): Path<(String, String)>,
        State(state): State<Arc<AppState>>,
    ) -> Response {
        match state.document_usecase.copy_document(&doc_id, &source).await {
            Ok(_) => StatusCode::OK.into_response(),
            Err(err) => {
                handle_service_error(&format!("copy_document [{} <- {}]", doc_id, source), err)
            }
        }
    }

    pub async fn import_document(
        Path(doc_id): Path<String>,
        State(state): State<Arc<AppState>>,
        Json(request): Json<ImportDocumentRequest>,
    ) -> Response {
        match state
            .document_usecase
            .import_document(&doc_id, &request.data)
            .await
        {
            Ok(_) => StatusCode::OK.into_response(),
            Err(err) => handle_service_error(&format!("import_document [{}]", doc_id), err),
        }
    }
}

fn handle_service_error(context: &str, err: DocumentUseCaseError) -> Response {
    match err {
        DocumentUseCaseError::NotFound { .. } => {
            warn!("{}: {}", context, err);
            (StatusCode::NOT_FOUND, format!("Error: {err}")).into_response()
        }
        DocumentUseCaseError::InvalidRequest { .. } => {
            warn!("{}: {}", context, err);
            (StatusCode::BAD_REQUEST, format!("Error: {err}")).into_response()
        }
        DocumentUseCaseError::Unexpected { message, source } => {
            error!("{}: {}; source: {:?}", context, message, source);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error: {}", message),
            )
                .into_response()
        }
    }
}
