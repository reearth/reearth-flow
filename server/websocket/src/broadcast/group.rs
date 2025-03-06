use crate::storage::kv::DocOps;
//use crate::storage::sqlite::SqliteStore;
use crate::storage::gcs::GcsStore;
use crate::AwarenessRef;
use anyhow::anyhow;
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use rand;
use redis::aio::MultiplexedConnection as RedisConnection;
use redis::AsyncCommands;
use serde::Deserialize;
use serde_json;
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

// Redis stream names and keys
const REDIS_STREAM_PREFIX: &str = "yjs";
const REDIS_WORKER_GROUP: &str = "yjs:worker";

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RedisConfig {
    /// Redis URL
    pub url: String,
    /// Cache TTL in seconds
    pub ttl: u64,
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
    redis: Option<Arc<Mutex<RedisConnection>>>,
    doc_name: Option<String>,
    redis_ttl: Option<usize>,
    storage_rx: Option<tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>>,
    // Buffer to store updates until disconnect
    pending_updates: Arc<Mutex<Vec<Vec<u8>>>>,
    // Redis subscriber task for receiving updates from other instances
    redis_subscriber_task: Option<JoinHandle<()>>,
    // Consumer name for Redis streams
    redis_consumer_name: Option<String>,
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
        self.connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_connections(&self) -> usize {
        self.connections.fetch_sub(1, Ordering::Relaxed)
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
            redis: None,
            doc_name: None,
            redis_ttl: None,
            storage_rx: Some(storage_rx),
            pending_updates,
            redis_subscriber_task: None,
            redis_consumer_name: None,
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

        tracing::info!("Loading document '{}' directly from GCS storage", doc_name);
        Self::load_from_storage(&store, &doc_name, &group.awareness_ref).await;

        let redis = if let Some(redis_config) = config.redis_config {
            match Self::init_redis_connection(&redis_config.url).await {
                Ok(conn) => {
                    tracing::info!("Successfully initialized Redis connection for pending updates");

                    let redis_key = format!("pending_updates:{}", doc_name);
                    let conn_clone = conn.clone();
                    let awareness_clone = group.awareness_ref.clone();
                    let pending_clone = group.pending_updates.clone();

                    tokio::spawn(async move {
                        let mut redis_conn = conn_clone.lock().await;
                        match redis_conn
                            .lrange::<_, Vec<Vec<u8>>>(&redis_key, 0, -1)
                            .await
                        {
                            Ok(updates) => {
                                tracing::info!(
                                    "Loaded {} pending updates from Redis",
                                    updates.len()
                                );

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
                                            } else {
                                                tracing::debug!(
                                                    "Successfully applied update from Redis"
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

                    // Initialize Redis Stream group for multi-instance sync
                    let stream_name = Self::compute_redis_stream_name(&doc_name);
                    let consumer_name = format!("instance-{}", rand::random::<u32>());

                    // Try to create the stream and consumer group
                    let mut redis_conn = conn.lock().await;
                    let create_result: redis::RedisResult<String> = redis::cmd("XGROUP")
                        .arg("CREATE")
                        .arg(&stream_name)
                        .arg(REDIS_WORKER_GROUP)
                        .arg("0")
                        .arg("MKSTREAM")
                        .query_async(&mut *redis_conn)
                        .await;

                    match create_result {
                        Ok(_) => {
                            tracing::info!(
                                "Created Redis stream and consumer group for document '{}'",
                                doc_name
                            );
                        }
                        Err(e) => {
                            // Group may already exist, which is fine
                            tracing::debug!("Redis group creation result: {}", e);
                        }
                    }
                    drop(redis_conn); // 释放锁，避免借用冲突

                    // Set up Redis subscriber task to receive updates from other instances
                    let awareness_for_sub = group.awareness_ref.clone();
                    let sender_for_sub = group.sender.clone();
                    let doc_name_for_sub = doc_name.clone();
                    let redis_url = redis_config.url.clone();

                    let redis_subscriber_task = tokio::spawn(async move {
                        // 使用新的连接而不是借用现有连接
                        tracing::info!(
                            "Starting Redis subscriber task for document '{}'",
                            doc_name_for_sub
                        );

                        // Create a separate connection for pub/sub
                        match redis::Client::open(redis_url) {
                            Ok(client) => {
                                // Use get_async_pubsub() instead of get_multiplexed_async_connection()
                                match client.get_async_pubsub().await {
                                    Ok(mut pubsub) => {
                                        // Subscribe to the document's channel
                                        let channel = format!("yjs:updates:{}", doc_name_for_sub);
                                        match pubsub.subscribe(&channel).await {
                                            Ok(_) => {
                                                let mut stream = pubsub.on_message();

                                                tracing::info!(
                                                    "Subscribed to Redis channel: {}",
                                                    channel
                                                );

                                                // Listen for messages
                                                while let Some(msg) = stream.next().await {
                                                    match msg.get_payload::<Vec<u8>>() {
                                                        Ok(payload) => {
                                                            // Apply the update to our local document
                                                            let awareness =
                                                                awareness_for_sub.write().await;
                                                            let mut txn =
                                                                awareness.doc().transact_mut();

                                                            if let Ok(decoded) =
                                                                Update::decode_v1(&payload)
                                                            {
                                                                if let Err(e) =
                                                                    txn.apply_update(decoded)
                                                                {
                                                                    tracing::warn!("Failed to apply update from Redis: {}", e);
                                                                } else {
                                                                    // Broadcast to local clients
                                                                    let _ = sender_for_sub
                                                                        .send(payload);
                                                                    tracing::debug!("Applied and broadcasted update from Redis");
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            tracing::error!("Failed to get payload from Redis message: {}", e);
                                                        }
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                tracing::error!(
                                                    "Failed to subscribe to Redis channel: {}",
                                                    e
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            "Failed to get async connection to Redis: {}",
                                            e
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to open Redis client: {}", e);
                            }
                        }
                    });

                    group.redis_subscriber_task = Some(redis_subscriber_task);
                    group.redis_consumer_name = Some(consumer_name);

                    Some(conn.clone())
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
        group.redis_ttl = redis_ttl;

        group.storage_rx = None;

        Ok(group)
    }

    async fn init_redis_connection(
        url: &str,
    ) -> Result<Arc<Mutex<RedisConnection>>, redis::RedisError> {
        let client = redis::Client::open(url)?;
        let conn = client.get_multiplexed_async_connection().await?;
        Ok(Arc::new(Mutex::new(conn)))
    }

    #[allow(dead_code)]
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
        tracing::info!(
            "handle_update called for document '{}' with update size {} bytes",
            doc_name,
            update.len()
        );

        tracing::info!("Pushing update to GCS for document '{}'", doc_name);
        let store_future = store.push_update(doc_name, &update);

        let redis_future = if let (Some(redis), Some(ttl)) = (redis, redis_ttl) {
            let cache_key = format!("doc:{}", doc_name);
            let redis_key = format!("pending_updates:{}", doc_name);
            let channel = format!("yjs:updates:{}", doc_name);
            let stream_name = Self::compute_redis_stream_name(doc_name);
            let redis = redis.clone();
            let update = update.clone();

            tracing::info!("Preparing to update Redis for document '{}'", doc_name);

            Some(async move {
                let mut conn = redis.lock().await;

                if let Err(e) = conn
                    .set_ex::<_, _, String>(&cache_key, update.as_slice(), ttl.try_into().unwrap())
                    .await
                {
                    tracing::error!("Failed to update Redis cache: {}", e);
                } else {
                    tracing::info!(
                        "Successfully updated Redis cache for document '{}'",
                        doc_name
                    );
                }

                if let Err(e) = conn.lpush::<_, _, ()>(&redis_key, update.as_slice()).await {
                    tracing::error!(
                        "Failed to store update to Redis pending_updates list: {}",
                        e
                    );
                } else {
                    tracing::debug!("Successfully stored update to Redis pending_updates list");
                    let _ = conn
                        .expire::<_, ()>(&redis_key, ttl.try_into().unwrap())
                        .await;
                }

                if let Err(e) = conn.publish::<_, _, ()>(&channel, update.as_slice()).await {
                    tracing::error!("Failed to publish update to Redis channel: {}", e);
                } else {
                    tracing::debug!("Successfully published update to Redis channel");
                }

                let cmd_result: redis::RedisResult<String> = redis::cmd("XADD")
                    .arg(&stream_name)
                    .arg("*")
                    .arg("message")
                    .arg(update.as_slice())
                    .query_async(&mut *conn)
                    .await;

                if let Err(e) = cmd_result {
                    tracing::error!("Failed to add update to Redis stream: {}", e);
                } else {
                    tracing::debug!("Successfully added update to Redis stream");
                }
            })
        } else {
            tracing::info!("Redis not configured for document '{}'", doc_name);
            None
        };

        match redis_future {
            Some(redis_future) => {
                tracing::info!(
                    "Executing GCS and Redis operations concurrently for document '{}'",
                    doc_name
                );
                let (store_result, _) = tokio::join!(store_future, redis_future);
                if let Err(e) = store_result {
                    tracing::error!(
                        "Failed to store update in GCS for document '{}': {}",
                        doc_name,
                        e
                    );
                } else {
                    tracing::info!(
                        "Successfully stored update in GCS for document '{}': {:?}",
                        doc_name,
                        store_result
                    );
                }
            }
            None => {
                tracing::info!("Executing GCS operation only for document '{}'", doc_name);
                if let Err(e) = store_future.await {
                    tracing::error!(
                        "Failed to store update in GCS for document '{}': {}",
                        doc_name,
                        e
                    );
                } else {
                    tracing::info!(
                        "Successfully stored update in GCS for document '{}'",
                        doc_name
                    );
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
            let redis_ttl = self.redis_ttl;

            tokio::spawn(async move {
                while let Some(res) = stream.next().await {
                    if let Ok(data) = res.map_err(|e| Error::Other(Box::new(e))) {
                        if let Ok(msg) = Message::decode_v1(&data) {
                            if let Message::Sync(SyncMessage::Update(update)) = &msg {
                                tracing::debug!(
                                    "Received update, sending to Redis, size: {} bytes",
                                    update.len()
                                );

                                if let (Some(redis), Some(ttl), Some(doc_name)) =
                                    (&redis, redis_ttl, &doc_name)
                                {
                                    let redis_key = format!("pending_updates:{}", doc_name);
                                    let redis = redis.clone();
                                    let update = update.clone();
                                    let channel = format!("yjs:updates:{}", doc_name);
                                    let stream_name =
                                        format!("{}:room:{}", REDIS_STREAM_PREFIX, doc_name);

                                    tokio::spawn(async move {
                                        let mut conn = redis.lock().await;

                                        match conn
                                            .lpush::<_, _, ()>(&redis_key, update.as_slice())
                                            .await
                                        {
                                            Ok(_) => {
                                                tracing::debug!("Successfully stored update to Redis pending_updates list");
                                                let _ = conn
                                                    .expire::<_, ()>(
                                                        &redis_key,
                                                        ttl.try_into().unwrap(),
                                                    )
                                                    .await;
                                            }
                                            Err(e) => {
                                                tracing::error!(
                                                    "Failed to store update to Redis: {}",
                                                    e
                                                );
                                            }
                                        }

                                        match conn
                                            .publish::<_, _, ()>(&channel, update.as_slice())
                                            .await
                                        {
                                            Ok(_) => {
                                                tracing::debug!("Successfully published update to Redis channel");
                                            }
                                            Err(e) => {
                                                tracing::error!(
                                                    "Failed to publish update to Redis channel: {}",
                                                    e
                                                );
                                            }
                                        }

                                        let cmd_result: redis::RedisResult<String> =
                                            redis::cmd("XADD")
                                                .arg(&stream_name)
                                                .arg("*")
                                                .arg("message")
                                                .arg(update.as_slice())
                                                .query_async(&mut *conn)
                                                .await;

                                        match cmd_result {
                                            Ok(id) => {
                                                tracing::debug!("Successfully added update to Redis stream with ID: {}", id);
                                            }
                                            Err(e) => {
                                                tracing::error!(
                                                    "Failed to add update to Redis stream: {}",
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
        if let (Some(store), Some(doc_name)) = (&self.storage, &self.doc_name) {
            tracing::info!("Manually flushing updates for document '{}'", doc_name);

            let updates = {
                let mut pending = self.pending_updates.lock().await;
                tracing::info!(
                    "Found {} pending updates for document '{}'",
                    pending.len(),
                    doc_name
                );
                if pending.is_empty() {
                    if let Some(redis) = &self.redis {
                        let redis_key = format!("pending_updates:{}", doc_name);
                        let mut redis_conn = redis.lock().await;
                        match redis_conn
                            .lrange::<_, Vec<Vec<u8>>>(&redis_key, 0, -1)
                            .await
                        {
                            Ok(redis_updates) => {
                                if !redis_updates.is_empty() {
                                    tracing::info!(
                                        "Found {} pending updates in Redis for document '{}'",
                                        redis_updates.len(),
                                        doc_name
                                    );
                                    return self
                                        .flush_redis_updates(redis_updates, doc_name, store)
                                        .await;
                                }
                            }
                            Err(e) => {
                                tracing::warn!("Failed to load pending updates from Redis: {}", e);
                            }
                        }
                    }

                    tracing::info!("No updates to store for document '{}'", doc_name);
                    return Ok(());
                }
                std::mem::take(&mut *pending)
            };

            let result = self.apply_and_store_updates(updates, doc_name, store).await;

            if let Some(redis) = &self.redis {
                let redis_key = format!("pending_updates:{}", doc_name);
                let mut redis_conn = redis.lock().await;
                if let Err(e) = redis_conn.del::<_, ()>(&redis_key).await {
                    tracing::warn!("Failed to clear pending updates from Redis: {}", e);
                } else {
                    tracing::info!(
                        "Cleared pending updates from Redis for document '{}'",
                        doc_name
                    );
                }
            }

            result
        } else {
            tracing::info!("No storage or doc_name available, not storing updates");
            Ok(())
        }
    }

    async fn apply_and_store_updates(
        &self,
        updates: Vec<Vec<u8>>,
        doc_name: &str,
        store: &Arc<GcsStore>,
    ) -> Result<(), Error> {
        let doc = Doc::new();
        let mut txn = doc.transact_mut();

        let mut has_updates = false;
        for (i, update) in updates.iter().enumerate() {
            if let Ok(decoded) = Update::decode_v1(update) {
                if let Err(e) = txn.apply_update(decoded) {
                    tracing::warn!("Failed to apply update {} during manual flush: {}", i, e);
                } else {
                    has_updates = true;
                    tracing::info!(
                        "Successfully applied update {} for document '{}'",
                        i,
                        doc_name
                    );
                }
            } else {
                tracing::warn!("Failed to decode update {} during manual flush", i);
            }
        }

        if !has_updates {
            tracing::info!("No valid updates to store for document '{}'", doc_name);
            return Ok(());
        }

        let state_vector = StateVector::default();
        let merged_update = txn.encode_state_as_update_v1(&state_vector);
        tracing::info!(
            "Created merged update of size {} bytes for document '{}'",
            merged_update.len(),
            doc_name
        );

        tracing::info!("Storing merged update for document '{}'", doc_name);
        Self::handle_update(merged_update, doc_name, store, &self.redis, self.redis_ttl).await;
        tracing::info!("Stored merged updates for document '{}'", doc_name);

        Ok(())
    }

    async fn flush_redis_updates(
        &self,
        updates: Vec<Vec<u8>>,
        doc_name: &str,
        store: &Arc<GcsStore>,
    ) -> Result<(), Error> {
        let result = self.apply_and_store_updates(updates, doc_name, store).await;

        if let Some(redis) = &self.redis {
            let redis_key = format!("pending_updates:{}", doc_name);
            let mut redis_conn = redis.lock().await;
            if let Err(e) = redis_conn.del::<_, ()>(&redis_key).await {
                tracing::warn!("Failed to clear pending updates from Redis: {}", e);
            } else {
                tracing::info!(
                    "Cleared pending updates from Redis for document '{}'",
                    doc_name
                );
            }
        }

        result
    }

    fn compute_redis_stream_name(doc_name: &str) -> String {
        format!("{}:room:{}", REDIS_STREAM_PREFIX, doc_name)
    }
}

impl Drop for BroadcastGroup {
    fn drop(&mut self) {
        tracing::info!("BroadcastGroup::drop called");

        if let Some(sub) = self.doc_sub.take() {
            drop(sub);
        }

        if let Some(sub) = self.awareness_sub.take() {
            drop(sub);
        }

        if let Some(task) = self.redis_subscriber_task.take() {
            task.abort();
            tracing::info!("Aborted Redis subscriber task");
        }

        self.awareness_updater.abort();

        if let (Some(store), Some(doc_name)) = (&self.storage, &self.doc_name) {
            let doc_name_log = doc_name.clone();
            tracing::info!("Preparing to store updates for document '{}'", doc_name_log);

            if let Ok(rt) = tokio::runtime::Handle::try_current() {
                let store = store.clone();
                let doc_name = doc_name.clone();
                let redis = self.redis.clone();
                let redis_ttl = self.redis_ttl;
                let pending_updates = self.pending_updates.clone();

                rt.spawn(async move {
                    let updates = {
                        let mut pending = pending_updates.lock().await;
                        tracing::info!(
                            "Found {} pending updates for document '{}'",
                            pending.len(),
                            doc_name
                        );

                        if pending.is_empty() && redis.is_some() {
                            let redis_key = format!("pending_updates:{}", doc_name);
                            let mut redis_conn = redis.as_ref().unwrap().lock().await;
                            match redis_conn
                                .lrange::<_, Vec<Vec<u8>>>(&redis_key, 0, -1)
                                .await
                            {
                                Ok(redis_updates) => {
                                    if !redis_updates.is_empty() {
                                        tracing::info!(
                                            "Found {} pending updates in Redis for document '{}'",
                                            redis_updates.len(),
                                            doc_name
                                        );
                                        redis_updates
                                    } else {
                                        Vec::new()
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Failed to load pending updates from Redis: {}",
                                        e
                                    );
                                    Vec::new()
                                }
                            }
                        } else if pending.is_empty() {
                            tracing::info!("No updates to store for document '{}'", doc_name);
                            Vec::new()
                        } else {
                            std::mem::take(&mut *pending)
                        }
                    };

                    if updates.is_empty() {
                        return;
                    }

                    let doc = Doc::new();
                    let mut txn = doc.transact_mut();

                    let mut has_updates = false;
                    for (i, update) in updates.iter().enumerate() {
                        if let Ok(decoded) = Update::decode_v1(update) {
                            if let Err(e) = txn.apply_update(decoded) {
                                tracing::warn!(
                                    "Failed to apply update {} during merge in Drop: {}",
                                    i,
                                    e
                                );
                            } else {
                                has_updates = true;
                                tracing::info!(
                                    "Successfully applied update {} for document '{}'",
                                    i,
                                    doc_name
                                );
                            }
                        } else {
                            tracing::warn!("Failed to decode update {} during merge in Drop", i);
                        }
                    }

                    if !has_updates {
                        tracing::info!("No valid updates to store for document '{}'", doc_name);
                        return;
                    }

                    let state_vector = StateVector::default();
                    let merged_update = txn.encode_state_as_update_v1(&state_vector);
                    tracing::info!(
                        "Created merged update of size {} bytes for document '{}'",
                        merged_update.len(),
                        doc_name
                    );

                    tracing::info!("Storing merged update for document '{}'", doc_name);
                    Self::handle_update(merged_update, &doc_name, &store, &redis, redis_ttl).await;
                    tracing::info!(
                        "Stored merged updates on disconnect for document '{}'",
                        doc_name
                    );

                    if let Some(redis) = &redis {
                        let redis_key = format!("pending_updates:{}", doc_name);
                        let mut redis_conn = redis.lock().await;
                        if let Err(e) = redis_conn.del::<_, ()>(&redis_key).await {
                            tracing::warn!("Failed to clear pending updates from Redis: {}", e);
                        } else {
                            tracing::info!(
                                "Cleared pending updates from Redis for document '{}'",
                                doc_name
                            );
                        }
                    }
                });
                tracing::info!(
                    "Spawned task to store updates for document '{}'",
                    doc_name_log
                );
            } else {
                tracing::error!("Failed to get tokio runtime handle for storing merged updates");
            }
        } else {
            tracing::info!("No storage or doc_name available, not storing updates");
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
