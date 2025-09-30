use crate::shared::errors::AppError;
use crate::shared::result::AppResult;

pub fn normalize_id(value: &str) -> String {
    value
        .strip_suffix(":main")
        .unwrap_or(value)
        .trim()
        .to_string()
}

pub fn ensure_not_empty(value: &str, field: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        Err(AppError::invalid_input(format!("{field} cannot be empty")))
    } else {
        Ok(())
    }
}
