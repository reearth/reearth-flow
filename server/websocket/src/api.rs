use crate::storage::gcs::GcsStore;
use crate::storage::kv::DocOps;
use crate::storage::redis::{RedisStore, StreamMessageResult};
use anyhow::Result;
use serde_json;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, warn};
use uuid::Uuid;
use yrs::sync::awareness::Awareness;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{Doc, ReadTxn, Transact, Update};

/// Compare Redis IDs like JavaScript version
/// Returns true if a < b
pub fn is_smaller_redis_id(a: &str, b: &str) -> bool {
    let a_parts: Vec<&str> = a.split('-').collect();
    let b_parts: Vec<&str> = b.split('-').collect();

    let a1 = a_parts.first().unwrap_or(&"0").parse::<u64>().unwrap_or(0);
    let a2 = a_parts.get(1).unwrap_or(&"0").parse::<u64>().unwrap_or(0);
    let b1 = b_parts.first().unwrap_or(&"0").parse::<u64>().unwrap_or(0);
    let b2 = b_parts.get(1).unwrap_or(&"0").parse::<u64>().unwrap_or(0);

    a1 < b1 || (a1 == b1 && a2 < b2)
}

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub redis_task_debounce: u64,        // Default: 10 seconds
    pub redis_min_message_lifetime: u64, // Default: 60 seconds
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            redis_task_debounce: 10000,        // 10 seconds in ms
            redis_min_message_lifetime: 60000, // 1 minute in ms
        }
    }
}

pub struct Api {
    pub redis_store: Arc<RedisStore>,
    pub storage: Arc<GcsStore>,
    pub config: ApiConfig,
    pub consumer_name: String,
    pub redis_worker_stream_name: String,
    pub redis_worker_group_name: String,
    pub prefix: String,
    pub _destroyed: Arc<Mutex<bool>>,
}

#[derive(Debug)]
pub struct GetDocResult {
    pub ydoc: Doc,
    pub awareness: Awareness,
    pub redis_last_id: String,
    pub store_references: Option<Vec<String>>,
    pub doc_changed: bool,
}

impl Api {
    pub async fn new(
        redis_store: Arc<RedisStore>,
        storage: Arc<GcsStore>,
        config: Option<ApiConfig>,
    ) -> Result<Self> {
        let config = config.unwrap_or_default();
        let prefix = redis_store.get_prefix();
        let consumer_name = Uuid::new_v4().to_string();
        let redis_worker_stream_name = format!("{}:worker", prefix);
        let redis_worker_group_name = format!("{}:worker", prefix);

        // Try to create consumer group
        if let Ok(mut conn) = redis_store.get_pool().get().await {
            let _: Result<(), redis::RedisError> = redis::cmd("XGROUP")
                .arg("CREATE")
                .arg(&redis_worker_stream_name)
                .arg(&redis_worker_group_name)
                .arg("0")
                .arg("MKSTREAM")
                .query_async(&mut *conn)
                .await;
        }

        Ok(Api {
            redis_store,
            storage,
            config,
            consumer_name,
            redis_worker_stream_name,
            redis_worker_group_name,
            prefix,
            _destroyed: Arc::new(Mutex::new(false)),
        })
    }

    /// Get messages from multiple streams like JavaScript version
    pub async fn get_messages(
        &self,
        streams: Vec<(String, String)>, // (stream_key, id) pairs
    ) -> Result<Vec<StreamMessageResult>> {
        self.redis_store.get_messages(streams).await
    }

    /// Add message to stream like JavaScript version
    pub async fn add_message(&self, room: &str, docid: &str, message: &[u8]) -> Result<()> {
        let mut conn = self.redis_store.create_dedicated_connection().await?;
        let stream_key = self.redis_store.compute_redis_room_stream_name(room, docid);

        // Handle sync step 2 like a normal update message (like JavaScript)
        let mut message_to_send = message.to_vec();
        if message.len() >= 2 && message[0] == 0 && message[1] == 1 {
            // sync step 2
            if message.len() < 4 {
                // message does not contain any content, don't distribute
                return Ok(());
            }
            message_to_send[1] = 2; // change to sync update
        }

        self.redis_store
            .add_message(
                &mut conn,
                &stream_key,
                &message_to_send,
                &self.redis_worker_stream_name,
            )
            .await
    }

    /// Get state vector from storage
    pub async fn get_state_vector(&self, room: &str, docid: &str) -> Result<Option<Vec<u8>>> {
        // Try to load document from GCS storage and compute state vector
        let temp_doc = Doc::new();
        {
            let mut txn = temp_doc.transact_mut();
            let loaded = self
                .storage
                .load_doc(&format!("{}/{}", room, docid), &mut txn)
                .await?;
            if !loaded {
                return Ok(None);
            }
        }

        let txn = temp_doc.transact();
        let state_vector = txn.state_vector().encode_v1();
        Ok(Some(state_vector))
    }

    /// Get document like JavaScript version
    pub async fn get_doc(&self, room: &str, docid: &str) -> Result<GetDocResult> {
        debug!("getDoc({}, {})", room, docid);

        let stream_key = self.redis_store.compute_redis_room_stream_name(room, docid);
        let streams = vec![(stream_key, "0".to_string())];
        let messages = self.get_messages(streams).await?;

        debug!("getDoc({}, {}) - retrieved messages", room, docid);

        let doc_messages = messages.first();

        // Load existing document state from storage
        let ydoc = Doc::new();
        let mut doc_loaded = false;
        {
            let mut txn = ydoc.transact_mut();
            let doc_key = format!("{}/{}", room, docid);
            match self.storage.load_doc(&doc_key, &mut txn).await {
                Ok(loaded) => {
                    doc_loaded = loaded;
                    if !loaded {
                        debug!(
                            "No existing document found in storage for {}/{}",
                            room, docid
                        );
                    } else {
                        debug!(
                            "Loaded existing document from storage for {}/{}",
                            room, docid
                        );
                    }
                }
                Err(e) => {
                    warn!("Failed to load document from storage: {}", e);
                }
            }
        }

        debug!("getDoc({}, {}) - retrieved doc from storage", room, docid);

        let awareness = Awareness::new(ydoc.clone());
        let _ = awareness.set_local_state(None::<serde_json::Value>); // we don't want to propagate awareness state

        let mut doc_changed = false;
        let mut redis_last_id = "0".to_string();

        if let Some(messages_result) = doc_messages {
            redis_last_id = messages_result.last_id.clone();

            let mut txn = ydoc.transact_mut();
            let initial_state = txn.state_vector();

            for message in &messages_result.messages {
                // Try to decode and apply the message
                // This is a simplified version - you might need more sophisticated message parsing
                if let Ok(update) = Update::decode_v1(message) {
                    if let Err(e) = txn.apply_update(update) {
                        warn!("Failed to apply update: {}", e);
                    }
                }
            }

            let final_state = txn.state_vector();
            doc_changed = initial_state != final_state;
        }

        // Generate references for tracking document versions
        let store_references = if doc_loaded {
            Some(vec![uuid::Uuid::new_v4().to_string()])
        } else {
            None
        };

        Ok(GetDocResult {
            ydoc,
            awareness,
            redis_last_id,
            store_references,
            doc_changed,
        })
    }

    pub async fn destroy(&self) -> Result<()> {
        let mut destroyed = self._destroyed.lock().await;
        *destroyed = true;
        Ok(())
    }

    /// Consume worker queue like JavaScript version
    pub async fn consume_worker_queue(&self, opts: &WorkerOpts) -> Result<Vec<String>> {
        let mut tasks = Vec::new();
        let mut conn = self.redis_store.get_pool().get().await?;

        // Try to reclaim tasks using XAUTOCLAIM
        let reclaimed_result: redis::Value = redis::cmd("XAUTOCLAIM")
            .arg(&self.redis_worker_stream_name)
            .arg(&self.redis_worker_group_name)
            .arg(&self.consumer_name)
            .arg(self.config.redis_task_debounce)
            .arg("0")
            .arg("COUNT")
            .arg(opts.try_claim_count)
            .query_async(&mut *conn)
            .await?;

        // Parse reclaimed tasks
        let mut task_streams = Vec::new();
        if let redis::Value::Array(reclaimed_array) = reclaimed_result {
            if reclaimed_array.len() >= 2 {
                if let redis::Value::Array(messages) = &reclaimed_array[1] {
                    for message in messages {
                        if let redis::Value::Array(msg_data) = message {
                            if msg_data.len() >= 2 {
                                let id = match &msg_data[0] {
                                    redis::Value::BulkString(id_bytes) => {
                                        std::str::from_utf8(id_bytes).unwrap_or("").to_string()
                                    }
                                    _ => continue,
                                };

                                if let redis::Value::Array(fields) = &msg_data[1] {
                                    for chunk in fields.chunks(2) {
                                        if chunk.len() == 2 {
                                            if let (
                                                redis::Value::BulkString(key),
                                                redis::Value::BulkString(value),
                                            ) = (&chunk[0], &chunk[1])
                                            {
                                                if key == b"compact" {
                                                    let stream = std::str::from_utf8(value)
                                                        .unwrap_or("")
                                                        .to_string();
                                                    task_streams.push(stream.clone());
                                                    tasks.push(id.clone());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if task_streams.is_empty() {
            debug!("No tasks available, pausing..");
            return Ok(vec![]);
        }

        debug!("Accepted {} tasks", task_streams.len());

        // Process each task stream
        for (i, stream) in task_streams.iter().enumerate() {
            let stream_len: usize = redis::cmd("XLEN")
                .arg(stream)
                .query_async(&mut *conn)
                .await?;

            if stream_len == 0 {
                // Stream is empty, remove the task
                let _: () = redis::cmd("DEL")
                    .arg(stream)
                    .query_async(&mut *conn)
                    .await?;

                if let Some(task_id) = tasks.get(i) {
                    let _: () = redis::cmd("XDEL")
                        .arg(&self.redis_worker_stream_name)
                        .arg(task_id)
                        .query_async(&mut *conn)
                        .await?;
                }

                debug!(
                    "Stream {} still empty, removing recurring task from queue",
                    stream
                );
            } else {
                // Process the stream - compact the document
                if let Err(e) = self.process_stream_for_compaction(stream).await {
                    warn!("Failed to compact stream {}: {}", stream, e);
                }

                // Re-add the task to the queue for future processing
                let _: () = redis::cmd("XADD")
                    .arg(&self.redis_worker_stream_name)
                    .arg("*")
                    .arg("compact")
                    .arg(stream)
                    .query_async(&mut *conn)
                    .await?;

                // Remove the current task
                if let Some(task_id) = tasks.get(i) {
                    let _: () = redis::cmd("XDEL")
                        .arg(&self.redis_worker_stream_name)
                        .arg(task_id)
                        .query_async(&mut *conn)
                        .await?;
                }

                debug!("Compacted stream: {}", stream);
            }
        }

        Ok(task_streams)
    }

    /// Process a stream for document compaction like JavaScript version
    async fn process_stream_for_compaction(&self, stream: &str) -> Result<()> {
        // Extract room and docid from stream name (format: prefix:room:room:docid)
        let stream_parts: Vec<&str> = stream.split(':').collect();
        if stream_parts.len() >= 4 && stream_parts[1] == "room" {
            let room = urlencoding::decode(stream_parts[2])?.into_owned();
            let docid = urlencoding::decode(stream_parts[3])?.into_owned();

            debug!(
                "Processing stream for compaction: room={}, docid={}",
                room, docid
            );

            // Get the document state and persist it if changed
            let doc_result = self.get_doc(&room, &docid).await?;
            if doc_result.doc_changed {
                // Save the document to persistent storage
                let doc_key = format!("{}/{}", room, docid);
                if let Err(e) = self
                    .storage
                    .flush_doc_v2(&doc_key, &doc_result.ydoc.transact())
                    .await
                {
                    warn!("Failed to persist document {}: {}", doc_key, e);
                } else {
                    debug!(
                        "Successfully persisted document {} after compaction",
                        doc_key
                    );
                }

                // Trim the Redis stream to remove old entries
                let last_id = if doc_result.redis_last_id != "0" {
                    doc_result.redis_last_id
                } else {
                    "0".to_string()
                };

                if last_id != "0" {
                    let min_id = (last_id
                        .split('-')
                        .next()
                        .unwrap_or("0")
                        .parse::<u64>()
                        .unwrap_or(0)
                        .saturating_sub(self.config.redis_min_message_lifetime))
                    .to_string();

                    let mut conn = self.redis_store.create_dedicated_connection().await?;
                    let _: () = redis::cmd("XTRIM")
                        .arg(stream)
                        .arg("MINID")
                        .arg(&min_id)
                        .query_async(&mut conn)
                        .await?;

                    debug!("Trimmed stream {} before ID {}", stream, min_id);
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct WorkerOpts {
    pub try_claim_count: usize,
}

impl Default for WorkerOpts {
    fn default() -> Self {
        Self { try_claim_count: 5 }
    }
}

pub struct Worker {
    pub api: Arc<Api>,
    pub opts: WorkerOpts,
}

impl Worker {
    pub fn new(api: Arc<Api>, opts: Option<WorkerOpts>) -> Self {
        let opts = opts.unwrap_or_default();
        Self { api, opts }
    }
}
