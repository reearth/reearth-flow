use tokio::select;
use tokio::task::JoinHandle;
use yrs::sync::protocol::Error;

pub struct Subscription {
    pub sink_task: JoinHandle<Result<(), Error>>,
    pub stream_task: JoinHandle<Result<(), Error>>,
}

impl Subscription {
    pub async fn completed(self) -> Result<(), Error> {
        let res = select! {
            r1 = self.sink_task => r1,
            r2 = self.stream_task => r2,
        };
        res.map_err(|e| Error::Other(e.into()))?
    }
}

pub struct ShutdownHandle {
    pub awareness_updater: JoinHandle<()>,
    pub awareness_shutdown_tx: tokio::sync::oneshot::Sender<()>,
    pub redis_subscriber_task: JoinHandle<()>,
    pub redis_subscriber_shutdown_tx: tokio::sync::oneshot::Sender<()>,
    pub heartbeat_task: JoinHandle<()>,
    pub heartbeat_shutdown_tx: tokio::sync::oneshot::Sender<()>,
    pub sync_task: JoinHandle<()>,
    pub sync_shutdown_tx: tokio::sync::oneshot::Sender<()>,
}

impl ShutdownHandle {
    pub fn shutdown_sync(self) {
        let _ = self.awareness_shutdown_tx.send(());
        let _ = self.heartbeat_shutdown_tx.send(());
        let _ = self.redis_subscriber_shutdown_tx.send(());
        let _ = self.sync_shutdown_tx.send(());

        self.awareness_updater.abort();
        self.redis_subscriber_task.abort();
        self.heartbeat_task.abort();
        self.sync_task.abort();
    }
}
