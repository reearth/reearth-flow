use geo_types::LineString;

use crate::types::line::Line2D;

use super::GeoNum;

mod basic;
mod difference;

pub trait LineOps: Sized {
    type Scalar: GeoNum;

    fn length(&self) -> Self::Scalar;

    //fn intersection(&self, other: &Self) -> Vec<Self>;

    //fn difference(&self, other: &Self) -> Vec<Self>;

    //fn projection(&self, point: &Coordinate2D<Self::Scalar>) -> Coordinate2D<Self::Scalar>;

    //fn split(&self, point: &Coordinate2D<Self::Scalar>, torelance: Self::Scalar) -> Vec<Self>;
}

impl<T: GeoNum> LineOps for Line2D<T> {
    type Scalar = T;

    fn length(&self) -> T {
        basic::line_length_2d(*self)
    }
}

impl<T: GeoNum> LineOps for LineString<T> {
    type Scalar = T;

    fn length(&self) -> T {
        let mut length = T::zero();
        for i in 0..self.0.len() - 1 {
            length = length + Line2D::new_(self.0[i], self.0[i + 1]).length();
        }
        length
    }
}
