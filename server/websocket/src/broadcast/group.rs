use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::storage::redis::{RedisConfig, RedisStore};
use crate::AwarenessRef;
use anyhow::anyhow;
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use rand;
use serde_json;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
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
    redis: Option<Arc<RedisStore>>,
    doc_name: Option<String>,
    storage_rx: Option<tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>>,
    shutdown_complete: AtomicBool,
}

impl std::fmt::Debug for BroadcastGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BroadcastGroup")
            .field("connections", &self.connections)
            .field("awareness_ref", &self.awareness_ref)
            .field("doc_name", &self.doc_name)
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
        let prev_count = self.connections.fetch_sub(1, Ordering::Relaxed);

        if let (Some(store), Some(doc_name)) = (&self.storage, &self.doc_name) {
            let store_clone = store.clone();
            let doc_name_clone = doc_name.clone();
            let redis = self.redis.clone();

            tokio::spawn(async move {
                if let Some(redis) = redis {
                    if let Ok(updates) = redis.load_updates(&doc_name_clone).await {
                        if !updates.is_empty() {
                            let doc = Doc::new();
                            let mut txn = doc.transact_mut();

                            let mut has_updates = false;
                            for (i, update) in updates.iter().enumerate() {
                                if let Ok(decoded) = Update::decode_v1(update) {
                                    if let Err(e) = txn.apply_update(decoded) {
                                        tracing::warn!(
                                            "Failed to apply update {} during flush: {}",
                                            i,
                                            e
                                        );
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

                                Self::handle_update(
                                    merged_update,
                                    &doc_name_clone,
                                    &store_clone,
                                    &Some(redis.clone()),
                                )
                                .await;
                            }

                            if let Err(e) = redis.clear_updates(&doc_name_clone).await {
                                tracing::warn!("Failed to clear updates from Redis: {}", e);
                            }
                        }
                    }
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

        Ok(Self {
            connections: Arc::new(AtomicUsize::new(0)),
            awareness_ref: awareness,
            sender,
            awareness_updater,
            doc_sub: Some(doc_sub),
            awareness_sub: Some(awareness_sub),
            storage: None,
            redis: None,
            doc_name: None,
            storage_rx: Some(storage_rx),
            shutdown_complete: AtomicBool::new(false),
        })
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

        Self::load_from_storage(&store, &doc_name, &group.awareness_ref).await;

        let redis = if let Some(redis_config) = config.redis_config {
            match RedisStore::new(redis_config).await {
                Ok(redis) => {
                    let redis = Arc::new(redis);
                    let redis_clone = redis.clone();
                    if let Err(e) = redis_clone.subscribe_to_updates(&doc_name).await {
                        tracing::error!("Failed to subscribe to Redis updates: {}", e);
                    }
                    Some(redis)
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
        group.redis = redis;
        group.doc_name = Some(doc_name.clone());
        group.storage_rx = None;

        Ok(group)
    }

    async fn load_from_storage(store: &Arc<GcsStore>, doc_name: &str, awareness: &AwarenessRef) {
        let awareness = awareness.write().await;
        let mut txn = awareness.doc().transact_mut();

        let mut attempts = 0;
        let max_attempts = 3;
        let timeout_duration = std::time::Duration::from_secs(5);

        loop {
            attempts += 1;

            match tokio::time::timeout(timeout_duration, store.load_doc(doc_name, &mut txn)).await {
                Ok(result) => match result {
                    Ok(_) => {
                        break;
                    }
                    Err(e) => {
                        if attempts >= max_attempts {
                            tracing::error!(
                                "Failed to load document '{}' from storage after {} attempts: {}",
                                doc_name,
                                max_attempts,
                                e
                            );
                            break;
                        } else {
                            tracing::warn!(
                                "Failed to load document '{}' from storage (attempt {}/{}): {}. Retrying...", 
                                doc_name,
                                attempts,
                                max_attempts,
                                e
                            );
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                        }
                    }
                },
                Err(_) => {
                    if attempts >= max_attempts {
                        tracing::error!(
                            "Timed out loading document '{}' from storage after {} attempts",
                            doc_name,
                            max_attempts
                        );
                        break;
                    } else {
                        tracing::warn!(
                            "Timed out loading document '{}' from storage (attempt {}/{}). Retrying...", 
                            doc_name,
                            attempts,
                            max_attempts
                        );
                    }
                }
            }
        }
    }

    async fn handle_update(
        update: Vec<u8>,
        doc_name: &str,
        store: &Arc<GcsStore>,
        redis: &Option<Arc<RedisStore>>,
    ) {
        let store_future = Some(store.push_update(doc_name, &update));

        let redis_future = redis
            .as_ref()
            .map(|redis| redis.push_update(doc_name, &update));

        match (store_future, redis_future) {
            (Some(store_future), Some(redis_future)) => {
                let (store_result, redis_result) = tokio::join!(store_future, redis_future);
                if let Err(e) = store_result {
                    tracing::error!(
                        "Failed to store update in GCS for document '{}': {}",
                        doc_name,
                        e
                    );
                }
                if let Err(e) = redis_result {
                    tracing::error!(
                        "Failed to store update in Redis for document '{}': {}",
                        doc_name,
                        e
                    );
                }
            }
            (Some(store_future), None) => {
                if let Err(e) = store_future.await {
                    tracing::error!(
                        "Failed to store update in GCS for document '{}': {}",
                        doc_name,
                        e
                    );
                }
            }
            (None, Some(redis_future)) => {
                if let Err(e) = redis_future.await {
                    tracing::error!(
                        "Failed to store update in Redis for document '{}': {}",
                        doc_name,
                        e
                    );
                }
            }
            (None, None) => {}
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
            let redis = self.redis.clone();
            let doc_name = self.doc_name.clone();

            tokio::spawn(async move {
                while let Some(res) = stream.next().await {
                    if let Ok(data) = res.map_err(|e| Error::Other(Box::new(e))) {
                        if let Ok(msg) = Message::decode_v1(&data) {
                            if let Message::Sync(SyncMessage::Update(update)) = &msg {
                                if let (Some(redis), Some(doc_name)) = (&redis, &doc_name) {
                                    let redis = redis.clone();
                                    let update = update.clone();
                                    let doc_name_clone = doc_name.to_string();
                                    tokio::spawn(async move {
                                        match redis
                                            .add_to_stream(&doc_name_clone, update.as_slice())
                                            .await
                                        {
                                            Ok(_) => {}
                                            Err(e) => {
                                                tracing::error!(
                                                    "Failed to add update to Redis: {}",
                                                    e
                                                );
                                            }
                                        }
                                    });
                                }
                            }

                            if let Ok(Some(reply)) =
                                Self::handle_msg(&protocol, &awareness, msg).await
                            {
                                let mut sink = sink.lock().await;
                                let _ = sink.send(reply.encode_v1()).await;
                            }
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

    pub async fn flush_updates(&self) -> Result<(), Error> {
        if self.connection_count() > 0 {
            return Ok(());
        }

        if let (Some(store), Some(doc_name), Some(redis)) =
            (&self.storage, &self.doc_name, &self.redis)
        {
            let updates = match redis.load_updates(doc_name).await {
                Ok(updates) => updates,
                Err(e) => {
                    tracing::error!("Failed to load updates from Redis: {}", e);
                    return Ok(());
                }
            };

            if !updates.is_empty() {
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
                    Self::handle_update(merged_update, doc_name, store, &Some(redis.clone())).await;
                }

                if let Err(e) = redis.clear_updates(doc_name).await {
                    tracing::warn!("Failed to clear updates from Redis: {}", e);
                }
            }
        }

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), Error> {
        if self
            .shutdown_complete
            .swap(true, std::sync::atomic::Ordering::SeqCst)
        {
            tracing::info!("BroadcastGroup already shut down");
            return Ok(());
        }

        tracing::info!("BroadcastGroup::shutdown called");

        if let Some(sub) = &self.doc_sub {
            drop(sub.clone());
        }

        if let Some(sub) = &self.awareness_sub {
            drop(sub.clone());
        }

        self.awareness_updater.abort();

        if self.connection_count() == 0 {
            self.flush_updates().await?;
        } else {
            tracing::info!("Not flushing updates during shutdown as connections are still active");
        }

        tracing::info!("BroadcastGroup shutdown complete");
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

        if !self
            .shutdown_complete
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            tracing::warn!(
                "BroadcastGroup dropped without calling shutdown() first. Pending updates may be lost."
            );

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
