mod cleanup;
mod heartbeat;
mod message_handler;
mod room_handler;
mod socket_handler;
mod types;

use crate::handlers::socket_handler::handle_socket;
use crate::state::AppState;
use axum::extract::{ws::WebSocketUpgrade, ConnectInfo, Path, Query, State};
use axum::response::IntoResponse;
use axum::response::Response;
use flow_websocket_infra::types::user::User;
use std::{net::SocketAddr, sync::Arc};
use types::WebSocketQuery;

pub async fn handle_upgrade(
    ws: WebSocketUpgrade,
    query: Query<WebSocketQuery>,
    Path(room_id): Path<String>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let user = User::new(query.user_id().to_string(), None, None);
    ws.on_upgrade(move |socket| {
        handle_socket(socket, addr, room_id, state, query.project_id(), user)
    })
    .into_response()
}
