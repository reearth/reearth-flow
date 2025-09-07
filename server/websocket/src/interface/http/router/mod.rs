use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::{interface::http::handlers::DocumentHandler, AppState};

pub fn document_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/document/{doc_id}", get(DocumentHandler::get_latest))
        .route(
            "/document/{doc_id}/history/metadata",
            get(DocumentHandler::get_history_metadata),
        )
        .route(
            "/document/{doc_id}/history/version/{version}",
            get(DocumentHandler::get_history_by_version),
        )
        .route(
            "/document/{doc_id}/rollback",
            post(DocumentHandler::rollback),
        )
        .route(
            "/document/{doc_id}/flush",
            post(DocumentHandler::flush_to_gcs),
        )
        .route("/document/snapshot", post(DocumentHandler::create_snapshot))
}
