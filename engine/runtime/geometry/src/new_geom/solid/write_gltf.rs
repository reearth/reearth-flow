use super::Solid;
use crate::new_geom::ops::{GltfBuffer, UnsupportedOperation, WriteGltf};

impl WriteGltf for Solid {
    fn write_gltf(&self, out: &mut GltfBuffer) -> Result<(), UnsupportedOperation> {
        self.exterior.emit_gltf(out);
        for m in &self.interiors {
            m.emit_gltf(out);
        }
        Ok(())
    }
}
