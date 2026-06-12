use super::LineString2D;
use crate::new_geom::ops::{Aabb, BoundingBox, UnsupportedOperation};

impl BoundingBox for LineString2D {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        let mut acc = None;
        for p in &self.pts {
            Aabb::extend(&mut acc, [p[0], p[1], 0.0]);
        }
        acc.ok_or(UnsupportedOperation {
            geometry: "LineString2D",
            operation: "bounding_box",
        })
    }
}
