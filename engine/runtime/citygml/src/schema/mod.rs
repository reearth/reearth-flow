//! A model of XML Schema content rules, loaded at runtime.
//!
//! Knows XSD, not CityGML: what a type's children may be, in what order, how
//! often, and which elements may stand in for which.

use std::fmt;

// TODO: remove once the loader uses the bundle (a later task in this series).
#[allow(dead_code)]
pub(crate) mod bundle;
mod error;
// The Task 3 model builds on the parser; until then only tests use it.
#[allow(dead_code)]
pub(crate) mod xsd;

pub use error::SchemaError;

/// A namespace-qualified name.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct QName {
    pub namespace: String,
    pub local: String,
}

impl QName {
    pub fn new(namespace: impl Into<String>, local: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            local: local.into(),
        }
    }
}

impl fmt::Display for QName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{{}}}{}", self.namespace, self.local)
    }
}

/// An upper occurrence bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaxOccurs {
    Bounded(u32),
    Unbounded,
}
