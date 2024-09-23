use std::{sync::Arc, time::Duration};

use tokio::sync::{
    broadcast::{Receiver, Sender},
    Notify,
};

use crate::node::NodeHandle;

#[derive(Debug, Clone)]
pub enum Event {
    SinkFlushed { node: NodeHandle },
    SinkFinished { node: NodeHandle },
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

pub async fn subscribe_event(receiver: &mut Receiver<Event>, notify: Arc<Notify>) {
    loop {
        tokio::select! {
            _ = notify.notified() => {
                return;
            },
            _ = tokio::time::sleep(Duration::from_millis(100)) => {}
        }
        let Ok(_) = receiver.recv().await else {
            continue;
        };
    }
}
