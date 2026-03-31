pub mod ast;
pub mod error;
pub mod eval;
pub mod lexer;
pub mod parser;

pub use error::{Error, Result};
pub use eval::{eval, Context};
pub use parser::parse;
