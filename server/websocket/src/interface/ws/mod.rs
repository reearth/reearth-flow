use crate::application::dto::AppState;
use crate::infrastructure::websocket::ws_handler;
use axum::{routing::get, Router};
use std::sync::Arc;

pub fn create_ws_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/{doc_id}", get(ws_handler))
        .with_state(state)
}
