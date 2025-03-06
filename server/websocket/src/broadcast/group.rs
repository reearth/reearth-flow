use crate::storage::kv::DocOps;
//use crate::storage::sqlite::SqliteStore;
use crate::storage::gcs::GcsStore;
use crate::AwarenessRef;
use anyhow::anyhow;
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use md5;
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
    redis_pubsub: Option<Arc<Mutex<RedisConnection>>>,
    redis_pubsub_task: Option<JoinHandle<()>>,
    doc_name: Option<String>,
    redis_ttl: Option<usize>,
    storage_rx: Option<tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>>,
    // Buffer to store updates until disconnect
    pending_updates: Arc<Mutex<Vec<Vec<u8>>>>,
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
            redis_pubsub: None,
            redis_pubsub_task: None,
            doc_name: None,
            redis_ttl: None,
            storage_rx: Some(storage_rx),
            pending_updates,
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
        let redis_url = config.redis_config.as_ref().map(|c| c.url.clone());

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

                    Some(conn)
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

        if let Some(url) = redis_url {
            match Self::init_redis_connection(&url).await {
                Ok(pubsub_conn) => {
                    tracing::info!("Successfully initialized Redis pub/sub connection");
                    let pubsub_task = group.setup_redis_pubsub(doc_name.clone(), url).await;
                    group.redis_pubsub = Some(pubsub_conn);
                    group.redis_pubsub_task = Some(pubsub_task);

                    group.awareness_updater.abort();
                    group.awareness_updater = group.setup_awareness_updater_with_redis();
                }
                Err(e) => {
                    tracing::error!("Failed to initialize Redis pub/sub connection: {}", e);
                }
            }
        }

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

    async fn setup_redis_pubsub(&self, doc_name: String, redis_url: String) -> JoinHandle<()> {
        let doc_channel = format!("doc:updates:{}", doc_name);
        let awareness_channel = format!("awareness:updates:{}", doc_name);

        let awareness_ref = self.awareness_ref.clone();
        let sender = self.sender.clone();
        let pending_updates = self.pending_updates.clone();

        let instance_id = format!(
            "{:x}",
            md5::compute(format!("{}:{}", doc_name, rand::random::<u64>()))
        );

        let processed_msg_ids = Arc::new(Mutex::new(std::collections::HashSet::<String>::new()));

        tokio::spawn(async move {
            struct PubSubParams<'a> {
                redis_url: &'a str,
                doc_channel: &'a str,
                awareness_channel: &'a str,
                instance_id: &'a str,
                processed_msg_ids: &'a Arc<Mutex<std::collections::HashSet<String>>>,
                awareness_ref: &'a AwarenessRef,
                sender: &'a Sender<Vec<u8>>,
                pending_updates: &'a Arc<Mutex<Vec<Vec<u8>>>>,
            }

            async fn handle_pubsub_connection(params: PubSubParams<'_>) -> bool {
                let client = match redis::Client::open(params.redis_url) {
                    Ok(client) => client,
                    Err(e) => {
                        tracing::error!("Failed to create Redis client for pub/sub: {}", e);
                        return true;
                    }
                };

                let mut pubsub = match client.get_async_pubsub().await {
                    Ok(pubsub) => pubsub,
                    Err(e) => {
                        tracing::error!("Failed to get Redis pub/sub connection: {}", e);
                        return true;
                    }
                };

                if let Err(e) = pubsub
                    .subscribe(&[params.doc_channel, params.awareness_channel])
                    .await
                {
                    tracing::error!("Failed to subscribe to Redis channels: {}", e);
                    return true;
                }

                tracing::info!(
                    "Subscribed to Redis channels: {} and {} with instance ID: {}",
                    params.doc_channel,
                    params.awareness_channel,
                    params.instance_id
                );

                let mut msg_stream = pubsub.on_message();

                while let Some(msg) = msg_stream.next().await {
                    let channel: String = msg.get_channel().unwrap_or_default();

                    let payload_result: redis::RedisResult<(String, String, Vec<u8>)> =
                        msg.get_payload();

                    match payload_result {
                        Ok((src_instance_id, message_id, payload)) => {
                            if src_instance_id == params.instance_id {
                                tracing::debug!("Skipping message from our own instance");
                                continue;
                            }

                            let should_process = {
                                let mut processed_ids = params.processed_msg_ids.lock().await;
                                if processed_ids.contains(&message_id) {
                                    false
                                } else {
                                    processed_ids.insert(message_id.clone());
                                    if processed_ids.len() > 1000 {
                                        let to_remove: Vec<String> = processed_ids
                                            .iter()
                                            .take(processed_ids.len() - 500)
                                            .cloned()
                                            .collect();
                                        for id in to_remove {
                                            processed_ids.remove(&id);
                                        }
                                    }
                                    true
                                }
                            };

                            if !should_process {
                                tracing::debug!(
                                    "Skipping already processed message: {}",
                                    message_id
                                );
                                continue;
                            }

                            if channel == params.doc_channel {
                                tracing::debug!(
                                    "Received document update from Redis pub/sub, size: {} bytes, id: {}",
                                    payload.len(),
                                    message_id
                                );

                                if let Ok(update) = Update::decode_v1(&payload) {
                                    let awareness = params.awareness_ref.write().await;
                                    let mut txn = awareness.doc().transact_mut();

                                    if let Err(e) = txn.apply_update(update) {
                                        tracing::warn!(
                                            "Failed to apply update from Redis pub/sub: {}",
                                            e
                                        );
                                    } else {
                                        tracing::debug!(
                                            "Successfully applied update from Redis pub/sub"
                                        );

                                        let mut updates = params.pending_updates.lock().await;
                                        updates.push(payload.clone());

                                        let mut encoder = EncoderV1::new();
                                        encoder.write_var(MSG_SYNC);
                                        encoder.write_var(MSG_SYNC_UPDATE);
                                        encoder.write_buf(&payload);
                                        let msg = encoder.to_vec();

                                        if params.sender.send(msg).is_err() {
                                            tracing::warn!(
                                                "Failed to forward Redis pub/sub update to clients"
                                            );
                                        }
                                    }
                                } else {
                                    tracing::warn!("Failed to decode update from Redis pub/sub");
                                }
                            } else if channel == params.awareness_channel {
                                tracing::debug!(
                                    "Received awareness update from Redis pub/sub, size: {} bytes, id: {}",
                                    payload.len(),
                                    message_id
                                );

                                if params.sender.send(payload.clone()).is_err() {
                                    tracing::warn!(
                                        "Failed to forward Redis pub/sub awareness update to clients"
                                    );
                                }
                            }
                        }
                        Err(_) => {
                            let payload: Vec<u8> = match msg.get_payload() {
                                Ok(data) => data,
                                Err(e) => {
                                    tracing::error!(
                                        "Failed to get payload from Redis pub/sub message: {}",
                                        e
                                    );
                                    continue;
                                }
                            };

                            if channel == params.doc_channel {
                                tracing::debug!(
                                    "Received legacy document update from Redis pub/sub, size: {} bytes",
                                    payload.len()
                                );

                                if let Ok(update) = Update::decode_v1(&payload) {
                                    let awareness = params.awareness_ref.write().await;
                                    let mut txn = awareness.doc().transact_mut();

                                    if let Err(e) = txn.apply_update(update) {
                                        tracing::warn!(
                                            "Failed to apply legacy update from Redis pub/sub: {}",
                                            e
                                        );
                                    } else {
                                        tracing::debug!(
                                            "Successfully applied legacy update from Redis pub/sub"
                                        );

                                        let mut updates = params.pending_updates.lock().await;
                                        updates.push(payload.clone());

                                        let mut encoder = EncoderV1::new();
                                        encoder.write_var(MSG_SYNC);
                                        encoder.write_var(MSG_SYNC_UPDATE);
                                        encoder.write_buf(&payload);
                                        let msg = encoder.to_vec();

                                        if params.sender.send(msg).is_err() {
                                            tracing::warn!("Failed to forward legacy Redis pub/sub update to clients");
                                        }
                                    }
                                } else {
                                    tracing::warn!(
                                        "Failed to decode legacy update from Redis pub/sub"
                                    );
                                }
                            } else if channel == params.awareness_channel {
                                tracing::debug!(
                                    "Received legacy awareness update from Redis pub/sub, size: {} bytes",
                                    payload.len()
                                );

                                if params.sender.send(payload.clone()).is_err() {
                                    tracing::warn!(
                                        "Failed to forward legacy Redis pub/sub awareness update to clients"
                                    );
                                }
                            }
                        }
                    }
                }

                tracing::warn!("Redis pub/sub connection closed");
                true
            }

            let mut reconnect_attempts = 0;

            loop {
                let params = PubSubParams {
                    redis_url: &redis_url,
                    doc_channel: &doc_channel,
                    awareness_channel: &awareness_channel,
                    instance_id: &instance_id,
                    processed_msg_ids: &processed_msg_ids,
                    awareness_ref: &awareness_ref,
                    sender: &sender,
                    pending_updates: &pending_updates,
                };

                let should_retry = handle_pubsub_connection(params).await;

                if !should_retry {
                    break;
                }

                reconnect_attempts += 1;
                let delay = std::cmp::min(30, 2u64.pow(reconnect_attempts as u32));

                if reconnect_attempts > 10 {
                    tracing::error!("Too many Redis reconnection attempts, giving up");
                    break;
                }

                tracing::warn!("Reconnecting to Redis in {} seconds...", delay);
                tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
            }

            tracing::warn!("Redis pub/sub task terminated");
        })
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
            let pubsub_channel = format!("doc:updates:{}", doc_name);
            let redis = redis.clone();
            let update_clone = update.clone();
            tracing::info!(
                "Preparing to update Redis cache for document '{}'",
                doc_name
            );
            Some(async move {
                let mut conn = redis.lock().await;

                if let Err(e) = conn
                    .set_ex::<_, _, String>(
                        &cache_key,
                        update_clone.as_slice(),
                        ttl.try_into().unwrap(),
                    )
                    .await
                {
                    tracing::error!("Failed to update Redis cache: {}", e);
                } else {
                    tracing::info!(
                        "Successfully updated Redis cache for document '{}'",
                        doc_name
                    );
                }

                if let Err(e) = conn
                    .publish::<_, _, ()>(&pubsub_channel, update_clone.as_slice())
                    .await
                {
                    tracing::error!("Failed to publish update to Redis pub/sub: {}", e);
                } else {
                    tracing::info!(
                        "Successfully published update to Redis pub/sub for document '{}'",
                        doc_name
                    );
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
                    tracing::debug!(
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

    fn setup_awareness_updater_with_redis(&self) -> JoinHandle<()> {
        let awareness_c = Arc::downgrade(&self.awareness_ref);
        let sink = self.sender.clone();
        let redis = self.redis.clone();
        let doc_name = self.doc_name.clone();
        let connections = self.connections.clone();
        let awareness_ref = self.awareness_ref.clone();
        let awareness_sub = self.awareness_sub.clone();

        tokio::task::spawn(async move {
            let lock = awareness_ref.write().await;

            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

            let _awareness_sub = lock.on_update(move |_awareness, event, _origin| {
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
                    tracing::debug!("Failed to send awareness update - receiver likely dropped");
                }
            });
            drop(lock);

            if let Some(sub) = awareness_sub {
                drop(sub);
            }

            while let Some(changed_clients) = rx.recv().await {
                if let Some(awareness) = awareness_c.upgrade() {
                    let awareness = awareness.read().await;
                    match awareness.update_with_clients(changed_clients) {
                        Ok(update) => {
                            let encoded_update = Message::Awareness(update).encode_v1();

                            let connection_count = connections.load(Ordering::Relaxed);
                            if connection_count > 0 {
                                if let Err(e) = sink.send(encoded_update.clone()) {
                                    tracing::error!("Failed to send awareness update: {}", e);
                                }
                            }

                            if let Some(redis) = &redis {
                                if let Some(doc_name) = &doc_name {
                                    let awareness_channel =
                                        format!("awareness:updates:{}", doc_name);
                                    let redis_clone = redis.clone();
                                    let encoded_update_clone = encoded_update.clone();
                                    let doc_name_clone = doc_name.clone();

                                    tokio::spawn(async move {
                                        let mut conn = redis_clone.lock().await;
                                        if let Err(e) = conn
                                            .publish::<_, _, ()>(
                                                &awareness_channel,
                                                encoded_update_clone.as_slice(),
                                            )
                                            .await
                                        {
                                            tracing::error!(
                                                "Failed to publish awareness update to Redis: {}",
                                                e
                                            );
                                        } else {
                                            tracing::debug!(
                                                "Published awareness update to Redis for document '{}'",
                                                doc_name_clone
                                            );
                                        }
                                    });
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to update awareness: {}", e);
                        }
                    }
                }
            }
        })
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
        if let Some(task) = self.redis_pubsub_task.take() {
            task.abort();
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
                    let mut processed_updates = std::collections::HashSet::new();
                    let mut all_updates = Vec::new();

                    {
                        let mut pending = pending_updates.lock().await;
                        tracing::info!(
                            "Found {} pending updates for document '{}'",
                            pending.len(),
                            doc_name
                        );

                        for update in pending.iter() {
                            let update_hash = format!("{:x}", md5::compute(update));
                            if processed_updates.insert(update_hash) {
                                all_updates.push(update.clone());
                            }
                        }
                        pending.clear();
                    }

                    if let Some(redis_conn) = &redis {
                        let redis_key = format!("pending_updates:{}", doc_name);
                        match redis_conn.lock().await.lrange::<_, Vec<Vec<u8>>>(&redis_key, 0, -1).await {
                            Ok(redis_updates) => {
                                tracing::info!(
                                    "Found {} pending updates in Redis for document '{}'",
                                    redis_updates.len(),
                                    doc_name
                                );

                                for update in redis_updates {
                                    let update_hash = format!("{:x}", md5::compute(&update));
                                    if processed_updates.insert(update_hash) {
                                        all_updates.push(update);
                                    }
                                }

                                if let Err(e) = redis_conn.lock().await.del::<_, ()>(&redis_key).await {
                                    tracing::error!("Failed to clear Redis pending updates: {}", e);
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to load updates from Redis: {}", e);
                            }
                        }
                        
                        let pubsub_channel = format!("doc:status:{}", doc_name);
                        let status_message = serde_json::json!({
                            "type": "instance_shutdown",
                            "timestamp": chrono::Utc::now().timestamp(),
                            "doc_id": doc_name
                        }).to_string();
                        
                        if let Err(e) = redis_conn.lock().await.publish::<_, _, ()>(&pubsub_channel, status_message).await {
                            tracing::error!("Failed to publish shutdown status to Redis: {}", e);
                        } else {
                            tracing::info!("Published shutdown status to Redis for document '{}'", doc_name);
                        }
                    }

                    if all_updates.is_empty() {
                        tracing::info!("No updates to store for document '{}'", doc_name);
                        return;
                    }

                    let doc = Doc::new();
                    let mut txn = doc.transact_mut();
                    let mut has_updates = false;

                    for (i, update) in all_updates.iter().enumerate() {
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

                    let max_retries = 3;
                    let mut retry_count = 0;
                    let mut last_error = None;

                    while retry_count < max_retries {
                        tracing::info!(
                            "Attempt {} to store merged update for document '{}'",
                            retry_count + 1,
                            doc_name
                        );

                        match store.push_update(&doc_name, &merged_update).await {
                            Ok(_) => {
                                tracing::info!(
                                    "Successfully stored merged updates on disconnect for document '{}'",
                                    doc_name
                                );

                                if let (Some(redis_conn), Some(ttl)) = (&redis, redis_ttl) {
                                    let cache_key = format!("doc:{}", doc_name);
                                    if let Err(e) = redis_conn.lock().await
                                        .set_ex::<_, _, String>(&cache_key, merged_update.as_slice(), ttl.try_into().unwrap())
                                        .await
                                    {
                                        tracing::error!("Failed to update Redis cache: {}", e);
                                    }
                                    
                                    // Also publish the final merged update to Redis pub/sub
                                    let pubsub_channel = format!("doc:updates:{}", doc_name);
                                    if let Err(e) = redis_conn.lock().await
                                        .publish::<_, _, ()>(&pubsub_channel, merged_update.as_slice())
                                        .await
                                    {
                                        tracing::error!("Failed to publish final update to Redis pub/sub: {}", e);
                                    } else {
                                        tracing::info!("Published final merged update to Redis pub/sub for document '{}'", doc_name);
                                    }
                                }

                                return;
                            }
                            Err(e) => {
                                last_error = Some(e.to_string());
                                tracing::error!(
                                    "Failed to store update in GCS (attempt {}/{}): {}",
                                    retry_count + 1,
                                    max_retries,
                                    last_error.as_ref().unwrap()
                                );
                                retry_count += 1;
                                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                            }
                        }
                    }

                    tracing::error!(
                        "CRITICAL: Failed to store updates after {} attempts: {}",
                        max_retries,
                        last_error.unwrap_or_else(|| "Unknown error".to_string())
                    );
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
