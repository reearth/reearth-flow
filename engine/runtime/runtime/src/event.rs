use std::sync::Arc;

use tokio::sync::{
    broadcast::{Receiver, Sender},
    Notify,
};
use tracing::{error, info};

use crate::node::{EdgeId, NodeHandle};

#[derive(Debug, Clone)]
pub enum Event {
    SourceFlushed,
    ProcessorFinished {
        node: NodeHandle,
        name: String,
    },
    SinkFinished {
        node: NodeHandle,
        name: String,
    },
    EdgePassThrough {
        feature_id: uuid::Uuid,
        edge_id: EdgeId,
    },
}

#[derive(Debug)]
pub struct EventHub {
    pub sender: Sender<Event>,
    pub receiver: Receiver<Event>,
}

impl EventHub {
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = tokio::sync::broadcast::channel(capacity);
        Self { sender, receiver }
    }
}

impl Clone for EventHub {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: self.receiver.resubscribe(),
        }
    }
}

#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    async fn on_event(&self, event: &Event);
    async fn on_shutdown(&self) {}
}

pub async fn subscribe_event(
    receiver: &mut Receiver<Event>,
    notify: Arc<Notify>,
    event_handlers: &[Arc<dyn EventHandler>],
) {
    loop {
        tokio::select! {
            _ = notify.notified() => {
                let shutdown_futures = event_handlers.iter()
                    .map(|handler| handler.on_shutdown());
                match tokio::time::timeout(
                    std::time::Duration::from_secs(5),
                    futures::future::join_all(shutdown_futures)
                ).await {
                    Ok(_) => info!("All handlers shut down successfully"),
                    Err(_) => error!("Shutdown timed out for some handlers"),
                }
                return;
            },
            Ok(ev) = receiver.recv() => {
                for handler in event_handlers.iter() {
                    handler.on_event(&ev).await;
                }
            },
        }
    }
}
