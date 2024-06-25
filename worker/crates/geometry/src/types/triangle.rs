use approx::{AbsDiffEq, RelativeEq};
use serde::{Deserialize, Serialize};

use crate::polygon;

use super::coordinate::Coordinate;
use super::coordnum::CoordNum;
use super::line::Line;
use super::no_value::NoValue;
use super::polygon::Polygon;
use super::traits::Surface;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Triangle<T: CoordNum = f64, Z: CoordNum = f64>(
    pub Coordinate<T, Z>,
    pub Coordinate<T, Z>,
    pub Coordinate<T, Z>,
);

pub type Triangle2D<T> = Triangle<T, NoValue>;
pub type Triangle3D<T> = Triangle<T, T>;

impl<T: CoordNum, Z: CoordNum> Triangle<T, Z> {
    /// Instantiate Self from the raw content value
    pub fn new(v1: Coordinate<T, Z>, v2: Coordinate<T, Z>, v3: Coordinate<T, Z>) -> Self {
        Self(v1, v2, v3)
    }

    pub fn to_array(&self) -> [Coordinate<T, Z>; 3] {
        [self.0, self.1, self.2]
    }

    pub fn to_polygon(self) -> Polygon<T, Z> {
        polygon![self.0, self.1, self.2, self.0]
    }

    pub fn to_lines(&self) -> [Line<T, Z>; 3] {
        [
            Line::new_(self.0, self.1),
            Line::new_(self.1, self.2),
            Line::new_(self.2, self.0),
        ]
    }
}

impl<IC: Into<Coordinate<T, Z>> + Copy, T: CoordNum, Z: CoordNum> From<[IC; 3]> for Triangle<T, Z> {
    fn from(array: [IC; 3]) -> Self {
        Self(array[0].into(), array[1].into(), array[2].into())
    }
}

impl<T: CoordNum, Z: CoordNum> Surface for Triangle<T, Z> {}

impl<T, Z> RelativeEq for Triangle<T, Z>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum + RelativeEq,
    Z: AbsDiffEq<Epsilon = Z> + CoordNum + RelativeEq,
{
    #[inline]
    fn default_max_relative() -> Self::Epsilon {
        T::default_max_relative()
    }

    #[inline]
    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        if !self.0.relative_eq(&other.0, epsilon, max_relative) {
            return false;
        }
        if !self.1.relative_eq(&other.1, epsilon, max_relative) {
            return false;
        }
        if !self.2.relative_eq(&other.2, epsilon, max_relative) {
            return false;
        }

        true
    }
}

impl<T, Z> AbsDiffEq for Triangle<T, Z>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum,
    Z: AbsDiffEq<Epsilon = Z> + CoordNum,
    T::Epsilon: Copy,
    Z::Epsilon: Copy,
{
    type Epsilon = T;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        if !self.0.abs_diff_eq(&other.0, epsilon) {
            return false;
        }
        if !self.1.abs_diff_eq(&other.1, epsilon) {
            return false;
        }
        if !self.2.abs_diff_eq(&other.2, epsilon) {
            return false;
        }

        true
    }
}
