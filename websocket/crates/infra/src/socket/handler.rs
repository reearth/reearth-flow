use std::{net::SocketAddr, sync::Arc};

use super::errors::{Result, WsError};
use super::state::AppState;
use axum::http::{Method, StatusCode, Uri};
use axum::BoxError;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tag", content = "content")]
enum Event {
    Join { room_id: String },
    Leave,
    Emit { event: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct FlowMessage {
    event: Event,
}

pub async fn handle_upgrade(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

async fn handle_socket(mut socket: WebSocket, addr: SocketAddr, state: Arc<AppState>) {
    if socket.send(Message::Ping(vec![4])).await.is_ok() {
        println!("pinned to {addr}");
    } else {
        println!("couldn't ping to {addr}");
        return;
    }

    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            if handle_message(msg, addr, state).await.is_err() {
                return;
            }
        } else {
            println!("client {addr} disconnected");
            return;
        }
    }
}

async fn handle_message(msg: Message, addr: SocketAddr, state: Arc<AppState>) -> Result<()> {
    match msg {
        Message::Text(t) => {
            let msg: FlowMessage = serde_json::from_str(&t).unwrap();

            match msg.event {
                Event::Join { room_id } => state
                    .rooms
                    .try_lock()
                    .or_else(|_| Err(WsError::WsError))?
                    .get_mut(&room_id)
                    .ok_or(WsError::WsError)?
                    .join("brabrabra".to_string()),
                Event::Leave => unimplemented!(),
                Event::Emit { event } => unimplemented!(),
            }
        }
        Message::Binary(d) => {
            println!(">>> {} sent {} bytes: {:?}", addr, d.len(), d);
            Ok(())
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                println!(
                    ">>> {} sent close with code {} and reason `{}`",
                    addr, cf.code, cf.reason
                );
            } else {
                println!(">>> {addr} somehow sent close message without CloseFrame");
            }
            Err(WsError::WsError)
        }
        // reply to ping automatically
        _ => Ok(()),
    }
}

pub async fn handle_error(_method: Method, _uri: Uri, err: BoxError) -> (StatusCode, String) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (StatusCode::REQUEST_TIMEOUT, "timeout".to_string())
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("unhandled error: {err}"),
        )
    }
}
