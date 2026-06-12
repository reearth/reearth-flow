use super::Solid;
use crate::new_geom::ops::{Aabb, BoundingBox, UnsupportedOperation};

impl BoundingBox for Solid {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        let mut acc = None;
        self.exterior.extend_aabb(&mut acc);
        for m in &self.interiors {
            m.extend_aabb(&mut acc);
        }
        acc.ok_or(UnsupportedOperation {
            geometry: "Solid",
            operation: "bounding_box",
        })
    }
}
