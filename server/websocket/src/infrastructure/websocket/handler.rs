use anyhow::Result;
use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::Response,
};
use bytes::Bytes;

use crate::application::dto::AppState;
use crate::infrastructure::websocket::connection::Connection;
use crate::infrastructure::websocket::error::WebSocketError;
use crate::infrastructure::{BroadcastGroup, BroadcastPool};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

#[repr(transparent)]
pub struct WarpConn(Connection<WarpSink, WarpStream>);

#[derive(Debug)]
pub struct WarpSink(SplitSink<WebSocket, Message>);

impl From<SplitSink<WebSocket, Message>> for WarpSink {
    fn from(sink: SplitSink<WebSocket, Message>) -> Self {
        WarpSink(sink)
    }
}

impl From<WarpSink> for SplitSink<WebSocket, Message> {
    fn from(val: WarpSink) -> Self {
        val.0
    }
}

impl futures_util::Sink<Bytes> for WarpSink {
    type Error = WebSocketError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match Pin::new(&mut self.0).poll_ready(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
            Poll::Ready(_) => Poll::Ready(Ok(())),
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: Bytes) -> Result<(), Self::Error> {
        Pin::new(&mut self.0)
            .start_send(Message::Binary(item))
            .map_err(|e| e.into())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match Pin::new(&mut self.0).poll_flush(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
            Poll::Ready(_) => Poll::Ready(Ok(())),
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match Pin::new(&mut self.0).poll_close(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
            Poll::Ready(_) => Poll::Ready(Ok(())),
        }
    }
}

#[derive(Debug)]
pub struct WarpStream(SplitStream<WebSocket>, Option<mpsc::Sender<Message>>);

impl From<SplitStream<WebSocket>> for WarpStream {
    fn from(stream: SplitStream<WebSocket>) -> Self {
        WarpStream(stream, None)
    }
}

impl From<WarpStream> for SplitStream<WebSocket> {
    fn from(val: WarpStream) -> Self {
        val.0
    }
}

impl WarpStream {
    pub fn with_pong_sender(stream: SplitStream<WebSocket>, sender: mpsc::Sender<Message>) -> Self {
        WarpStream(stream, Some(sender))
    }
}

impl Stream for WarpStream {
    type Item = Result<Bytes, WebSocketError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.0).poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(res)) => match res {
                Ok(msg) => match msg {
                    Message::Binary(data) => Poll::Ready(Some(Ok(data))),
                    Message::Ping(ping_data) => {
                        if let Some(sender) = &self.1 {
                            let pong_msg = Message::Pong(ping_data.clone());
                            let tx = sender.clone();
                            tokio::spawn(async move {
                                if let Err(e) = tx.send(pong_msg).await {
                                    warn!("Failed to send pong message: {}", e);
                                } else {
                                    info!("Pong response sent");
                                }
                            });
                        }
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    Message::Pong(_) | Message::Text(_) => {
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                    Message::Close(_) => Poll::Ready(None),
                },
                Err(e) => Poll::Ready(Some(Err(e.into()))),
            },
        }
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(doc_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let doc_id = normalize_doc_id(&doc_id);

    let bcast = match state.pool.get_group(&doc_id).await {
        Ok(group) => group,
        Err(e) => {
            error!("Failed to get or create group for {}: {}", doc_id, e);
            return Response::builder()
                .status(500)
                .body(axum::body::Body::empty())
                .unwrap();
        }
    };

    ws.on_upgrade(move |socket| handle_socket(socket, bcast, doc_id, Arc::clone(&state.pool)))
}

async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    bcast: Arc<BroadcastGroup>,
    doc_id: String,
    pool: Arc<BroadcastPool>,
) {
    let (sender, receiver) = socket.split();

    let (pong_tx, _pong_rx) = mpsc::channel::<Message>(64);

    let sink = WarpSink(sender);
    let stream = WarpStream::with_pong_sender(receiver, pong_tx);

    let conn = Connection::new(bcast.clone(), sink, stream).await;

    if let Err(e) = bcast.increment_connections().await {
        error!("Failed to increment connections: {}", e);
    }

    // tracing::info!("WebSocket connection established for document '{}'", doc_id);

    if let Err(e) = conn.await {
        error!(
            "WebSocket connection error for document '{}': {}",
            doc_id, e
        );
    }

    // tracing::info!("WebSocket connection closed for document '{}'", doc_id);

    let _ = bcast.decrement_connections().await;

    let count = bcast.connection_count();

    if count == 0 {
        if let Err(e) = pool.cleanup_empty_group(&doc_id).await {
            error!("Failed to cleanup empty group for {}: {}", doc_id, e);
        }
    }
}

fn normalize_doc_id(doc_id: &str) -> String {
    doc_id.strip_suffix(":main").unwrap_or(doc_id).to_string()
}
