use super::GeometryCollection;
use crate::new_geom::ops::{Aabb, BoundingBox, UnsupportedOperation};

impl BoundingBox for GeometryCollection {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        let mut acc: Option<Aabb> = None;
        for g in &self.0 {
            if let Ok(b) = g.bounding_box() {
                Aabb::extend(&mut acc, b.min);
                Aabb::extend(&mut acc, b.max);
            }
        }
        acc.ok_or(UnsupportedOperation {
            geometry: "GeometryCollection",
            operation: "bounding_box",
        })
    }
}
