pub mod operator;
pub mod resolve;
pub mod storage;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("ResolveError: {0}")]
    Resolve(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
