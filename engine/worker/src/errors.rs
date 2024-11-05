#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to create tokio runtime: {0}")]
    FailedToCreateTokioRuntime(#[source] std::io::Error),

    #[error("Failed to download metadata: {0}")]
    FailedToDownloadMetadata(#[source] object_store::Error),

    #[error("Failed to download workflow file: {0}")]
    FailedToDownloadWorkflow(#[source] object_store::Error),

    #[error("Failed to download asset files: {0}")]
    FailedToDownloadAssetFiles(String),

    #[error("Failed to create workflow: {0}")]
    FailedToCreateWorkflow(String),

    #[error("Failed to encode: {0}")]
    FailedToEncode(#[source] serde_json::Error),

    #[error("Failed to initialize cli: {0}")]
    Init(String),

    #[error("Failed to run cli: {0}")]
    Run(String),

    #[error("Failed to cleanup: {0}")]
    Cleanup(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    pub(crate) fn init<T: ToString>(message: T) -> Self {
        Self::Init(message.to_string())
    }

    pub(crate) fn failed_to_create_workflow<T: ToString>(message: T) -> Self {
        Self::FailedToCreateWorkflow(message.to_string())
    }

    pub(crate) fn run<T: ToString>(message: T) -> Self {
        Self::Run(message.to_string())
    }

    pub(crate) fn failed_to_download_asset_files<T: ToString>(message: T) -> Self {
        Self::FailedToDownloadAssetFiles(message.to_string())
    }

    pub(crate) fn cleanup<T: ToString>(message: T) -> Self {
        Self::Cleanup(message.to_string())
    }
}
