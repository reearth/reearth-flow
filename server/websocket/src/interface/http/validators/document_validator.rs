use crate::interface::http::dto::{CreateSnapshotRequest, RollbackRequest};

/// Document Validator - Interface Layer
/// Validates incoming HTTP requests before they reach the controllers
pub struct DocumentValidator;

impl DocumentValidator {
    /// Validate create snapshot request
    pub fn validate_create_snapshot(request: &CreateSnapshotRequest) -> Result<(), String> {
        if request.doc_id.is_empty() {
            return Err("Document ID cannot be empty".to_string());
        }

        if request.doc_id.len() > 255 {
            return Err("Document ID too long".to_string());
        }

        Ok(())
    }

    /// Validate rollback request
    pub fn validate_rollback(request: &RollbackRequest) -> Result<(), String> {
        if request.version == 0 {
            return Err("Version must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Validate document ID parameter
    pub fn validate_doc_id(doc_id: &str) -> Result<(), String> {
        if doc_id.is_empty() {
            return Err("Document ID cannot be empty".to_string());
        }

        if doc_id.len() > 255 {
            return Err("Document ID too long".to_string());
        }

        // Check for valid characters (alphanumeric, hyphens, underscores)
        if !doc_id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
            return Err("Document ID contains invalid characters".to_string());
        }

        Ok(())
    }
}
