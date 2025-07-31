use crate::domain::value_objects::document_name::DocumentName;
use anyhow::Result;
use async_trait::async_trait;
use futures_util::Sink;
use futures_util::Stream;
use std::sync::Arc;
use yrs::sync::Message;

/// Repository interface for WebSocket subscription management
#[async_trait]
pub trait WebSocketRepository: Send + Sync {
    type Sink: Sink<Message> + Send + Sync + Unpin;
    type Stream: Stream<Item = Result<Message, Message>> + Send + Sync + Unpin;

    /// Create WebSocket subscription for Y.js protocol
    async fn create_subscription(
        &self,
        document_name: &DocumentName,
        sink: Arc<tokio::sync::Mutex<Self::Sink>>,
        stream: Self::Stream,
        user_token: Option<String>,
    ) -> Result<crate::Subscription>;

    /// Handle Y.js protocol message
    async fn handle_protocol_message(
        &self,
        document_name: &DocumentName,
        message: yrs::sync::Message,
    ) -> Result<Option<yrs::sync::Message>>;
}
