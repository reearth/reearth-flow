use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::AwarenessRef;
use anyhow::anyhow;
use anyhow::Result;
use bb8_redis::{bb8, RedisConnectionManager};
use futures_util::{SinkExt, StreamExt};
use rand;
use redis::AsyncCommands;
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

const REDIS_STREAM_PREFIX: &str = "yjs";
const REDIS_WORKER_GROUP: &str = "yjs:worker";

type RedisStreamEntry = (String, Vec<(String, Vec<u8>)>);
type RedisStreamData = Vec<RedisStreamEntry>;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub ttl: u64,
}

impl RedisConfig {
    pub fn new(url: String) -> Self {
        Self { url, ttl: 3600 }
    }
}

#[derive(Debug, Clone)]
pub struct BroadcastConfig {
    pub storage_enabled: bool,
    pub doc_name: Option<String>,
    pub redis_config: Option<RedisConfig>,
}

type RedisPool = bb8::Pool<RedisConnectionManager>;

pub struct BroadcastGroup {
    connections: Arc<AtomicUsize>,
    awareness_ref: AwarenessRef,
    sender: Sender<Vec<u8>>,
    awareness_updater: JoinHandle<()>,
    doc_sub: Option<yrs::Subscription>,
    awareness_sub: Option<yrs::Subscription>,
    storage: Option<Arc<GcsStore>>,
    redis_pool: Option<Arc<RedisPool>>,
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
        let prev_count = self.connections.fetch_sub(1, Ordering::Relaxed);

        if let (Some(store), Some(doc_name)) = (&self.storage, &self.doc_name) {
            let store_clone = store.clone();
            let doc_name_clone = doc_name.clone();
            let redis = self.redis_pool.clone();
            let redis_ttl = self.redis_ttl;
            let pending_updates = self.pending_updates.clone();

            tokio::spawn(async move {
                let updates = {
                    let mut pending = pending_updates.lock().await;
                    if pending.is_empty() {
                        if let Some(redis_pool) = &redis {
                            let redis_key = format!("pending_updates:{}", doc_name_clone);
                            let mut redis_conn = redis_pool.get().await.unwrap();
                            match redis_conn
                                .lrange::<_, Vec<Vec<u8>>>(&redis_key, 0, -1)
                                .await
                            {
                                Ok(redis_updates) => {
                                    if !redis_updates.is_empty() {
                                        let doc = Doc::new();
                                        let mut txn = doc.transact_mut();

                                        let mut has_updates = false;
                                        for (i, update) in redis_updates.iter().enumerate() {
                                            if let Ok(decoded) = Update::decode_v1(update) {
                                                if let Err(e) = txn.apply_update(decoded) {
                                                    tracing::warn!("Failed to apply update {} during flush: {}", i, e);
                                                } else {
                                                    has_updates = true;
                                                }
                                            } else {
                                                tracing::warn!(
                                                    "Failed to decode update {} during flush",
                                                    i
                                                );
                                            }
                                        }

                                        if has_updates {
                                            let state_vector = StateVector::default();
                                            let merged_update =
                                                txn.encode_state_as_update_v1(&state_vector);

                                            Self::handle_update(
                                                merged_update,
                                                &doc_name_clone,
                                                &store_clone,
                                                &redis,
                                                redis_ttl,
                                            )
                                            .await;
                                        }

                                        if let Err(e) = redis_conn.del::<_, ()>(&redis_key).await {
                                            tracing::warn!(
                                                "Failed to clear pending updates from Redis: {}",
                                                e
                                            );
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "Failed to load pending updates from Redis: {}",
                                        e
                                    );
                                }
                            }
                        }
                        return;
                    }
                    std::mem::take(&mut *pending)
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

                    Self::handle_update(
                        merged_update,
                        &doc_name_clone,
                        &store_clone,
                        &redis,
                        redis_ttl,
                    )
                    .await;
                }

                if let Some(redis_pool) = &redis {
                    let redis_key = format!("pending_updates:{}", doc_name_clone);
                    let mut redis_conn = redis_pool.get().await.unwrap();
                    if let Err(e) = redis_conn.del::<_, ()>(&redis_key).await {
                        tracing::warn!("Failed to clear pending updates from Redis: {}", e);
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
            redis_pool: None,
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

        Self::load_from_storage(&store, &doc_name, &group.awareness_ref).await;

        let redis_pool = if let Some(redis_config) = config.redis_config {
            match Self::init_redis_connection(&redis_config.url).await {
                Ok(conn) => {
                    let redis_key = format!("pending_updates:{}", doc_name);
                    let conn_clone = conn.clone();
                    let awareness_clone = group.awareness_ref.clone();
                    let pending_clone = group.pending_updates.clone();

                    tokio::spawn(async move {
                        let mut redis_conn = match conn_clone.get().await {
                            Ok(conn) => conn,
                            Err(e) => {
                                tracing::error!("Failed to get Redis connection: {}", e);
                                return;
                            }
                        };
                        match redis_conn
                            .lrange::<_, Vec<Vec<u8>>>(&redis_key, 0, -1)
                            .await
                        {
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

                    let stream_name = Self::compute_redis_stream_name(&doc_name);
                    let consumer_name = format!("instance-{}", rand::random::<u32>());

                    let mut redis_conn = conn.get().await.unwrap();
                    let _: redis::RedisResult<String> = redis::cmd("XGROUP")
                        .arg("CREATE")
                        .arg(&stream_name)
                        .arg(REDIS_WORKER_GROUP)
                        .arg("0")
                        .arg("MKSTREAM")
                        .query_async(&mut *redis_conn)
                        .await;
                    drop(redis_conn);

                    let awareness_for_sub = group.awareness_ref.clone();
                    let sender_for_sub = group.sender.clone();
                    let doc_name_for_sub = doc_name.clone();
                    let redis_url = redis_config.url.clone();

                    let redis_subscriber_task = tokio::spawn(async move {
                        match redis::Client::open(redis_url) {
                            Ok(client) => match client.get_async_pubsub().await {
                                Ok(mut pubsub) => {
                                    let channel = format!("yjs:updates:{}", doc_name_for_sub);
                                    match pubsub.subscribe(&channel).await {
                                        Ok(_) => {
                                            let mut stream = pubsub.on_message();

                                            while let Some(msg) = stream.next().await {
                                                match msg.get_payload::<Vec<u8>>() {
                                                    Ok(payload) => {
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
                                                                let _ =
                                                                    sender_for_sub.send(payload);
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
                            },
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
        group.redis_pool = redis_pool;
        group.doc_name = Some(doc_name.clone());
        group.redis_ttl = redis_ttl;

        group.storage_rx = None;

        Ok(group)
    }

    async fn init_redis_connection(url: &str) -> Result<Arc<RedisPool>, redis::RedisError> {
        let manager = RedisConnectionManager::new(url)?;
        let pool = bb8::Pool::builder()
            .max_size(100)
            .min_idle(5)
            .connection_timeout(std::time::Duration::from_secs(5))
            .idle_timeout(Some(std::time::Duration::from_secs(500)))
            .max_lifetime(Some(std::time::Duration::from_secs(7200)))
            .build(manager)
            .await
            .map_err(|e| {
                redis::RedisError::from(std::io::Error::new(std::io::ErrorKind::Other, e))
            })?;
        Ok(Arc::new(pool))
    }

    #[allow(dead_code)]
    async fn load_from_redis(
        redis: &Arc<RedisPool>,
        doc_name: &str,
        awareness: &AwarenessRef,
    ) -> Result<(), anyhow::Error> {
        let mut conn = redis.get().await?;
        let cache_key = format!("doc:{}", doc_name);

        let cached_data: Vec<u8> = conn
            .get(&cache_key)
            .await
            .map_err(|e| Error::Other(e.into()))?;
        let update = Update::decode_v1(&cached_data)?;

        let awareness_guard = awareness.write().await;
        let mut txn = awareness_guard.doc().transact_mut();
        txn.apply_update(update)?;

        Ok(())
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
        redis: &Option<Arc<RedisPool>>,
        redis_ttl: Option<usize>,
    ) {
        let store_future = Some(store.push_update(doc_name, &update));

        let redis_future = if let (Some(redis), Some(ttl)) = (redis, redis_ttl) {
            let stream_name = Self::compute_redis_stream_name(doc_name);
            let redis = redis.clone();
            let update = update.clone();

            Some(async move {
                let mut conn = redis.get().await.unwrap();
                let extended_ttl = ttl.saturating_mul(2);

                let mut pipe = redis::pipe();

                pipe.cmd("XADD")
                    .arg(&stream_name)
                    .arg("*")
                    .arg("update")
                    .arg(update.as_slice());

                pipe.cmd("EXPIRE")
                    .arg(&stream_name)
                    .arg(extended_ttl as i64);

                let pipe_result: redis::RedisResult<(String, bool)> =
                    pipe.query_async(&mut *conn).await;

                if let Err(e) = pipe_result {
                    tracing::error!("Failed to add update to Redis stream: {}", e);
                }
            })
        } else {
            None
        };

        match (store_future, redis_future) {
            (Some(store_future), Some(redis_future)) => {
                let (store_result, _) = tokio::join!(store_future, redis_future);
                if let Err(e) = store_result {
                    tracing::error!(
                        "Failed to store update in GCS for document '{}': {}",
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
                redis_future.await;
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
            let redis = self.redis_pool.clone();
            let doc_name = self.doc_name.clone();
            let redis_ttl = self.redis_ttl;

            tokio::spawn(async move {
                while let Some(res) = stream.next().await {
                    if let Ok(data) = res.map_err(|e| Error::Other(Box::new(e))) {
                        if let Ok(msg) = Message::decode_v1(&data) {
                            if let Message::Sync(SyncMessage::Update(update)) = &msg {
                                if let (Some(redis), Some(ttl), Some(doc_name)) =
                                    (&redis, redis_ttl, &doc_name)
                                {
                                    let stream_name = Self::compute_redis_stream_name(doc_name);
                                    let redis = redis.clone();
                                    let update = update.clone();

                                    tokio::spawn(async move {
                                        let mut conn = redis.get().await.unwrap();

                                        let cmd_result: redis::RedisResult<String> =
                                            redis::cmd("XADD")
                                                .arg(&stream_name)
                                                .arg("*")
                                                .arg("update")
                                                .arg(update.as_slice())
                                                .query_async(&mut *conn)
                                                .await;

                                        match cmd_result {
                                            Ok(id) => {
                                                let _ = conn
                                                    .expire::<_, ()>(
                                                        &stream_name,
                                                        ttl.try_into().unwrap(),
                                                    )
                                                    .await;
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
        if self.connection_count() > 0 {
            return Ok(());
        }

        if let (Some(store), Some(doc_name)) = (&self.storage, &self.doc_name) {
            let stream_name = Self::compute_redis_stream_name(doc_name);

            if let Some(redis) = &self.redis_pool {
                let mut redis_conn = redis.get().await.unwrap();

                let stream_data: RedisStreamData = redis::cmd("XRANGE")
                    .arg(&stream_name)
                    .arg("-")
                    .arg("+")
                    .query_async(&mut *redis_conn)
                    .await
                    .unwrap_or_default();

                if !stream_data.is_empty() {
                    let updates: Vec<Vec<u8>> = stream_data
                        .into_iter()
                        .filter_map(|(_, pairs)| {
                            pairs
                                .into_iter()
                                .find(|(field, _)| field == "update")
                                .map(|(_, value)| value)
                        })
                        .collect();

                    if !updates.is_empty() {
                        let result = self.apply_and_store_updates(updates, doc_name, store).await;

                        let _: Result<(), redis::RedisError> = redis::cmd("DEL")
                            .arg(&stream_name)
                            .query_async(&mut *redis_conn)
                            .await;

                        return result;
                    }
                }
            }

            Ok(())
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

        tracing::debug!("Storing merged update for document '{}'", doc_name);
        Self::handle_update(
            merged_update,
            doc_name,
            store,
            &self.redis_pool,
            self.redis_ttl,
        )
        .await;
        tracing::debug!("Stored merged updates for document '{}'", doc_name);

        Ok(())
    }

    fn compute_redis_stream_name(doc_name: &str) -> String {
        format!("{}:room:{}", REDIS_STREAM_PREFIX, doc_name)
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

        if let Some(task) = &self.redis_subscriber_task {
            task.abort();
            tracing::info!("Aborted Redis subscriber task");
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

        if let Some(task) = self.redis_subscriber_task.take() {
            task.abort();
        }

        self.awareness_updater.abort();

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
