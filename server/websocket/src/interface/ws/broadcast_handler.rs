use crate::application::service::broadcast::BroadcastGroupService;
use crate::domain::entity::BroadcastConfig;
use crate::domain::repository::{AwarenessRepository, BroadcastRepository, WebSocketRepository};
use crate::domain::repository::{kv, redis};
use crate::domain::value_objects::document_name::DocumentName;
use crate::Subscription;
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use yrs::sync::{DefaultProtocol, Message, Protocol};

/// WebSocket handler for Y.js collaborative editing using DDD architecture
pub struct BroadcastWebSocketHandler<S, R, B, A, W>
where
    S: kv::KVStore + Send + Sync + 'static,
    R: redis::RedisRepository + Send + Sync + 'static,
    B: BroadcastRepository + Send + Sync + 'static,
    A: AwarenessRepository + Send + Sync + 'static,
    W: WebSocketRepository + Send + Sync + 'static,
{
    service: Arc<BroadcastGroupService<S, R, B, A, W>>,
}

impl<S, R, B, A, W> BroadcastWebSocketHandler<S, R, B, A, W>
where
    S: kv::KVStore + Send + Sync + 'static,
    R: redis::RedisRepository + Send + Sync + 'static,
    B: BroadcastRepository + Send + Sync + 'static,
    A: AwarenessRepository + Send + Sync + 'static,
    W: WebSocketRepository + Send + Sync + 'static,
{
    pub fn new(service: Arc<BroadcastGroupService<S, R, B, A, W>>) -> Self {
        Self { service }
    }

    /// Handle WebSocket connection for Y.js collaborative editing
    pub async fn handle_connection(
        &self,
        document_name: DocumentName,
        sink: Arc<Mutex<W::Sink>>,
        stream: W::Stream,
        user_token: Option<String>,
    ) -> Result<Subscription> {
        // Get or create broadcast group with awareness
        let (group, awareness) = self
            .service
            .get_or_create_group_with_awareness(document_name.clone())
            .await?;

        // Increment connection count
        self.service.increment_connections(&group).await?;

        // Start background tasks for the group
        self.service
            .start_background_tasks(group.clone(), awareness.clone())
            .await?;

        // Create WebSocket subscription using the service
        let subscription = self
            .service
            .create_websocket_subscription(&document_name, sink, stream, user_token)
            .await?;

        Ok(subscription)
    }

    /// Handle Y.js protocol message
    pub async fn handle_message(
        &self,
        document_name: &DocumentName,
        message: Message,
    ) -> Result<Option<Message>> {
        self.service
            .handle_protocol_message(document_name, message)
            .await
    }

    /// Subscribe to document updates
    pub async fn subscribe_to_updates(
        &self,
        document_name: &DocumentName,
    ) -> Result<tokio::sync::broadcast::Receiver<bytes::Bytes>> {
        self.service.subscribe_to_group(document_name).await
    }

    /// Broadcast message to all clients
    pub async fn broadcast_message(
        &self,
        document_name: &DocumentName,
        message: bytes::Bytes,
    ) -> Result<()> {
        self.service.broadcast_message(document_name, message).await
    }
}

/// Factory for creating WebSocket handlers with proper DDD dependencies
pub struct BroadcastWebSocketHandlerFactory {
    config: BroadcastConfig,
}

impl BroadcastWebSocketHandlerFactory {
    pub fn new(config: BroadcastConfig) -> Self {
        Self { config }
    }

    /// Create a new WebSocket handler with all dependencies
    pub fn create_handler<S, R, B, A, W>(
        &self,
        storage_repo: Arc<S>,
        redis_repo: Arc<R>,
        broadcast_repo: Arc<B>,
        awareness_repo: Arc<A>,
        websocket_repo: Arc<W>,
    ) -> BroadcastWebSocketHandler<S, R, B, A, W>
    where
        S: kv::KVStore + Send + Sync + 'static,
        R: redis::RedisRepository + Send + Sync + 'static,
        B: BroadcastRepository + Send + Sync + 'static,
        A: AwarenessRepository + Send + Sync + 'static,
        W: WebSocketRepository + Send + Sync + 'static,
    {
        let service = Arc::new(BroadcastGroupService::with_config(
            broadcast_repo,
            storage_repo,
            redis_repo,
            awareness_repo,
            websocket_repo,
            self.config.clone(),
        ));

        BroadcastWebSocketHandler::new(service)
    }
}

/// Simplified WebSocket handler that uses the existing infrastructure
pub struct SimpleBroadcastHandler {
    broadcast_repo: Arc<dyn BroadcastRepository>,
    awareness_repo: Arc<dyn AwarenessRepository>,
}

impl SimpleBroadcastHandler {
    pub fn new(
        broadcast_repo: Arc<dyn BroadcastRepository>,
        awareness_repo: Arc<dyn AwarenessRepository>,
    ) -> Self {
        Self {
            broadcast_repo,
            awareness_repo,
        }
    }

    /// Handle a simple WebSocket connection with basic Y.js support
    pub async fn handle_simple_connection(
        &self,
        document_name: DocumentName,
        mut websocket_stream: tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        user_token: Option<String>,
    ) -> Result<()> {
        // Get or create broadcast group
        let group = match self.broadcast_repo.get_group(&document_name).await? {
            Some(group) => group,
            None => {
                let instance_id = crate::domain::value_objects::instance_id::InstanceId::new();
                self.broadcast_repo
                    .create_group(document_name.clone(), instance_id)
                    .await?
            }
        };

        // Load awareness
        let awareness = self.awareness_repo.load_awareness(&document_name).await?;

        // Subscribe to broadcast messages
        let mut receiver = self.broadcast_repo.subscribe(&document_name).await?;

        // Split WebSocket stream
        let (mut ws_sender, mut ws_receiver) = websocket_stream.split();

        // Handle incoming WebSocket messages
        let doc_name_clone = document_name.clone();
        let broadcast_repo_clone = self.broadcast_repo.clone();
        let awareness_clone = awareness.clone();

        let incoming_task = tokio::spawn(async move {
            while let Some(msg) = ws_receiver.next().await {
                match msg {
                    Ok(tokio_tungstenite::tungstenite::Message::Binary(data)) => {
                        // Try to decode as Y.js message
                        if let Ok(yjs_msg) = yrs::sync::Message::decode_v1(&data) {
                            // Handle the Y.js message
                            match yjs_msg {
                                yrs::sync::Message::Sync(sync_msg) => {
                                    // Handle sync messages
                                    let _ = broadcast_repo_clone
                                        .broadcast_message(&doc_name_clone, data.into())
                                        .await;
                                }
                                yrs::sync::Message::Awareness(awareness_update) => {
                                    // Handle awareness updates
                                    let _ = broadcast_repo_clone
                                        .broadcast_message(&doc_name_clone, data.into())
                                        .await;
                                }
                                _ => {}
                            }
                        }
                    }
                    Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => {
                        break;
                    }
                    Err(_) => {
                        break;
                    }
                    _ => {}
                }
            }
        });

        // Handle outgoing messages
        let outgoing_task = tokio::spawn(async move {
            while let Ok(message) = receiver.recv().await {
                let ws_msg = tokio_tungstenite::tungstenite::Message::Binary(message.to_vec());
                if ws_sender.send(ws_msg).await.is_err() {
                    break;
                }
            }
        });

        // Wait for either task to complete
        tokio::select! {
            _ = incoming_task => {},
            _ = outgoing_task => {},
        }

        Ok(())
    }
}
