//! A model of XML Schema content rules, loaded at runtime.
//!
//! Knows XSD, not CityGML: what a type's children may be, in what order, how
//! often, and which elements may stand in for which.

use std::fmt;

use fastxml::schema::types::{WildcardConstraint, WildcardNamespace};

pub(crate) mod bundle;
mod error;
mod model;
mod slots;

pub use error::SchemaError;
pub use model::{AdeSource, ClassModel, Placement, SchemaSet, Slot, SlotAccepts};

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
    /// fastxml has already resolved `##targetNamespace` and `##local` in a
    /// list, and records the declaring schema's namespace for `##other`.
    pub(crate) fn from_fastxml(constraint: &WildcardConstraint) -> Self {
        match &constraint.namespace {
            WildcardNamespace::Any => Self::Any,
            WildcardNamespace::Other => {
                Self::Other(constraint.target_namespace.clone().unwrap_or_default())
            }
            WildcardNamespace::List(namespaces) => Self::In(namespaces.clone()),
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
