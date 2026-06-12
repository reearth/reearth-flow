use super::Csg;
use crate::new_geom::ops::{Aabb, BoundingBox, UnsupportedOperation};

impl BoundingBox for Csg {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        let mut acc = None;
        self.root.extend_aabb(&mut acc);
        acc.ok_or(UnsupportedOperation {
            geometry: "Csg",
            operation: "bounding_box",
        })
    }
}
