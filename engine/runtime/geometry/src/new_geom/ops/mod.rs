//! Operation traits. Each operation is a separate trait with a default method
//! that returns [`UnsupportedOperation`], so a leaf opts in by overriding and
//! opts out with an empty impl block (collapsed via the `unsupported!` macro in
//! the parent module).

pub mod bounding_box;
pub mod reproject;
pub mod write_gltf;

pub use bounding_box::{Aabb, BoundingBox};
pub use reproject::Reproject;
pub use write_gltf::{GltfBuffer, WriteGltf};

/// Returned by an operation a given geometry type does not support. Carries the
/// concrete type name (via `type_name`) and the operation name for diagnostics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedOperation {
    pub geometry: &'static str,
    pub operation: &'static str,
}

impl core::fmt::Display for UnsupportedOperation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "`{}` is not supported by `{}`",
            self.operation, self.geometry
        )
    }
}

impl std::error::Error for UnsupportedOperation {}
