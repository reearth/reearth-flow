use super::{Point2D, Point3D};
use crate::new_geom::ops::{Aabb, BoundingBox, UnsupportedOperation};

impl BoundingBox for Point2D {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        Ok(Aabb::point(self.x, self.y, self.z.unwrap_or(0.0)))
    }
}

impl BoundingBox for Point3D {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        Ok(Aabb::point(self.x, self.y, self.z))
    }
}
