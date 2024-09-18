use std::{net::SocketAddr, sync::Arc};

use super::errors::{Result, WsError};
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
use yrs::sync::{Awareness, SyncMessage};
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{ReadTxn, Transact, Update};

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
    let _ = state.join(&room_id).await;

    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match handle_message(msg, addr, &room_id, state.clone()).await {
                Ok(Some(msg)) => {
                    let _ = socket.send(Message::Binary(msg)).await;
                    continue;
                }
                Ok(_) => continue,
                Err(_) => {
                    debug!("client {addr} disconnected");
                    return;
                }
            }
        } else {
            println!("client {addr} disconnected");
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

            let _ = match msg.event {
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

            let rooms = state.rooms.try_lock().unwrap();
            let room = rooms.get(room_id).unwrap();
            let doc = room.get_doc();
            match yrs::sync::Message::decode_v1(&d) {
                Ok(yrs::sync::Message::Sync(SyncMessage::SyncStep1(sv))) => {
                    trace!("Sync");
                    let txn = doc.transact();
                    let update = txn.encode_state_as_update_v1(&sv);
                    let sync2 = SyncMessage::SyncStep2(update).encode_v1();
                    Ok(Some(sync2))
                }
                Ok(yrs::sync::Message::Sync(SyncMessage::Update(data))) => {
                    trace!("Update");
                    let mut txn = doc.transact_mut();
                    txn.apply_update(Update::decode_v1(&data).unwrap());
                    Ok(None)
                }
                Ok(yrs::sync::Message::Awareness(update)) => {
                    let mut awareness = Awareness::new((*doc).clone());
                    let _ = awareness.apply_update(update);
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
            Err(WsError::Error)
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
        let _ = state.timeout().await;
        (StatusCode::REQUEST_TIMEOUT, "timeout".to_string())
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("unhandled error: {err}"),
        )
    }
}

impl AppState {
    async fn _on_disconnect(&self) {
        unimplemented!()
    }
    async fn join(&self, room_id: &str) -> Result<()> {
        self.rooms
            .try_lock()
            .map_err(|_| WsError::Error)?
            .get_mut(room_id)
            .ok_or(WsError::Error)?
            .join("brabrabra".to_string())
            .await?;
        Ok(())
    }
    async fn leave(&self, _room_id: &str) -> Result<()> {
        unimplemented!()
    }
    async fn emit(&self, _data: &str) -> Result<()> {
        unimplemented!()
    }
    async fn timeout(&self) -> Result<()> {
        unimplemented!()
    }
}
