//! Geometric predicates over point leaves.

use super::{Point2D, Point3D};
use crate::predicates::{self, Equal};

impl Equal for Point2D {
    fn equal(&self, rhs: &Self, tolerance: f64) -> predicates::Result<bool> {
        predicates::require_tolerance(tolerance)?;
        predicates::require_same_frame(self.frame(), rhs.frame())?;
        let (a, b) = (self.position(), rhs.position());
        let d = [b[0] - a[0], b[1] - a[1]];
        Ok((d[0] * d[0] + d[1] * d[1]).sqrt() <= tolerance)
    }
}

impl Equal for Point3D {
    fn equal(&self, rhs: &Self, tolerance: f64) -> predicates::Result<bool> {
        predicates::require_tolerance(tolerance)?;
        predicates::require_same_frame(self.frame(), rhs.frame())?;
        let (a, b) = (self.position(), rhs.position());
        let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        Ok((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() <= tolerance)
    }
}
