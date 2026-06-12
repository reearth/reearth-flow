use super::Point2D;
use crate::new_geom::ops::{Reproject, UnsupportedOperation};
use crate::new_geom::Coordinate;

impl Reproject for Point2D {
    fn reproject(&mut self, target_epsg: u32) -> Result<(), UnsupportedOperation> {
        // Reads its own frame (`self.coord`), transforms, writes the new frame.
        // No `coord` parameter is needed because the leaf owns it.
        let _from = &self.coord;
        self.x += 1000.0; // placeholder transform
        self.y += 1000.0;
        self.coord = Coordinate::Crs(target_epsg);
        Ok(())
    }
}

// Point3D::reproject is left unsupported (see point/mod.rs `unsupported!`).
