use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::storage::redis::pubsub::RedisPubSub;
use crate::storage::redis::{RedisConfig, RedisStore};
use crate::AwarenessRef;
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use rand;
use serde::Deserialize;
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
use yrs::{Doc, ReadTxn, StateVector, Transact, Update};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UserInfo {
    pub id: Option<String>,
    pub name: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BroadcastConfig {
    pub storage_enabled: bool,
    pub doc_name: Option<String>,
    pub redis_config: Option<RedisConfig>,
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
    pub fn increment_connections(&self) {
        let prev_count = self.connections.fetch_add(1, Ordering::Relaxed);

        if prev_count == 0 {
            if let (Some(store), Some(doc_name)) = (&self.storage, &self.doc_name) {
                let store_clone = store.clone();
                let doc_name_clone = doc_name.clone();
                let awareness_clone = self.awareness_ref.clone();

                tokio::spawn(async move {
                    Self::load_from_storage(&store_clone, &doc_name_clone, &awareness_clone).await;
                });
            }
        }
    }

    pub fn decrement_connections(&self) -> usize {
        let prev_count = self.connections.fetch_sub(1, Ordering::SeqCst);
        let current_count = prev_count - 1;

        if current_count == 0 {
            if let (Some(redis_store), Some(doc_name)) = (&self.redis_store, &self.doc_name) {
                let redis_store = redis_store.clone();
                let doc_name = doc_name.clone();
                let store = self.storage.clone();
                let pending_updates = self.pending_updates.clone();

                tokio::spawn(async move {
                    let updates = {
                        let mut pending = pending_updates.lock().await;
                        std::mem::take(&mut *pending)
                    };

                    if updates.is_empty() {
                        if let Some(store) = &store {
                            if let Ok(Some(_)) = store.flush_doc(&doc_name).await {
                                tracing::debug!("Flushed document {} to storage", doc_name);
                            }
                        }
                        return;
                    };

                    let doc = Doc::new();
                    let mut txn = doc.transact_mut();

                    let mut has_updates = false;
                    for (i, update) in updates.iter().enumerate() {
                        if let Ok(decoded) = Update::decode_v1(update) {
                            if let Err(e) = txn.apply_update(decoded) {
                                tracing::warn!("Failed to apply update {} during flush: {}", i, e);
                            } else {
                                has_updates = true;
                            }
                        } else {
                            tracing::warn!("Failed to decode update {} during flush", i);
                        }
                    }

                    if has_updates {
                        let state_vector = StateVector::default();
                        let merged_update = txn.encode_state_as_update_v1(&state_vector);

                        if let Some(store) = &store {
                            Self::handle_gcs_update(merged_update, &doc_name, store).await;
                        }
                    }

                    if let Err(e) = redis_store.clear_pending_updates(&doc_name).await {
                        tracing::warn!("Failed to clear pending updates from Redis: {}", e);
                    }
                });
            }
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
            lock.doc_mut().observe_update_v1(move |_txn, u| {
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
            })?
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
                    match awareness.update_with_clients(changed_clients) {
                        Ok(update) => {
                            if sink.send(Message::Awareness(update).encode_v1()).is_err() {
                                tracing::warn!("couldn't broadcast awareness update");
                            }
                        }
                        Err(e) => {
                            tracing::warn!("error while computing awareness update: {}", e)
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
        config: BroadcastConfig,
    ) -> Result<Self> {
        if !config.storage_enabled {
            return Self::new(awareness, buffer_capacity).await;
        }

        let mut group = Self::new(awareness, buffer_capacity).await?;

        let doc_name = config
            .doc_name
            .expect("doc_name required when storage enabled");
        let redis_ttl = config.redis_config.as_ref().map(|c| c.ttl as usize);

        group.doc_name = Some(doc_name.clone());

        Self::load_from_storage(
            &store,
            group.doc_name.as_ref().unwrap(),
            &group.awareness_ref,
        )
        .await;

        let redis_store = if let Some(redis_config) = config.redis_config {
            let mut store = RedisStore::new(Some(redis_config.clone()));
            match store.init().await {
                Ok(()) => {
                    let store = Arc::new(store);
                    let doc_name_str = group.doc_name.as_ref().unwrap().clone();
                    let store_clone = store.clone();
                    let awareness_clone = group.awareness_ref.clone();
                    let pending_clone = group.pending_updates.clone();

                    tokio::spawn(async move {
                        match store_clone.get_pending_updates(&doc_name_str).await {
                            Ok(updates) => {
                                if !updates.is_empty() {
                                    let awareness = awareness_clone.write().await;
                                    let mut txn = awareness.doc().transact_mut();

                                    for update in &updates {
                                        if let Ok(decoded) = Update::decode_v1(update) {
                                            if let Err(e) = txn.apply_update(decoded) {
                                                tracing::warn!(
                                                    "Failed to apply update from Redis: {}",
                                                    e
                                                );
                                            }
                                        } else {
                                            tracing::warn!("Failed to decode update from Redis");
                                        }
                                    }

                                    let mut pending = pending_clone.lock().await;
                                    pending.extend(updates);
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Failed to load pending updates from Redis: {}", e);
                            }
                        }
                    });

                    let consumer_name = format!("instance-{}", rand::random::<u32>());

                    let doc_name_for_sub = group.doc_name.as_ref().unwrap().clone();
                    let awareness_for_sub = group.awareness_ref.clone();
                    let (tx, mut rx) = tokio::sync::mpsc::channel(1024);
                    let sender_for_sub = group.sender.clone();
                    let redis_url = redis_config.url.clone();

                    let mut redis_pubsub =
                        RedisPubSub::new(redis_url, awareness_for_sub, tx, doc_name_for_sub);

                    // Set up a task to forward messages from mpsc to broadcast
                    let sender_clone = sender_for_sub.clone();
                    tokio::spawn(async move {
                        while let Some(msg) = rx.recv().await {
                            let _ = sender_clone.send(msg);
                        }
                    });

                    redis_pubsub.start();
                    group.redis_consumer_name = Some(consumer_name);

                    let redis_pubsub = std::sync::Arc::new(tokio::sync::Mutex::new(redis_pubsub));
                    let redis_pubsub_clone = redis_pubsub.clone();

                    group.redis_subscriber_task = Some(tokio::spawn(async move {
                        let _redis_pubsub = redis_pubsub_clone;
                        loop {
                            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                        }
                    }));

                    Some(store)
                }
                Err(e) => {
                    tracing::error!("Failed to initialize Redis connection: {}", e);
                    None
                }
            }
        } else {
            None
        };

        group.storage = Some(store);
        group.redis_store = redis_store;
        group.redis_ttl = redis_ttl;

        group.storage_rx = None;

        Ok(group)
    }

    async fn load_from_storage(store: &Arc<GcsStore>, doc_name: &str, awareness: &AwarenessRef) {
        let awareness = awareness.write().await;
        let mut txn = awareness.doc().transact_mut();

        if let Err(e) = store.load_doc(doc_name, &mut txn).await {
            tracing::error!("Failed to load document '{}' from storage: {}", doc_name, e);
        }
    }

    async fn handle_gcs_update(update: Vec<u8>, doc_name: &str, store: &Arc<GcsStore>) {
        if let Err(e) = store.push_update(doc_name, &update).await {
            tracing::error!(
                "Failed to store update in GCS for document '{}': {}",
                doc_name,
                e
            );
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
        &self,
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

        self.subscribe_with(sink, stream, DefaultProtocol)
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
            let redis = self.redis_store.clone();
            let doc_name = self.doc_name.clone();
            let storage = self.storage.clone();
            let protocol = Arc::new(protocol);
            let sender = self.sender.clone();
            let pending_updates = self.pending_updates.clone();

            tokio::spawn(async move {
                while let Some(msg_result) = stream.next().await {
                    match msg_result {
                        Ok(msg) => {
                            let awareness_clone = awareness.clone();
                            let protocol_clone = protocol.clone();
                            let sender_clone = sender.clone();
                            let pending_clone = pending_updates.clone();
                            let redis_clone = redis.clone();
                            let doc_name_str = doc_name.clone();
                            let storage_clone = storage.clone();

                            let handle = tokio::spawn(async move {
                                let protocol_ref: &P = &protocol_clone;

                                if let Ok(decoded_msg) = Message::decode_v1(&msg) {
                                    if let Ok(Some(response)) = Self::handle_msg(
                                        protocol_ref,
                                        &awareness_clone,
                                        decoded_msg,
                                    )
                                    .await
                                    {
                                        let encoded_response = response.encode_v1();
                                        if let Some(doc_name) = doc_name_str.as_ref() {
                                            let doc_name_str = doc_name.clone();

                                            if let Some(redis_store) = &redis_clone {
                                                let _ = redis_store
                                                    .add_update(&doc_name_str, &encoded_response)
                                                    .await;
                                            }

                                            if let Some(storage) = &storage_clone {
                                                let mut pending = pending_clone.lock().await;
                                                pending.push(encoded_response.clone());

                                                if pending.len() >= 10 {
                                                    let updates = std::mem::take(&mut *pending);
                                                    drop(pending);

                                                    let doc = Doc::new();
                                                    let mut txn = doc.transact_mut();

                                                    let mut has_updates = false;
                                                    for (i, update) in updates.iter().enumerate() {
                                                        if let Ok(decoded) =
                                                            Update::decode_v1(update)
                                                        {
                                                            if let Err(e) =
                                                                txn.apply_update(decoded)
                                                            {
                                                                tracing::warn!(
                                                                    "Failed to apply update {} during flush: {}",
                                                                    i,
                                                                    e
                                                                );
                                                            } else {
                                                                has_updates = true;
                                                            }
                                                        }
                                                    }

                                                    if has_updates {
                                                        let state_vector = StateVector::default();
                                                        let merged_update = txn
                                                            .encode_state_as_update_v1(
                                                                &state_vector,
                                                            );
                                                        Self::handle_gcs_update(
                                                            merged_update,
                                                            doc_name,
                                                            storage,
                                                        )
                                                        .await;
                                                    }

                                                    if let Some(redis_store) = &redis_clone {
                                                        let _ = redis_store
                                                            .clear_pending_updates(&doc_name_str)
                                                            .await;
                                                    }
                                                }
                                            }
                                        }
                                        let _ = sender_clone.send(encoded_response);
                                    }
                                }
                            });

                            if let Err(e) = handle.await {
                                if e.is_cancelled() {
                                    // todo: handle cancellation
                                }
                            }
                        }
                        Err(_) => {
                            break;
                        }
                    }
                }
                Ok(())
            })
        };

        Subscription {
            sink_task,
            stream_task,
        }
    }

    async fn handle_msg<P: Protocol>(
        protocol: &P,
        awareness: &AwarenessRef,
        msg: Message,
    ) -> Result<Option<Message>, Error> {
        match msg {
            Message::Sync(msg) => match msg {
                SyncMessage::SyncStep1(state_vector) => {
                    let awareness = awareness.read().await;
                    protocol.handle_sync_step1(&awareness, state_vector)
                }
                SyncMessage::SyncStep2(update) => {
                    let awareness = awareness.write().await;
                    let update = Update::decode_v1(&update)?;
                    protocol.handle_sync_step2(&awareness, update)
                }
                SyncMessage::Update(update) => {
                    let awareness = awareness.write().await;
                    let update = Update::decode_v1(&update)?;
                    protocol.handle_sync_step2(&awareness, update)
                }
            },
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

        if !self
            .shutdown_complete
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            if let (Some(_store), Some(doc_name)) = (&self.storage, &self.doc_name) {
                tracing::warn!(
                    "Document '{}' may have pending updates that weren't flushed",
                    doc_name
                );
            }
        }
    }
}

pub struct Subscription {
    sink_task: JoinHandle<Result<(), Error>>,
    stream_task: JoinHandle<Result<(), Error>>,
}

impl Subscription {
    pub async fn completed(self) -> Result<(), Error> {
        let res = select! {
            r1 = self.sink_task => r1,
            r2 = self.stream_task => r2,
        };
        res.map_err(|e| Error::Other(e.into()))?
    }
}
