//! `WriteGltf`: "computation into the same target".
//!
//! The canonical enum-dispatch shape: every leaf folds itself into one shared
//! output sink. The return type is uniform across variants (just `Result`), so
//! it composes cleanly through the nested dispatch.

use super::UnsupportedOperation;

/// Stand-in for a glTF binary buffer assembled across many geometries.
#[derive(Default, Debug)]
pub struct GltfBuffer {
    pub bytes: Vec<u8>,
    pub prim_count: usize,
}

#[enum_dispatch::enum_dispatch]
pub trait WriteGltf {
    fn write_gltf(&self, _out: &mut GltfBuffer) -> Result<(), UnsupportedOperation> {
        Err(UnsupportedOperation {
            geometry: core::any::type_name::<Self>(),
            operation: "write_gltf",
        })
    }
}
