use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub(crate) enum HttpProcessorError {
    #[error("HTTPCaller Factory error: {0}")]
    HttpCallerFactory(String),
    #[error("HTTPCaller error: {0}")]
    HttpCaller(String),
    #[error("HTTP request error: {0}")]
    Request(String),
    #[error("HTTP response error: {0}")]
    Response(String),
    #[error("Expression evaluation error: {0}")]
    Expression(String),
}
