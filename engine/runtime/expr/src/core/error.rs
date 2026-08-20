use thiserror::Error;

use super::value::Value;

/// Sentinel: pos not yet assigned by an AST node.
pub(crate) const POS_UNSET: usize = usize::MAX;

pub fn eval_error(msg: impl Into<String>) -> Error {
    Error::Eval {
        pos: POS_UNSET,
        msg: msg.into(),
    }
}

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
