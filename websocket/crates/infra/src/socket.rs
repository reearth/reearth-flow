use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use errors::WsError;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

mod errors;

struct Room {
    users: Mutex<HashSet<String>>,
    tx: broadcast::Sender<String>,
}

impl Room {
    fn new() -> Self {
        Room {
            users: Mutex::new(HashSet::new()),
            tx: broadcast::Sender::new(100),
        }
    }

    fn join(&mut self, user_id: String) -> Result<(), WsError> {
        self.users
            .try_lock()
            .or_else(|_| Err(WsError::WsError))?
            .insert(user_id);
        Ok(())
    }

    fn leave(&mut self, user_id: String) -> Result<(), WsError> {
        self.users
            .try_lock()
            .or_else(|_| Err(WsError::WsError))?
            .remove(&user_id);
        Ok(())
    }

    fn broadcast(&mut self, msg: String) -> Result<(), WsError> {
        self.tx.send(msg).or_else(|_| Err(WsError::WsError))?;
        Ok(())
    }
}

pub struct AppState {
    rooms: Mutex<HashMap<String, Room>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            rooms: Mutex::new(HashMap::new()),
        }
    }

    fn make_room(&mut self) -> Result<String, WsError> {
        let id = uuid::Uuid::new_v4().to_string();
        let room = Room::new();
        self.rooms
            .try_lock()
            .or_else(|_| Err(WsError::WsError))?
            .insert(id.clone(), room);
        Ok(id)
    }

    fn delete_room(&mut self, id: String) -> Result<(), WsError> {
        self.rooms
            .try_lock()
            .or_else(|_| Err(WsError::WsError))?
            .remove(&id);
        Ok(())
    }
}

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

pub async fn ws_handler(
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

async fn handle_message(
    msg: Message,
    addr: SocketAddr,
    state: Arc<AppState>,
) -> Result<(), WsError> {
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
        _ => Ok(()),
    }
}
