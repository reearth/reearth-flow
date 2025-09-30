use crate::shared::errors::AppError;

pub type AppResult<T> = Result<T, AppError>;
