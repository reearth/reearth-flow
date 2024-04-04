mod convert;
mod decl;
mod display;
mod dom_impl;
pub mod error;
mod mutex;
mod name;
mod namespace;
mod node;
mod options;
pub mod parser;
mod syntax;
mod text;
mod trait_impl;
mod traits;

pub use crate::node::RefNode;

pub(crate) type Result<T, E = error::Error> = std::result::Result<T, E>;
