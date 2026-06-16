#![recursion_limit = "2048"]
extern crate alloc;

pub mod algorithm;
pub mod error;
pub mod types;
pub mod utils;
pub mod validation;

#[macro_use]
pub mod macros;

pub mod _alloc {
    pub use ::alloc::vec;
}

// TODO(new-geometry): replace this placeholder with the real geometry types.
/// The top-level geometry type (work in progress).
#[cfg(feature = "new-geometry")]
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Geometry {
    // Intentionally empty for now.
}

#[cfg(feature = "new-geometry")]
impl Geometry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        true
    }
}
