//! Point constructors.
//!
//! A `Point` is a single position in a frame — readers (CityGML `gml:Point`, OBJ
//! vertices, 2D/3D point features) hand over exactly that, so construction just
//! pairs the position with its coordinate frame.

use crate::coordinate::CoordinateFrame;

use super::{Point2D, Point3D};

impl Point2D {
    /// A 2D point at `position`, in `frame`.
    pub fn new(frame: CoordinateFrame, position: [f64; 2]) -> Self {
        Self { frame, position }
    }
}

impl Point3D {
    /// A 3D point at `position`, in `frame`.
    pub fn new(frame: CoordinateFrame, position: [f64; 3]) -> Self {
        Self { frame, position }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::EpsgCode;

    #[test]
    fn new_stores_position_and_frame() {
        let p = Point2D::new(CoordinateFrame::Euclidean, [1.0, 2.0]);
        assert_eq!(p.position, [1.0, 2.0]);
        assert_eq!(p.frame, CoordinateFrame::Euclidean);

        let q = Point3D::new(CoordinateFrame::Crs(EpsgCode::new(4326)), [1.0, 2.0, 3.0]);
        assert_eq!(q.position, [1.0, 2.0, 3.0]);
        assert_eq!(q.frame, CoordinateFrame::Crs(EpsgCode::new(4326)));
    }
}
