#![allow(dead_code)]
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::Mutex;
use tracing::{debug, error};
use yrs::sync::Error;

use crate::domain::value_objects::sub::Subscription;
use crate::BroadcastGroup;

type CompletionFuture = Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>;

pub struct Connection<Sink, Stream> {
    broadcast_sub: Option<Subscription>,
    completion_future: Option<CompletionFuture>,
    user_token: Option<String>,
    broadcast_group: Option<Arc<BroadcastGroup>>,
    sink: PhantomData<Sink>,
    stream: PhantomData<Stream>,
}

impl<Sink, Stream, E> Connection<Sink, Stream>
where
    Sink: SinkExt<Bytes, Error = E> + Send + Sync + Unpin + 'static,
    Stream: StreamExt<Item = Result<Bytes, E>> + Send + Sync + Unpin + 'static,
    E: std::error::Error + Into<Error> + Send + Sync + 'static,
{
    pub async fn new(
        broadcast_group: Arc<BroadcastGroup>,
        sink: Sink,
        stream: Stream,
        user_token: Option<String>,
    ) -> Self {
        let sink = Arc::new(Mutex::new(sink));
        let group_clone = broadcast_group.clone();

        let broadcast_sub = broadcast_group.subscribe(sink.clone(), stream).await;

        Connection {
            broadcast_sub: Some(broadcast_sub),
            completion_future: None,
            user_token,
            broadcast_group: Some(group_clone),
            sink: PhantomData,
            stream: PhantomData,
        }
    }
}

impl<Sink, Stream> Future for Connection<Sink, Stream> {
    type Output = Result<(), Error>;

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

impl<Sink, Stream> Unpin for Connection<Sink, Stream> {}

impl<Sink, Stream> Drop for Connection<Sink, Stream> {
    fn drop(&mut self) {
        if let Some(group) = self.broadcast_group.take() {
            let group_clone = group.clone();
            tokio::spawn(async move {
                if let Err(e) = group_clone.cleanup_client_awareness().await {
                    error!("Failed to cleanup awareness: {}", e);
                }
            });
        }
    }
}
