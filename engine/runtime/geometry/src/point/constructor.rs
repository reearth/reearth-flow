//! Point constructors.
//!
//! A `Point` is a single position in a frame — readers (CityGML `gml:Point`, OBJ
//! vertices, 2D/3D point features) hand over exactly that, so construction just
//! pairs the position with its coordinate frame.

use crate::coordinate::Coordinate;

use super::{Point2D, Point3D};

impl Point2D {
    /// A 2D point at `position`, in `coordinate`.
    pub fn new(coordinate: Coordinate, position: [f64; 2]) -> Self {
        Self {
            coordinate,
            position,
        }
    }
}

impl Point3D {
    /// A 3D point at `position`, in `coordinate`.
    pub fn new(coordinate: Coordinate, position: [f64; 3]) -> Self {
        Self {
            coordinate,
            position,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_position_and_frame() {
        let p = Point2D::new(Coordinate::Euclidean, [1.0, 2.0]);
        assert_eq!(p.position, [1.0, 2.0]);
        assert_eq!(p.coordinate, Coordinate::Euclidean);

        let q = Point3D::new(Coordinate::Crs(4326), [1.0, 2.0, 3.0]);
        assert_eq!(q.position, [1.0, 2.0, 3.0]);
        assert_eq!(q.coordinate, Coordinate::Crs(4326));
    }
}
