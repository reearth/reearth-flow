use crate::domain::entity::gcs::GcsStore;
use crate::domain::entity::BroadcastGroup;
use crate::domain::repository::awareness::AwarenessRepository;
use crate::domain::repository::broadcast::BroadcastRepository;
use crate::domain::repository::websocket::WebSocketRepository;
use crate::domain::services::kv::DocOps;
use crate::domain::value_objects::document_name::DocumentName;
use crate::domain::value_objects::instance_id::InstanceId;
use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use yrs::sync::Awareness;

/// In-memory implementation of BroadcastRepository using broadcast channels
pub struct BroadcastRepositoryImpl {
    groups: Arc<DashMap<String, Arc<BroadcastGroup>>>,
    channels: Arc<DashMap<String, broadcast::Sender<Bytes>>>,
    buffer_capacity: usize,
}

impl BroadcastRepositoryImpl {
    pub fn new(buffer_capacity: usize) -> Self {
        Self {
            groups: Arc::new(DashMap::new()),
            channels: Arc::new(DashMap::new()),
            buffer_capacity,
        }
    }
}

#[async_trait]
impl BroadcastRepository for BroadcastRepositoryImpl {
    async fn create_group(
        &self,
        document_name: DocumentName,
        instance_id: InstanceId,
    ) -> Result<Arc<BroadcastGroup>> {
        let doc_name_str = document_name.as_str().to_string();

        // Check if group already exists
        if let Some(existing_group) = self.groups.get(&doc_name_str) {
            return Ok(existing_group.clone());
        }

        // Create new broadcast channel
        let (sender, _) = broadcast::channel(self.buffer_capacity);

        // Create new broadcast group
        let group = Arc::new(BroadcastGroup::new(document_name, instance_id));

        // Store group and channel
        self.groups.insert(doc_name_str.clone(), group.clone());
        self.channels.insert(doc_name_str, sender);

        Ok(group)
    }

    async fn get_group(&self, document_name: &DocumentName) -> Result<Option<Arc<BroadcastGroup>>> {
        let doc_name_str = document_name.as_str();
        Ok(self.groups.get(doc_name_str).map(|entry| entry.clone()))
    }

    async fn remove_group(&self, document_name: &DocumentName) -> Result<()> {
        let doc_name_str = document_name.as_str();
        self.groups.remove(doc_name_str);
        self.channels.remove(doc_name_str);
        Ok(())
    }

    async fn broadcast_message(&self, document_name: &DocumentName, message: Bytes) -> Result<()> {
        let doc_name_str = document_name.as_str();
        if let Some(sender) = self.channels.get(doc_name_str) {
            let _ = sender.send(message);
        }
        Ok(())
    }

    async fn subscribe(&self, document_name: &DocumentName) -> Result<broadcast::Receiver<Bytes>> {
        let doc_name_str = document_name.as_str();

        // Get or create channel
        let sender = match self.channels.get(doc_name_str) {
            Some(sender) => sender.clone(),
            None => {
                let (sender, _) = broadcast::channel(self.buffer_capacity);
                self.channels
                    .insert(doc_name_str.to_string(), sender.clone());
                sender
            }
        };

        Ok(sender.subscribe())
    }
}

/// Implementation of AwarenessRepository using GCS storage
pub struct AwarenessRepositoryImpl {
    storage: Arc<GcsStore>,
}

impl AwarenessRepositoryImpl {
    pub fn new(storage: Arc<GcsStore>) -> Self {
        Self { storage }
    }
}

#[async_trait]
impl AwarenessRepository for AwarenessRepositoryImpl {
    async fn load_awareness(&self, document_name: &DocumentName) -> Result<Arc<RwLock<Awareness>>> {
        // Create a new Y.js document
        let doc = yrs::Doc::new();
        
        // Load document from GCS storage
        let doc_data = match self.storage.load_doc(document_name.as_str()).await {
            Ok(data) => data,
            Err(_) => {
                // If document doesn't exist, create a new one
                Vec::new()
            }
        };

        // Apply updates to the document if any exist
        if !doc_data.is_empty() {
            use yrs::Transact;
            let mut txn = doc.transact_mut();
            if let Err(e) = txn.apply_update(yrs::Update::decode_v1(&doc_data)?) {
                warn!("Failed to apply document update: {}", e);
            }
        };

        // Create awareness from document
        let awareness = Awareness::new(doc);
        Ok(Arc::new(RwLock::new(awareness)))
    }

    async fn save_awareness_state(
        &self,
        document_name: &DocumentName,
        awareness: &Awareness,
        redis: &dyn RedisRepository<Error = anyhow::Error>,
    ) -> Result<()> {
        // Save the document state to GCS
        let doc = awareness.doc();
        let state = {
            use yrs::{ReadTxn, Transact};
            let txn = doc.transact();
            txn.encode_state_as_update_v1(&yrs::StateVector::default())
        };

        // Store the update in GCS
        self.gcs_store
            .store_doc_update(document_name.as_str(), &state)
            .await?;
        Ok(())
    }

    async fn get_awareness_update(&self, document_name: &DocumentName) -> Result<Option<Bytes>> {
        // This would typically get the latest awareness changes
        // For now, return None as this requires more complex awareness state tracking
        Ok(None)
    }
}

/// Implementation of WebSocketRepository for Y.js protocol handling
pub struct WebSocketRepositoryImpl {
    broadcast_repo: Arc<dyn BroadcastRepository>,
    awareness_repo: Arc<dyn AwarenessRepository>,
}

impl WebSocketRepositoryImpl {
    pub fn new(
        broadcast_repo: Arc<dyn BroadcastRepository>,
        awareness_repo: Arc<dyn AwarenessRepository>,
    ) -> Self {
        Self {
            broadcast_repo,
            awareness_repo,
        }
    }
}

#[async_trait]
impl WebSocketRepository for WebSocketRepositoryImpl {
    type Sink = futures_util::sink::SinkMapErr<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
        fn(tokio_tungstenite::tungstenite::Error) -> yrs::sync::Error,
    >;
    type Stream = futures_util::stream::MapErr<
        futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
        >,
        fn(tokio_tungstenite::tungstenite::Error) -> yrs::sync::Error,
    >;

    async fn create_subscription(
        &self,
        document_name: &DocumentName,
        sink: Arc<Mutex<Self::Sink>>,
        stream: Self::Stream,
        user_token: Option<String>,
    ) -> Result<Subscription> {
        // Load awareness for the document
        let awareness = self.awareness_repo.load_awareness(document_name).await?;

        // Subscribe to broadcast messages
        let receiver = self.broadcast_repo.subscribe(document_name).await?;

        // Create Y.js protocol handler
        let protocol = yrs::sync::DefaultProtocol;

        // Start message handling loop
        let doc_name = document_name.clone();
        let broadcast_repo = self.broadcast_repo.clone();

        let handle = tokio::spawn(async move {
            // Handle incoming WebSocket messages and Y.js protocol
            // This is a simplified version - full implementation would handle:
            // - Y.js sync messages
            // - Awareness updates
            // - Connection management
            // - Error handling
        });

        Ok(Subscription::new(handle))
    }

    async fn handle_protocol_message(
        &self,
        document_name: &DocumentName,
        message: yrs::sync::Message,
    ) -> Result<Option<yrs::sync::Message>> {
        use yrs::sync::{Message, SyncMessage};

        match message {
            Message::Sync(sync_msg) => {
                // Handle Y.js sync messages
                match sync_msg {
                    SyncMessage::SyncStep1(state_vector) => {
                        // Load awareness and generate sync response
                        let awareness = self.awareness_repo.load_awareness(document_name).await?;
                        let awareness_guard = awareness.read().await;
                        let doc = awareness_guard.doc();

                        use yrs::{ReadTxn, Transact};
                        let txn = doc.transact();
                        let update = txn.encode_state_as_update_v1(&state_vector);

                        if !update.is_empty() {
                            return Ok(Some(Message::Sync(SyncMessage::SyncStep2(update))));
                        }
                    }
                    SyncMessage::SyncStep2(update) => {
                        // Apply update to document
                        let awareness = self.awareness_repo.load_awareness(document_name).await?;
                        let awareness_guard = awareness.write().await;
                        let doc = awareness_guard.doc();

                        use yrs::{Transact, Update};
                        let mut txn = doc.transact_mut();
                        if let Ok(update) = Update::decode_v1(&update) {
                            txn.apply_update(update);
                        }

                        // Broadcast the update to other clients
                        let _ = self
                            .broadcast_repo
                            .broadcast_message(document_name, Bytes::from(update))
                            .await;
                    }
                    SyncMessage::Update(update) => {
                        // Apply and broadcast update
                        let awareness = self.awareness_repo.load_awareness(document_name).await?;
                        let awareness_guard = awareness.write().await;
                        let doc = awareness_guard.doc();

                        use yrs::{Transact, Update};
                        let mut txn = doc.transact_mut();
                        if let Ok(parsed_update) = Update::decode_v1(&update) {
                            txn.apply_update(parsed_update);
                        }

                        // Broadcast to other clients
                        let _ = self
                            .broadcast_repo
                            .broadcast_message(document_name, Bytes::from(update))
                            .await;
                    }
                }
            }
            Message::Awareness(awareness_update) => {
                // Handle awareness updates
                let _ = self
                    .broadcast_repo
                    .broadcast_message(document_name, Bytes::from(awareness_update.encode_v1()))
                    .await;
            }
            Message::Custom(_) => {
                // Handle custom messages if needed
            }
        }

        Ok(None)
    }
}
