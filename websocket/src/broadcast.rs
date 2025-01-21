use crate::storage::kv::DocOps;
//use crate::storage::sqlite::SqliteStore;
use crate::storage::gcs::GcsStore;
use crate::AwarenessRef;
use futures_util::{SinkExt, StreamExt};
use redis::aio::MultiplexedConnection as RedisConnection;
use redis::AsyncCommands;
use serde::Deserialize;
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
use yrs::{Transact, Update};

/// Redis cache configuration
#[derive(Debug, Clone, Default, Deserialize)]
pub struct RedisConfig {
    /// Redis URL
    pub url: String,
    /// Cache TTL in seconds
    pub ttl: u64,
}

/// Connection configuration options
#[derive(Debug, Clone)]
pub struct BroadcastConfig {
    /// Whether to enable persistent storage
    pub storage_enabled: bool,
    /// Document name for storage (required if storage enabled)
    pub doc_name: Option<String>,
    /// Redis configuration (optional)
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
    redis: Option<Arc<Mutex<RedisConnection>>>,
    doc_name: Option<String>,
    redis_ttl: Option<usize>,
    storage_rx: Option<tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>>,
}

unsafe impl Send for BroadcastGroup {}
unsafe impl Sync for BroadcastGroup {}

impl BroadcastGroup {
    pub fn increment_connections(&self) {
        self.connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_connections(&self) -> usize {
        self.connections.fetch_sub(1, Ordering::Relaxed)
    }

    pub fn connection_count(&self) -> usize {
        self.connections.load(Ordering::Relaxed)
    }

    pub async fn new(awareness: AwarenessRef, buffer_capacity: usize) -> Self {
        let (sender, _receiver) = channel(buffer_capacity);
        let awareness_c = Arc::downgrade(&awareness);
        let mut lock = awareness.write().await;
        let sink = sender.clone();

        let (storage_tx, storage_rx) = tokio::sync::mpsc::unbounded_channel();

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

                    // Send update to storage channel
                    if let Err(e) = storage_tx.send(u.update.clone()) {
                        tracing::error!("Failed to send update to storage channel: {}", e);
                    }
                })
                .unwrap()
        };

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let sink = sender.clone();

        let awareness_sub = lock.on_update(move |_awareness, event, _origin| {
            let added = event.added();
            let updated = event.updated();
            let removed = event.removed();
            // Pre-calculate total capacity needed
            let total_len = added.len() + updated.len() + removed.len();
            if total_len == 0 {
                return; // Early return if no changes
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

        BroadcastGroup {
            connections: Arc::new(AtomicUsize::new(0)),
            awareness_ref: awareness,
            awareness_updater,
            sender,
            doc_sub: Some(doc_sub),
            awareness_sub: Some(awareness_sub),
            storage: None,
            redis: None,
            doc_name: None,
            redis_ttl: None,
            storage_rx: Some(storage_rx),
        }
    }

    pub async fn with_storage(
        awareness: AwarenessRef,
        buffer_capacity: usize,
        store: Arc<GcsStore>,
        config: BroadcastConfig,
    ) -> Self {
        if !config.storage_enabled {
            return Self::new(awareness, buffer_capacity).await;
        }

        let mut group = Self::new(awareness, buffer_capacity).await;

        // Set storage-related fields
        let doc_name = config
            .doc_name
            .expect("doc_name required when storage enabled");
        let redis_ttl = config.redis_config.as_ref().map(|c| c.ttl as usize);

        // Initialize Redis and load data
        let redis = if let Some(redis_config) = config.redis_config {
            match Self::init_redis_connection(&redis_config.url).await {
                Ok(conn) => {
                    if (Self::load_from_redis(&conn, &doc_name, &group.awareness_ref).await)
                        .is_err()
                    {
                        Self::load_from_storage(&store, &doc_name, &group.awareness_ref).await;
                    }
                    Some(conn)
                }
                Err(e) => {
                    tracing::error!("Failed to initialize Redis connection: {}", e);
                    Self::load_from_storage(&store, &doc_name, &group.awareness_ref).await;
                    None
                }
            }
        } else {
            Self::load_from_storage(&store, &doc_name, &group.awareness_ref).await;
            None
        };

        group.storage = Some(store);
        group.redis = redis;
        group.doc_name = Some(doc_name.clone());
        group.redis_ttl = redis_ttl;

        // Start storage processing task
        if let Some(mut rx) = group.storage_rx.take() {
            let store = group.storage.clone().unwrap();
            let redis = group.redis.clone();
            let redis_ttl = group.redis_ttl;

            tokio::spawn(async move {
                while let Some(update) = rx.recv().await {
                    Self::handle_update(update, &doc_name, &store, &redis, redis_ttl).await;
                }
            });
        }

        group
    }

    async fn init_redis_connection(
        url: &str,
    ) -> Result<Arc<Mutex<RedisConnection>>, redis::RedisError> {
        let client = redis::Client::open(url)?;
        let conn = client.get_multiplexed_async_connection().await?;
        Ok(Arc::new(Mutex::new(conn)))
    }

    async fn load_from_redis(
        redis: &Arc<Mutex<RedisConnection>>,
        doc_name: &str,
        awareness: &AwarenessRef,
    ) -> Result<(), Error> {
        let mut conn = redis.lock().await;
        let cache_key = format!("doc:{}", doc_name);

        let cached_data: Vec<u8> = conn
            .get(&cache_key)
            .await
            .map_err(|e| Error::Other(e.into()))?;
        let update = Update::decode_v1(&cached_data)?;

        let awareness_guard = awareness.write().await;
        let mut txn = awareness_guard.doc().transact_mut();
        txn.apply_update(update)?;

        tracing::debug!("Successfully loaded document from Redis cache");
        Ok(())
    }

    async fn load_from_storage(store: &Arc<GcsStore>, doc_name: &str, awareness: &AwarenessRef) {
        let awareness = awareness.write().await;
        let mut txn = awareness.doc().transact_mut();
        match store.load_doc(doc_name, &mut txn).await {
            Ok(_) => {
                tracing::debug!("Successfully loaded document '{}' from storage", doc_name);
            }
            Err(e) => {
                tracing::error!("Failed to load document '{}' from storage: {}", doc_name, e);
            }
        }
    }

    async fn handle_update(
        update: Vec<u8>,
        doc_name: &str,
        store: &Arc<GcsStore>,
        redis: &Option<Arc<Mutex<RedisConnection>>>,
        redis_ttl: Option<usize>,
    ) {
        // Store in persistent storage and update Redis cache in parallel
        let store_future = store.push_update(doc_name, &update);

        let redis_future = if let (Some(redis), Some(ttl)) = (redis, redis_ttl) {
            let cache_key = format!("doc:{}", doc_name);
            let redis = redis.clone();
            let update = update.clone();
            Some(async move {
                let mut conn = redis.lock().await;
                if let Err(e) = conn
                    .set_ex::<_, _, String>(&cache_key, update.as_slice(), ttl.try_into().unwrap())
                    .await
                {
                    tracing::error!("Failed to update Redis cache: {}", e);
                }
            })
        } else {
            None
        };

        // Execute both operations concurrently if Redis is enabled
        match redis_future {
            Some(redis_future) => {
                let (store_result, _) = tokio::join!(store_future, redis_future);
                if let Err(e) = store_result {
                    tracing::error!("Failed to store update in GCS: {}", e);
                } else {
                    tracing::debug!("Successfully stored update in GCS {:?}", store_result);
                }
            }
            None => {
                if let Err(e) = store_future.await {
                    tracing::error!("Failed to store update in GCS: {}", e);
                } else {
                    tracing::debug!("Successfully stored update in GCS");
                }
            }
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
            let sink = sink.clone();
            let mut receiver = self.sender.subscribe();
            tokio::spawn(async move {
                while let Ok(msg) = receiver.recv().await {
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
            tokio::spawn(async move {
                while let Some(res) = stream.next().await {
                    if let Ok(data) = res.map_err(|e| Error::Other(Box::new(e))) {
                        if let Ok(msg) = Message::decode_v1(&data) {
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
}

impl Drop for BroadcastGroup {
    fn drop(&mut self) {
        // Cancel all subscriptions
        if let Some(sub) = self.doc_sub.take() {
            drop(sub);
        }
        if let Some(sub) = self.awareness_sub.take() {
            drop(sub);
        }
        self.awareness_updater.abort();
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
