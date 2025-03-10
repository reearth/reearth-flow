use crate::broadcast::group::BroadcastGroup;
use crate::conn::Connection;
use axum::extract::ws::{Message, WebSocket};
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    response::Response,
};

#[cfg(feature = "auth")]
use axum::extract::Query;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{Stream, StreamExt};
use redis::AsyncCommands;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use yrs::sync::Error;

use crate::{pool::BroadcastPool, AppState};

#[cfg(feature = "auth")]
use crate::AuthQuery;

/// Connection Wrapper over a [WebSocket], which implements a Yjs/Yrs awareness and update exchange
/// protocol.
#[repr(transparent)]
pub struct WarpConn(Connection<WarpSink, WarpStream>);

impl WarpConn {
    pub fn new(broadcast_group: Arc<BroadcastGroup>, socket: WebSocket) -> Self {
        let (sink, stream) = socket.split();
        let conn = Connection::new(broadcast_group, WarpSink(sink), WarpStream(stream));
        WarpConn(conn)
    }
}

impl core::future::Future for WarpConn {
    type Output = Result<(), Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}

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

impl futures_util::Sink<Vec<u8>> for WarpSink {
    type Error = Error;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match Pin::new(&mut self.0).poll_ready(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) => Poll::Ready(Err(Error::Other(e.into()))),
            Poll::Ready(_) => Poll::Ready(Ok(())),
        }
    }

    fn start_send(mut self: Pin<&mut Self>, item: Vec<u8>) -> Result<(), Self::Error> {
        if let Err(e) = Pin::new(&mut self.0).start_send(Message::Binary(item.into())) {
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
    type Item = Result<Vec<u8>, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match Pin::new(&mut self.0).poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Ready(Some(res)) => match res {
                Ok(msg) => match msg {
                    Message::Binary(data) => Poll::Ready(Some(Ok(data.to_vec()))),
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

    // Verify token
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

    let mut redis_conn_for_lock = None;
    let lock_key = format!("lock:workflow:{}", doc_id);
    let mut lock_acquired = false;

    if let Some(redis_config) = state.pool.get_redis_config() {
        let redis_key = format!("doc_id:{}", doc_id);

        if let Ok(manager) = redis::Client::open(redis_config.url.clone()) {
            if let Ok(mut conn) = manager.get_multiplexed_async_connection().await {
                match conn.set_nx::<_, _, bool>(&lock_key, "1").await {
                    Ok(true) => {
                        lock_acquired = true;
                        tracing::debug!("Acquired lock for workflow '{}' creation", doc_id);

                        if let Err(e) = conn.expire::<_, bool>(&lock_key, 300).await {
                            tracing::warn!(
                                "Failed to set expiration on lock for workflow '{}': {}",
                                doc_id,
                                e
                            );
                        }

                        if let Err(e) = conn
                            .set_ex::<_, _, ()>(&redis_key, "1", redis_config.ttl)
                            .await
                        {
                            tracing::warn!("Failed to store doc_id in Redis: {}", e);
                        }

                        redis_conn_for_lock = Some(conn);
                    }
                    Ok(false) => {
                        let mut retries = 0;
                        const MAX_LOCK_RETRIES: usize = 30;
                        const LOCK_RETRY_DELAY_MS: u64 = 500;

                        tracing::info!(
                            "Waiting for lock on workflow '{}' to be released...",
                            doc_id
                        );

                        while retries < MAX_LOCK_RETRIES {
                            match conn.exists::<_, bool>(&lock_key).await {
                                Ok(false) => {
                                    match conn.set_nx::<_, _, bool>(&lock_key, "1").await {
                                        Ok(true) => {
                                            lock_acquired = true;
                                            tracing::debug!(
                                                "Acquired lock for workflow '{}' after waiting",
                                                doc_id
                                            );

                                            if let Err(e) =
                                                conn.expire::<_, bool>(&lock_key, 300).await
                                            {
                                                tracing::warn!("Failed to set expiration on lock for workflow '{}': {}", doc_id, e);
                                            }

                                            if let Err(e) = conn
                                                .set_ex::<_, _, ()>(
                                                    &redis_key,
                                                    "1",
                                                    redis_config.ttl,
                                                )
                                                .await
                                            {
                                                tracing::warn!(
                                                    "Failed to store doc_id in Redis: {}",
                                                    e
                                                );
                                            }

                                            redis_conn_for_lock = Some(conn);
                                            break;
                                        }
                                        Ok(false) => {
                                            tracing::debug!("Lock for workflow '{}' still held by another process, retrying...", doc_id);
                                        }
                                        Err(e) => {
                                            tracing::warn!(
                                                "Failed to acquire lock for workflow '{}': {}",
                                                doc_id,
                                                e
                                            );
                                            break;
                                        }
                                    }
                                }
                                Ok(true) => {
                                    tracing::debug!(
                                        "Waiting for lock on workflow '{}' to be released...",
                                        doc_id
                                    );
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Failed to check lock status for workflow '{}': {}",
                                        doc_id,
                                        e
                                    );
                                    break;
                                }
                            }

                            tokio::time::sleep(tokio::time::Duration::from_millis(
                                LOCK_RETRY_DELAY_MS,
                            ))
                            .await;
                            retries += 1;
                        }

                        if retries == MAX_LOCK_RETRIES {
                            tracing::warn!(
                                "Timed out waiting for lock on workflow '{}', proceeding anyway",
                                doc_id
                            );
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to acquire lock for workflow '{}': {}", doc_id, e);
                    }
                }
            } else {
                tracing::warn!("Failed to get Redis connection");
            }
        } else {
            tracing::warn!("Failed to open Redis client");
        }
    }

    let bcast = match state.pool.get_or_create_group(&doc_id).await {
        Ok(group) => group,
        Err(e) => {
            if lock_acquired {
                if let Some(mut conn) = redis_conn_for_lock {
                    if let Err(e) = conn.del::<_, ()>(&lock_key).await {
                        tracing::warn!("Failed to release lock for workflow '{}': {}", doc_id, e);
                    } else {
                        tracing::debug!("Released lock for workflow '{}' after failure", doc_id);
                    }
                }
            }

            tracing::error!("Failed to get or create group for {}: {}", doc_id, e);
            return Response::builder()
                .status(500)
                .body(axum::body::Body::empty())
                .unwrap();
        }
    };

    let lock_key_clone = lock_key.clone();
    let redis_conn_clone = redis_conn_for_lock;
    let doc_id_clone = doc_id.clone();
    let lock_acquired_clone = lock_acquired;

    ws.on_upgrade(move |socket| {
        let release_lock_future = async move {
            if lock_acquired_clone {
                if let Some(mut conn) = redis_conn_clone {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                    if let Err(e) = conn.del::<_, ()>(&lock_key_clone).await {
                        tracing::warn!(
                            "Failed to release lock for workflow '{}': {}",
                            doc_id_clone,
                            e
                        );
                    } else {
                        tracing::debug!(
                            "Released lock for workflow '{}' after connection established",
                            doc_id_clone
                        );
                    }
                }
            }
        };

        async move {
            release_lock_future.await;

            handle_socket(socket, bcast, doc_id, state.pool.clone(), user_token).await;
        }
    })
}

async fn handle_socket(
    socket: axum::extract::ws::WebSocket,
    bcast: Arc<crate::BroadcastGroup>,
    doc_id: String,
    pool: Arc<BroadcastPool>,
    user_token: Option<String>,
) {
    bcast.increment_connections();

    let (sender, receiver) = socket.split();

    let conn = crate::conn::Connection::with_broadcast_group_and_user(
        bcast.clone(),
        WarpSink(sender),
        WarpStream(receiver),
        user_token,
    )
    .await;

    if let Err(e) = conn.await {
        tracing::error!("WebSocket connection error: {}", e);
    }
    pool.remove_connection(&doc_id).await;
}

fn normalize_doc_id(doc_id: &str) -> String {
    doc_id.strip_suffix(":main").unwrap_or(doc_id).to_string()
}
