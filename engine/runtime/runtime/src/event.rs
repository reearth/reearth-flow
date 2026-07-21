use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use tokio::sync::{
    broadcast::{
        error::{RecvError, TryRecvError},
        Receiver, Sender,
    },
    Notify,
};
use tracing::{error, info, Level, Span};

use crate::node::{NodeHandle, NodeStatus};

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
        /// Per-node completion counters, populated only on the terminal
        /// status emit (`Completed`/`Failed`); `None` otherwise.
        metrics: Option<NodeMetrics>,
    },
    /// Structured diagnostic signal, rendered into action logs by
    /// `LogEventHandler`. `Arc`'d since `Diagnostic` is large and every
    /// broadcast subscriber gets its own clone of `Event`.
    Diagnostic(Arc<reearth_flow_diagnostics::Diagnostic>),
}

/// Per-node completion counters carried on the terminal `NodeStatusChanged`;
/// previously only reachable by regexing the log line
/// (`worker/src/action_log_parser.rs`), now carried structurally.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct NodeMetrics {
    /// Successfully processed feature count (`Processor` only).
    pub features_processed: u64,
    /// Successfully written feature count (`Sink` only).
    pub features_written: u64,
    /// Feature count emitted downstream during `finish()` (`Processor` only).
    pub finish_feature_count: u64,
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

    /// Send a structured diagnostic as an `Event::Diagnostic`, wrapping it in
    /// the `Arc` the broadcast channel needs to clone cheaply per receiver.
    pub fn diagnostic(&self, diagnostic: reearth_flow_diagnostics::Diagnostic) {
        self.send(Event::Diagnostic(Arc::new(diagnostic)));
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

    pub fn warn_log_with_node_info<T: ToString>(
        &self,
        span: Option<Span>,
        node_handle: NodeHandle,
        node_name: String,
        message: T,
    ) {
        self.send(Event::Log {
            level: Level::WARN,
            span,
            node_handle: Some(node_handle),
            node_name: Some(node_name),
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

/// Runs every handler's `on_shutdown`, bounded by a 5s timeout so a wedged
/// handler can't hang the subscriber forever.
async fn run_shutdown(event_handlers: &[Arc<dyn EventHandler>]) {
    let shutdown_futures = event_handlers.iter().map(|handler| handler.on_shutdown());
    match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        futures::future::join_all(shutdown_futures),
    )
    .await
    {
        Ok(_) => info!("All handlers shut down successfully"),
        Err(_) => error!("Shutdown timed out for some handlers"),
    }
}

/// Drains every event still queued in the broadcast ring via non-blocking
/// `try_recv`, so in-flight events reach handlers before shutdown runs.
async fn drain_queued(
    receiver: &mut Receiver<Event>,
    event_handlers: &[Arc<dyn EventHandler>],
    dropped: &Arc<AtomicU64>,
) {
    loop {
        match receiver.try_recv() {
            Ok(ev) => {
                for handler in event_handlers.iter() {
                    handler.on_event(&ev).await;
                }
            }
            Err(TryRecvError::Lagged(n)) => {
                dropped.fetch_add(n, Ordering::Relaxed);
            }
            Err(TryRecvError::Empty) | Err(TryRecvError::Closed) => break,
        }
    }
}

pub async fn subscribe_event(
    receiver: &mut Receiver<Event>,
    notify: Arc<Notify>,
    event_handlers: &[Arc<dyn EventHandler>],
    dropped: Arc<AtomicU64>,
) {
    loop {
        tokio::select! {
            _ = notify.notified() => {
                // Flush whatever is still queued before running on_shutdown,
                // so completeness no longer depends on a fixed-delay sleep.
                drain_queued(receiver, event_handlers, &dropped).await;
                run_shutdown(event_handlers).await;
                return;
            },
            result = receiver.recv() => {
                match result {
                    Ok(ev) => {
                        for handler in event_handlers.iter() {
                            handler.on_event(&ev).await;
                        }
                    }
                    Err(RecvError::Lagged(n)) => {
                        dropped.fetch_add(n, Ordering::Relaxed);
                    }
                    Err(RecvError::Closed) => {
                        // The hub sender is gone; run the same clean shutdown
                        // as an explicit notify, so a stray sender clone
                        // outliving the notify permit still terminates.
                        run_shutdown(event_handlers).await;
                        return;
                    }
                }
            },
        }
    }
}

#[cfg(test)]
mod subscribe_event_tests {
    use std::sync::Mutex;
    use std::time::Duration;

    use super::*;

    /// Records every `on_event`/`on_shutdown` call, in order, so tests can
    /// assert both counts and ordering.
    #[derive(Default)]
    struct RecordingHandler {
        log: Mutex<Vec<String>>,
    }

    impl RecordingHandler {
        fn event_count(&self) -> usize {
            self.log
                .lock()
                .unwrap()
                .iter()
                .filter(|entry| entry.starts_with("event:"))
                .count()
        }

        fn shutdown_called(&self) -> bool {
            self.log.lock().unwrap().iter().any(|e| e == "shutdown")
        }

        /// True iff every recorded event precedes the (single) shutdown entry.
        fn events_precede_shutdown(&self) -> bool {
            let log = self.log.lock().unwrap();
            let Some(shutdown_pos) = log.iter().position(|e| e == "shutdown") else {
                return false;
            };
            log[..shutdown_pos].iter().all(|e| e.starts_with("event:"))
                && log[shutdown_pos + 1..].is_empty()
        }
    }

    #[async_trait::async_trait]
    impl EventHandler for RecordingHandler {
        async fn on_event(&self, _event: &Event) {
            self.log.lock().unwrap().push("event:seen".to_string());
        }

        async fn on_shutdown(&self) {
            self.log.lock().unwrap().push("shutdown".to_string());
        }
    }

    fn log_event(message: &str) -> Event {
        Event::Log {
            level: Level::INFO,
            span: None,
            node_handle: None,
            node_name: None,
            message: message.to_string(),
        }
    }

    /// A capacity-2 channel flooded with 5 sends before the subscriber runs:
    /// the drain must report 3 dropped while dispatching the 2 survivors.
    #[tokio::test]
    async fn lagged_events_are_counted_and_survivors_dispatched() {
        let (sender, mut receiver) = tokio::sync::broadcast::channel::<Event>(2);
        for i in 0..5 {
            sender.send(log_event(&format!("event-{i}"))).unwrap();
        }

        let handler = Arc::new(RecordingHandler::default());
        let handlers: Vec<Arc<dyn EventHandler>> = vec![handler.clone()];
        let notify = Arc::new(Notify::new());
        // A stored permit is observed by `notified()` regardless of timing.
        notify.notify_one();
        let dropped = Arc::new(AtomicU64::new(0));

        subscribe_event(&mut receiver, notify, &handlers, Arc::clone(&dropped)).await;

        assert!(
            dropped.load(Ordering::Relaxed) >= 3,
            "expected at least 3 dropped events (5 sent - capacity 2), got {}",
            dropped.load(Ordering::Relaxed)
        );
        assert_eq!(
            handler.event_count(),
            2,
            "the 2 surviving events must still reach the handler"
        );
        assert!(handler.shutdown_called());
        assert!(handler.events_precede_shutdown());
    }

    /// (b) Normal flow: sends that fit comfortably within capacity are
    /// dispatched with no lag at all, so the counter must stay at 0.
    #[tokio::test]
    async fn normal_dispatch_keeps_dropped_counter_at_zero() {
        let (sender, mut receiver) = tokio::sync::broadcast::channel::<Event>(8);
        sender.send(log_event("a")).unwrap();
        sender.send(log_event("b")).unwrap();
        sender.send(log_event("c")).unwrap();

        let handler = Arc::new(RecordingHandler::default());
        let handlers: Vec<Arc<dyn EventHandler>> = vec![handler.clone()];
        let notify = Arc::new(Notify::new());
        notify.notify_one();
        let dropped = Arc::new(AtomicU64::new(0));

        subscribe_event(&mut receiver, notify, &handlers, Arc::clone(&dropped)).await;

        assert_eq!(dropped.load(Ordering::Relaxed), 0);
        assert_eq!(handler.event_count(), 3);
        assert!(handler.shutdown_called());
    }

    /// (c) After notify fires, any events still queued must reach handlers
    /// via the drain loop strictly before `on_shutdown` runs.
    #[tokio::test]
    async fn queued_events_reach_handlers_before_shutdown() {
        let (sender, mut receiver) = tokio::sync::broadcast::channel::<Event>(4);
        sender.send(log_event("first")).unwrap();
        sender.send(log_event("second")).unwrap();

        let handler = Arc::new(RecordingHandler::default());
        let handlers: Vec<Arc<dyn EventHandler>> = vec![handler.clone()];
        let notify = Arc::new(Notify::new());
        notify.notify_one();
        let dropped = Arc::new(AtomicU64::new(0));

        subscribe_event(&mut receiver, notify, &handlers, dropped).await;

        assert_eq!(handler.event_count(), 2);
        assert!(handler.events_precede_shutdown());
    }

    /// The hub sender being dropped (`RecvError::Closed`) must run the same
    /// shutdown path as an explicit notify, not silently vanish.
    #[tokio::test]
    async fn closed_sender_triggers_shutdown_path() {
        let (sender, mut receiver) = tokio::sync::broadcast::channel::<Event>(4);
        drop(sender);

        let handler = Arc::new(RecordingHandler::default());
        let handlers: Vec<Arc<dyn EventHandler>> = vec![handler.clone()];
        let notify = Arc::new(Notify::new());
        let dropped = Arc::new(AtomicU64::new(0));

        subscribe_event(&mut receiver, notify, &handlers, dropped).await;

        assert!(handler.shutdown_called());
        assert_eq!(handler.event_count(), 0);
    }

    /// Regression test: `notify_waiters()` stores no permit, so a
    /// notification fired before the subscriber parks is lost. Reproduces
    /// that race and asserts prompt termination, timeout-bounded so a real
    /// regression fails fast instead of hanging CI.
    #[tokio::test]
    async fn notify_fired_before_any_waiter_is_not_lost() {
        let (sender, mut receiver) = tokio::sync::broadcast::channel::<Event>(4);
        sender.send(log_event("early")).unwrap();

        let handler = Arc::new(RecordingHandler::default());
        let handlers: Vec<Arc<dyn EventHandler>> = vec![handler.clone()];
        let notify = Arc::new(Notify::new());
        // Stores a permit before `subscribe_event` is even running — the
        // exact scenario `notify_waiters()` would silently drop.
        notify.notify_one();
        let dropped = Arc::new(AtomicU64::new(0));

        let result = tokio::time::timeout(
            Duration::from_secs(5),
            subscribe_event(&mut receiver, notify, &handlers, dropped),
        )
        .await;

        assert!(
            result.is_ok(),
            "subscribe_event must terminate promptly on a pre-stored notify permit, not hang"
        );
        assert_eq!(handler.event_count(), 1);
        assert!(handler.shutdown_called());
        assert!(handler.events_precede_shutdown());
    }
}
