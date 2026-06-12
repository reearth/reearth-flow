#![recursion_limit = "2048"]
extern crate alloc;

pub mod algorithm;
pub mod error;
/// Reference prototype for the new geometry type (design doc §3.3, §4).
/// Standalone; not wired into the production types.
pub mod new_geom;
pub mod types;
pub mod utils;
pub mod validation;

#[macro_use]
pub mod macros;

pub mod _alloc {
    pub use ::alloc::vec;
}
