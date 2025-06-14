use std::sync::Arc;

use tokio::sync::{
    broadcast::{Receiver, Sender},
    Notify,
};
use tracing::{error, info, Level, Span};

use crate::node::{EdgeId, NodeHandle, NodeStatus};

#[derive(Debug, Clone)]
pub enum Event {
    SourceFlushed,
    ProcessorFinished {
        node: NodeHandle,
        name: String,
    },
    ProcessorFailed {
        node: NodeHandle,
        name: String,
    },
    SinkFinishFailed {
        name: String,
    },
    SinkFinished {
        node: NodeHandle,
        name: String,
    },
    EdgeCompleted {
        feature_id: uuid::Uuid,
        edge_id: EdgeId,
    },
    EdgePassThrough {
        feature_id: uuid::Uuid,
        edge_id: EdgeId,
    },
    Log {
        level: Level,
        span: Option<Span>,
        node_handle: Option<NodeHandle>,
        node_name: Option<String>,
        message: String,
    },
    NodeStatusChanged {
        node_handle: NodeHandle,
        status: NodeStatus,
        feature_id: Option<uuid::Uuid>,
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

    pub fn send(&self, event: Event) {
        let _ = self.sender.send(event);
    }

    pub fn info_log<T: ToString>(&self, span: Option<Span>, message: T) {
        self.send(Event::Log {
            level: Level::INFO,
            span,
            node_handle: None,
            node_name: None,
            message: message.to_string(),
        });
    }

    pub fn info_log_with_node_handle<T: ToString>(
        &self,
        span: Option<Span>,
        node_handle: NodeHandle,
        message: T,
    ) {
        self.send(Event::Log {
            level: Level::INFO,
            span,
            node_handle: Some(node_handle),
            node_name: None,
            message: message.to_string(),
        });
    }

    pub fn info_log_with_node_info<T: ToString>(
        &self,
        span: Option<Span>,
        node_handle: NodeHandle,
        node_name: String,
        message: T,
    ) {
        self.send(Event::Log {
            level: Level::INFO,
            span,
            node_handle: Some(node_handle),
            node_name: Some(node_name),
            message: message.to_string(),
        });
    }

    pub fn debug_log<T: ToString>(&self, span: Option<Span>, message: T) {
        self.send(Event::Log {
            level: Level::DEBUG,
            span,
            node_handle: None,
            node_name: None,
            message: message.to_string(),
        });
    }

    pub fn debug_log_with_node_handle<T: ToString>(
        &self,
        span: Option<Span>,
        node_handle: NodeHandle,
        message: T,
    ) {
        self.send(Event::Log {
            level: Level::DEBUG,
            span,
            node_handle: Some(node_handle),
            node_name: None,
            message: message.to_string(),
        });
    }

    pub async fn simple_flush(&self, delay_ms: u64) {
        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
    }

    pub async fn enhanced_flush(&self, max_wait_ms: u64) {
        let start = std::time::Instant::now();
        let max_duration = tokio::time::Duration::from_millis(max_wait_ms);

        while start.elapsed() < max_duration {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            if self.sender.receiver_count() == 0 {
                break;
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    pub fn warn_log<T: ToString>(&self, span: Option<Span>, message: T) {
        self.send(Event::Log {
            level: Level::WARN,
            span,
            node_handle: None,
            node_name: None,
            message: message.to_string(),
        });
    }

    pub fn warn_log_with_node_handle<T: ToString>(
        &self,
        span: Option<Span>,
        node_handle: NodeHandle,
        message: T,
    ) {
        self.send(Event::Log {
            level: Level::WARN,
            span,
            node_handle: Some(node_handle),
            node_name: None,
            message: message.to_string(),
        });
    }

    pub fn error_log<T: ToString>(&self, span: Option<Span>, message: T) {
        self.send(Event::Log {
            level: Level::ERROR,
            span,
            node_handle: None,
            node_name: None,
            message: message.to_string(),
        });
    }

    pub fn error_log_with_node_handle<T: ToString>(
        &self,
        span: Option<Span>,
        node_handle: NodeHandle,
        message: T,
    ) {
        self.send(Event::Log {
            level: Level::ERROR,
            span,
            node_handle: Some(node_handle),
            node_name: None,
            message: message.to_string(),
        });
    }

    pub fn error_log_with_node_info<T: ToString>(
        &self,
        span: Option<Span>,
        node_handle: NodeHandle,
        node_name: String,
        message: T,
    ) {
        self.send(Event::Log {
            level: Level::ERROR,
            span,
            node_handle: Some(node_handle),
            node_name: Some(node_name),
            message: message.to_string(),
        });
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
