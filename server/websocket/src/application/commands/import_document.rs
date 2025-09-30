use crate::shared::errors::AppError;
use crate::shared::result::AppResult;
use crate::shared::utils::ensure_not_empty;

#[derive(Debug, Clone)]
pub struct ImportDocumentCommand {
    pub doc_id: String,
    pub payload: Vec<u8>,
}

impl ImportDocumentCommand {
    pub fn new(doc_id: impl Into<String>, payload: Vec<u8>) -> Self {
        Self {
            doc_id: doc_id.into(),
            payload,
        }
    }

    pub fn validate(&self) -> AppResult<()> {
        ensure_not_empty(&self.doc_id, "doc_id")?;
        if self.payload.is_empty() {
            return Err(AppError::invalid_input("payload cannot be empty"));
        }
        Ok(())
    }
}
