//! A model of XML Schema content rules, loaded at runtime.
//!
//! Knows XSD, not CityGML: what a type's children may be, in what order, how
//! often, and which elements may stand in for which.

// TODO: remove once the loader uses the bundle (a later task in this series).
#[allow(dead_code)]
pub(crate) mod bundle;
