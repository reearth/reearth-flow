use crate::storage::redis::RedisStore;
use anyhow::Result;
use bytes::Bytes;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::Mutex;
use tokio::time::{sleep, Instant};
use tracing::warn;
use yrs::{updates::decoder::Decode, Doc, ReadTxn, StateVector, Transact, Update};

pub struct Publish {
    count: Arc<Mutex<u32>>,
    doc: Arc<Mutex<Doc>>,
    new_data_sender: mpsc::Sender<()>,
    _timer_task: Option<tokio::task::JoinHandle<()>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl Publish {
    pub fn new(
        redis_store: Arc<RedisStore>,
        stream_key: String,
        instance_id: String,
        conn: &mut redis::aio::MultiplexedConnection,
    ) -> Self {
        let doc = Arc::new(Mutex::new(Doc::new()));
        let doc_clone = doc.clone();
        let stream_key_clone = stream_key.clone();
        let count = Arc::new(Mutex::new(0));
        let count_clone = count.clone();
        let instance_id_clone = instance_id.clone();
        let mut conn_clone = conn.clone();
        let mut first_publish = true;

        let (new_data_sender, mut new_data_receiver) = mpsc::channel(32);
        let (shutdown_tx, mut shutdown_rx) = oneshot::channel();

        let timer_task = tokio::spawn(async move {
            let mut last_data_time: Option<Instant> = None;

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        break;
                    }
                    _ = new_data_receiver.recv() => {
                        last_data_time = Some(Instant::now());

                        let should_flush_now = {
                            let count_value = *count_clone.lock().await;
                            count_value > 14
                        };

                        if should_flush_now {
                            let mut doc_lock = doc_clone.lock().await;
                            let count_value = *count_clone.lock().await;
                            if count_value > 0 {
                                let update = {
                                    let txn = doc_lock.transact_mut();
                                    txn.encode_state_as_update_v1(&StateVector::default())
                                };

                                if first_publish {
                                    first_publish = false;
                                    if let Err(e) = redis_store.publish_update_with_ttl(&mut conn_clone, &stream_key_clone, &update, &instance_id_clone, 43200).await {
                                        warn!("Failed to flush first document: {}", e);
                                    }
                                } else if let Err(e) = redis_store.publish_update(&mut conn_clone, &stream_key_clone, &update, &instance_id_clone).await {
                                    warn!("Failed to flush document: {}", e);
                                }

                                *doc_lock = Doc::new();
                                let mut count = count_clone.lock().await;
                                *count = 0;
                                last_data_time = None;
                            }
                        }
                    }
                    _ = sleep(Duration::from_millis(10)) => {
                        if let Some(last_time) = last_data_time {
                            if last_time.elapsed() >= Duration::from_millis(20) {
                                let mut doc_lock = doc_clone.lock().await;
                                let count_value = *count_clone.lock().await;
                                if count_value > 0 {
                                    let update = {
                                        let txn = doc_lock.transact_mut();
                                        txn.encode_state_as_update_v1(&StateVector::default())
                                    };

                                    if first_publish {
                                        first_publish = false;
                                        if let Err(e) = redis_store.publish_update_with_ttl(&mut conn_clone, &stream_key_clone, &update, &instance_id_clone, 43200).await {
                                            warn!("Failed to flush first document: {}", e);
                                        }
                                    } else if let Err(e) = redis_store.publish_update(&mut conn_clone, &stream_key_clone, &update, &instance_id_clone).await {
                                        warn!("Failed to flush document: {}", e);
                                    }

                                    *doc_lock = Doc::new();
                                    let mut count = count_clone.lock().await;
                                    *count = 0;
                                    last_data_time = None;
                                }
                            }
                        }
                    }
                }
            }
        });

        Self {
            count,
            doc,
            new_data_sender,
            _timer_task: Some(timer_task),
            shutdown_tx: Some(shutdown_tx),
        }
    }

    pub async fn insert(&mut self, bytes: Bytes) -> Result<()> {
        let update = Update::decode_v1(&bytes)?;

        {
            let doc = self.doc.lock().await;
            doc.transact_mut().apply_update(update)?;
        }

        {
            let mut count = self.count.lock().await;
            *count += 1;
        }

        let _ = self.new_data_sender.send(()).await;

        Ok(())
    }
}

impl Drop for Publish {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            if let Err(e) = tx.send(()) {
                warn!("Failed to send publish timer shutdown signal: {:?}", e);
                if let Some(task) = self._timer_task.take() {
                    task.abort();
                }
            }
        }
    }
}
