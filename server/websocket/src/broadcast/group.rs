use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::storage::redis::RedisStore;
use crate::AwarenessRef;
use anyhow::anyhow;
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use rand;
use redis::AsyncCommands;

use serde_json;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast::error::SendError;
use tokio::sync::broadcast::{channel, Sender};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use yrs::encoding::write::Write;
use yrs::sync::protocol::{MSG_SYNC, MSG_SYNC_UPDATE};
use yrs::sync::{DefaultProtocol, Error, Message, Protocol, SyncMessage};
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::{Encode, Encoder, EncoderV1};
use yrs::{Doc, ReadTxn, Transact, Update};

#[derive(Debug, Clone)]
pub struct BroadcastConfig {
    pub storage_enabled: bool,
    pub doc_name: Option<String>,
}

pub struct BroadcastGroup {
    connections: Arc<AtomicUsize>,
    awareness_ref: AwarenessRef,
    sender: Sender<Vec<u8>>,
    awareness_updater: JoinHandle<()>,
    doc_sub: Option<yrs::Subscription>,
    awareness_sub: Option<yrs::Subscription>,
    storage: Option<Arc<GcsStore>>,
    redis_store: Option<Arc<RedisStore>>,
    doc_name: Option<String>,
    redis_ttl: Option<usize>,
    storage_rx: Option<tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>>,
    pending_updates: Arc<Mutex<Vec<Vec<u8>>>>,
    redis_subscriber_task: Option<JoinHandle<()>>,
    redis_consumer_name: Option<String>,
    shutdown_complete: AtomicBool,
}

impl std::fmt::Debug for BroadcastGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BroadcastGroup")
            .field("connections", &self.connections)
            .field("awareness_ref", &self.awareness_ref)
            .field("doc_name", &self.doc_name)
            .field("redis_ttl", &self.redis_ttl)
            .finish()
    }
}

unsafe impl Send for BroadcastGroup {}
unsafe impl Sync for BroadcastGroup {}

impl BroadcastGroup {
    pub async fn increment_connections(&self) -> Result<()> {
        let _ = self.connections.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    pub fn decrement_connections(&self) -> usize {
        let prev_count = self.connections.fetch_sub(1, Ordering::Relaxed);

        if let (Some(store), Some(doc_name)) = (&self.storage, &self.doc_name) {
            let store_clone = store.clone();
            let doc_name_clone = doc_name.clone();
            let awareness = self.awareness_ref.clone();
            let shutdown_flag = Arc::new(AtomicBool::new(false));
            let shutdown_flag_clone = shutdown_flag.clone();

            tokio::spawn(async move {
                let awareness = awareness.write().await;
                let awareness_doc = awareness.doc();
                let _awareness_state = awareness_doc.transact().state_vector();

                let gcs_doc = Doc::new();
                let mut gcs_txn = gcs_doc.transact_mut();

                if let Err(e) = store_clone.load_doc(&doc_name_clone, &mut gcs_txn).await {
                    tracing::warn!("Failed to load current state from GCS: {}", e);
                }

                let gcs_state = gcs_txn.state_vector();

                let awareness_txn = awareness_doc.transact();
                let update = awareness_txn.encode_state_as_update_v1(&gcs_state);
                Self::handle_gcs_update(update, &doc_name_clone, &store_clone).await;

                shutdown_flag_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            });

            tokio::spawn(async move {
                for _ in 0..10 {
                    if shutdown_flag.load(std::sync::atomic::Ordering::SeqCst) {
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
            });
        }

        prev_count
    }

    pub fn connection_count(&self) -> usize {
        self.connections.load(Ordering::Relaxed)
    }

    pub async fn new(awareness: AwarenessRef, buffer_capacity: usize) -> Result<Self> {
        let (sender, _receiver) = channel(buffer_capacity);
        let awareness_c = Arc::downgrade(&awareness);
        let mut lock = awareness.write().await;
        let sink = sender.clone();

        let (_storage_tx, storage_rx) = tokio::sync::mpsc::unbounded_channel();
        let pending_updates = Arc::new(Mutex::new(Vec::new()));
        let pending_updates_clone = pending_updates.clone();

        let doc_sub = {
            lock.doc_mut()
                .observe_update_v1(move |_txn, u| {
                    let mut encoder = EncoderV1::new();
                    encoder.write_var(MSG_SYNC);
                    encoder.write_var(MSG_SYNC_UPDATE);
                    encoder.write_buf(&u.update);
                    let msg = encoder.to_vec();
                    if let Err(_e) = sink.send(msg) {
                        tracing::warn!("broadcast channel closed");
                    }

                    let update_clone = u.update.clone();
                    let updates_arc = pending_updates_clone.clone();

                    tokio::spawn(async move {
                        let mut updates = updates_arc.lock().await;
                        updates.push(update_clone.clone());
                    });
                })
                .map_err(|e| anyhow!("Failed to observe document updates: {}", e))?
        };

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let sink = sender.clone();

        let awareness_sub = lock.on_update(move |_awareness, event, _origin| {
            let added = event.added();
            let updated = event.updated();
            let removed = event.removed();
            let total_len = added.len() + updated.len() + removed.len();
            if total_len == 0 {
                return;
            }

            let mut changed = Vec::with_capacity(total_len);
            changed.extend_from_slice(added);
            changed.extend_from_slice(updated);
            changed.extend_from_slice(removed);

            if tx.send(changed).is_err() {
                tracing::warn!("failed to send awareness update");
            }
        });
        drop(lock);

        let awareness_updater = tokio::task::spawn(async move {
            while let Some(changed_clients) = rx.recv().await {
                if let Some(awareness) = awareness_c.upgrade() {
                    let awareness = awareness.read().await;
                    if let Ok(update) = awareness.update_with_clients(changed_clients) {
                        if sink.send(Message::Awareness(update).encode_v1()).is_err() {
                            tracing::warn!("couldn't broadcast awareness update");
                        }
                    }
                } else {
                    return;
                }
            }
        });

        let result = Self {
            connections: Arc::new(AtomicUsize::new(0)),
            awareness_ref: awareness,
            sender,
            awareness_updater,
            doc_sub: Some(doc_sub),
            awareness_sub: Some(awareness_sub),
            storage: None,
            redis_store: None,
            doc_name: None,
            redis_ttl: None,
            storage_rx: Some(storage_rx),
            pending_updates,
            redis_subscriber_task: None,
            redis_consumer_name: None,
            shutdown_complete: AtomicBool::new(false),
        };

        Ok(result)
    }

    pub async fn with_storage(
        awareness: AwarenessRef,
        buffer_capacity: usize,
        store: Arc<GcsStore>,
        redis_store: Option<Arc<RedisStore>>,
        config: BroadcastConfig,
    ) -> Result<Self> {
        if !config.storage_enabled {
            return Self::new(awareness, buffer_capacity).await;
        }

        let mut group = Self::new(awareness, buffer_capacity).await?;

        let doc_name = config.doc_name.unwrap_or_default();

        let redis_ttl = redis_store
            .as_ref()
            .and_then(|rs| rs.get_config())
            .map(|c| c.ttl as usize);

        Self::load_from_storage(&store, &doc_name, &group.awareness_ref).await;

        if let Some(redis_store) = redis_store {
            group.redis_store = Some(redis_store.clone());

            if let Some(pool) = redis_store.get_pool() {
                let redis_key = format!("pending_updates:{}", doc_name);
                let awareness_clone = group.awareness_ref.clone();
                let pending_clone = group.pending_updates.clone();

                tokio::spawn(async move {
                    let mut redis_conn = match pool.get().await {
                        Ok(conn) => conn,
                        Err(e) => {
                            tracing::error!("Failed to get Redis connection: {}", e);
                            return;
                        }
                    };
                    if let Ok(updates) = redis_conn
                        .lrange::<_, Vec<Vec<u8>>>(&redis_key, 0, -1)
                        .await
                    {
                        if !updates.is_empty() {
                            let awareness = awareness_clone.write().await;
                            let mut txn = awareness.doc().transact_mut();

                            for update in &updates {
                                if let Ok(decoded) = Update::decode_v1(update) {
                                    if let Err(e) = txn.apply_update(decoded) {
                                        tracing::warn!("Failed to apply update from Redis: {}", e);
                                    }
                                }
                            }

                            let mut pending = pending_clone.lock().await;
                            pending.extend(updates);
                        }
                    }
                });

                let redis_url = redis_store
                    .get_config()
                    .map(|config| config.url.clone())
                    .unwrap_or_default();

                let consumer_name = format!("instance-{}", rand::random::<u32>());
                let awareness_for_sub = group.awareness_ref.clone();
                let sender_for_sub = group.sender.clone();
                let doc_name_for_sub = doc_name.clone();

                let redis_subscriber_task = tokio::spawn(async move {
                    if let Ok(client) = redis::Client::open(redis_url) {
                        if let Ok(mut pubsub) = client.get_async_pubsub().await {
                            let channel = format!("yjs:updates:{}", doc_name_for_sub);
                            if (pubsub.subscribe(&channel).await).is_ok() {
                                let mut stream = pubsub.on_message();
                                while let Some(msg) = stream.next().await {
                                    if let Ok(payload) = msg.get_payload::<Vec<u8>>() {
                                        let awareness = awareness_for_sub.write().await;
                                        let mut txn = awareness.doc().transact_mut();

                                        if let Ok(decoded) = Update::decode_v1(&payload) {
                                            if let Err(e) = txn.apply_update(decoded) {
                                                tracing::warn!(
                                                    "Failed to apply update from Redis: {}",
                                                    e
                                                );
                                            } else {
                                                let _ = sender_for_sub.send(payload);
                                            }
                                        }
                                    } else {
                                        tracing::error!("Failed to get payload from Redis message");
                                    }
                                }
                            } else {
                                tracing::error!("Failed to subscribe to Redis channel");
                            }
                        } else {
                            tracing::error!("Failed to get async connection to Redis");
                        }
                    } else {
                        tracing::error!("Failed to open Redis client");
                    }
                });

                group.redis_subscriber_task = Some(redis_subscriber_task);
                group.redis_consumer_name = Some(consumer_name);
            }
        }

        group.storage = Some(store);
        group.doc_name = Some(doc_name.clone());
        group.redis_ttl = redis_ttl;

        group.storage_rx = None;

        Ok(group)
    }

    async fn load_from_storage(store: &Arc<GcsStore>, doc_name: &str, awareness: &AwarenessRef) {
        let awareness = awareness.write().await;
        let mut txn = awareness.doc().transact_mut();

        if let Err(e) = store.load_doc(doc_name, &mut txn).await {
            tracing::error!("Error loading document '{}' from storage: {}", doc_name, e);
        }
    }

    async fn handle_gcs_update(update: Vec<u8>, doc_name: &str, store: &Arc<GcsStore>) {
        if let Err(e) = store.push_update(doc_name, &update).await {
            tracing::error!("Failed to store update for document '{}': {}", doc_name, e);
        }
    }

    pub fn awareness(&self) -> &AwarenessRef {
        &self.awareness_ref
    }

    pub fn broadcast(&self, msg: Vec<u8>) -> Result<(), SendError<Vec<u8>>> {
        self.sender.send(msg)?;
        Ok(())
    }

    pub fn subscribe<Sink, Stream, E>(&self, sink: Arc<Mutex<Sink>>, stream: Stream) -> Subscription
    where
        Sink: SinkExt<Vec<u8>> + Send + Sync + Unpin + 'static,
        Stream: StreamExt<Item = Result<Vec<u8>, E>> + Send + Sync + Unpin + 'static,
        <Sink as futures_util::Sink<Vec<u8>>>::Error: std::error::Error + Send + Sync,
        E: std::error::Error + Send + Sync + 'static,
    {
        self.subscribe_with(sink, stream, DefaultProtocol)
    }

    pub fn subscribe_with_user<Sink, Stream, E>(
        self: Arc<Self>,
        sink: Arc<Mutex<Sink>>,
        stream: Stream,
        user_token: Option<String>,
    ) -> Subscription
    where
        Sink: SinkExt<Vec<u8>> + Send + Sync + Unpin + 'static,
        Stream: StreamExt<Item = Result<Vec<u8>, E>> + Send + Sync + Unpin + 'static,
        <Sink as futures_util::Sink<Vec<u8>>>::Error: std::error::Error + Send + Sync,
        E: std::error::Error + Send + Sync + 'static,
    {
        if let Some(token) = user_token {
            let awareness = self.awareness().clone();
            let client_id = rand::random::<u64>();

            tokio::spawn(async move {
                let awareness = awareness.write().await;
                let mut local_state = std::collections::HashMap::new();

                local_state.insert(
                    "user",
                    serde_json::json!({
                        "id": token,
                        "name": format!("User-{}", client_id % 1000),
                    }),
                );

                if let Err(e) = awareness.set_local_state(Some(local_state)) {
                    tracing::error!("Failed to set awareness state: {}", e);
                }
            });
        }

        let subscription = self.subscribe_with(sink, stream, DefaultProtocol);
        let (tx, rx) = tokio::sync::oneshot::channel();

        let self_clone = self.clone();

        tokio::spawn(async move {
            if let Err(e) = self_clone.increment_connections().await {
                tracing::error!("Failed to increment connections: {}", e);
            }
            let _ = tx.send(());
        });

        Subscription {
            sink_task: subscription.sink_task,
            stream_task: subscription.stream_task,
            sync_complete: Some(rx),
        }
    }

    pub fn subscribe_with<Sink, Stream, E, P>(
        &self,
        sink: Arc<Mutex<Sink>>,
        mut stream: Stream,
        protocol: P,
    ) -> Subscription
    where
        Sink: SinkExt<Vec<u8>> + Send + Sync + Unpin + 'static,
        Stream: StreamExt<Item = Result<Vec<u8>, E>> + Send + Sync + Unpin + 'static,
        <Sink as futures_util::Sink<Vec<u8>>>::Error: std::error::Error + Send + Sync,
        E: std::error::Error + Send + Sync + 'static,
        P: Protocol + Send + Sync + 'static,
    {
        let sink_task = {
            let sender = self.sender.clone();
            let sink = sink.clone();
            tokio::spawn(async move {
                let mut rx = sender.subscribe();
                while let Ok(msg) = rx.recv().await {
                    let mut sink = sink.lock().await;
                    if sink.send(msg).await.is_err() {
                        return Ok(());
                    }
                }
                Ok(())
            })
        };

        let stream_task = {
            let awareness = self.awareness().clone();
            let redis_store = self.redis_store.clone();
            let doc_name = self.doc_name.clone();
            let redis_ttl = self.redis_ttl;
            let gcs_store = self.storage.clone();

            tokio::spawn(async move {
                while let Some(res) = stream.next().await {
                    let data = match res.map_err(|e| Error::Other(Box::new(e))) {
                        Ok(data) => data,
                        Err(e) => {
                            tracing::warn!("Error receiving message: {}", e);
                            continue;
                        }
                    };

                    let msg = match Message::decode_v1(&data) {
                        Ok(msg) => msg,
                        Err(e) => {
                            tracing::warn!("Failed to decode message: {}", e);
                            continue;
                        }
                    };

                    match Self::handle_msg(
                        &protocol,
                        &awareness,
                        msg,
                        redis_store.as_ref(),
                        doc_name.as_ref(),
                        redis_ttl,
                        gcs_store.as_ref(),
                    )
                    .await
                    {
                        Ok(Some(reply)) => {
                            let mut sink_lock = sink.lock().await;
                            if let Err(e) = sink_lock.send(reply.encode_v1()).await {
                                tracing::warn!("Failed to send reply: {}", e);
                            }
                        }
                        Err(e) => tracing::warn!("Error handling message: {}", e),
                        _ => {}
                    }
                }
                Ok(())
            })
        };

        Subscription {
            sink_task,
            stream_task,
            sync_complete: None,
        }
    }

    async fn handle_msg<P: Protocol>(
        protocol: &P,
        awareness: &AwarenessRef,
        msg: Message,
        redis_store: Option<&Arc<RedisStore>>,
        doc_name: Option<&String>,
        redis_ttl: Option<usize>,
        _gcs_store: Option<&Arc<GcsStore>>,
    ) -> Result<Option<Message>, Error> {
        match msg {
            Message::Sync(msg) => {
                if let (Some(redis_store), Some(doc_name), Some(ttl)) =
                    (redis_store, doc_name, redis_ttl)
                {
                    let rs = redis_store.clone();
                    let dn = doc_name.clone();
                    let update_bytes = match &msg {
                        SyncMessage::Update(update) => update.clone(),
                        SyncMessage::SyncStep2(update) => update.clone(),
                        _ => Vec::new(),
                    };

                    if !update_bytes.is_empty() {
                        let ttl_clone = ttl as u64;
                        tokio::spawn(async move {
                            if let Err(e) =
                                rs.add_publish_update(&dn, &update_bytes, ttl_clone).await
                            {
                                tracing::error!("Redis update failed: {}", e);
                            }
                        });
                    }
                }

                match msg {
                    SyncMessage::SyncStep1(state_vector) => {
                        let awareness = awareness.read().await;
                        protocol.handle_sync_step1(&awareness, state_vector)
                    }
                    SyncMessage::SyncStep2(update) => {
                        let awareness = awareness.write().await;
                        let decoded_update = Update::decode_v1(&update)?;

                        protocol.handle_sync_step2(&awareness, decoded_update)
                    }
                    SyncMessage::Update(update) => {
                        let awareness = awareness.write().await;
                        let update = Update::decode_v1(&update)?;
                        protocol.handle_sync_step2(&awareness, update)
                    }
                }
            }
            Message::Auth(deny_reason) => {
                let awareness = awareness.read().await;
                protocol.handle_auth(&awareness, deny_reason)
            }
            Message::AwarenessQuery => {
                let awareness = awareness.read().await;
                protocol.handle_awareness_query(&awareness)
            }
            Message::Awareness(update) => {
                let awareness = awareness.write().await;
                protocol.handle_awareness_update(&awareness, update)
            }
            Message::Custom(tag, data) => {
                let awareness = awareness.write().await;
                protocol.missing_handle(&awareness, tag, data)
            }
        }
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.shutdown_complete
            .store(true, std::sync::atomic::Ordering::SeqCst);

        if let (Some(store), Some(doc_name)) = (&self.storage, &self.doc_name) {
            let awareness = self.awareness_ref.read().await;
            let awareness_doc = awareness.doc();

            let gcs_doc = Doc::new();
            let mut gcs_txn = gcs_doc.transact_mut();

            if let Err(e) = store.load_doc(doc_name, &mut gcs_txn).await {
                tracing::warn!("Failed to load current state from GCS: {}", e);
            }

            let gcs_state = gcs_txn.state_vector();

            let awareness_txn = awareness_doc.transact();
            let update = awareness_txn.encode_state_as_update_v1(&gcs_state);

            if let Err(e) = store.push_update(doc_name, &update).await {
                tracing::error!(
                    "Failed to store final state for document '{}': {}",
                    doc_name,
                    e
                );
                return Err(anyhow!("Failed to store final state: {}", e));
            }

            if let Some(redis_store) = &self.redis_store {
                if let Err(e) = redis_store.clear_pending_updates(doc_name).await {
                    tracing::warn!("Failed to clear pending updates from Redis: {}", e);
                }
            }
        }

        Ok(())
    }
}

impl Drop for BroadcastGroup {
    fn drop(&mut self) {
        if let Some(sub) = self.doc_sub.take() {
            drop(sub);
        }

        if let Some(sub) = self.awareness_sub.take() {
            drop(sub);
        }

        if let Some(task) = self.redis_subscriber_task.take() {
            task.abort();
        }

        self.awareness_updater.abort();

        self.shutdown_complete
            .store(true, std::sync::atomic::Ordering::SeqCst);

        if let (Some(_store), Some(doc_name)) = (&self.storage, &self.doc_name) {
            let pending_count = {
                match self.pending_updates.try_lock() {
                    Ok(guard) => guard.len(),
                    Err(_) => 0,
                }
            };

            if pending_count > 0 {
                tracing::warn!(
                    "Document '{}' may have pending updates that weren't flushed (count: {})",
                    doc_name,
                    pending_count
                );
            }
        }
    }
}

pub struct Subscription {
    sink_task: JoinHandle<Result<(), Error>>,
    stream_task: JoinHandle<Result<(), Error>>,
    sync_complete: Option<tokio::sync::oneshot::Receiver<()>>,
}

impl Subscription {
    pub async fn completed(mut self) -> Result<(), Error> {
        if let Some(sync_complete) = self.sync_complete.take() {
            let _ = sync_complete.await;
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let res = select! {
            r1 = self.sink_task => r1,
            r2 = self.stream_task => r2,
        };
        res.map_err(|e| Error::Other(e.into()))?
    }
}
