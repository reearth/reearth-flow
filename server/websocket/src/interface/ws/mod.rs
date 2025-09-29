use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::Response,
};
use bytes::Bytes;

#[cfg(feature = "auth")]
use axum::extract::Query;

use crate::{AppState, WebsocketServiceError};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::mpsc;
use tracing::{debug, error, warn};
use yrs::sync::Error;

#[cfg(feature = "auth")]
use crate::AuthQuery;

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
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match Pin::new(&mut self.0).poll_ready(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(Error::Other(e.into()))),
            Poll::Ready(_) => Poll::Ready(Ok(())),
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: Bytes) -> Result<(), Self::Error> {
        if let Err(e) = Pin::new(&mut self.0).start_send(Message::Binary(item)) {
            Err(Error::Other(e.into()))
        } else {
            Ok(())
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match Pin::new(&mut self.0).poll_flush(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(Error::Other(e.into()))),
            Poll::Ready(_) => Poll::Ready(Ok(())),
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match Pin::new(&mut self.0).poll_close(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(Error::Other(e.into()))),
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
    type Item = Result<Bytes, Error>;

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
                                    debug!("Pong response sent");
                                }
                            });
                        }
                        self.poll_next(cx)
                    }
                    Message::Pong(_) | Message::Text(_) => self.poll_next(cx),
                    Message::Close(_) => Poll::Ready(None),
                },
                Err(e) => Poll::Ready(Some(Err(Error::Other(e.into())))),
            },
        }
    }
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(doc_id): Path<String>,
    #[cfg(feature = "auth")] Query(query): Query<AuthQuery>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let doc_id = normalize_doc_id(&doc_id);

    #[cfg(feature = "auth")]
    let user_token = Some(query.token.clone());

    #[cfg(not(feature = "auth"))]
    let user_token: Option<String> = None;

    #[cfg(feature = "auth")]
    {
        let authorized = state.auth.verify_token(&query.token).await;
        match authorized {
            Ok(true) => {
                debug!("Token verified successfully");
            }
            Ok(false) => {
                error!("Token verification failed");
                return Response::builder()
                    .status(401)
                    .body(axum::body::Body::empty())
                    .unwrap();
            }
            Err(e) => {
                error!("Token verification error: {}", e);
                return Response::builder()
                    .status(500)
                    .body(axum::body::Body::empty())
                    .unwrap();
            }
        }
    }

    let websocket_service = state.websocket_service.clone();

    let group = match websocket_service.get_group(&doc_id).await {
        Ok(group) => group,
        Err(err) => {
            error!("Failed to get or create group for {}: {}", doc_id, err);
            return Response::builder()
                .status(500)
                .body(axum::body::Body::empty())
                .unwrap();
        }
    };

    let user_token_clone = user_token.clone();

    ws.on_upgrade(move |socket| {
        let websocket_service = websocket_service.clone();
        let group = group.clone();
        let doc_id = doc_id.clone();
        let user_token = user_token_clone.clone();

        async move {
            let (sender, receiver) = socket.split();
            let (pong_tx, _pong_rx) = mpsc::channel::<Message>(64);

            let sink = WarpSink::from(sender);
            let stream = WarpStream::with_pong_sender(receiver, pong_tx);

            if let Err(err) = websocket_service
                .handle_connection(group, sink, stream, &doc_id, user_token)
                .await
            {
                match err {
                    WebsocketServiceError::BroadcastGroup { source, .. } => {
                        error!(
                            "Broadcast group error during connection for '{}': {}",
                            doc_id, source
                        );
                    }
                    WebsocketServiceError::Connection { source, .. } => {
                        error!("WebSocket connection error for '{}': {}", doc_id, source);
                    }
                }
            }
        }
    })
}

fn normalize_doc_id(doc_id: &str) -> String {
    doc_id.strip_suffix(":main").unwrap_or(doc_id).to_string()
}
