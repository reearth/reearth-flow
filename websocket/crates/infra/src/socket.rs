use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    ops::ControlFlow,
    sync::Mutex,
};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use tokio::sync::broadcast;

struct Room {
    users: Mutex<HashSet<String>>,
    tx: broadcast::Sender<String>,
}

struct AppState {
    rooms: HashMap<String, Room>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket(mut socket: WebSocket, addr: SocketAddr) {
    if socket.send(Message::Ping(vec![4])).await.is_ok() {
        println!("pinned to {addr}");
    } else {
        println!("couldn't ping to {addr}");
        return;
    }

    if let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            if handle_message(msg, addr).await.is_break() {
                return;
            }
        } else {
            println!("client {addr} disconnected");
            return;
        }
    }
}

async fn handle_message(msg: Message, addr: SocketAddr) -> ControlFlow<(), ()> {
    match msg {
        Message::Text(t) => {
            println!(">>> {addr} sent str: {t:?}");
        }
        Message::Binary(d) => {
            println!(">>> {} sent {} bytes: {:?}", addr, d.len(), d);
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
            return ControlFlow::Break(());
        }
        _ => (),
    }
    ControlFlow::Continue(())
}
