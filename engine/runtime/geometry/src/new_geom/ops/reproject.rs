//! `Reproject`: a coord-dependent, in-place (`&mut self`) edit.
//!
//! This is the operation that motivates storing `Coordinate` inside each leaf:
//! the method signature carries no `coord` argument, yet the impl needs the
//! source frame. Because every leaf owns its `coord`, the impl just reads
//! `self.coord`, transforms its own coordinates, and writes the new frame back.

use super::UnsupportedOperation;

/// Coordinate-dependent in-place operation. `enum_dispatch` forwards `&mut self`
/// down to the concrete leaf unchanged.
#[enum_dispatch::enum_dispatch]
pub trait Reproject {
    fn reproject(&mut self, _target_epsg: u32) -> Result<(), UnsupportedOperation> {
        Err(UnsupportedOperation {
            geometry: core::any::type_name::<Self>(),
            operation: "reproject",
        })
    }
}
