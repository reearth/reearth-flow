use super::message::EncodableMessage;

#[async_trait::async_trait]
pub trait Publisher: Send + Sync {
    async fn publish<M: EncodableMessage>(&self, message: M) -> Result<(), super::errors::Error>;
    async fn shutdown(&self);
}
