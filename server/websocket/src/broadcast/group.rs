#![allow(dead_code)]
use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::storage::redis::RedisStore;
use crate::{AwarenessRef, Subscription};

use anyhow::Result;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use rand;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};
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
    doc_name: String,
    instance_id: String,
    last_read_id: Arc<Mutex<String>>,
    awareness_updater: Option<JoinHandle<()>>,
    awareness_shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    redis_subscriber_task: Option<JoinHandle<()>>,
    redis_subscriber_shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
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
        let (sender, _) = channel(buffer_capacity.max(512));
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
                if let Err(e) = sink.send(msg) {
                    error!("broadcast channel closed: {}", e);
                }
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

            if let Err(e) = tx.send(changed) {
                warn!("failed to send awareness update: {}", e);
            }
        });
        drop(lock);

        let instance_id = format!("instance-{}", rand::random::<u64>());
        let instance_id_for_awareness = instance_id.clone();
        let redis_store_for_awareness = redis_store.clone();
        let doc_name_for_awareness = config.doc_name.clone().unwrap_or_default();
        let mut conn = redis_store_for_awareness
            .create_dedicated_connection()
            .await?;

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
                                        if let Err(e) = sink.send(msg_bytes) {
                                            error!("couldn't broadcast awareness update {}", e);
                                            return;
                                        }

                                        let update_bytes = update.encode_v1();
                                        if let Err(e) = redis_store_for_awareness
                                            .set_awareness(
                                                &doc_name_for_awareness,
                                                &instance_id_for_awareness,
                                                &mut conn,
                                                &update_bytes,
                                                300,
                                            )
                                            .await
                                        {
                                            warn!("Failed to store awareness update in Redis: {}", e);
                                        }
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

        let doc_name = config.doc_name.unwrap_or_default();

        let doc_name_for_sub = doc_name.clone();
        let redis_store_for_sub = redis_store.clone();
        let (heartbeat_shutdown_tx, mut heartbeat_shutdown_rx) = tokio::sync::oneshot::channel();
        let awareness_clone = Arc::clone(&awareness);

        let heartbeat_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            let client_id = awareness_clone.read().await.client_id();
            loop {
                select! {
                    _ = &mut heartbeat_shutdown_rx => {
                        break;
                    },
                    _ = interval.tick() => {
                        if let Err(e) = redis_store_for_sub
                            .update_instance_heartbeat(&doc_name_for_sub, &client_id)
                            .await
                        {
                            warn!("Failed to update instance heartbeat: {}", e);
                        }
                    }
                }
            }
        });

        let last_read_id = Arc::new(Mutex::new("0".to_string()));
        let last_read_id_clone = Arc::clone(&last_read_id);
        let awareness_clone = Arc::clone(&awareness);
        let instance_id_clone = instance_id.clone();
        let redis_store_for_sub_clone = Arc::clone(&redis_store);
        let doc_name_for_sub_clone = doc_name.clone();
        let (redis_subscriber_shutdown_tx, mut redis_subscriber_shutdown_rx) =
            tokio::sync::oneshot::channel();

        let redis_subscriber_task = tokio::spawn(async move {
            let stream_key = format!("yjs:stream:{doc_name_for_sub_clone}");
            let awareness_last_read_id = Arc::new(Mutex::new("0".to_string()));

            loop {
                select! {
                    _ = &mut redis_subscriber_shutdown_rx => {
                        break;
                    },
                    _ = async {
                        let result = redis_store_for_sub_clone
                            .read_and_filter(
                                &stream_key,
                                512,
                                &instance_id_clone,
                                &last_read_id_clone,
                            )
                            .await;

                        match result {
                            Ok(updates) => {
                                let update_count = updates.len();
                                let mut decoded_updates = Vec::with_capacity(update_count);

                                for update in updates.iter() {
                                    if let Ok(decoded) = Update::decode_v1(update) {
                                        decoded_updates.push(decoded);
                                    }
                                }

                                if !decoded_updates.is_empty() {
                                    let awareness = awareness_clone.write().await;
                                    let mut txn = awareness.doc().transact_mut();

                                    for decoded in decoded_updates {
                                        if let Err(e) = txn.apply_update(decoded) {
                                            warn!("Failed to apply update from Redis: {}", e);
                                        }
                                    }
                                    drop(txn);
                                    drop(awareness);
                                }
                            },
                            Err(e) => {
                                error!("Error reading from Redis Stream '{}': {}", stream_key, e);
                                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                            },
                        }
                    } => {},
                    _ = async {
                        let result = redis_store_for_sub_clone
                            .read_awareness_updates(
                                &doc_name_for_sub_clone,
                                &awareness_last_read_id,
                                500,
                                Some(instance_id_clone.as_str()),
                            )
                            .await;

                        match result {
                            Ok(awareness_updates) => {
                                let update_count = awareness_updates.len();
                                if update_count > 0 {
                                    let awareness = awareness_clone.write().await;
                                    for (_instance_id, data) in awareness_updates {
                                        if let Some(data) = data {
                                            if let Ok(awareness_update) = yrs::sync::awareness::AwarenessUpdate::decode_v1(&data) {
                                                if let Err(e) = awareness.apply_update(awareness_update) {
                                                    warn!("Failed to apply awareness update from Redis: {}", e);
                                                }
                                            } else {
                                                warn!("Failed to decode awareness update from Redis");
                                            }
                                        }
                                    }
                                    drop(awareness);
                                }
                            },
                            Err(e) => {
                                warn!("Error reading awareness updates from Redis: {}", e);
                                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                            },
                        }
                    } => {}
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                tokio::task::yield_now().await;
            }
        });

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
            doc_name,
            instance_id,
            last_read_id,
            awareness_updater: Some(awareness_updater),
            awareness_shutdown_tx: Some(awareness_shutdown_tx),
            redis_subscriber_task: Some(redis_subscriber_task),
            redis_subscriber_shutdown_tx: Some(redis_subscriber_shutdown_tx),
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

    pub fn get_last_read_id(&self) -> &Arc<Mutex<String>> {
        &self.last_read_id
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
            let redis_store = self.redis_store.clone();
            let doc_name = self.doc_name.clone();
            let stream_key = format!("yjs:stream:{doc_name}");
            let instance_id = self.instance_id.clone();
            let mut conn = match redis_store.create_dedicated_connection().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to create dedicated Redis connection: {}", e);
                    return Subscription {
                        sink_task: tokio::spawn(async { Ok(()) }),
                        stream_task: tokio::spawn(async { Ok(()) }),
                    };
                }
            };
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

                    match Self::handle_msg(
                        &protocol,
                        &awareness,
                        msg,
                        &redis_store,
                        &mut conn,
                        &stream_key,
                        &instance_id,
                    )
                    .await
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
        redis_store: &RedisStore,
        conn: &mut redis::aio::MultiplexedConnection,
        stream_key: &str,
        instance_id: &str,
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
                        .publish_update_with_ttl(
                            conn,
                            stream_key,
                            &update_bytes,
                            instance_id,
                            43200,
                        )
                        .await
                    {
                        warn!("Failed to publish update to Redis: {}", e);
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
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        let client_id = {
            let awareness_read = self.awareness_ref.read().await;
            awareness_read.client_id()
        };
        self.redis_store
            .remove_instance_heartbeat(&self.doc_name, &client_id)
            .await?;

        let conn_count = self
            .redis_store
            .get_active_instances(&self.doc_name, 60)
            .await?;
        if conn_count <= 0 {
            let lock_id = format!("gcs:lock:{}", self.doc_name);
            let instance_id = format!("instance-{}", rand::random::<u64>());

            let lock_acquired = self
                .redis_store
                .acquire_doc_lock(&lock_id, &instance_id)
                .await?;

            if lock_acquired {
                let awareness = self.awareness_ref.write().await;
                let awareness_doc = awareness.doc();

                {
                    let last_stream_id = self
                        .redis_store
                        .get_stream_last_id(&self.doc_name)
                        .await
                        .ok()
                        .flatten();

                    if let Some(ref id) = last_stream_id {
                        info!("Got last stream ID before GCS save: {}", id);
                    } else {
                        info!("No stream ID found before GCS save");
                    }

                    let gcs_doc = Doc::new();
                    let mut gcs_txn = gcs_doc.transact_mut();

                    if let Err(e) = self.storage.load_doc(&self.doc_name, &mut gcs_txn).await {
                        warn!("Failed to load current state from GCS: {}", e);
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
                        let update_future = self.storage.push_update(
                            &self.doc_name,
                            &update_bytes,
                            &self.redis_store,
                        );
                        let flush_future =
                            self.storage.flush_doc_v2(&self.doc_name, &awareness_txn);

                        let (update_result, flush_result) =
                            tokio::join!(update_future, flush_future);

                        if let Err(e) = flush_result {
                            warn!("Failed to flush document directly to storage: {}", e);
                        }
                        if let Err(e) = update_result {
                            warn!("Failed to update document in storage: {}", e);
                        }

                        if let Some(last_id) = last_stream_id {
                            if let Err(e) = self
                                .redis_store
                                .trim_stream_before(&self.doc_name, &last_id)
                                .await
                            {
                                warn!("Failed to trim Redis stream after GCS save: {}", e);
                            }
                        }
                    }
                }
            }

            if let Err(e) = self
                .redis_store
                .release_doc_lock(&lock_id, &instance_id)
                .await
            {
                warn!("Failed to release GCS lock: {}", e);
            }
            self.redis_store
                .safe_delete_stream(&self.doc_name, &self.instance_id)
                .await?;
            self.redis_store
                .delete_awareness_stream(&self.doc_name)
                .await?;
        }

        Ok(())
    }
}

impl Drop for BroadcastGroup {
    fn drop(&mut self) {
        if let Some(tx) = self.awareness_shutdown_tx.take() {
            if let Err(e) = tx.send(()) {
                warn!("Failed to send awareness shutdown signal: {:?}", e);
                if let Some(task) = self.awareness_updater.take() {
                    task.abort();
                }
            }
        }
        if let Some(tx) = self.heartbeat_shutdown_tx.take() {
            if let Err(e) = tx.send(()) {
                warn!("Failed to send heartbeat shutdown signal: {:?}", e);
                if let Some(task) = self.heartbeat_task.take() {
                    task.abort();
                }
            }
        }
        if let Some(tx) = self.redis_subscriber_shutdown_tx.take() {
            if let Err(e) = tx.send(()) {
                warn!("Failed to send redis subscriber shutdown signal: {:?}", e);
                if let Some(task) = self.redis_subscriber_task.take() {
                    task.abort();
                }
            }
        }
        if let Some(tx) = self.sync_shutdown_tx.take() {
            if let Err(e) = tx.send(()) {
                warn!("Failed to send sync shutdown signal: {:?}", e);
                if let Some(task) = self.sync_task.take() {
                    task.abort();
                }
            }
        }
    }
}
