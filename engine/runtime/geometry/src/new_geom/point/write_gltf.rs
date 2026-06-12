use super::Point2D;
use crate::new_geom::ops::{GltfBuffer, UnsupportedOperation, WriteGltf};

impl WriteGltf for Point2D {
    fn write_gltf(&self, out: &mut GltfBuffer) -> Result<(), UnsupportedOperation> {
        out.bytes.extend(self.x.to_le_bytes());
        out.bytes.extend(self.y.to_le_bytes());
        out.prim_count += 1;
        Ok(())
    }
}

// Point3D::write_gltf is left unsupported (see point/mod.rs `unsupported!`).
