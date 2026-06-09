use thiserror::Error;

use super::value::Value;

/// Internal error produced by eval helpers; converted to `Error::Eval` with pos by `eval_inner`.
#[derive(Debug)]
pub struct InnerError {
    pub msg: String,
}

impl InnerError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { msg: msg.into() }
    }
}

pub type InnerResult<T> = std::result::Result<T, InnerError>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("lex error at position {pos}: {msg}")]
    Lex { pos: usize, msg: String },

    #[error("parse error at position {pos}: {msg}")]
    Parse { pos: usize, msg: String },

    #[error("eval error at position {pos}: {msg}")]
    Eval { pos: usize, msg: String },

    /// Control flow: `return <value>` — not a real error.
    #[error("return")]
    Return(Value),
}

pub type Result<T> = std::result::Result<T, Error>;
