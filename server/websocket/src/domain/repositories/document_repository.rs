use crate::domain::models::{Document, DocumentId};
use anyhow::Result;
use async_trait::async_trait;

/// 文档仓储接口
#[async_trait]
pub trait DocumentRepository: Send + Sync {
    /// 获取文档
    async fn get(&self, id: &DocumentId) -> Result<Option<Document>>;

    /// 保存文档
    async fn save(&self, document: &Document) -> Result<()>;

    /// 删除文档
    async fn delete(&self, id: &DocumentId) -> Result<()>;

    /// 检查文档是否存在
    async fn exists(&self, id: &DocumentId) -> Result<bool>;
}
