use crate::api::{Api, WorkerOpts};
use anyhow::Result;
use std::sync::Arc;
use tokio::time::{interval, Duration, MissedTickBehavior};
use tracing::{debug, error};

pub struct Worker {
    api: Arc<Api>,
    opts: WorkerOpts,
    running: Arc<tokio::sync::Mutex<bool>>,
}

impl Worker {
    pub fn new(api: Arc<Api>, opts: Option<WorkerOpts>) -> Self {
        let opts = opts.unwrap_or_default();
        Self {
            api,
            opts,
            running: Arc::new(tokio::sync::Mutex::new(true)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        let api = Arc::clone(&self.api);
        let opts = self.opts.clone();
        let running = Arc::clone(&self.running);

        let mut worker_interval = interval(Duration::from_millis(1000));
        worker_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        tokio::spawn(async move {
            while *running.lock().await {
                tokio::select! {
                    _ = worker_interval.tick() => {
                        match api.consume_worker_queue(&opts).await {
                            Ok(tasks) => {
                                if !tasks.is_empty() {
                                    debug!("Processed {} worker tasks", tasks.len());
                                }
                            },
                            Err(e) => {
                                error!("Error consuming worker queue: {}", e);
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.lock().await;
        *running = false;
        Ok(())
    }
}

pub async fn create_worker(api: Arc<Api>, opts: Option<WorkerOpts>) -> Result<Worker> {
    Ok(Worker::new(api, opts))
}
