use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentName(String);

impl DocumentName {
    pub fn new(name: String) -> Result<Self, DocumentNameError> {
        if name.is_empty() {
            return Err(DocumentNameError::Empty);
        }
        if name.len() > 255 {
            return Err(DocumentNameError::TooLong);
        }
        if !name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err(DocumentNameError::InvalidCharacters);
        }
        Ok(Self(name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0.into_bytes()
    }
}

impl fmt::Display for DocumentName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for DocumentName {
    type Error = DocumentNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for DocumentName {
    type Error = DocumentNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_string())
    }
}

#[derive(Debug, Error)]
pub enum DocumentNameError {
    #[error("Document name cannot be empty")]
    Empty,
    #[error("Document name cannot exceed 255 characters")]
    TooLong,
    #[error(
        "Document name can only contain alphanumeric characters, hyphens, underscores, and dots"
    )]
    InvalidCharacters,
}
