use crate::shared::result::AppResult;
use crate::shared::utils::ensure_not_empty;

#[derive(Debug, Clone)]
pub struct RollbackDocumentCommand {
    pub doc_id: String,
    pub version: u64,
}

impl RollbackDocumentCommand {
    pub fn new(doc_id: impl Into<String>, version: u64) -> Self {
        Self {
            doc_id: doc_id.into(),
            version,
        }
    }

    pub fn validate(&self) -> AppResult<()> {
        ensure_not_empty(&self.doc_id, "doc_id")?;
        Ok(())
    }
}
