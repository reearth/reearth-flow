use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::storage::redis::RedisStore;
use crate::AwarenessRef;

use anyhow::Result;
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

#[derive(Debug, Clone)]
pub struct BroadcastConfig {
    pub storage_enabled: bool,
    pub doc_name: Option<String>,
}

pub struct BroadcastGroup {
    connections: Arc<AtomicUsize>,
    awareness_ref: AwarenessRef,
    sender: Sender<Vec<u8>>,
    pub awareness_updater: JoinHandle<()>,
    doc_sub: Option<yrs::Subscription>,
    awareness_sub: Option<yrs::Subscription>,
    storage: Option<Arc<GcsStore>>,
    redis_store: Option<Arc<RedisStore>>,
    doc_name: Option<String>,
    redis_ttl: Option<usize>,
    storage_rx: Option<tokio::sync::mpsc::UnboundedReceiver<Vec<u8>>>,
    pub redis_subscriber_task: Option<JoinHandle<()>>,
    redis_consumer_name: Option<String>,
    redis_group_name: Option<String>,
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
        let prev_count = self.connections.fetch_add(1, Ordering::Relaxed);
        let new_count = prev_count + 1;

        tracing::info!(
            "Connection count increased: {} -> {}",
            prev_count,
            new_count
        );

        if let (Some(redis_store), Some(doc_name)) = (&self.redis_store, &self.doc_name) {
            if let Err(e) = redis_store.increment_doc_connections(doc_name).await {
                tracing::warn!("Failed to increment Redis global connection count: {}", e);
            }
        }

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

        if let (Some(redis_store), Some(doc_name)) = (&self.redis_store, &self.doc_name) {
            match redis_store.decrement_doc_connections(doc_name).await {
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("Failed to decrement Redis global connection count: {}", e);
                }
            }
        }

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
                let update = awareness_txn.encode_diff_v1(&gcs_state);
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

        new_count
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
            lock.doc_mut().observe_update_v1(move |_txn, u| {
                let mut encoder = EncoderV1::new();
                encoder.write_var(MSG_SYNC);
                encoder.write_var(MSG_SYNC_UPDATE);
                encoder.write_buf(&u.update);
                let msg = encoder.to_vec();
                if let Err(_e) = sink.send(msg) {
                    tracing::debug!("broadcast channel closed");
                }
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
            redis_subscriber_task: None,
            redis_consumer_name: None,
            redis_group_name: None,
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

        let doc_name = config.doc_name.clone().unwrap_or_default();

        let redis_ttl = redis_store
            .as_ref()
            .and_then(|rs| rs.get_config())
            .map(|c| c.ttl as usize);

        Self::load_from_storage(&store, &doc_name, &group.awareness_ref).await;

        if let Some(redis_store) = redis_store.clone() {
            group.redis_store = Some(redis_store.clone());

            let consumer_name = format!("instance-{}", rand::random::<u32>());
            let consumer_name_clone = consumer_name.clone();
            let awareness_for_sub = group.awareness_ref.clone();
            let sender_for_sub = group.sender.clone();
            let doc_name_for_sub = doc_name.clone();
            let redis_store_for_sub = redis_store.clone();

            let group_name = format!("yjs-group-{}", consumer_name);
            let group_name_clone = group_name.clone();

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

                loop {
                    match redis_store_for_sub
                        .read_and_ack_with_lua(
                            &doc_name_for_sub,
                            &group_name_clone,
                            &consumer_name_clone,
                            15,
                        )
                        .await
                    {
                        Ok(updates) => {
                            consecutive_errors = 0;
                            if !updates.is_empty() {
                                let decoded_updates: Vec<_> = updates
                                    .iter()
                                    .map(|update| (update.clone(), Update::decode_v1(update)))
                                    .collect();

                                for (update, _) in &decoded_updates {
                                    if sender_for_sub.send(update.clone()).is_err() {
                                        tracing::warn!("Failed to broadcast Redis update");
                                    }
                                }

                                let awareness = awareness_for_sub.write().await;
                                let mut txn = awareness.doc().transact_mut();

                                for (_, decoded) in decoded_updates {
                                    if let Ok(update) = decoded {
                                        if let Err(e) = txn.apply_update(update) {
                                            tracing::warn!(
                                                "Failed to apply update from Redis: {}",
                                                e
                                            );
                                        }
                                    } else if let Err(e) = decoded {
                                        tracing::warn!("Failed to decode update from Redis: {}", e);
                                    }
                                }
                            }
                        }
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
                }
            });

            group.redis_subscriber_task = Some(redis_subscriber_task);
            group.redis_consumer_name = Some(consumer_name);
            group.redis_group_name = Some(group_name);
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

    pub fn get_redis_store(&self) -> Option<Arc<RedisStore>> {
        self.redis_store.clone()
    }

    pub fn get_redis_group_name(&self) -> Option<String> {
        self.redis_group_name.clone()
    }

    pub fn get_doc_name(&self) -> Option<String> {
        self.doc_name.clone()
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
        let doc_id = self
            .doc_name
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let current_count = self.connection_count();

        tracing::info!(
            "Creating new subscription for doc '{}', current count: {}",
            doc_id,
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

        let subscription = self.subscribe_with(sink, stream, DefaultProtocol);
        let (tx, rx) = tokio::sync::oneshot::channel();

        let self_clone = self.clone();
        let doc_id_clone = self
            .doc_name
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        tokio::spawn(async move {
            if let Err(e) = self_clone.increment_connections().await {
                tracing::error!("Failed to increment connections: {}", e);
            } else {
                let new_count = self_clone.connection_count();
                tracing::info!(
                    "New connection count for doc '{}': {}",
                    doc_id_clone,
                    new_count
                );
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
        _gcs_store: Option<&Arc<GcsStore>>,
    ) -> Result<Option<Message>, Error> {
        match msg {
            Message::Sync(msg) => {
                if let (Some(redis_store), Some(doc_name)) = (redis_store, doc_name) {
                    let rs = redis_store.clone();
                    let dn = doc_name.clone();
                    let update_bytes = match &msg {
                        SyncMessage::Update(update) => update.clone(),
                        SyncMessage::SyncStep2(update) => update.clone(),
                        _ => Vec::new(),
                    };

                    if !update_bytes.is_empty() {
                        tokio::spawn(async move {
                            if let Err(e) = rs.publish_update(&dn, &update_bytes).await {
                                tracing::error!("Redis Stream update failed: {}", e);
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

        if let Some(task) = &self.redis_subscriber_task {
            task.abort();
        }

        if let (Some(redis_store), Some(doc_name), Some(consumer_name), Some(group_name)) = (
            &self.redis_store,
            &self.doc_name,
            &self.redis_consumer_name,
            &self.redis_group_name,
        ) {
            match redis_store
                .read_pending_messages(doc_name, group_name, consumer_name, 100)
                .await
            {
                Ok(pending) => {
                    if !pending.is_empty() {
                        tracing::debug!(
                            "Acknowledging {} pending messages for '{}'",
                            pending.len(),
                            doc_name
                        );
                        for (msg_id, _) in pending {
                            let _ = redis_store.ack_message(doc_name, group_name, &msg_id).await;
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Error reading pending messages: {}", e);
                }
            }

            let redis_store_clone = redis_store.clone();
            let doc_name_clone = doc_name.clone();
            let group_name_clone = group_name.clone();
            let consumer_name_clone = consumer_name.clone();

            tokio::spawn(async move {
                match redis_store_clone
                    .delete_consumer(&doc_name_clone, &group_name_clone, &consumer_name_clone)
                    .await
                {
                    Ok(1) => {
                        tracing::info!(
                            "Successfully deleted consumer '{}' from group '{}'",
                            consumer_name_clone,
                            group_name_clone
                        );
                    }
                    Ok(0) => {
                        tracing::debug!(
                            "Consumer '{}' not found in group '{}'",
                            consumer_name_clone,
                            group_name_clone
                        );
                    }
                    Ok(n) => {
                        tracing::info!(
                            "Deleted {} pending messages for consumer '{}' in group '{}'",
                            n,
                            consumer_name_clone,
                            group_name_clone
                        );
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to delete consumer '{}' from group '{}': {}",
                            consumer_name_clone,
                            group_name_clone,
                            e
                        );
                    }
                }

                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            });
        }

        if let (Some(store), Some(doc_name)) = (&self.storage, &self.doc_name) {
            let awareness = self.awareness_ref.read().await;
            let awareness_doc = awareness.doc();

            let gcs_doc = Doc::new();
            let mut gcs_txn = gcs_doc.transact_mut();

            if let Err(e) = store.load_doc(doc_name, &mut gcs_txn).await {
                tracing::warn!("Failed to load document state for final save: {}", e);
            }

            let gcs_state = gcs_txn.state_vector();

            let awareness_txn = awareness_doc.transact();
            let update = awareness_txn.encode_diff_v1(&gcs_state);

            if let Err(e) = store.push_update(doc_name, &update).await {
                tracing::warn!("Failed to save final document state: {}", e);
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

            if let (Some(redis_store), Some(doc_name), Some(consumer_name), Some(group_name)) = (
                &self.redis_store,
                &self.doc_name,
                &self.redis_consumer_name,
                &self.redis_group_name,
            ) {
                let rs = redis_store.clone();
                let dn = doc_name.clone();
                let cn = consumer_name.clone();
                let gn = group_name.clone();

                tokio::spawn(async move {
                    match rs.read_pending_messages(&dn, &gn, &cn, 100).await {
                        Ok(pending) => {
                            if !pending.is_empty() {
                                tracing::debug!(
                                    "Drop: Acknowledging {} pending messages",
                                    pending.len()
                                );
                                for (msg_id, _) in pending {
                                    let _ = rs.ack_message(&dn, &gn, &msg_id).await;
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Error reading pending messages during drop: {}", e);
                        }
                    }

                    match rs.delete_consumer(&dn, &gn, &cn).await {
                        Ok(n) => {
                            if n > 0 {
                                tracing::debug!(
                                    "Drop: Successfully deleted consumer '{}' from group '{}'",
                                    cn,
                                    gn
                                );
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Drop: Failed to delete consumer: {}", e);
                        }
                    }
                });
            }
        }

        self.awareness_updater.abort();

        self.shutdown_complete
            .store(true, std::sync::atomic::Ordering::SeqCst);
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
