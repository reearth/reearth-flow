use std::fmt;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use thiserror::Error;
use tracing::error;

use crate::domain::entity::doc::Document;
use crate::domain::repository::document::DocumentRepository;
use crate::domain::value_objects::http::HistoryItem;

#[derive(Debug, Error)]
pub enum DocumentUseCaseError {
    #[error("document not found: {document_id}")]
    NotFound { document_id: String },
    #[error("invalid request: {message}")]
    InvalidRequest { message: String },
    #[error("{message}")]
    Unexpected {
        message: String,
        #[source]
        source: anyhow::Error,
    },
}

pub struct DocumentUseCase {
    repository: Arc<dyn DocumentRepository>,
}

impl DocumentUseCase {
    pub fn new(repository: Arc<dyn DocumentRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_snapshot(
        &self,
        doc_id: &str,
        version: u64,
    ) -> Result<Document, DocumentUseCaseError> {
        match self.repository.create_snapshot(doc_id, version).await {
            Ok(Some(document)) => Ok(document),
            Ok(None) => Err(DocumentUseCaseError::NotFound {
                document_id: doc_id.to_string(),
            }),
            Err(err) => Err(DocumentUseCaseError::Unexpected {
                message: format!(
                    "failed to create snapshot for document '{}' at version {}",
                    doc_id, version
                ),
                source: err,
            }),
        }
    }

    pub async fn get_latest_document(
        &self,
        doc_id: &str,
    ) -> Result<Document, DocumentUseCaseError> {
        if let Err(err) = self.repository.flush_to_gcs(doc_id).await {
            error!(
                "failed to flush websocket changes for '{}' before fetching latest: {}",
                doc_id, err
            );
        }

        match self.repository.fetch_latest(doc_id).await {
            Ok(Some(document)) => Ok(document),
            Ok(None) => Err(DocumentUseCaseError::NotFound {
                document_id: doc_id.to_string(),
            }),
            Err(err) => Err(DocumentUseCaseError::Unexpected {
                message: format!("failed to load latest document '{}'", doc_id),
                source: err,
            }),
        }
    }

    pub async fn get_history(
        &self,
        doc_id: &str,
    ) -> Result<Vec<HistoryItem>, DocumentUseCaseError> {
        self.repository.fetch_history(doc_id).await.map_err(|err| {
            DocumentUseCaseError::Unexpected {
                message: format!("failed to load history for document '{}'", doc_id),
                source: err,
            }
        })
    }

    pub async fn rollback(
        &self,
        doc_id: &str,
        version: u64,
    ) -> Result<Document, DocumentUseCaseError> {
        self.repository
            .rollback(doc_id, version)
            .await
            .map_err(|err| {
                let message = err.to_string();
                if message.contains("not found") {
                    DocumentUseCaseError::NotFound {
                        document_id: doc_id.to_string(),
                    }
                } else if message.contains("version") {
                    DocumentUseCaseError::InvalidRequest { message }
                } else {
                    DocumentUseCaseError::Unexpected {
                        message: format!(
                            "failed to rollback document '{}' to version {}",
                            doc_id, version
                        ),
                        source: err,
                    }
                }
            })
    }

    pub async fn get_history_metadata(
        &self,
        doc_id: &str,
    ) -> Result<Vec<(u32, DateTime<Utc>)>, DocumentUseCaseError> {
        self.repository
            .fetch_history_metadata(doc_id)
            .await
            .map_err(|err| DocumentUseCaseError::Unexpected {
                message: format!("failed to load history metadata for '{}'", doc_id),
                source: err,
            })
    }

    pub async fn get_history_by_version(
        &self,
        doc_id: &str,
        version: u64,
    ) -> Result<HistoryItem, DocumentUseCaseError> {
        match self.repository.fetch_history_version(doc_id, version).await {
            Ok(Some(item)) => Ok(item),
            Ok(None) => Err(DocumentUseCaseError::NotFound {
                document_id: format!("{}@{}", doc_id, version),
            }),
            Err(err) => Err(DocumentUseCaseError::Unexpected {
                message: format!(
                    "failed to load history version {} for document '{}'",
                    version, doc_id
                ),
                source: err,
            }),
        }
    }

    pub async fn flush_to_gcs(&self, doc_id: &str) -> Result<(), DocumentUseCaseError> {
        self.repository
            .flush_to_gcs(doc_id)
            .await
            .map_err(|err| DocumentUseCaseError::Unexpected {
                message: format!("failed to flush updates for document '{}'", doc_id),
                source: err,
            })
    }

    pub async fn save_snapshot(&self, doc_id: &str) -> Result<(), DocumentUseCaseError> {
        self.repository.save_snapshot(doc_id).await.map_err(|err| {
            DocumentUseCaseError::Unexpected {
                message: format!("failed to save snapshot for document '{}'", doc_id),
                source: err,
            }
        })
    }

    pub async fn copy_document(
        &self,
        doc_id: &str,
        source: &str,
    ) -> Result<(), DocumentUseCaseError> {
        self.repository
            .copy_document(doc_id, source)
            .await
            .map_err(|err| DocumentUseCaseError::Unexpected {
                message: format!("failed to copy document '{}'", doc_id),
                source: err,
            })
    }

    pub async fn import_document(
        &self,
        doc_id: &str,
        data: &[u8],
    ) -> Result<(), DocumentUseCaseError> {
        self.repository
            .import_document(doc_id, data)
            .await
            .map_err(|err| DocumentUseCaseError::Unexpected {
                message: format!("failed to import document '{}'", doc_id),
                source: err,
            })
    }
}

impl Clone for DocumentUseCase {
    fn clone(&self) -> Self {
        Self {
            repository: Arc::clone(&self.repository),
        }
    }
}

impl fmt::Debug for DocumentUseCase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DocumentUseCase").finish_non_exhaustive()
    }
}
