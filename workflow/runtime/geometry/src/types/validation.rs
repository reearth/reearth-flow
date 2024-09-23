#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Validation {
    pub is_valid: bool,
    pub errors: Vec<String>,
}
