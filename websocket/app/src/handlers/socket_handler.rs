use crate::handlers::message_handler::handle_message;
use crate::state::AppState;
use axum::extract::ws::WebSocket;
use flow_websocket_infra::types::user::User;
use futures_util::StreamExt;
use std::sync::atomic::AtomicBool;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use tracing::debug;

use super::cleanup::perform_cleanup;
use super::heartbeat::start_heartbeat;

pub async fn handle_socket(
    socket: WebSocket,
    addr: SocketAddr,
    room_id: String,
    state: Arc<AppState>,
    project_id: Option<String>,
    user: User,
) {
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));
    let is_cleaning_up = Arc::new(AtomicBool::new(false));
    let (cleanup_tx, mut cleanup_rx) = mpsc::channel(1);

    let cleanup = {
        let is_cleaning_up = is_cleaning_up.clone();
        let room_id = room_id.clone();
        let user = user.clone();
        let project_id = project_id.clone();
        let state = state.clone();
        let cleanup_tx = cleanup_tx.clone();

        Arc::new(move || {
            let _ = perform_cleanup(
                is_cleaning_up.clone(),
                room_id.clone(),
                user.clone(),
                project_id.clone(),
                state.clone(),
                cleanup_tx.clone(),
            );
        }) as Arc<dyn Fn() + Send + Sync>
    };

    let heartbeat_task = start_heartbeat(sender, addr, move || cleanup());

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(msg) => {
                if let Err(e) = handle_message(
                    msg,
                    addr,
                    &room_id,
                    project_id.clone(),
                    state.clone(),
                    user.clone(),
                )
                .await
                {
                    debug!("Error handling message: {:?}", e);
                    continue;
                }
            }
            Err(e) => {
                debug!("Error receiving message: {:?}", e);
                debug!("Message from {addr} is not ok");
                continue;
            }
        }
    }

    let _ = cleanup_rx.recv().await;
    heartbeat_task.abort();
}
