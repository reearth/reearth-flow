use axum::extract::ws::{Message, WebSocket};
use bytes::Bytes;
use futures_util::stream::SplitSink;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::{Mutex, RwLock};
use tokio::time::interval;
use tracing::{info, trace, warn};

const PING_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct SignalingService(Arc<RwLock<HashMap<Arc<str>, HashSet<WsSinkKey>>>>);

impl SignalingService {
    pub fn new() -> Self {
        SignalingService(Arc::new(RwLock::new(Default::default())))
    }
}

impl Default for SignalingService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct WsSink(Arc<Mutex<SplitSink<WebSocket, Message>>>);

impl WsSink {
    fn new(sink: SplitSink<WebSocket, Message>) -> Self {
        WsSink(Arc::new(Mutex::new(sink)))
    }

    async fn try_send(&self, msg: Message) -> Result<(), axum::Error> {
        let mut sink = self.0.lock().await;
        sink.send(msg).await
    }

    async fn close(&self) -> Result<(), axum::Error> {
        let mut sink = self.0.lock().await;
        sink.close().await
    }
}

#[allow(clippy::mutable_key_type)]
#[derive(Debug, Clone)]
struct WsSinkKey(WsSink);

impl From<WsSink> for WsSinkKey {
    fn from(ws: WsSink) -> Self {
        WsSinkKey(ws)
    }
}

impl Hash for WsSinkKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let ptr = Arc::as_ptr(&self.0 .0) as usize;
        ptr.hash(state);
    }
}

impl PartialEq<Self> for WsSinkKey {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0 .0, &other.0 .0)
    }
}

impl Eq for WsSinkKey {}

/// Handle incoming signaling connection
pub async fn handle_signaling_connection(
    socket: WebSocket,
    service: SignalingService,
) -> Result<(), axum::Error> {
    let mut topics = service.0;
    let (sink, mut stream) = socket.split();
    let ws = WsSink::new(sink);
    let mut ping_interval = interval(PING_TIMEOUT);
    let mut state = ConnState::default();

    loop {
        select! {
            _ = ping_interval.tick() => {
                if !state.pong_received {
                    ws.close().await?;
                    return Ok(());
                } else {
                    state.pong_received = false;
                    if ws.try_send(Message::Ping(Bytes::from_static(b""))).await.is_err() {
                        ws.close().await?;
                        return Ok(());
                    }
                }
            },
            res = stream.next() => {
                match res {
                    None => {
                        info!("Stream ended, closing connection");
                        ws.close().await?;
                        return Ok(());
                    },
                    Some(Err(e)) => {
                        warn!("❌ WebSocket error: {:?}", e);
                        ws.close().await?;
                        return Ok(());
                    },
                    Some(Ok(msg)) => {
                        trace!("📨 Received message: {:?}", msg);
                        if let Err(e) = process_msg(msg, &ws, &mut state, &mut topics).await {
                            warn!("❌ Error processing message, closing connection: {:?}", e);
                            ws.close().await?;
                            return Ok(());
                        }
                    }
                }
            }
        }
    }
}

const PONG_MSG: &str = r#"{"type":"pong"}"#;

#[allow(clippy::mutable_key_type)]
async fn process_msg(
    msg: Message,
    ws: &WsSink,
    state: &mut ConnState,
    topics: &mut Arc<RwLock<HashMap<Arc<str>, HashSet<WsSinkKey>>>>,
) -> Result<(), axum::Error> {
    if let Message::Text(text) = msg {
        let json = text.to_string();
        if let Ok(signal) = serde_json::from_str::<Signal>(&json) {
            match signal {
                Signal::Subscribe {
                    topics: topic_names,
                } => {
                    if !topic_names.is_empty() {
                        info!(
                            "📥 Client subscribing to {} topics: {:?}",
                            topic_names.len(),
                            topic_names
                        );
                        let mut topics_guard = topics.write().await;
                        for topic in topic_names {
                            if let Some((key, _)) = topics_guard.get_key_value(topic) {
                                state.subscribed_topics.insert(key.clone());
                                let subs = topics_guard.get_mut(topic).unwrap();
                                subs.insert(ws.clone().into());
                                info!(
                                    "✅ Added to existing room '{}', total clients: {}",
                                    topic,
                                    subs.len()
                                );
                            } else {
                                let topic: Arc<str> = topic.into();
                                state.subscribed_topics.insert(topic.clone());
                                let mut subs = HashSet::new();
                                subs.insert(ws.clone().into());
                                topics_guard.insert(topic.clone(), subs);
                                info!("🆕 Created new room '{}'", topic);
                            };
                        }
                    }
                }
                Signal::Unsubscribe {
                    topics: topic_names,
                } => {
                    if !topic_names.is_empty() {
                        let mut topics_guard = topics.write().await;
                        for topic in topic_names {
                            if let Some(subs) = topics_guard.get_mut(topic) {
                                trace!("unsubscribing client from '{topic}'");
                                subs.remove(&ws.clone().into());
                            }
                        }
                    }
                }
                Signal::Publish { topic } => {
                    let mut failed = Vec::new();
                    {
                        let topics_guard = topics.read().await;
                        if let Some(receivers) = topics_guard.get(topic) {
                            let client_count = receivers.len();
                            trace!(
                                "publishing on {} clients at '{}': {}",
                                client_count,
                                topic,
                                json
                            );

                            for receiver in receivers.iter() {
                                if let Err(e) = receiver
                                    .0
                                    .try_send(Message::Text(json.clone().into()))
                                    .await
                                {
                                    info!(
                                        "failed to publish message {} on '{}': {:?}",
                                        json, topic, e
                                    );
                                    failed.push(receiver.clone());
                                }
                            }
                        }
                    }
                    if !failed.is_empty() {
                        let mut topics_guard = topics.write().await;
                        if let Some(receivers) = topics_guard.get_mut(topic) {
                            for f in failed {
                                receivers.remove(&f);
                            }
                        }
                    }
                }
                Signal::Ping => {
                    trace!("Received ping, sending pong");
                    ws.try_send(Message::Text(PONG_MSG.into())).await?;
                }
                Signal::Pong => {
                    trace!("Received pong, updating state");
                    state.pong_received = true;
                }
            }
        }
    } else if let Message::Close(_) = msg {
        let mut topics_guard = topics.write().await;
        for topic in state.subscribed_topics.drain() {
            if let Some(subs) = topics_guard.get_mut(&topic) {
                subs.remove(&ws.clone().into());
                if subs.is_empty() {
                    topics_guard.remove(&topic);
                }
            }
        }
        state.closed = true;
    } else if let Message::Ping(_) = msg {
        trace!("Received WebSocket ping, sending pong");
        ws.try_send(Message::Pong(Bytes::from_static(b""))).await?;
    } else if let Message::Pong(_) = msg {
        trace!("Received WebSocket pong");
        state.pong_received = true;
    }
    Ok(())
}

#[derive(Debug)]
struct ConnState {
    closed: bool,
    pong_received: bool,
    subscribed_topics: HashSet<Arc<str>>,
}

impl Default for ConnState {
    fn default() -> Self {
        ConnState {
            closed: false,
            pong_received: true,
            subscribed_topics: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
enum Signal<'a> {
    #[serde(rename = "publish")]
    Publish { topic: &'a str },
    #[serde(rename = "subscribe")]
    Subscribe { topics: Vec<&'a str> },
    #[serde(rename = "unsubscribe")]
    Unsubscribe { topics: Vec<&'a str> },
    #[serde(rename = "ping")]
    Ping,
    #[serde(rename = "pong")]
    Pong,
}
