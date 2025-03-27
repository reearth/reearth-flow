use tokio::task::JoinHandle;

pub struct BackgroundTasks {
    awareness_updater: JoinHandle<()>,
    awareness_shutdown_tx: tokio::sync::mpsc::Sender<()>,
    redis_subscriber_task: Option<JoinHandle<()>>,
    redis_subscriber_shutdown_tx: Option<tokio::sync::mpsc::Sender<()>>,
    heartbeat_task: Option<JoinHandle<()>>,
    heartbeat_shutdown_tx: Option<tokio::sync::mpsc::Sender<()>>,
}

impl BackgroundTasks {
    pub fn new(
        awareness_updater: JoinHandle<()>,
        awareness_shutdown_tx: tokio::sync::mpsc::Sender<()>,
    ) -> Self {
        Self {
            awareness_updater,
            awareness_shutdown_tx,
            redis_subscriber_task: None,
            redis_subscriber_shutdown_tx: None,
            heartbeat_task: None,
            heartbeat_shutdown_tx: None,
        }
    }

    pub fn set_redis_subscriber(
        &mut self,
        task: JoinHandle<()>,
        shutdown_tx: tokio::sync::mpsc::Sender<()>,
    ) {
        self.redis_subscriber_task = Some(task);
        self.redis_subscriber_shutdown_tx = Some(shutdown_tx);
    }

    pub fn set_heartbeat(
        &mut self,
        task: JoinHandle<()>,
        shutdown_tx: tokio::sync::mpsc::Sender<()>,
    ) {
        self.heartbeat_task = Some(task);
        self.heartbeat_shutdown_tx = Some(shutdown_tx);
    }

    pub fn stop_redis_subscriber(&mut self) {
        if let Some(tx) = self.redis_subscriber_shutdown_tx.take() {
            let _ = tx.try_send(());
            if let Some(task) = self.redis_subscriber_task.take() {
                task.abort();
            }
        } else if let Some(task) = self.redis_subscriber_task.take() {
            task.abort();
        }
    }

    pub fn stop_heartbeat(&mut self) {
        if let Some(tx) = self.heartbeat_shutdown_tx.take() {
            let _ = tx.try_send(());
            if let Some(task) = self.heartbeat_task.take() {
                task.abort();
            }
        } else if let Some(task) = self.heartbeat_task.take() {
            task.abort();
        }
    }

    pub fn stop_awareness_updater(&mut self) {
        let _ = self.awareness_shutdown_tx.try_send(());
        self.awareness_updater.abort();
    }

    pub fn stop_all(&mut self) {
        self.stop_redis_subscriber();
        self.stop_heartbeat();
        self.stop_awareness_updater();
    }
}

impl Drop for BackgroundTasks {
    fn drop(&mut self) {
        self.stop_all();
    }
}
