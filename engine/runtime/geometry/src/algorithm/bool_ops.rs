mod op;
use op::*;
mod assembly;
use assembly::*;
mod spec;
use spec::*;

use crate::types::{
    multi_line_string::MultiLineString2D, multi_polygon::MultiPolygon2D, polygon::Polygon2D,
};

use super::{coords_iter::CoordsIter, GeoFloat, GeoNum};

pub trait BooleanOps: Sized {
    type Scalar: GeoNum;

    fn boolean_op(&self, other: &Self, op: OpType) -> MultiPolygon2D<Self::Scalar>;
    fn intersection(&self, other: &Self) -> MultiPolygon2D<Self::Scalar> {
        self.boolean_op(other, OpType::Intersection)
    }
    fn union(&self, other: &Self) -> MultiPolygon2D<Self::Scalar> {
        self.boolean_op(other, OpType::Union)
    }
    fn xor(&self, other: &Self) -> MultiPolygon2D<Self::Scalar> {
        self.boolean_op(other, OpType::Xor)
    }
    fn difference(&self, other: &Self) -> MultiPolygon2D<Self::Scalar> {
        self.boolean_op(other, OpType::Difference)
    }

    /// Clip a 1-D geometry with self.
    ///
    /// Returns the portion of `ls` that lies within `self` (known as the set-theoeretic
    /// intersection) if `invert` is false, and the difference (`ls - self`) otherwise.
    fn clip(
        &self,
        ls: &MultiLineString2D<Self::Scalar>,
        invert: bool,
    ) -> MultiLineString2D<Self::Scalar>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpType {
    Intersection,
    Union,
    Difference,
    Xor,
}

impl<T: GeoFloat> BooleanOps for Polygon2D<T> {
    type Scalar = T;

    fn boolean_op(&self, other: &Self, op: OpType) -> MultiPolygon2D<Self::Scalar> {
        let spec = BoolOp::from(op);
        let mut bop = Proc::new(spec, self.coords_count() + other.coords_count());
        bop.add_polygon(self, 0);
        bop.add_polygon(other, 1);
        bop.sweep()
    }

    fn clip(
        &self,
        ls: &MultiLineString2D<Self::Scalar>,
        invert: bool,
    ) -> MultiLineString2D<Self::Scalar> {
        let spec = ClipOp::new(invert);
        let mut bop = Proc::new(spec, self.coords_count() + ls.coords_count());
        bop.add_polygon(self, 0);
        ls.0.iter().enumerate().for_each(|(idx, l)| {
            bop.add_line_string(l, idx + 1);
        });
        bop.sweep()
    }
}
impl<T: GeoFloat> BooleanOps for MultiPolygon2D<T> {
    type Scalar = T;

    fn boolean_op(&self, other: &Self, op: OpType) -> MultiPolygon2D<Self::Scalar> {
        let spec = BoolOp::from(op);
        let mut bop = Proc::new(spec, self.coords_count() + other.coords_count());
        bop.add_multi_polygon(self, 0);
        bop.add_multi_polygon(other, 1);
        bop.sweep()
    }

    fn clip(
        &self,
        ls: &MultiLineString2D<Self::Scalar>,
        invert: bool,
    ) -> MultiLineString2D<Self::Scalar> {
        let spec = ClipOp::new(invert);
        let mut bop = Proc::new(spec, self.coords_count() + ls.coords_count());
        bop.add_multi_polygon(self, 0);
        ls.0.iter().enumerate().for_each(|(idx, l)| {
            bop.add_line_string(l, idx + 1);
        });
        bop.sweep()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{coordinate::Coordinate2D, line_string::LineString2D};
    #[test]
    fn test_polygon_boolean_ops_two_squares() {
        let poly1 = MultiPolygon2D::new(vec![Polygon2D::new(LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(2.0, 0.0),
            Coordinate2D::new_(2.0, 2.0),
            Coordinate2D::new_(0.0, 2.0),
            Coordinate2D::new_(0.0, 0.0),
        ]), vec![])]);
        let poly2 = MultiPolygon2D::new(vec![Polygon2D::new(LineString2D::new(vec![
            Coordinate2D::new_(1.0, 1.0),
            Coordinate2D::new_(3.0, 1.0),
            Coordinate2D::new_(3.0, 3.0),
            Coordinate2D::new_(1.0, 3.0),
            Coordinate2D::new_(1.0, 1.0),
        ]), vec![])]);
        let intersection = poly1.intersection(&poly2);
        assert_eq!(intersection.0.len(), 1);
        assert_eq!(intersection.0[0].exterior().0.len(), 5);

        let diff = poly1.difference(&poly2);
        assert_eq!(diff.0.len(), 1);
        assert_eq!(diff.0[0].exterior().0.len(), 7);
    }

    #[test]
    fn test_polygon_boolean_ops_two_triangles() {
        let poly1 = MultiPolygon2D::new(vec![Polygon2D::new(LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(2.0, 0.0),
            Coordinate2D::new_(1.0, 2.0),
            Coordinate2D::new_(0.0, 0.0),
        ]), vec![])]);
        let poly2 = MultiPolygon2D::new(vec![Polygon2D::new(LineString2D::new(vec![
            Coordinate2D::new_(0.0, 0.0),
            Coordinate2D::new_(2.0, 0.0),
            Coordinate2D::new_(1.0, 1.0),
            Coordinate2D::new_(0.0, 0.0),
        ]), vec![])]);
        let intersection = poly1.intersection(&poly2);
        assert_eq!(intersection.0.len(), 1);
        assert_eq!(intersection.0[0].exterior().0.len(), 4);

        let diff = poly1.difference(&poly2);
        assert_eq!(diff.0.len(), 1);
        assert_eq!(diff.0[0].exterior().0.len(), 5);
    }
}
