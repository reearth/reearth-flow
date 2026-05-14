use thiserror::Error;

/// Internal error produced by eval helpers; converted to `Error::Eval` with pos by `eval_inner`.
#[derive(Debug)]
pub struct EvalHelperError {
    pub msg: String,
}

impl EvalHelperError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { msg: msg.into() }
    }
}

pub type HResult<T> = std::result::Result<T, EvalHelperError>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("lex error at position {pos}: {msg}")]
    Lex { pos: usize, msg: String },

    #[error("parse error at position {pos}: {msg}")]
    Parse { pos: usize, msg: String },

    #[error("eval error at position {pos}: {msg}")]
    Eval { pos: usize, msg: String },

    #[error("eval_string error: {msg}")]
    EvalString { msg: String },
}

pub type Result<T> = std::result::Result<T, Error>;
