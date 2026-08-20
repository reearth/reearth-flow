#![recursion_limit = "256"]
// TODO(new-geometry): remove after migration. Gating each ported action's
// `process`/`finish`/`start` leaves geometry-only imports and helpers unused
// under the flag; silence that noise (feature-scoped, so the default build keeps
// full lint coverage).
#![cfg_attr(feature = "new-geometry", allow(unused_imports, dead_code))]

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
