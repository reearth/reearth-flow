//! Point leaves.

use serde::{Deserialize, Serialize};

use super::coordinate::Coordinate;

mod constructor;

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
