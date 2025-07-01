use anyhow::Result;
use tokio::select;
use tokio::task::JoinHandle;

pub struct Subscription {
    pub sink_task: JoinHandle<Result<()>>,
    pub stream_task: JoinHandle<Result<()>>,
}

impl Subscription {
    pub async fn completed(self) -> Result<()> {
        let res = select! {
            r1 = self.sink_task => r1,
            r2 = self.stream_task => r2,
        };
        res.map_err(|e| anyhow::anyhow!(e))?
    }
}
