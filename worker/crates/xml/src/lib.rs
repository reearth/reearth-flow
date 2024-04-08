pub mod convert;
pub mod decl;
mod display;
mod dom_impl;
pub mod error;
mod mutex;
pub mod name;
pub mod namespace;
pub mod node;
mod options;
pub mod parser;
mod syntax;
mod text;
mod trait_impl;
pub mod traits;

pub use crate::node::RefNode;

pub(crate) type Result<T, E = error::Error> = std::result::Result<T, E>;
