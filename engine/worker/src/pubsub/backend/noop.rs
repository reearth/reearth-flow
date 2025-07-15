use crate::pubsub::message::EncodableMessage;

#[derive(thiserror::Error, Debug)]
pub enum NoopPubSubError {}

pub struct NoopPubSub {}

#[async_trait::async_trait]
impl crate::pubsub::publisher::Publisher for NoopPubSub {
    type Error = NoopPubSubError;

    async fn publish<M: EncodableMessage>(&self, _message: M) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn shutdown(&self) {}
}
