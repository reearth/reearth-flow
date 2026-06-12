use super::GeometryCollection;
use crate::new_geom::ops::{GltfBuffer, UnsupportedOperation, WriteGltf};

impl WriteGltf for GeometryCollection {
    fn write_gltf(&self, out: &mut GltfBuffer) -> Result<(), UnsupportedOperation> {
        // Children that cannot emit (e.g. CSG) are skipped, not fatal.
        for g in &self.0 {
            let _ = g.write_gltf(out);
        }
        Ok(())
    }
}
