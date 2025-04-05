use crate::storage::redis::RedisStore;
use crate::AwarenessRef;
use bytes::Bytes;
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::{broadcast::Sender, mpsc, Mutex};
use tokio::task::JoinHandle;
use tracing::{debug, error, warn};
use yrs::updates::decoder::Decode;
use yrs::{Transact, Update};

pub struct RedisSubscriberTask {
    task: StdMutex<Option<JoinHandle<()>>>,
    shutdown_tx: StdMutex<Option<mpsc::Sender<()>>>,
}

impl RedisSubscriberTask {
    pub fn new() -> Self {
        Self {
            task: StdMutex::new(None),
            shutdown_tx: StdMutex::new(None),
        }
    }

    pub fn start(
        &self,
        doc_name: String,
        redis_store: Arc<RedisStore>,
        awareness_ref: AwarenessRef,
        sender: Sender<Bytes>,
        last_read_id: Arc<Mutex<String>>,
    ) {
        if self.task.lock().unwrap().is_some() {
            return;
        }

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        *self.shutdown_tx.lock().unwrap() = Some(shutdown_tx);

        let stream_key = format!("yjs:stream:{}", doc_name);

        let task = tokio::spawn(async move {
            let mut conn = if let Ok(conn) = redis_store.get_pool().get().await {
                conn
            } else {
                error!("Failed to get Redis connection");
                return;
            };

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        break;
                    },
                    _ = async {
                        let result = redis_store
                            .read_and_ack(
                                &mut conn,
                                &stream_key,
                                512,
                                &last_read_id,
                            )
                            .await;

                        match result {
                            Ok(updates) => {
                                let update_count = updates.len();
                                let mut decoded_updates = Vec::with_capacity(update_count);

                                for update in &updates {
                                    if let Ok(decoded) = Update::decode_v1(update) {
                                        decoded_updates.push(decoded);
                                    }

                                    if sender.send(update.clone()).is_err() {
                                        debug!("Failed to broadcast Redis update");
                                    }
                                }

                                if !decoded_updates.is_empty() {
                                    let awareness = awareness_ref.write().await;
                                    let mut txn = awareness.doc().transact_mut();

                                    for decoded in decoded_updates {
                                        if let Err(e) = txn.apply_update(decoded) {
                                            warn!("Failed to apply update from Redis: {}", e);
                                        }
                                    }
                                }

                            },
                            Err(e) => {
                                error!("Error reading from Redis Stream: {}", e);
                                tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
                            },
                        }

                        tokio::task::yield_now().await;
                    } => {}
                }
            }
            debug!("Redis subscriber task exited gracefully");
        });

        *self.task.lock().unwrap() = Some(task);
    }

    pub fn shutdown(&self) {
        let mut shutdown_lock = self.shutdown_tx.lock().unwrap();
        if let Some(tx) = shutdown_lock.take() {
            let _ = tx.try_send(());
            let mut task_lock = self.task.lock().unwrap();
            if let Some(task) = task_lock.take() {
                task.abort();
            }
        }
    }
}

impl Drop for RedisSubscriberTask {
    fn drop(&mut self) {
        self.shutdown();
    }
}

pub fn create_redis_subscriber_task(
    doc_name: String,
    redis_store: Arc<RedisStore>,
    awareness_ref: AwarenessRef,
    sender: Sender<Bytes>,
    last_read_id: Arc<Mutex<String>>,
) -> RedisSubscriberTask {
    let task = RedisSubscriberTask::new();
    task.start(doc_name, redis_store, awareness_ref, sender, last_read_id);
    task
}
