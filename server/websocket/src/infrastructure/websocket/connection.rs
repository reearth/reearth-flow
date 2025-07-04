#![allow(dead_code)]
use anyhow::Result;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::Mutex;

use crate::infrastructure::broadcast::group::BroadcastGroup;
use crate::infrastructure::broadcast::sub::Subscription;

type CompletionFuture = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

pub struct Connection<Sink, Stream> {
    broadcast_sub: Option<Subscription>,
    completion_future: Option<CompletionFuture>,
    sink: PhantomData<Sink>,
    stream: PhantomData<Stream>,
}

impl<Sink, Stream, E> Connection<Sink, Stream>
where
    Sink: SinkExt<Bytes, Error = E> + Send + Sync + Unpin + 'static,
    Stream: StreamExt<Item = Result<Bytes, E>> + Send + Sync + Unpin + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    pub async fn new(broadcast_group: Arc<BroadcastGroup>, sink: Sink, stream: Stream) -> Self {
        let sink = Arc::new(Mutex::new(sink));
        let broadcast_sub = broadcast_group.subscribe(sink.clone(), stream).await;

        Connection {
            broadcast_sub: Some(broadcast_sub),
            completion_future: None,
            sink: PhantomData,
            stream: PhantomData,
        }
    }
}

impl<Sink, Stream> Future for Connection<Sink, Stream> {
    type Output = Result<()>;

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
                    tracing::debug!("Connection future completed with result: {:?}", result);
                }
                Poll::Pending => {
                    tracing::debug!("Connection future is pending");
                }
            }
            poll_result
        } else {
            Poll::Ready(Ok(()))
        }
    }
}

impl<Sink, Stream> Unpin for Connection<Sink, Stream> {}
