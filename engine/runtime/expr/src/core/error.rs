use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("lex error at position {pos}: {msg}")]
    Lex { pos: usize, msg: String },

    #[error("parse error at position {pos}: {msg}")]
    Parse { pos: usize, msg: String },

    #[error("eval error: {msg}")]
    Eval { msg: String },
}

pub type Result<T> = std::result::Result<T, Error>;
