use super::message::EncodableMessage;

#[async_trait::async_trait]
pub trait Publisher: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn publish<M: EncodableMessage>(&self, message: M) -> Result<(), Self::Error>;
    async fn shutdown(&self);
}
