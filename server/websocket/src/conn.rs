#![allow(dead_code)]
use futures_util::{SinkExt, StreamExt};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::Mutex;
use yrs::sync::Error;

use crate::group::{BroadcastGroup, Subscription};

type CompletionFuture = Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>;

/// Connection handler over a pair of message streams, which implements a Yjs/Yrs awareness and
/// update exchange protocol.
pub struct Connection<Sink, Stream> {
    broadcast_sub: Option<Subscription>,
    completion_future: Option<CompletionFuture>,
    user_token: Option<String>,
    _sink: PhantomData<Sink>,
    _stream: PhantomData<Stream>,
}

impl<Sink, Stream, E> Connection<Sink, Stream>
where
    Sink: SinkExt<Vec<u8>, Error = E> + Send + Sync + Unpin + 'static,
    Stream: StreamExt<Item = Result<Vec<u8>, E>> + Send + Sync + Unpin + 'static,
    E: std::error::Error + Into<Error> + Send + Sync + 'static,
{
    /// Creates a new connection using a BroadcastGroup with user information
    pub async fn with_broadcast_group_and_user(
        broadcast_group: Arc<BroadcastGroup>,
        sink: Sink,
        stream: Stream,
        user_token: Option<String>,
    ) -> Self {
        let sink = Arc::new(Mutex::new(sink));
        let broadcast_sub = Some(broadcast_group.clone().subscribe_with_user(
            sink,
            stream,
            user_token.clone(),
        ));

        Connection {
            broadcast_sub,
            completion_future: None,
            user_token,
            _sink: PhantomData,
            _stream: PhantomData,
        }
    }

    /// Creates a new connection using a BroadcastGroup
    pub async fn with_broadcast_group(
        broadcast_group: Arc<BroadcastGroup>,
        sink: Sink,
        stream: Stream,
    ) -> Self {
        let sink = Arc::new(Mutex::new(sink));
        let broadcast_sub = Some(broadcast_group.subscribe(sink, stream));

        Connection {
            broadcast_sub,
            completion_future: None,
            user_token: None,
            _sink: PhantomData,
            _stream: PhantomData,
        }
    }

    /// Creates a new connection with default protocol
    pub fn new(broadcast_group: Arc<BroadcastGroup>, sink: Sink, stream: Stream) -> Self {
        let sink = Arc::new(Mutex::new(sink));
        let broadcast_sub = Some(broadcast_group.subscribe(sink, stream));

        Connection {
            broadcast_sub,
            completion_future: None,
            user_token: None,
            _sink: PhantomData,
            _stream: PhantomData,
        }
    }
}

impl<Sink, Stream> Future for Connection<Sink, Stream> {
    type Output = Result<(), Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.completion_future.is_none() {
            if let Some(sub) = self.broadcast_sub.take() {
                self.completion_future = Some(Box::pin(sub.completed()));
            }
        }

        if let Some(fut) = self.completion_future.as_mut() {
            fut.as_mut().poll(cx)
        } else {
            Poll::Ready(Ok(()))
        }
    }
}

impl<Sink, Stream> Drop for Connection<Sink, Stream> {
    fn drop(&mut self) {
        if let Some(sub) = self.broadcast_sub.take() {
            drop(sub);
        }
    }
}

impl<Sink, Stream> Unpin for Connection<Sink, Stream> {}
