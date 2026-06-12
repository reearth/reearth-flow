use super::LineString2D;
use crate::new_geom::ops::{Reproject, UnsupportedOperation};
use crate::new_geom::Coordinate;

impl Reproject for LineString2D {
    fn reproject(&mut self, target_epsg: u32) -> Result<(), UnsupportedOperation> {
        for p in &mut self.pts {
            p[0] += 1000.0;
            p[1] += 1000.0;
        }
        self.coord = Coordinate::Crs(target_epsg);
        Ok(())
    }
}
