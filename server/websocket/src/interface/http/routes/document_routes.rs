use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::application::dto::AppState;
use crate::interface::http::controllers::DocumentController;

/// Document Routes - Interface Layer
/// Defines HTTP routes and maps them to controller methods
pub fn document_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/document/{doc_id}", get(DocumentController::get_latest))
        .route(
            "/document/{doc_id}/history",
            get(DocumentController::get_history),
        )
        .route(
            "/document/{doc_id}/history/metadata",
            get(DocumentController::get_history_metadata),
        )
        .route(
            "/document/{doc_id}/history/version/{version}",
            get(DocumentController::get_history_by_version),
        )
        .route(
            "/document/{doc_id}/rollback",
            post(DocumentController::rollback),
        )
        .route(
            "/document/{doc_id}/flush",
            post(DocumentController::flush_to_gcs),
        )
        .route("/document/snapshot", post(DocumentController::create_snapshot))
}
