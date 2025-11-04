use crate::shared::result::AppResult;
use crate::shared::utils::ensure_not_empty;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocId(String);

impl DocId {
    pub fn new(value: impl Into<String>) -> AppResult<Self> {
        let value = value.into();
        ensure_not_empty(&value, "doc_id")?;
        Ok(Self(value))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for DocId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
