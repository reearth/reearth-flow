use crate::types::{coordinate::Coordinate2D, line::Line2D};

use super::GeoFloat;

mod difference;
mod join;

pub trait LineOps: Sized {
    type Scalar: GeoFloat;

    /// Remove the overlapping part.
    fn difference(&self, other: &Self, torelance: Self::Scalar) -> Vec<Self>;

    /// Split the line at the point. If the point is not on the line, return the original line
    fn split(&self, point: &Coordinate2D<Self::Scalar>, torelance: Self::Scalar) -> Vec<Self>;

    /// Join the lines if the end of the first line is close to the start of the second line.
    fn join_forward(&self, other: &Self, torelance: Self::Scalar) -> Option<Self>;
}

impl<T: GeoFloat> LineOps for Line2D<T> {
    type Scalar = T;

    fn difference(&self, other: &Self, torelance: T) -> Vec<Self> {
        difference::line_difference_2d(*self, *other, torelance)
    }

    fn split(&self, point: &Coordinate2D<T>, torelance: T) -> Vec<Self> {
        fn point_on_line_2d<T: GeoFloat>(
            line: Line2D<T>,
            point: Coordinate2D<T>,
            torelance: T,
        ) -> bool {
            let line_1 = Line2D::new(line.start, point);
            let line_2 = Line2D::new(point, line.end);

            (line_1.length() + line_2.length() - line.length()).abs() < torelance
        }

        if !point_on_line_2d(*self, *point, torelance) {
            return vec![*self];
        }

        let first = Line2D::new(self.start, *point);
        let second = Line2D::new(*point, self.end);

        vec![first, second]
    }

    fn join_forward(&self, other: &Self, torelance: T) -> Option<Self> {
        if Line2D::new(self.end, other.start).length() < torelance {
            Some(Line2D::new(self.start, other.end))
        } else {
            None
        }
    }
}
