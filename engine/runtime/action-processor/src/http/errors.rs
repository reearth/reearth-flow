use thiserror::Error;

pub(crate) type Result<T, E = HttpProcessorError> = std::result::Result<T, E>;

#[derive(Error, Debug, Clone)]
pub(crate) enum HttpProcessorError {
    #[error("HttpCaller factory error: {0}")]
    CallerFactory(String),

    #[error("HttpCaller error: {0}")]
    #[allow(dead_code)]
    Caller(String),

    #[error("HttpCaller request error: {0}")]
    Request(String),

    #[error("HttpCaller response error: {0}")]
    Response(String),
}
