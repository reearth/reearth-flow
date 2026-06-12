use super::GeometryCollection;
use crate::new_geom::ops::{Reproject, UnsupportedOperation};

impl Reproject for GeometryCollection {
    fn reproject(&mut self, target_epsg: u32) -> Result<(), UnsupportedOperation> {
        // Each child reprojects using ITS OWN frame, read from its own leaf.
        // Nothing here passes a parent coord down, so mixed-frame collections
        // are handled correctly.
        self.0.iter_mut().try_for_each(|g| g.reproject(target_epsg))
    }
}
