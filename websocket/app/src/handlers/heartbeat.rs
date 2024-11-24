use axum::extract::ws::{Message, WebSocket};
use futures_util::stream::SplitSink;
use futures_util::SinkExt;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tracing::debug;

pub fn start_heartbeat(
    sender: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    addr: std::net::SocketAddr,
    cleanup: impl Fn() + Send + Clone + 'static,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            if sender
                .lock()
                .await
                .send(Message::Ping(vec![1]))
                .await
                .is_err()
            {
                debug!("Ping failed for client {addr}");
                cleanup();
                break;
            }
        }
    })
}
