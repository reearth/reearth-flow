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
