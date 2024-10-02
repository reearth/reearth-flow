use std::sync::Arc;

use tokio::sync::{
    broadcast::{Receiver, Sender},
    Notify,
};

use crate::node::NodeHandle;

#[derive(Debug, Clone)]
pub enum Event {
    SourceFlushed { node: NodeHandle, name: String },
    ProcessorStarted { node: NodeHandle, name: String },
    ProcessorFinished { node: NodeHandle, name: String },
    SinkFlushed { node: NodeHandle, name: String },
    SinkFinished { node: NodeHandle, name: String },
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
}

pub async fn subscribe_event(
    receiver: &mut Receiver<Event>,
    notify: Arc<Notify>,
    event_handlers: &[Box<dyn EventHandler>],
) {
    loop {
        tokio::select! {
            _ = notify.notified() => {
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
