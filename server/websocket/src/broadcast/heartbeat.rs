use crate::storage::redis::RedisStore;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::warn;

pub struct HeartbeatTask {
    task: Mutex<Option<JoinHandle<()>>>,
    shutdown_tx: Mutex<Option<mpsc::Sender<()>>>,
}

impl HeartbeatTask {
    pub fn new() -> Self {
        Self {
            task: Mutex::new(None),
            shutdown_tx: Mutex::new(None),
        }
    }

    pub fn start(&self, doc_name: String, instance_id: String, redis_store: Arc<RedisStore>) {
        if self.task.lock().unwrap().is_some() {
            return;
        }

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        *self.shutdown_tx.lock().unwrap() = Some(shutdown_tx);

        let task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(56));

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        break;
                    },
                    _ = interval.tick() => {
                        if let Err(e) = redis_store
                            .update_instance_heartbeat(&doc_name, &instance_id)
                            .await
                        {
                            warn!("Failed to update instance heartbeat: {}", e);
                        }
                    }
                }
            }
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

impl Drop for HeartbeatTask {
    fn drop(&mut self) {
        self.shutdown();
    }
}

pub fn create_heartbeat_task(
    doc_name: String,
    instance_id: String,
    redis_store: Arc<RedisStore>,
) -> HeartbeatTask {
    let task = HeartbeatTask::new();
    task.start(doc_name, instance_id, redis_store);
    task
}
