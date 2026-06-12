use super::Solid;
use crate::new_geom::ops::{Reproject, UnsupportedOperation};
use crate::new_geom::Coordinate;

impl Reproject for Solid {
    fn reproject(&mut self, target_epsg: u32) -> Result<(), UnsupportedOperation> {
        // Transform every coordless shell, then update the single frame.
        self.exterior.translate([1000.0, 0.0, 0.0]);
        for m in &mut self.interiors {
            m.translate([1000.0, 0.0, 0.0]);
        }
        self.coord = Coordinate::Crs(target_epsg);
        Ok(())
    }
}
