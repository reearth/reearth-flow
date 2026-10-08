//! A model of XML Schema content rules, loaded at runtime.
//!
//! Knows XSD, not CityGML: what a type's children may be, in what order, how
//! often, and which elements may stand in for which.

use std::fmt;

pub(crate) mod bundle;
mod error;
mod model;
pub(crate) mod xsd;

pub use error::SchemaError;
pub use model::{ClassModel, Placement, SchemaSet, Slot, SlotAccepts};

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

/// An `xs:any` namespace constraint, with `##targetNamespace` and `##other`
/// already resolved against the target namespace of the schema that declared
/// the wildcard, so it judges a child the same wherever it is inherited.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Wildcard {
    /// `##any`: every element.
    Any,
    /// `##other`: every namespace-qualified element outside this namespace.
    Other(String),
    /// A list of namespaces; `##local` is the empty namespace.
    In(Vec<String>),
}

impl Wildcard {
    /// Read a `namespace` attribute as written in a schema whose target
    /// namespace is `target_namespace`.
    pub(crate) fn parse(constraint: &str, target_namespace: &str) -> Self {
        match constraint.trim() {
            "##any" => Self::Any,
            "##other" => Self::Other(target_namespace.to_owned()),
            list => Self::In(
                list.split_whitespace()
                    .map(|token| match token {
                        "##targetNamespace" => target_namespace.to_owned(),
                        "##local" => String::new(),
                        uri => uri.to_owned(),
                    })
                    .collect(),
            ),
        }
    }

    pub fn accepts(&self, child: &QName) -> bool {
        match self {
            Self::Any => true,
            Self::Other(namespace) => !child.namespace.is_empty() && &child.namespace != namespace,
            Self::In(namespaces) => namespaces.iter().any(|ns| ns == &child.namespace),
        }
    }
}
