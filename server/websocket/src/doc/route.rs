use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use crate::doc::handler::DocumentHandler;
use crate::AppState;

pub fn document_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/document/:doc_id", get(DocumentHandler::get_latest))
        .route(
            "/document/:doc_id/history",
            get(DocumentHandler::get_history),
        )
        .route(
            "/document/:doc_id/rollback",
            post(DocumentHandler::rollback),
        )
}
