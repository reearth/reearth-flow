use crate::shared::result::AppResult;
use crate::shared::utils::ensure_not_empty;

#[derive(Debug, Clone)]
pub struct CopyDocumentCommand {
    pub doc_id: String,
    pub source_id: String,
}

impl CopyDocumentCommand {
    pub fn new(doc_id: impl Into<String>, source_id: impl Into<String>) -> Self {
        Self {
            doc_id: doc_id.into(),
            source_id: source_id.into(),
        }
    }

    pub fn validate(&self) -> AppResult<()> {
        ensure_not_empty(&self.doc_id, "doc_id")?;
        ensure_not_empty(&self.source_id, "source_id")
    }
}
