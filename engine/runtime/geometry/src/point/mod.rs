//! Point leaves.

use nusamai_projection::crs::EpsgCode;
use serde::{Deserialize, Serialize};

use super::coordinate::Coordinate;
use crate::error::Result;
use crate::ops::reproject::Transformer;

mod constructor;
mod ops;

/// A single position in 2D space.
/// Used for CityGML `gml:Point` and 2D point features.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Point2D {
    /// Coordinate frame this position is expressed in.
    coordinate: Coordinate,
    position: [f64; 2],
}

/// A single position in 3D space.
/// Used for CityGML `gml:Point`, OBJ vertices, and 3D point features.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Point3D {
    /// Coordinate frame this position is expressed in.
    coordinate: Coordinate,
    position: [f64; 3],
}

crate::unsupported!(Point2D: Triangulate);
crate::unsupported!(Point3D: Triangulate);
impl Point2D {
    pub(crate) fn reproject(&mut self, target: EpsgCode, cache: &mut Transformer) -> Result<()> {
        let from = self.coordinate.require_crs()?;
        if from != target {
            let [x, y] = self.position;
            let [nx, ny, _] = cache.transform(from, target, [x, y, 0.0])?;
            self.position = [nx, ny];
            self.coordinate = Coordinate::Crs(target);
        }
        Ok(())
    }
}

impl Point3D {
    pub(crate) fn reproject(&mut self, target: EpsgCode, cache: &mut Transformer) -> Result<()> {
        let from = self.coordinate.require_crs()?;
        if from != target {
            self.position = cache.transform(from, target, self.position)?;
            self.coordinate = Coordinate::Crs(target);
        }
        Ok(())
    }
}
