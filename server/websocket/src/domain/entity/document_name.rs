use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DocumentName(String);

impl DocumentName {
    pub fn new(name: String) -> Result<Self, &'static str> {
        if name.is_empty() {
            return Err("Document name cannot be empty");
        }
        if name.len() > 255 {
            return Err("Document name cannot exceed 255 characters");
        }
        if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.') {
            return Err("Document name can only contain alphanumeric characters, hyphens, underscores, and dots");
        }
        Ok(Self(name))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for DocumentName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for DocumentName {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for DocumentName {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_string())
    }
}
