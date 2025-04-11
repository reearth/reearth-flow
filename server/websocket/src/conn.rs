#![allow(dead_code)]
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};
use yrs::sync::Error;

use crate::broadcast::sub::Subscription;
use crate::group::BroadcastGroup;

type CompletionFuture = Pin<Box<dyn Future<Output = Result<(), Error>> + Send>>;

pub struct Connection<Sink, Stream> {
    broadcast_sub: Option<Subscription>,
    completion_future: Option<CompletionFuture>,
    user_token: Option<String>,
    ping_interval: Duration,
    ping_task: Option<JoinHandle<()>>,
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
        let broadcast_sub = broadcast_group
            .subscribe(sink.clone(), stream, user_token.clone())
            .await;

        let ping_interval = Duration::from_secs(30);
        let ping_task = Some(Self::start_ping_task(sink, ping_interval));

        Connection {
            broadcast_sub: Some(broadcast_sub),
            completion_future: None,
            user_token,
            ping_interval,
            ping_task,
            sink: PhantomData,
            stream: PhantomData,
        }
    }

    fn start_ping_task(sink: Arc<Mutex<Sink>>, ping_interval: Duration) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = interval(ping_interval);
            loop {
                interval.tick().await;
                let ping_message = Bytes::from("ping");
                let mut sink_lock = sink.lock().await;
                if let Err(e) = sink_lock.send(ping_message).await {
                    tracing::warn!("Failed to send ping: {:?}", e);
                    break;
                }
            }
        })
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

impl<Sink, Stream> Drop for Connection<Sink, Stream> {
    fn drop(&mut self) {
        if let Some(task) = self.ping_task.take() {
            task.abort();
        }
    }
}

impl<Sink, Stream> Unpin for Connection<Sink, Stream> {}
