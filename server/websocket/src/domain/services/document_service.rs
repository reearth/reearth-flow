use crate::domain::models::{Document, DocumentId};
use crate::domain::repositories::DocumentRepository;
use anyhow::Result;
use std::sync::Arc;

pub struct DocumentService {
    repository: Arc<dyn DocumentRepository>,
}

impl DocumentService {
    pub fn new(repository: Arc<dyn DocumentRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_document(&self, id: DocumentId) -> Result<Document> {
        if self.repository.exists(&id).await? {
            anyhow::bail!("Document already exists: {:?}", id);
        }

        let document = Document::new(id);
        self.repository.save(&document).await?;
        Ok(document)
    }

    pub async fn get_or_create(&self, id: DocumentId) -> Result<Document> {
        if let Some(doc) = self.repository.get(&id).await? {
            Ok(doc)
        } else {
            self.create_document(id).await
        }
    }
}
