use google_cloud_pubsub::client::{Client, ClientConfig};

pub(crate) mod google;
pub(crate) mod noop;

#[derive(thiserror::Error, Debug, Clone)]
pub(crate) enum PubSubBackendError {
    #[error("Failed to try from : {0}")]
    TryFrom(String),
    #[error("Failed to create : {0}")]
    Create(String),
}

impl PubSubBackendError {
    pub(crate) fn create<T: ToString>(message: T) -> Self {
        Self::Create(message.to_string())
    }
}

#[derive(Clone)]
pub(crate) enum PubSubBackend {
    Google(google::CloudPubSub),
    Noop(noop::NoopPubSub),
}

impl PubSubBackend {
    pub(crate) async fn try_from(value: &str) -> Result<Self, PubSubBackendError> {
        let value = value.to_lowercase();
        match value.as_str() {
            "google" => {
                let config = ClientConfig::default()
                    .with_auth()
                    .await
                    .map_err(PubSubBackendError::create)?;
                let client = Client::new(config)
                    .await
                    .map_err(PubSubBackendError::create)?;
                Ok(Self::Google(google::CloudPubSub::new(client)))
            }
            "noop" => Ok(Self::Noop(noop::NoopPubSub {})),
            _ => Err(PubSubBackendError::TryFrom(value)),
        }
    }
}
