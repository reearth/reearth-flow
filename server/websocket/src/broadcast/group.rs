#![allow(dead_code)]
use crate::api::Api;
use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::storage::redis::RedisStore;
use crate::subscriber::{create_subscriber, Subscriber};
use crate::{AwarenessRef, Subscription};

use anyhow::Result;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use rand;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};
use yrs::types::ToJson;

use serde_json;
use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast::{channel, Sender};
use tokio::sync::Mutex;
use yrs::encoding::write::Write;
use yrs::sync::protocol::{MSG_SYNC, MSG_SYNC_UPDATE};
use yrs::sync::{DefaultProtocol, Error, Message, Protocol, SyncMessage};
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::{Encode, Encoder, EncoderV1};
use yrs::{Doc, ReadTxn, Transact, Update};

use super::types::BroadcastConfig;

pub struct BroadcastGroup {
    awareness_ref: AwarenessRef,
    sender: Sender<Bytes>,
    doc_sub: yrs::Subscription,
    awareness_sub: yrs::Subscription,
    storage: Arc<GcsStore>,
    redis_store: Arc<RedisStore>,
    api: Arc<Api>,
    subscriber: Arc<Subscriber>,
    room: String,
    doc_name: String,
    instance_id: String,
    initial_redis_sub_id: String,
    stream_name: String,
    is_closing: Arc<tokio::sync::Mutex<bool>>,
    awareness_updater: Option<JoinHandle<()>>,
    awareness_shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    redis_handler_task: Option<JoinHandle<()>>,
    redis_handler_shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    heartbeat_task: Option<JoinHandle<()>>,
    heartbeat_shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    sync_task: Option<JoinHandle<()>>,
    sync_shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl std::fmt::Debug for BroadcastGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BroadcastGroup")
            .field("awareness_ref", &self.awareness_ref)
            .field("doc_name", &self.doc_name)
            .finish()
    }
}

impl BroadcastGroup {
    pub async fn new(
        awareness: AwarenessRef,
        buffer_capacity: usize,
        redis_store: Arc<RedisStore>,
        storage: Arc<GcsStore>,
        config: BroadcastConfig,
    ) -> Result<Self> {
        // Create API and Subscriber like JavaScript version
        let api: Arc<Api> = Arc::new(Api::new(redis_store.clone(), storage.clone(), None).await?);
        let subscriber: Arc<Subscriber> =
            Arc::new(create_subscriber(redis_store.clone(), api.clone()).await?);

        let room = config.room_name.clone().unwrap_or_default();
        let doc_name = config.doc_name.clone().unwrap_or("index".to_string());

        // Compute stream name using JavaScript format
        let stream_name = redis_store.compute_redis_room_stream_name(&room, &doc_name);
        let (sender, _) = channel(buffer_capacity.max(512));
        let awareness_c = Arc::downgrade(&awareness);
        let mut lock = awareness.write().await;
        let sink = sender.clone();

        // Create is_closing flag to prevent Redis operations during shutdown
        let is_closing = Arc::new(tokio::sync::Mutex::new(false));

        // Doc update handler - when local doc changes, send to Redis
        let api_for_updates = api.clone();
        let room_for_updates = room.clone();
        let doc_name_for_updates = doc_name.clone();
        let is_closing_for_doc = is_closing.clone();
        let doc_sub = {
            lock.doc_mut().observe_update_v1(move |_txn, u| {
                let mut encoder = EncoderV1::new();
                encoder.write_var(MSG_SYNC);
                encoder.write_var(MSG_SYNC_UPDATE);
                encoder.write_buf(&u.update);
                let msg = Bytes::from(encoder.to_vec());

                // Send to local subscribers
                if let Err(e) = sink.send(msg.clone()) {
                    debug!("broadcast channel closed (likely during shutdown): {}", e);
                    return; // Don't continue if local broadcast fails
                }

                // Only send to Redis if not closing
                let api_clone = api_for_updates.clone();
                let room_clone = room_for_updates.clone();
                let doc_name_clone = doc_name_for_updates.clone();
                let is_closing_clone = is_closing_for_doc.clone();
                tokio::spawn(async move {
                    // Check if we're in shutdown state
                    if *is_closing_clone.lock().await {
                        debug!("Skipping Redis send during shutdown");
                        return;
                    }

                    if let Err(e) = api_clone
                        .add_message(&room_clone, &doc_name_clone, &msg)
                        .await
                    {
                        debug!(
                            "Failed to add message to Redis (likely during shutdown): {}",
                            e
                        );
                    }
                });
            })?
        };

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let sink = sender.clone();
        let (awareness_shutdown_tx, mut awareness_shutdown_rx) = tokio::sync::oneshot::channel();

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

            // Check if sender is still available before sending
            if tx.is_closed() {
                return; // Silently return if channel is closed
            }

            if tx.send(changed).is_err() {
                // Don't log warning as this is expected when client disconnects
            }
        });
        drop(lock);

        let instance_id = format!("instance-{}", rand::random::<u64>());

        // Register this instance in Redis for tracking like JavaScript version
        let doc_key = format!("{}/{}", room, doc_name);
        let client_id = {
            let awareness_read = awareness.read().await;
            awareness_read.client_id()
        };

        // Register instance heartbeat
        if let Err(e) = redis_store
            .update_instance_heartbeat(&doc_key, &client_id)
            .await
        {
            warn!("Failed to register initial instance heartbeat: {}", e);
        } else {
            debug!("Registered instance heartbeat for doc: {}", doc_key);
        }

        // Simplified awareness updater
        let api_for_awareness = api.clone();
        let room_for_awareness = room.clone();
        let doc_name_for_awareness = doc_name.clone();
        let is_closing_for_awareness = is_closing.clone();
        let awareness_updater = tokio::task::spawn(async move {
            loop {
                select! {
                    _ = &mut awareness_shutdown_rx => {
                        break;
                    },
                    client_update = rx.recv() => {
                        match client_update {
                            Some(changed_clients) => {
                                if let Some(awareness) = awareness_c.upgrade() {
                                    let awareness = awareness.read().await;
                                    if let Ok(update) = awareness.update_with_clients(changed_clients.clone()) {
                                        let msg_bytes = Bytes::from(Message::Awareness(update.clone()).encode_v1());

                                        // Check if the broadcast channel is still open
                                        if sink.receiver_count() == 0 {
                                            // No receivers, break the loop
                                            debug!("No receivers for awareness updates, stopping updater");
                                            break;
                                        }

                                        if sink.send(msg_bytes.clone()).is_err() {
                                            // Channel closed, stop the updater gracefully
                                            debug!("Awareness broadcast channel closed, stopping updater");
                                            return;
                                        }

                                        // Send awareness to Redis only if not closing
                                        let api_clone = api_for_awareness.clone();
                                        let room_clone = room_for_awareness.clone();
                                        let doc_name_clone = doc_name_for_awareness.clone();
                                        let is_closing_clone = is_closing_for_awareness.clone();
                                        tokio::spawn(async move {
                                            // Check if we're in shutdown state
                                            if *is_closing_clone.lock().await {
                                                debug!("Skipping Redis awareness send during shutdown");
                                                return;
                                            }

                                            if let Err(e) = api_clone.add_message(&room_clone, &doc_name_clone, &msg_bytes).await {
                                                debug!("Failed to add awareness message to Redis (likely during shutdown): {}", e);
                                            }
                                        });
                                    }
                                } else {
                                    break;
                                }
                            },
                            None => {
                                debug!("Awareness update channel closed");
                                break;
                            }
                        }
                    }
                }
            }
        });

        // Set up JavaScript-style subscription with proper ydoc sync
        let sender_for_sub = sender.clone();
        let awareness_for_redis = awareness.clone();
        let subscriber_for_messages = subscriber.clone();

        // Create a channel for async processing of Redis messages
        let (redis_msg_tx, mut redis_msg_rx) =
            tokio::sync::mpsc::unbounded_channel::<(String, Vec<Bytes>)>();

        // Spawn a task to handle Redis messages asynchronously
        let awareness_for_handler = awareness_for_redis.clone();
        let sender_for_handler = sender_for_sub.clone();
        let (redis_handler_shutdown_tx, mut redis_handler_shutdown_rx) =
            tokio::sync::oneshot::channel();

        let redis_handler_task = tokio::spawn(async move {
            loop {
                select! {
                    _ = &mut redis_handler_shutdown_rx => {
                        debug!("Redis handler received shutdown signal");
                        break;
                    }
                    msg = redis_msg_rx.recv() => {
                        match msg {
                            Some((stream, messages)) => {
                                debug!(
                                    "Processing {} Redis messages from stream: {}",
                                    messages.len(),
                                    stream
                                );

                                // Apply each message to local document AND broadcast
                                for message in messages {
                                    if message.is_empty() {
                                        continue;
                                    }

                                    // Try to decode and apply the message to local ydoc
                                    match yrs::sync::Message::decode_v1(&message) {
                                        Ok(parsed_msg) => {
                                            match parsed_msg {
                                                yrs::sync::Message::Sync(sync_msg) => {
                                                    match sync_msg {
                                                        yrs::sync::SyncMessage::Update(update) => {
                                                            // Apply sync update to local document
                                                            if let Ok(decoded_update) =
                                                                yrs::Update::decode_v1(&update)
                                                            {
                                                                let awareness_guard =
                                                                    awareness_for_handler.write().await;
                                                                let mut txn = awareness_guard.doc().transact_mut();
                                                                if let Err(e) = txn.apply_update(decoded_update) {
                                                                    warn!("Failed to apply Redis update to local doc: {}", e);
                                                                } else {
                                                                    debug!("Applied sync update from Redis to local doc");
                                                                }
                                                            }
                                                        }
                                                        yrs::sync::SyncMessage::SyncStep2(update) => {
                                                            // Apply sync step 2 to local document
                                                            if let Ok(decoded_update) =
                                                                yrs::Update::decode_v1(&update)
                                                            {
                                                                let awareness_guard =
                                                                    awareness_for_handler.write().await;
                                                                let mut txn = awareness_guard.doc().transact_mut();
                                                                if let Err(e) = txn.apply_update(decoded_update) {
                                                                    warn!("Failed to apply Redis sync step 2 to local doc: {}", e);
                                                                } else {
                                                                    debug!("Applied sync step 2 from Redis to local doc");
                                                                }
                                                            }
                                                        }
                                                        yrs::sync::SyncMessage::SyncStep1(_) => {
                                                            // Sync step 1 doesn't modify the document, just broadcast
                                                        }
                                                    }
                                                }
                                                yrs::sync::Message::Awareness(awareness_update) => {
                                                    // Apply awareness update to local awareness
                                                    let awareness_guard = awareness_for_handler.write().await;
                                                    if let Err(e) = awareness_guard.apply_update(awareness_update) {
                                                        warn!("Failed to apply Redis awareness update: {}", e);
                                                    } else {
                                                        debug!("Applied awareness update from Redis");
                                                    }
                                                }
                                                _ => {
                                                    debug!("Ignoring non-sync message from Redis");
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            warn!("Failed to decode Redis message: {}", e);
                                        }
                                    }

                                    // Check if sender is still available before broadcasting
                                    if sender_for_handler.receiver_count() == 0 {
                                        debug!("No more receivers, stopping Redis message processing");
                                        return;
                                    }

                                    // Broadcast the original message to connected clients
                                    if let Err(e) = sender_for_handler.send(message) {
                                        debug!("Channel closed, stopping Redis message processing: {}", e);
                                        return; // Exit gracefully when channel is closed
                                    }
                                }
                            }
                            None => {
                                debug!("Redis message channel closed");
                                break;
                            }
                        }
                    }
                }
            }
        });

        let initial_redis_sub_id = {
            let stream_key = stream_name.clone();
            let tx = redis_msg_tx.clone();
            let sub_result = subscriber_for_messages
                .subscribe(stream_key, move |stream: String, messages: Vec<Bytes>| {
                    // Send to async handler, but handle channel closure gracefully
                    if let Err(e) = tx.send((stream.clone(), messages)) {
                        // Only log as debug during shutdown to avoid spam
                        debug!(
                            "Redis messages channel closed during shutdown for stream {}: {}",
                            stream, e
                        );
                    }
                })
                .await;
            sub_result.redis_id
        };

        // Simplified heartbeat (optional - not in JavaScript version)
        let (heartbeat_shutdown_tx, mut heartbeat_shutdown_rx) = tokio::sync::oneshot::channel();
        let heartbeat_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                select! {
                    _ = &mut heartbeat_shutdown_rx => {
                        break;
                    },
                    _ = interval.tick() => {
                        // Optional heartbeat logic
                        debug!("Heartbeat tick");
                    }
                }
            }
        });

        // Optional sync task (simplified)
        let (sync_shutdown_tx, mut sync_shutdown_rx) = tokio::sync::oneshot::channel();
        let sender_clone = sender.clone();
        let awareness_clone = Arc::clone(&awareness);
        let sync_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                select! {
                    _ = &mut sync_shutdown_rx => {
                        break;
                    },
                    _ = interval.tick() => {
                        let awareness = awareness_clone.read().await;
                        let txn = awareness.doc().transact();
                        let state_vector = txn.state_vector();

                        let sync_msg = Message::Sync(SyncMessage::SyncStep1(state_vector));
                        let encoded_msg = sync_msg.encode_v1();

                        let msg = Bytes::from(encoded_msg);
                        if let Err(e) = sender_clone.send(msg) {
                            warn!("Failed to send periodic sync message: {}", e);
                            break;
                        }
                    }
                }
            }
        });

        Ok(BroadcastGroup {
            awareness_ref: awareness,
            sender,
            doc_sub,
            awareness_sub,
            storage,
            redis_store,
            api,
            subscriber,
            room,
            doc_name,
            instance_id,
            initial_redis_sub_id,
            stream_name,
            is_closing,
            awareness_updater: Some(awareness_updater),
            awareness_shutdown_tx: Some(awareness_shutdown_tx),
            redis_handler_task: Some(redis_handler_task),
            redis_handler_shutdown_tx: Some(redis_handler_shutdown_tx),
            heartbeat_task: Some(heartbeat_task),
            heartbeat_shutdown_tx: Some(heartbeat_shutdown_tx),
            sync_task: Some(sync_task),
            sync_shutdown_tx: Some(sync_shutdown_tx),
        })
    }

    pub fn awareness(&self) -> &AwarenessRef {
        &self.awareness_ref
    }

    pub fn get_redis_store(&self) -> &Arc<RedisStore> {
        &self.redis_store
    }

    pub fn get_doc_name(&self) -> String {
        self.doc_name.clone()
    }

    pub fn get_initial_redis_sub_id(&self) -> &str {
        &self.initial_redis_sub_id
    }

    pub fn get_active_connections(&self) -> usize {
        self.sender.receiver_count()
    }

    pub async fn subscribe<Sink, Stream, E>(
        self: Arc<Self>,
        sink: Arc<Mutex<Sink>>,
        stream: Stream,
    ) -> Subscription
    where
        Sink: SinkExt<Bytes> + Send + Sync + Unpin + 'static,
        Stream: StreamExt<Item = Result<Bytes, E>> + Send + Sync + Unpin + 'static,
        <Sink as futures_util::Sink<Bytes>>::Error: std::error::Error + Send + Sync,
        E: std::error::Error + Send + Sync + 'static,
    {
        self.listen(sink, stream, DefaultProtocol).await
    }

    pub async fn listen<Sink, Stream, E, P>(
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
            let api = self.api.clone();
            let room = self.room.clone();
            let doc_name = self.doc_name.clone();

            tokio::spawn(async move {
                while let Some(res) = stream.next().await {
                    let data = match res.map_err(anyhow::Error::from) {
                        Ok(data) => data,
                        Err(e) => {
                            warn!("Error receiving message: {}", e);
                            break;
                        }
                    };

                    let msg = match Message::decode_v1(&data) {
                        Ok(msg) => msg,
                        Err(e) => {
                            warn!("Failed to decode message: {}", e);
                            continue;
                        }
                    };

                    match Self::handle_msg(&protocol, &awareness, msg, &api, &room, &doc_name).await
                    {
                        Ok(Some(reply)) => {
                            let mut sink_lock = sink.lock().await;
                            if let Err(e) = sink_lock.send(Bytes::from(reply.encode_v1())).await {
                                warn!("Failed to send reply: {}", e);
                            }
                        }
                        Err(e) => warn!("Error handling message: {}", e),
                        _ => {}
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
        api: &Arc<Api>,
        room: &str,
        doc_name: &str,
    ) -> Result<Option<Message>, Error> {
        // Send all messages to Redis using JavaScript-style API
        let encoded_msg = msg.encode_v1();
        let api_clone = api.clone();
        let room_clone = room.to_string();
        let doc_name_clone = doc_name.to_string();

        // Send to Redis using JavaScript format
        tokio::spawn(async move {
            if let Err(e) = api_clone
                .add_message(&room_clone, &doc_name_clone, &encoded_msg)
                .await
            {
                warn!("Failed to add message to Redis: {}", e);
            }
        });

        // Handle the message locally
        match msg {
            Message::Sync(msg) => match msg {
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

    fn all_nodes_have_position(&self, doc: &Doc) -> bool {
        let map = doc.get_or_insert_map("workflows");
        let map_json = map.to_json(&doc.transact());

        let json_str = serde_json::to_string(&map_json).unwrap_or_else(|_| "{}".to_string());
        match serde_json::from_str::<serde_json::Value>(&json_str) {
            Ok(map_json_value) => {
                if let Some(main) = map_json_value["main"].as_object() {
                    if let Some(nodes) = main["nodes"].as_object() {
                        if nodes.is_empty() {
                            debug!("No nodes found");
                            return false;
                        }

                        for (_, node) in nodes {
                            if let Some(position) = node["position"].as_object() {
                                if let (Some(x), Some(y)) = (position.get("x"), position.get("y")) {
                                    if x.is_number() && y.is_number() {
                                        continue;
                                    }
                                }
                            }
                            return false;
                        }
                        return true;
                    }
                }
                false
            }
            Err(e) => {
                tracing::error!("Error parsing map_json: {:?}", e);
                false
            }
        }
    }

    pub async fn cleanup_client_awareness(&self) -> Result<()> {
        let awareness = self.awareness().clone();
        let awareness_read = awareness.read().await;
        awareness_read.clean_local_state();

        debug!("Cleaned up client awareness for room: {}", self.room);
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        // Set closing flag first to prevent new Redis operations
        {
            let mut closing = self.is_closing.lock().await;
            *closing = true;
            debug!("Set closing flag for BroadcastGroup shutdown");
        }

        let client_id = {
            let awareness_read = self.awareness_ref.read().await;
            awareness_read.client_id()
        };

        let doc_key = format!("{}/{}", self.room, self.doc_name);
        self.redis_store
            .remove_instance_heartbeat(&doc_key, &client_id)
            .await?;

        let conn_count = self.redis_store.get_active_instances(&doc_key, 60).await?;

        debug!("Active instances for doc {}: {}", doc_key, conn_count);

        if conn_count <= 0 {
            info!("Last user for doc {}, cleaning up Redis data...", doc_key);

            let lock_id = format!("cleanup:lock:{}", doc_key);
            let instance_id = format!("cleanup-{}", rand::random::<u64>());

            let lock_acquired = self
                .redis_store
                .acquire_doc_lock(&lock_id, &instance_id)
                .await?;

            if lock_acquired {
                let awareness = self.awareness_ref.write().await;
                let awareness_doc = awareness.doc();

                {
                    let stream_key = self
                        .redis_store
                        .compute_redis_room_stream_name(&self.room, &self.doc_name);
                    let last_stream_id = self
                        .redis_store
                        .get_stream_last_id(&doc_key)
                        .await
                        .ok()
                        .flatten();

                    if let Some(ref id) = last_stream_id {
                        info!("Got last stream ID before cleanup: {}", id);
                    }

                    let gcs_doc = Doc::new();
                    let mut gcs_txn = gcs_doc.transact_mut();

                    if let Err(e) = self.storage.load_doc(&doc_key, &mut gcs_txn).await {
                        warn!("Failed to load current state from storage: {}", e);
                    }

                    let gcs_state = gcs_txn.state_vector();
                    let awareness_txn = awareness_doc.transact();

                    let update = awareness_txn.encode_diff_v1(&gcs_state);
                    let update_bytes = Bytes::from(update);

                    if !(update_bytes.is_empty()
                        || (update_bytes.len() == 2
                            && update_bytes[0] == 0
                            && update_bytes[1] == 0))
                    {
                        info!("Saving final document state for {} before cleanup", doc_key);

                        let flush_result =
                            self.storage.flush_doc_v2(&doc_key, &awareness_txn).await;
                        if let Err(e) = flush_result {
                            warn!("Failed to flush final document state to storage: {}", e);
                        } else {
                            info!("Successfully saved final document state for {}", doc_key);
                        }

                        if let Some(last_id) = last_stream_id {
                            if let Err(e) = self
                                .redis_store
                                .trim_stream_before(&doc_key, &last_id)
                                .await
                            {
                                warn!("Failed to trim Redis stream after final save: {}", e);
                            } else {
                                info!("Trimmed Redis stream after final save");
                            }
                        }
                    }

                    info!("Cleaning up Redis data for document: {}", doc_key);

                    if let Err(e) = self.redis_store.delete_stream(&doc_key).await {
                        warn!("Failed to delete Redis stream: {}", e);
                    } else {
                        info!("Successfully deleted Redis stream for {}", doc_key);
                    }

                    let stream_key = self
                        .redis_store
                        .compute_redis_room_stream_name(&self.room, &self.doc_name);
                    let worker_cleanup_result = self
                        .redis_store
                        .safe_delete_stream(&doc_key, &self.instance_id)
                        .await;
                    if let Err(e) = worker_cleanup_result {
                        warn!("Failed to clean up worker queue for {}: {}", doc_key, e);
                    } else {
                        info!("Cleaned up worker queue for {}", doc_key);
                    }
                }

                // Release the cleanup lock
                if let Err(e) = self
                    .redis_store
                    .release_doc_lock(&lock_id, &instance_id)
                    .await
                {
                    warn!("Failed to release cleanup lock: {}", e);
                }

                info!("Completed cleanup for last user of document: {}", doc_key);
            } else {
                debug!(
                    "Another instance is already cleaning up document: {}",
                    doc_key
                );
            }
        } else {
            debug!(
                "Other users still active for doc {}, skipping cleanup",
                doc_key
            );
        }

        Ok(())
    }
}

impl Drop for BroadcastGroup {
    fn drop(&mut self) {
        info!("Dropping BroadcastGroup for room: {}", self.room);

        // Set closing flag to prevent new Redis operations
        let is_closing_clone = self.is_closing.clone();
        tokio::spawn(async move {
            let mut closing = is_closing_clone.lock().await;
            *closing = true;
            debug!("Set closing flag to prevent Redis operations");
        });

        // Unsubscribe from Redis to stop receiving new messages
        let subscriber_clone = self.subscriber.clone();
        let stream_name_clone = self.stream_name.clone();
        tokio::spawn(async move {
            // Use a dummy handler for unsubscribe (the function signature requires it)
            let dummy_handler = |_: String, _: Vec<Bytes>| {};
            subscriber_clone
                .unsubscribe(&stream_name_clone, dummy_handler)
                .await;
            info!("Unsubscribed from Redis stream: {}", stream_name_clone);
        });

        // Send shutdown signals and abort tasks if sending fails
        if let Some(tx) = self.awareness_shutdown_tx.take() {
            if tx.send(()).is_err() {
                debug!("Awareness shutdown channel already closed");
                if let Some(task) = self.awareness_updater.take() {
                    task.abort();
                }
            }
        }

        // Shutdown Redis handler task first to stop processing messages
        if let Some(tx) = self.redis_handler_shutdown_tx.take() {
            if tx.send(()).is_err() {
                info!("Redis handler shutdown channel already closed");
                if let Some(task) = self.redis_handler_task.take() {
                    task.abort();
                }
            }
        }

        if let Some(tx) = self.heartbeat_shutdown_tx.take() {
            if tx.send(()).is_err() {
                info!("Heartbeat shutdown channel already closed");
                if let Some(task) = self.heartbeat_task.take() {
                    task.abort();
                }
            }
        }

        if let Some(tx) = self.sync_shutdown_tx.take() {
            if tx.send(()).is_err() {
                info!("Sync shutdown channel already closed");
                if let Some(task) = self.sync_task.take() {
                    task.abort();
                }
            }
        }

        info!(
            "BroadcastGroup dropped successfully for room: {}",
            self.room
        );
    }
}
