use crate::storage::redis::RedisStore;
use anyhow::Result;
use bytes::Bytes;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::time::interval;
use tracing::warn;
use yrs::{updates::decoder::Decode, Doc, ReadTxn, StateVector, Transact, Update};

pub struct Publish {
    count: Arc<Mutex<u32>>,
    doc: Arc<Mutex<Doc>>,
    flush_sender: mpsc::Sender<()>,
    _timer_task: Option<tokio::task::JoinHandle<()>>,
}

impl Publish {
    pub fn new(redis_store: Arc<RedisStore>, stream_key: String, instance_id: String) -> Self {
        let doc = Arc::new(Mutex::new(Doc::new()));
        let doc_clone = doc.clone();
        let redis_clone = redis_store.clone();
        let stream_key_clone = stream_key.clone();
        let count = Arc::new(Mutex::new(0));
        let count_clone = count.clone();
        let instance_id_clone = instance_id.clone();

        let (flush_sender, mut flush_receiver) = mpsc::channel(32);

        let timer_task = tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(22));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        let mut doc_lock = doc_clone.lock().await;
                        if doc_lock.transact().encode_state_as_update_v1(&StateVector::default()).len() > 2 {
                            let update = {
                                let txn = doc_lock.transact_mut();
                                txn.encode_state_as_update_v1(&StateVector::default())
                            };

                            if let Err(e) = redis_clone.publish_update_with_origin(&stream_key_clone, &update, &instance_id_clone).await {
                                warn!("Failed to flush document: {}", e);
                            }

                            *doc_lock = Doc::new();
                            let mut count = count_clone.lock().await;
                            *count = 0;
                        }
                    }
                    _ = flush_receiver.recv() => {
                        let mut doc_lock = doc_clone.lock().await;
                        if doc_lock.transact().encode_state_as_update_v1(&StateVector::default()).len() > 2 {
                            let update = {
                                let txn = doc_lock.transact_mut();
                                txn.encode_state_as_update_v1(&StateVector::default())
                            };

                            if let Err(e) = redis_clone.publish_update_with_origin(&stream_key_clone, &update, &instance_id_clone).await {
                                warn!("Failed to flush document: {}", e);
                            }

                            *doc_lock = Doc::new();
                            let mut count = count_clone.lock().await;
                            *count = 0;
                        }
                    }
                }
            }
        });

        Self {
            count,
            doc,
            flush_sender,
            _timer_task: Some(timer_task),
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

            if *count > 5 {
                let _ = self.flush_sender.send(()).await;
            }
        }

        Ok(())
    }
}

impl Drop for Publish {
    fn drop(&mut self) {
        if let Some(task) = self._timer_task.take() {
            task.abort();
        }
    }
}
