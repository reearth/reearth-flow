use std::{net::SocketAddr, sync::Arc};

use super::errors::{Result, WsError};
use super::room::Room;
use super::state::AppState;
use axum::extract::{Path, Query};
use axum::http::{Method, StatusCode, Uri};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tower::BoxError;
use tracing::{debug, trace};
use yrs::updates::decoder::Decode;
use yrs::{ReadTxn, StateVector, Transact};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tag", content = "content")]
enum Event {
    Join { room_id: String },
    Leave,
    Emit { data: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct FlowMessage {
    event: Event,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WebSocketQuery {
    token: String,
}

pub async fn handle_upgrade(
    ws: WebSocketUpgrade,
    query: Query<WebSocketQuery>,
    Path(room_id): Path<String>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    debug!("{:?}", query);
    ws.on_upgrade(move |socket| {
        handle_socket(socket, addr, query.token.to_string(), room_id, state)
    })
}

async fn handle_socket(
    mut socket: WebSocket,
    addr: SocketAddr,
    token: String,
    room_id: String,
    state: Arc<AppState>,
) {
    if socket.send(Message::Ping(vec![4])).await.is_ok() {
        println!("pinned to {addr}");
    } else {
        println!("couldn't ping to {addr}");
        return;
    }

    // TODO: authentication
    if token != "nyaan" {
        return;
    }

    debug!("{:?}", state.make_room(room_id.clone()));
    state.join(&room_id).await;

    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match handle_message(msg, addr, &room_id, state.clone()).await {
                Ok(Some(msg)) => {
                    socket.send(Message::Binary(msg)).await;
                    continue;
                }
                Ok(_) => continue,
                Err(_) => return,
            }
        } else {
            println!("client {addr} disconnected");
            return;
        }
    }
}

async fn handle_message(
    msg: Message,
    addr: SocketAddr,
    room_id: &str,
    state: Arc<AppState>,
) -> Result<Option<Vec<u8>>> {
    match msg {
        Message::Text(t) => {
            let msg: FlowMessage = serde_json::from_str(&t).unwrap();

            match msg.event {
                Event::Join { room_id } => state.join(&room_id).await,
                Event::Leave => state.leave("brabra").await,
                Event::Emit { data } => state.emit(&data).await,
            };
            Ok(None)
        }
        Message::Binary(d) => {
            trace!("{} sent {} bytes: {:?}", addr, d.len(), d);
            if d.len() < 3 {
                return Ok(None);
            };
            match (d[0], d[1]) {
                (0, 0) => {
                    let rest = d[2..].to_vec();
                    trace!("SyncStep1");
                    let rooms = state.rooms.try_lock().unwrap();
                    let room = rooms.get(room_id).unwrap();
                    let doc = room.get_doc();
                    let txn = doc.transact();
                    let diff = txn.encode_diff_v1(&StateVector::decode_v1(&d).unwrap());

                    Ok(Some(diff))
                }
                (0, 1) => {
                    let rest = d[2..].to_vec();
                    trace!("SyncStep2");
                    Ok(None)
                }
                (0, 2) => {
                    let rest = d[2..].to_vec();
                    trace!("update");
                    Ok(None)
                }
                (1, _) => {
                    let rest = d[1..].to_vec();
                    trace!("awareness");
                    Ok(None)
                }
                _ => Ok(None),
            }
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
        _ => Ok(None),
    }
}

pub async fn handle_error(
    _method: Method,
    _uri: Uri,
    err: BoxError,
    state: Arc<AppState>,
) -> impl IntoResponse {
    if err.is::<tower::timeout::error::Elapsed>() {
        state.timeout();
        (StatusCode::REQUEST_TIMEOUT, "timeout".to_string())
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("unhandled error: {err}"),
        )
    }
}

impl AppState {
    async fn on_disconnect(&self) {
        unimplemented!()
    }
    async fn join(&self, room_id: &str) -> Result<()> {
        self.rooms
            .try_lock()
            .or_else(|_| Err(WsError::WsError))?
            .get_mut(room_id)
            .ok_or(WsError::WsError)?
            .join("brabrabra".to_string());
        Ok(())
    }
    async fn leave(&self, room_id: &str) -> Result<()> {
        unimplemented!()
    }
    async fn emit(&self, data: &str) -> Result<()> {
        unimplemented!()
    }
    async fn timeout(&self) -> Result<()> {
        unimplemented!()
    }
}
