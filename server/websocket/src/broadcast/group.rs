use crate::broadcast::tasks::BackgroundTasks;
use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::storage::redis::RedisStore;
use crate::AwarenessRef;

use anyhow::Result;
use bytes::Bytes;
use deadpool_redis::Connection;
use futures_util::{SinkExt, StreamExt};
use rand;

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

use super::types::BroadcastConfig;

pub struct BroadcastGroup {
    connections: Arc<AtomicUsize>,
    awareness_ref: AwarenessRef,
    sender: Sender<Bytes>,
    background_tasks: Arc<Mutex<BackgroundTasks>>,
    doc_sub: Option<yrs::Subscription>,
    awareness_sub: Option<yrs::Subscription>,
    storage: Arc<GcsStore>,
    redis_store: Arc<RedisStore>,
    doc_name: String,
    redis_consumer_name: Option<String>,
    redis_group_name: Option<String>,
    shutdown_complete: AtomicBool,
    instance_id: String,
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

impl BroadcastGroup {
    pub async fn increment_connections(&self) -> Result<()> {
        let prev_count = self.connections.fetch_add(1, Ordering::Relaxed);
        let new_count = prev_count + 1;

        tracing::debug!(
            "Connection count increased: {} -> {}",
            prev_count,
            new_count
        );

        Ok(())
    }

    pub async fn decrement_connections(&self) -> usize {
        let prev_count = self.connections.fetch_sub(1, Ordering::Relaxed);
        let new_count = prev_count - 1;

        tracing::debug!(
            "Connection count decreased: {} -> {}",
            prev_count,
            new_count
        );

        new_count
    }

    pub fn connection_count(&self) -> usize {
        self.connections.load(Ordering::Relaxed)
    }

    pub async fn new(
        awareness: AwarenessRef,
        buffer_capacity: usize,
        redis_store: Arc<RedisStore>,
        storage: Arc<GcsStore>,
        doc_name: String,
    ) -> Result<Self> {
        let (sender, _receiver) = channel(buffer_capacity);
        let awareness_c = Arc::downgrade(&awareness);
        let mut lock = awareness.write().await;
        let sink = sender.clone();

        let doc_sub = {
            lock.doc_mut().observe_update_v1(move |_txn, u| {
                let mut encoder = EncoderV1::new();
                encoder.write_var(MSG_SYNC);
                encoder.write_var(MSG_SYNC_UPDATE);
                encoder.write_buf(&u.update);
                let msg = Bytes::from(encoder.to_vec());
                if let Err(_e) = sink.send(msg) {
                    tracing::debug!("broadcast channel closed");
                }
            })?
        };

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let sink = sender.clone();

        let (awareness_shutdown_tx, mut awareness_shutdown_rx) = tokio::sync::mpsc::channel(1);

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
            loop {
                select! {
                    _ = awareness_shutdown_rx.recv() => {
                        tracing::debug!("Awareness updater received shutdown signal");
                        break;
                    },
                    client_update = rx.recv() => {
                        match client_update {
                            Some(changed_clients) => {
                                if let Some(awareness) = awareness_c.upgrade() {
                                    let awareness = awareness.read().await;
                                    if let Ok(update) = awareness.update_with_clients(changed_clients) {
                                        let msg_bytes = Bytes::from(Message::Awareness(update).encode_v1());
                                        if sink.send(msg_bytes).is_err() {
                                            tracing::warn!("couldn't broadcast awareness update");
                                        }
                                    }
                                } else {
                                    break;
                                }
                            },
                            None => {
                                tracing::debug!("Awareness update channel closed");
                                break;
                            }
                        }
                    }
                }
            }
            tracing::debug!("Awareness updater task exited gracefully");
        });

        let instance_id = format!("instance-{}", rand::random::<u64>());

        let background_tasks = Arc::new(Mutex::new(BackgroundTasks::new(
            awareness_updater,
            awareness_shutdown_tx,
        )));

        let result = Self {
            connections: Arc::new(AtomicUsize::new(0)),
            awareness_ref: awareness,
            sender,
            background_tasks,
            doc_sub: Some(doc_sub),
            awareness_sub: Some(awareness_sub),
            storage,
            redis_store,
            doc_name,
            redis_consumer_name: None,
            redis_group_name: None,
            shutdown_complete: AtomicBool::new(false),
            instance_id,
        };

        Ok(result)
    }

    pub async fn with_storage(
        awareness: AwarenessRef,
        buffer_capacity: usize,
        store: Arc<GcsStore>,
        redis_store: Arc<RedisStore>,
        config: BroadcastConfig,
    ) -> Result<Self> {
        let doc_name = config.doc_name.unwrap_or_default();
        if !config.storage_enabled {
            return Self::new(awareness, buffer_capacity, redis_store, store, doc_name).await;
        }

        let mut group = Self::new(
            awareness,
            buffer_capacity,
            redis_store.clone(),
            store.clone(),
            doc_name.clone(),
        )
        .await?;

        Self::load_from_storage(&store, &doc_name, &group.awareness_ref).await;

        let consumer_name = format!("instance-{}", rand::random::<u32>());
        let consumer_name_clone = consumer_name.clone();
        let awareness_for_sub = group.awareness_ref.clone();
        let sender_for_sub = group.sender.clone();
        let doc_name_for_sub = doc_name.clone();
        let redis_store_for_sub = redis_store.clone();

        let group_name = format!("yjs-group-{}", consumer_name);
        let group_name_clone = group_name.clone();

        let (redis_shutdown_tx, mut redis_shutdown_rx) = tokio::sync::mpsc::channel(1);

        let redis_subscriber_task = tokio::spawn(async move {
            if let Err(e) = redis_store_for_sub
                .create_consumer_group(&doc_name_for_sub, &group_name_clone)
                .await
            {
                tracing::error!("Failed to create Redis consumer group: {}", e);
                return;
            }

            let mut consecutive_errors = 0;
            let max_consecutive_errors = 5;
            let mut total_errors = 0;
            let max_total_errors = 10;
            let stream_key = format!("yjs:stream:{}", doc_name_for_sub);
            let mut conn = match redis_store_for_sub.get_pool().get().await {
                Ok(conn) => conn,
                Err(e) => {
                    tracing::error!("Failed to get Redis connection: {}", e);
                    return;
                }
            };

            loop {
                select! {
                    _ = redis_shutdown_rx.recv() => {
                        break;
                    },
                    _ = async {
                        match redis_store_for_sub
                            .read_and_ack(
                                &mut conn,
                                &stream_key,
                                &group_name_clone,
                                &consumer_name_clone,
                                64,
                            )
                            .await
                        {
                            Ok(updates) => {
                                consecutive_errors = 0;
                                if !updates.is_empty() {
                                    let mut decoded_updates = Vec::with_capacity(updates.len());

                                    for update in &updates {
                                        if let Ok(decoded) = Update::decode_v1(update) {
                                            decoded_updates.push(decoded);
                                        }

                                        if sender_for_sub.send(update.clone()).is_err() {
                                            tracing::debug!("Failed to broadcast Redis update");
                                        }
                                    }

                                    if !decoded_updates.is_empty() {
                                        let awareness = awareness_for_sub.write().await;
                                        let mut txn = awareness.doc().transact_mut();

                                        for decoded in decoded_updates {
                                            if let Err(e) = txn.apply_update(decoded) {
                                                tracing::warn!("Failed to apply update from Redis: {}", e);
                                            }
                                        }
                                    }
                                }                            }
                            Err(e) => {
                                tracing::error!("Error reading from Redis Stream: {}", e);

                                consecutive_errors += 1;
                                total_errors += 1;
                                if consecutive_errors >= max_consecutive_errors
                                    || total_errors >= max_total_errors
                                {
                                    tracing::warn!(
                                        "Too many Redis errors ({} total, {} consecutive), stopping subscriber",
                                        total_errors, consecutive_errors
                                    );
                                    return;
                                }

                                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                            }
                        }

                        tokio::task::yield_now().await;
                    } => {}
                }
            }
            tracing::debug!("Redis subscriber task exited gracefully");
        });

        {
            let mut background_tasks = group.background_tasks.lock().await;
            background_tasks.set_redis_subscriber(redis_subscriber_task, redis_shutdown_tx);
        }
        group.redis_consumer_name = Some(consumer_name);
        group.redis_group_name = Some(group_name);

        let (heartbeat_shutdown_tx, mut heartbeat_shutdown_rx) = tokio::sync::mpsc::channel(1);
        let instance_id = group.instance_id.clone();

        let heartbeat_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(56));

            loop {
                select! {
                    _ = heartbeat_shutdown_rx.recv() => {
                        break;
                    },
                    _ = interval.tick() => {
                        if let Err(e) = redis_store
                            .update_instance_heartbeat(&doc_name, &instance_id)
                            .await
                        {
                            tracing::warn!("Failed to update instance heartbeat: {}", e);
                        }
                    }
                }
            }
        });

        {
            let mut background_tasks = group.background_tasks.lock().await;
            background_tasks.set_heartbeat(heartbeat_task, heartbeat_shutdown_tx);
        }

        Ok(group)
    }

    async fn load_from_storage(store: &Arc<GcsStore>, doc_name: &str, awareness: &AwarenessRef) {
        let awareness = awareness.write().await;
        let mut txn = awareness.doc().transact_mut();

        if let Err(e) = store.load_doc(doc_name, &mut txn).await {
            tracing::error!("Error loading document '{}' from storage: {}", doc_name, e);
        }
    }

    async fn handle_gcs_update(update: Bytes, doc_name: &str, store: &Arc<GcsStore>) {
        if let Err(e) = store.push_update(doc_name, &update).await {
            tracing::error!("Failed to store update for document '{}': {}", doc_name, e);
        }
    }

    pub fn awareness(&self) -> &AwarenessRef {
        &self.awareness_ref
    }

    pub fn get_redis_store(&self) -> &Arc<RedisStore> {
        &self.redis_store
    }

    pub fn get_redis_group_name(&self) -> Option<&String> {
        self.redis_group_name.as_ref()
    }

    pub fn get_doc_name(&self) -> String {
        self.doc_name.clone()
    }

    pub fn broadcast(&self, msg: Bytes) -> Result<(), SendError<Bytes>> {
        self.sender.send(msg)?;
        Ok(())
    }

    pub fn subscribe<Sink, Stream, E>(
        self: Arc<Self>,
        sink: Arc<Mutex<Sink>>,
        stream: Stream,
        user_token: Option<String>,
    ) -> Subscription
    where
        Sink: SinkExt<Bytes> + Send + Sync + Unpin + 'static,
        Stream: StreamExt<Item = Result<Bytes, E>> + Send + Sync + Unpin + 'static,
        <Sink as futures_util::Sink<Bytes>>::Error: std::error::Error + Send + Sync,
        E: std::error::Error + Send + Sync + 'static,
    {
        let current_count = self.connection_count();

        tracing::debug!(
            "Creating new subscription for doc '{}', current count: {}",
            self.doc_name,
            current_count
        );

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

        let subscription = self.listen(sink, stream, DefaultProtocol);
        let (tx, rx) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            if let Err(e) = self.increment_connections().await {
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

    pub fn listen<Sink, Stream, E, P>(
        &self,
        sink: Arc<Mutex<Sink>>,
        mut stream: Stream,
        protocol: P,
    ) -> Subscription
    where
        Sink: SinkExt<Bytes> + Send + Sync + Unpin + 'static,
        Stream: StreamExt<Item = Result<Bytes, E>> + Send + Sync + Unpin + 'static,
        <Sink as futures_util::Sink<Bytes>>::Error: std::error::Error + Send + Sync,
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
            let stream_key = format!("yjs:stream:{}", doc_name);
            tokio::spawn(async move {
                let mut conn = redis_store.get_pool().get().await.unwrap();
                while let Some(res) = stream.next().await {
                    let data = match res.map_err(anyhow::Error::from) {
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
                        &redis_store,
                        &stream_key,
                        &mut conn,
                    )
                    .await
                    {
                        Ok(Some(reply)) => {
                            let mut sink_lock = sink.lock().await;
                            if let Err(e) = sink_lock.send(Bytes::from(reply.encode_v1())).await {
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
        redis_store: &Arc<RedisStore>,
        stream_key: &str,
        conn: &mut Connection,
    ) -> Result<Option<Message>, Error> {
        match msg {
            Message::Sync(msg) => {
                let update_bytes = match &msg {
                    SyncMessage::Update(update) => update.clone(),
                    SyncMessage::SyncStep2(update) => update.clone(),
                    _ => Vec::new(),
                };

                if !update_bytes.is_empty() {
                    if let Err(e) = redis_store
                        .publish_update(stream_key, &update_bytes, conn)
                        .await
                    {
                        tracing::error!("Redis Stream update failed: {}", e);
                    }
                }

                match msg {
                    SyncMessage::SyncStep1(state_vector) => {
                        let awareness = awareness.read().await;
                        protocol.handle_sync_step1(&awareness, state_vector)
                    }
                    SyncMessage::SyncStep2(update) => {
                        let decoded_update = Update::decode_v1(&update)?;
                        let awareness = awareness.write().await;
                        protocol.handle_sync_step2(&awareness, decoded_update)
                    }
                    SyncMessage::Update(update) => {
                        let update = Update::decode_v1(&update)?;
                        let awareness = awareness.write().await;
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

        if self.connection_count() == 0 {
            let redis_store_clone = self.redis_store.clone();
            let store_clone = self.storage.clone();

            if let Err(e) = redis_store_clone
                .remove_instance_heartbeat(&self.doc_name, &self.instance_id)
                .await
            {
                tracing::warn!(
                    "Failed to remove instance heartbeat before checking connections: {}",
                    e
                );
            } else {
                tracing::debug!(
                    "Removed heartbeat for instance {} before checking connections",
                    self.instance_id
                );
            }

            let should_save = match redis_store_clone
                .get_active_instances(&self.doc_name, 60)
                .await
            {
                Ok(connections) => {
                    if connections <= 0 {
                        tracing::debug!(
                            "All instances disconnected from '{}', proceeding with GCS save",
                            self.doc_name
                        );
                        true
                    } else {
                        tracing::debug!(
                            "Other instances still connected to '{}', skipping GCS save",
                            self.doc_name
                        );
                        false
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to get Redis connection count: {}", e);
                    true
                }
            };

            if should_save {
                let lock_id = format!("gcs:lock:{}", self.doc_name);
                let instance_id = format!("instance-{}", rand::random::<u64>());

                let lock_acquired = match redis_store_clone
                    .acquire_doc_lock(&lock_id, &instance_id)
                    .await
                {
                    Ok(true) => {
                        tracing::debug!("Acquired lock for GCS operations on {}", self.doc_name);
                        Some((redis_store_clone.clone(), lock_id, instance_id))
                    }
                    Ok(false) => {
                        tracing::warn!(
                            "Could not acquire lock for GCS operations, skipping update"
                        );
                        None
                    }
                    Err(e) => {
                        tracing::warn!("Error acquiring lock for GCS operations: {}", e);
                        None
                    }
                };

                if lock_acquired.is_some() {
                    let awareness = self.awareness_ref.write().await;
                    let awareness_doc = awareness.doc();
                    let _awareness_state = awareness_doc.transact().state_vector();

                    let gcs_doc = Doc::new();
                    let mut gcs_txn = gcs_doc.transact_mut();

                    if let Err(e) = store_clone.load_doc(&self.doc_name, &mut gcs_txn).await {
                        tracing::warn!("Failed to load current state from GCS: {}", e);
                    }

                    let gcs_state = gcs_txn.state_vector();

                    let awareness_txn = awareness_doc.transact();
                    let update = awareness_txn.encode_diff_v1(&gcs_state);
                    let update_bytes = Bytes::from(update);

                    if !(update_bytes.is_empty()
                        || update_bytes.len() == 2 && update_bytes[0] == 0 && update_bytes[1] == 0)
                    {
                        Self::handle_gcs_update(update_bytes, &self.doc_name, &store_clone).await;
                    }

                    if let Err(e) = store_clone
                        .flush_doc_direct(&self.doc_name, awareness_doc)
                        .await
                    {
                        tracing::warn!("Failed to flush document directly to storage: {}", e);
                    }
                }

                if let Some((redis, lock_id, instance_id)) = lock_acquired {
                    if let Err(e) = redis.release_doc_lock(&lock_id, &instance_id).await {
                        tracing::warn!("Failed to release GCS lock: {}", e);
                    }
                }
            }
        }

        {
            let mut background_tasks = self.background_tasks.lock().await;
            background_tasks.stop_all();
        }

        let redis_store_clone = self.redis_store.clone();
        let doc_name_clone = self.doc_name.clone();
        let instance_id = self.instance_id.clone();

        tokio::spawn(async move {
            match redis_store_clone
                .safe_delete_stream(&doc_name_clone, &instance_id)
                .await
            {
                Ok(deleted) => {
                    if deleted {
                        tracing::debug!(
                            "Successfully deleted Redis stream for '{}'",
                            doc_name_clone
                        );
                    } else {
                        tracing::debug!(
                            "Did not delete Redis stream for '{}' as it may still be in use",
                            doc_name_clone
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Error during safe Redis stream deletion for '{}': {}",
                        doc_name_clone,
                        e
                    );
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        });

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

        self.shutdown_complete
            .store(true, std::sync::atomic::Ordering::SeqCst);

        let background_tasks = self.background_tasks.clone();
        tokio::spawn(async move {
            let mut tasks = background_tasks.lock().await;
            tasks.stop_all();
        });
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
