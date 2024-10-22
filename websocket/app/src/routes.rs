use axum::routing::{get, Router};
use std::sync::Arc;

use crate::add_middleware;
use crate::handler::handle_upgrade;
use crate::state::AppState;

fn create_base_router() -> Router<Arc<AppState>> {
    Router::new().route("/:room", get(handle_upgrade))
}

pub fn create_router(state: Arc<AppState>) -> Router {
    let base_router = create_base_router();
    let router_with_middleware = add_middleware(base_router);
    router_with_middleware.with_state(state)
}
