#![recursion_limit = "256"]

pub(crate) mod citygml;
pub(crate) mod common;
pub mod mapping;
pub(crate) mod object_list;
pub mod plateau3;
pub mod plateau4;
pub mod plateau6;
pub mod solar;
pub(crate) mod types;

#[cfg(test)]
pub(crate) mod tests;
