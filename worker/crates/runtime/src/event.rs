use tokio::sync::broadcast::{Receiver, Sender};

use crate::node::NodeHandle;

#[derive(Debug, Clone)]
pub enum Event {
    SinkFlushed { node: NodeHandle },
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
            receiver: self.sender.subscribe(),
        }
    }
}
