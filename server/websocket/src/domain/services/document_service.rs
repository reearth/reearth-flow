use crate::domain::models::{Document, DocumentId};
use crate::domain::repositories::DocumentRepository;
use anyhow::Result;
use std::sync::Arc;

/// 文档领域服务
pub struct DocumentService {
    repository: Arc<dyn DocumentRepository>,
}

impl DocumentService {
    pub fn new(repository: Arc<dyn DocumentRepository>) -> Self {
        Self { repository }
    }

    /// 创建新文档
    pub async fn create_document(&self, id: DocumentId) -> Result<Document> {
        // 检查文档是否已存在
        if self.repository.exists(&id).await? {
            anyhow::bail!("Document already exists: {:?}", id);
        }

        let document = Document::new(id);
        self.repository.save(&document).await?;
        Ok(document)
    }

    /// 获取或创建文档
    pub async fn get_or_create(&self, id: DocumentId) -> Result<Document> {
        if let Some(doc) = self.repository.get(&id).await? {
            Ok(doc)
        } else {
            self.create_document(id).await
        }
    }

    /// 更新文档时钟
    pub async fn update_clock(&self, id: &DocumentId, clock: u32) -> Result<()> {
        if let Some(mut doc) = self.repository.get(id).await? {
            doc.update_clock(clock).await?;
            self.repository.save(&doc).await?;
            Ok(())
        } else {
            anyhow::bail!("Document not found: {:?}", id);
        }
    }
}
