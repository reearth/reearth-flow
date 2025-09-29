use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use anyhow::Error as AnyError;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::time;
use tracing::{debug, error, info, warn};
use yrs::sync::Error as YSyncError;

use crate::domain::repository::broadcast_pool::{BroadcastGroupHandle, BroadcastGroupProvider};
use crate::infrastructure::websocket::types::Subscription;

const CONNECTION_TIMEOUT: Duration = Duration::from_secs(86_400);

#[derive(Debug, Error)]
pub enum WebsocketServiceError {
    #[error("failed to access broadcast group for '{doc_id}'")]
    BroadcastGroup {
        doc_id: String,
        #[source]
        source: AnyError,
    },
    #[error("websocket connection error for '{doc_id}'")]
    Connection {
        doc_id: String,
        #[source]
        source: YSyncError,
    },
}

#[derive(Clone, Debug)]
pub struct WebsocketService<P>
where
    P: BroadcastGroupProvider + 'static,
{
    pool: Arc<P>,
}

impl<P> WebsocketService<P>
where
    P: BroadcastGroupProvider + 'static,
{
    pub fn new(pool: Arc<P>) -> Self {
        Self { pool }
    }

    pub async fn get_group(&self, doc_id: &str) -> Result<Arc<P::Group>, WebsocketServiceError> {
        self.pool
            .get_group(doc_id)
            .await
            .map_err(|err| WebsocketServiceError::BroadcastGroup {
                doc_id: doc_id.to_string(),
                source: err,
            })
    }

    pub async fn handle_connection<Sink, Stream, E>(
        &self,
        group: Arc<P::Group>,
        sink: Sink,
        stream: Stream,
        doc_id: &str,
        user_token: Option<String>,
    ) -> Result<(), WebsocketServiceError>
    where
        Sink: SinkExt<Bytes, Error = E> + Send + Sync + Unpin + 'static,
        Stream: StreamExt<Item = Result<Bytes, E>> + Send + Sync + Unpin + 'static,
        E: std::error::Error + Into<YSyncError> + Send + Sync + 'static,
    {
        group.increment_connections_count().await;

        let connection = BroadcastConnection::new(group.clone(), sink, stream, user_token).await;
        let doc_id_owned = doc_id.to_string();

        let connection_result = tokio::select! {
            result = connection => result,
            _ = time::sleep(CONNECTION_TIMEOUT) => {
                warn!("Connection timeout for document '{}' - possible stale connection", doc_id_owned);
                Err(YSyncError::Other("Connection timeout".into()))
            }
        };

        group.decrement_connections_count().await;

        let active_connections = group.get_connections_count().await;
        info!(
            "Active connections for document '{}': {}",
            doc_id_owned, active_connections
        );

        if active_connections == 0 {
            let pool = Arc::clone(&self.pool);
            let cleanup_doc_id = doc_id_owned.clone();
            tokio::spawn(async move {
                pool.cleanup_group(&cleanup_doc_id).await;
                info!("Cleaned up BroadcastGroup for doc_id: {}", cleanup_doc_id);
            });
        }

        match connection_result {
            Ok(_) => Ok(()),
            Err(err) => {
                error!(
                    "WebSocket connection error for document '{}': {}",
                    doc_id_owned, err
                );
                Err(WebsocketServiceError::Connection {
                    doc_id: doc_id_owned,
                    source: err,
                })
            }
        }
    }
}

type CompletionFuture = Pin<Box<dyn Future<Output = Result<(), YSyncError>> + Send>>;

struct BroadcastConnection<G, Sink, Stream>
where
    G: BroadcastGroupHandle + ?Sized + 'static,
{
    broadcast_sub: Option<Subscription>,
    completion_future: Option<CompletionFuture>,
    _user_token: Option<String>,
    broadcast_group: Option<Arc<G>>,
    sink: PhantomData<Sink>,
    stream: PhantomData<Stream>,
}

impl<G, Sink, Stream, E> BroadcastConnection<G, Sink, Stream>
where
    G: BroadcastGroupHandle + ?Sized + 'static,
    Sink: SinkExt<Bytes, Error = E> + Send + Sync + Unpin + 'static,
    Stream: StreamExt<Item = Result<Bytes, E>> + Send + Sync + Unpin + 'static,
    E: std::error::Error + Into<YSyncError> + Send + Sync + 'static,
{
    async fn new(
        broadcast_group: Arc<G>,
        sink: Sink,
        stream: Stream,
        user_token: Option<String>,
    ) -> Self {
        let sink = Arc::new(Mutex::new(sink));
        let group_clone = Arc::clone(&broadcast_group);

        let broadcast_sub = broadcast_group.subscribe(Arc::clone(&sink), stream).await;

        BroadcastConnection {
            broadcast_sub: Some(broadcast_sub),
            completion_future: None,
            _user_token: user_token,
            broadcast_group: Some(group_clone),
            sink: PhantomData,
            stream: PhantomData,
        }
    }
}

impl<G, Sink, Stream, E> Future for BroadcastConnection<G, Sink, Stream>
where
    G: BroadcastGroupHandle + ?Sized + 'static,
    Sink: SinkExt<Bytes, Error = E> + Send + Sync + Unpin + 'static,
    Stream: StreamExt<Item = Result<Bytes, E>> + Send + Sync + Unpin + 'static,
    E: std::error::Error + Into<YSyncError> + Send + Sync + 'static,
{
    type Output = Result<(), YSyncError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.completion_future.is_none() {
            if let Some(broadcast_sub) = self.as_mut().get_mut().broadcast_sub.take() {
                self.completion_future = Some(Box::pin(broadcast_sub.completed()));
            }
        }

        if let Some(fut) = self.completion_future.as_mut() {
            let poll_result = fut.as_mut().poll(cx);
            match &poll_result {
                Poll::Ready(result) => {
                    debug!("Connection future completed with result: {:?}", result);
                }
                Poll::Pending => {
                    debug!("Connection future is pending");
                }
            }
            poll_result
        } else {
            Poll::Ready(Ok(()))
        }
    }
}

impl<G, Sink, Stream> Drop for BroadcastConnection<G, Sink, Stream>
where
    G: BroadcastGroupHandle + ?Sized + 'static,
{
    fn drop(&mut self) {
        if let Some(group) = self.broadcast_group.take() {
            let group_clone = Arc::clone(&group);
            tokio::spawn(async move {
                if let Err(e) = group_clone.cleanup_client_awareness().await {
                    error!("Failed to cleanup awareness: {}", e);
                }
            });
        }
    }
}
