use crate::handlers::message_handler::handle_message;
use crate::state::AppState;
use axum::extract::ws::{Message, WebSocket};
use flow_websocket_infra::types::user::User;
use futures::SinkExt;
use futures_util::stream::SplitSink;
use futures_util::StreamExt;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::{mpsc, Mutex};
use tracing::debug;

use super::cleanup::perform_cleanup;
use super::heartbeat::start_heartbeat;

pub struct ConnectionState {
    pub sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    pub is_cleaning_up: Arc<AtomicBool>,
    pub cleanup_tx: mpsc::Sender<()>,
    pub current_project_id: Arc<Mutex<Option<String>>>,
}

impl ConnectionState {
    pub async fn send_message(&self, message: Message) -> Result<(), axum::Error> {
        let mut sender = self.sender.lock().await;
        sender.send(message).await?;
        Ok(())
    }

    pub fn start_cleanup(&self) {
        if self
            .is_cleaning_up
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            let _ = self.cleanup_tx.try_send(());
        }
    }
}

pub async fn handle_socket(
    mut socket: WebSocket,
    addr: SocketAddr,
    token: String,
    room_id: String,
    state: Arc<AppState>,
    user: User,
) {
    if !verify_connection(&mut socket, &addr, &token).await {
        return;
    }

    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(Mutex::new(sender));
    let is_cleaning_up = Arc::new(AtomicBool::new(false));
    let (cleanup_tx, mut cleanup_rx) = mpsc::channel(1);
    let current_project_id = Arc::new(Mutex::new(None));

    let conn_state = ConnectionState {
        sender: sender.clone(),
        is_cleaning_up: is_cleaning_up.clone(),
        cleanup_tx: cleanup_tx.clone(),
        current_project_id: current_project_id.clone(),
    };

    let cleanup = {
        let is_cleaning_up = is_cleaning_up.clone();
        let room_id = room_id.clone();
        let user = user.clone();
        let state = state.clone();
        let cleanup_tx = cleanup_tx.clone();
        let current_project_id = current_project_id.clone();

        move || {
            let is_cleaning_up = is_cleaning_up.clone();
            let room_id = room_id.clone();
            let user = user.clone();
            let state = state.clone();
            let cleanup_tx = cleanup_tx.clone();
            let current_project_id = current_project_id.clone();

            tokio::spawn(async move {
                let mut project_id_lock = current_project_id.lock().await;
                let project_id = project_id_lock.take();
                let cleanup_fn =
                    perform_cleanup(is_cleaning_up, room_id, user, project_id, state, cleanup_tx);
                cleanup_fn();
            });
        }
    };

    let heartbeat_task = start_heartbeat(sender, addr, cleanup);

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(msg) => {
                if let Err(e) = handle_message(
                    msg,
                    addr,
                    &room_id,
                    &conn_state,
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

async fn verify_connection(socket: &mut WebSocket, addr: &SocketAddr, token: &str) -> bool {
    if socket.send(Message::Ping(vec![4])).await.is_err() || token != "nyaan" {
        debug!("Connection failed for {addr}: ping failed or invalid token");
        return false;
    }
    true
}
