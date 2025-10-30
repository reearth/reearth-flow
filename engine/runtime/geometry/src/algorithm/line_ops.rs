use crate::types::{coordinate::Coordinate2D, coordnum::CoordNum, line::Line2D, no_value::NoValue};

use super::GeoFloat;

mod difference;

pub trait LineOps: Sized {
    type Scalar: GeoFloat;

    /// Remove the overlapping part.
    fn difference(&self, other: &Self, tolerance: Self::Scalar) -> Vec<Self>;

    /// Split the line at the point. If the point is not on the line, return the original line
    fn split(
        &self,
        point: &Coordinate2D<Self::Scalar>,
        tolerance: Self::Scalar,
    ) -> SplitResult<Self::Scalar>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum SplitResult<T: CoordNum> {
    Success([Line2D<T>; 2]),
    FailureNotOnLine(Line2D<T>),
}

impl<T: GeoFloat + From<NoValue>> LineOps for Line2D<T> {
    type Scalar = T;

    fn difference(&self, other: &Self, tolerance: T) -> Vec<Self> {
        difference::line_difference_2d(*self, *other, tolerance)
    }

    fn split(&self, point: &Coordinate2D<T>, tolerance: T) -> SplitResult<T> {
        fn point_on_line_2d<T: GeoFloat>(
            line: Line2D<T>,
            point: Coordinate2D<T>,
            tolerance: T,
        ) -> bool {
            let line_1 = Line2D::new(line.start, point);
            let line_2 = Line2D::new(point, line.end);

            (line_1.length() + line_2.length() - line.length()).abs() < tolerance
        }

        if !point_on_line_2d(*self, *point, tolerance) {
            return SplitResult::FailureNotOnLine(*self);
        }
        // We also exclude the case where the point is at the start or end of the line.
        if (*point - self.start).norm() < tolerance || (*point - self.end).norm() < tolerance {
            return SplitResult::FailureNotOnLine(*self);
        }

        let first = Line2D::new(self.start, *point);
        let second = Line2D::new(*point, self.end);

        SplitResult::Success([first, second])
    }
}
