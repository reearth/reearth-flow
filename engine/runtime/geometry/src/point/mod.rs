//! Point leaves.

use serde::{Deserialize, Serialize};

use super::coordinate::CoordinateFrame;

mod constructor;
mod ops;
#[cfg(feature = "new-geometry")]
mod validation;

/// A single position in 2D space.
/// Used for CityGML `gml:Point` and 2D point features.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Point2D {
    /// Coordinate frame this position is expressed in.
    frame: CoordinateFrame,
    position: [f64; 2],
}

/// A single position in 3D space.
/// Used for CityGML `gml:Point`, OBJ vertices, and 3D point features.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Point3D {
    /// Coordinate frame this position is expressed in.
    frame: CoordinateFrame,
    position: [f64; 3],
}

impl Point2D {
    /// The coordinate frame this position is expressed in.
    #[inline]
    pub fn frame(&self) -> &CoordinateFrame {
        &self.frame
    }

    /// The `[x, y]` position.
    #[inline]
    pub fn position(&self) -> [f64; 2] {
        self.position
    }
}

impl Point3D {
    /// The coordinate frame this position is expressed in.
    #[inline]
    pub fn frame(&self) -> &CoordinateFrame {
        &self.frame
    }

    /// The `[x, y, z]` position.
    #[inline]
    pub fn position(&self) -> [f64; 3] {
        self.position
    }
}

crate::unsupported!(Point2D: Triangulate);
crate::unsupported!(Point3D: Triangulate);
