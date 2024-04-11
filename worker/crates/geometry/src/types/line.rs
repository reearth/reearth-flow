use approx::{AbsDiffEq, RelativeEq};
use serde::{Deserialize, Serialize};

use super::coordinate::Coordinate;
use super::coordnum::CoordNum;
use super::{no_value::NoValue, point::Point};

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub struct Line<T: CoordNum = f64, Z: CoordNum = NoValue> {
    pub start: Coordinate<T, Z>,
    pub end: Coordinate<T, Z>,
}

pub type Line2D<T> = Line<T>;
pub type Line3D<T> = Line<T, T>;

impl<T: CoordNum, Z: CoordNum> Line<T, Z> {
    pub fn new<C>(start: C, end: C) -> Self
    where
        C: Into<Coordinate<T, Z>>,
    {
        Self {
            start: start.into(),
            end: end.into(),
        }
    }
}

impl<T: CoordNum> Line<T, T> {
    pub fn new_<C>(start: C, end: C) -> Self
    where
        C: Into<Coordinate<T, T>>,
    {
        Self {
            start: start.into(),
            end: end.into(),
        }
    }
}

impl<T: CoordNum, Z: CoordNum> Line<T, Z> {
    pub fn delta(&self) -> Coordinate<T, Z> {
        self.end - self.start
    }

    pub fn dx(&self) -> T {
        self.delta().x
    }

    pub fn dy(&self) -> T {
        self.delta().y
    }

    pub fn dz(&self) -> Z {
        self.delta().z
    }
}

impl<T: CoordNum> Line<T> {
    pub fn slope(&self) -> T {
        self.dy() / self.dx()
    }

    pub fn determinant(&self) -> T {
        self.start.x * self.end.y - self.start.y * self.end.x
    }
}

impl<T: CoordNum, Z: CoordNum> Line<T, Z> {
    pub fn start_point(&self) -> Point<T, Z> {
        Point::from(self.start)
    }

    pub fn end_point(&self) -> Point<T, Z> {
        Point::from(self.end)
    }

    pub fn points(&self) -> (Point<T, Z>, Point<T, Z>) {
        (self.start_point(), self.end_point())
    }
}

impl<T: CoordNum> From<[(T, T); 2]> for Line<T> {
    fn from(coord: [(T, T); 2]) -> Self {
        Line::new(coord[0], coord[1])
    }
}

impl<T: CoordNum> From<[(T, T, T); 2]> for Line<T, T> {
    fn from(coord: [(T, T, T); 2]) -> Self {
        Line::new_(coord[0], coord[1])
    }
}

impl<T> RelativeEq for Line<T, T>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum + RelativeEq,
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
        self.start.relative_eq(&other.start, epsilon, max_relative)
            && self.end.relative_eq(&other.end, epsilon, max_relative)
    }
}

impl<T: AbsDiffEq<Epsilon = T> + CoordNum> AbsDiffEq for Line<T, T> {
    type Epsilon = T;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.start.abs_diff_eq(&other.start, epsilon) && self.end.abs_diff_eq(&other.end, epsilon)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{coord, point};

    #[test]
    fn test_abs_diff_eq() {
        let delta = 1e-6;
        let line = Line::new(
            coord! { x: 0., y: 0., z: 0. },
            coord! { x: 1., y: 1., z: 1. },
        );
        let line_start_x = Line::new(
            point! {
                x: 0. + delta,
                y: 0.,
                z: 0.,
            },
            point! { x: 1., y: 1., z: 1. },
        );
        assert!(line.abs_diff_eq(&line_start_x, 1e-2));
        assert!(line.abs_diff_ne(&line_start_x, 1e-12));
    }

    #[test]
    fn test_relative_eq() {
        let delta = 1e-6;

        let line = Line::new(
            coord! { x: 0., y: 0., z: 0. },
            coord! { x: 1., y: 1., z: 1. },
        );
        let line_start_x = Line::new(
            point! {
                x: 0. + delta,
                y: 0.,
                z: 0.,
            },
            point! { x: 1., y: 1., z: 1. },
        );
        let line_start_y = Line::new(
            coord! {
                x: 0.,
                y: 0. + delta,
                z: 0.,
            },
            coord! { x: 1., y: 1., z: 1.},
        );

        assert!(line.relative_eq(&line_start_x, 1e-2, 1e-2));
        assert!(line.relative_ne(&line_start_x, 1e-12, 1e-12));

        assert!(line.relative_eq(&line_start_y, 1e-2, 1e-2));
        assert!(line.relative_ne(&line_start_y, 1e-12, 1e-12));
    }
}
