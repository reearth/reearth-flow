use crate::shared::result::AppResult;
use crate::shared::utils::ensure_not_empty;

#[derive(Debug, Clone)]
pub struct FlushDocumentCommand {
    pub doc_id: String,
}

impl FlushDocumentCommand {
    pub fn new(doc_id: impl Into<String>) -> Self {
        Self {
            doc_id: doc_id.into(),
        }
    }

    pub fn validate(&self) -> AppResult<()> {
        ensure_not_empty(&self.doc_id, "doc_id")
    }
}
