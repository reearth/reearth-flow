use crate::conn::Connection;
use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::Response,
};
use bytes::Bytes;

#[cfg(feature = "auth")]
use axum::extract::Query;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{Stream, StreamExt};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use yrs::sync::Error;

use crate::AppState;

#[cfg(feature = "auth")]
use crate::AuthQuery;

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
pub struct WarpStream(SplitStream<WebSocket>);

impl From<SplitStream<WebSocket>> for WarpStream {
    fn from(stream: SplitStream<WebSocket>) -> Self {
        WarpStream(stream)
    }
}

impl From<WarpStream> for SplitStream<WebSocket> {
    fn from(val: WarpStream) -> Self {
        val.0
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
                    Message::Ping(_) | Message::Pong(_) | Message::Text(_) => {
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
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
                tracing::debug!("Token verified successfully");
            }
            Ok(false) => {
                tracing::error!("Token verification failed");
                return Response::builder()
                    .status(401)
                    .body(axum::body::Body::empty())
                    .unwrap();
            }
            Err(e) => {
                tracing::error!("Token verification error: {}", e);
                return Response::builder()
                    .status(500)
                    .body(axum::body::Body::empty())
                    .unwrap();
            }
        }
    }

    let bcast = match state.pool.get_group(&doc_id).await {
        Ok(group) => group,
        Err(e) => {
            tracing::error!("Failed to get or create group for {}: {}", doc_id, e);
            return Response::builder()
                .status(500)
                .body(axum::body::Body::empty())
                .unwrap();
        }
    };

    ws.on_upgrade(move |socket| handle_socket(socket, bcast, doc_id, user_token))
}

async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    bcast: Arc<crate::BroadcastGroup>,
    doc_id: String,
    user_token: Option<String>,
) {
    let (sender, receiver) = socket.split();

    let conn = crate::conn::Connection::new(
        bcast.clone(),
        WarpSink(sender),
        WarpStream(receiver),
        user_token,
    )
    .await;

    let result = conn.await;
    if let Err(e) = result {
        tracing::error!("WebSocket connection error: {}", e);
    }

    let _ = bcast.decrement_connections().await;
    tracing::debug!("Connection decreased for document '{}'", doc_id);
}

fn normalize_doc_id(doc_id: &str) -> String {
    doc_id.strip_suffix(":main").unwrap_or(doc_id).to_string()
}
