use approx::{AbsDiffEq, RelativeEq};
use num_traits::{NumOps, One};
use serde::{Deserialize, Serialize};

use crate::{coord, polygon};

use super::coordinate::Coordinate;
use super::coordnum::CoordNum;
use super::line::Line;
use super::no_value::NoValue;
use super::polygon::Polygon;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub struct Rectangle<T: CoordNum = f64, Z: CoordNum = f64> {
    min: Coordinate<T, Z>,
    max: Coordinate<T, Z>,
}

pub type Rectangle2D<T> = Rectangle<T, NoValue>;
pub type Rectangle3D<T> = Rectangle<T, T>;

impl<T: CoordNum, Z: CoordNum> Rectangle<T, Z> {
    pub fn new<C>(c1: C, c2: C) -> Self
    where
        C: Into<Coordinate<T, Z>>,
    {
        let c1 = c1.into();
        let c2 = c2.into();
        let (min_x, max_x) = if c1.x < c2.x {
            (c1.x, c2.x)
        } else {
            (c2.x, c1.x)
        };
        let (min_y, max_y) = if c1.y < c2.y {
            (c1.y, c2.y)
        } else {
            (c2.y, c1.y)
        };
        let (min_z, max_z) = if c1.z < c2.z {
            (c1.z, c2.z)
        } else {
            (c2.z, c1.z)
        };
        Self {
            min: coord! { x: min_x, y: min_y, z: min_z, },
            max: coord! { x: max_x, y: max_y, z: max_z, },
        }
    }

    pub fn min(self) -> Coordinate<T, Z> {
        self.min
    }

    pub fn set_min<C: Into<Coordinate<T, Z>>>(&mut self, min: C) {
        self.min = min.into();
        self.assert_valid_bounds();
    }

    pub fn max(self) -> Coordinate<T, Z> {
        self.max
    }

    pub fn set_max<C: Into<Coordinate<T, Z>>>(&mut self, max: C) {
        self.max = max.into();
        self.assert_valid_bounds();
    }

    pub fn width(self) -> T {
        self.max().x - self.min().x
    }

    pub fn height(self) -> T {
        self.max().y - self.min().y
    }

    pub fn to_polygon(self) -> Polygon<T, NoValue> {
        polygon![
            (x: self.min.x, y: self.min.y),
            (x: self.min.x, y: self.max.y),
            (x: self.max.x, y: self.max.y),
            (x: self.max.x, y: self.min.y),
            (x: self.min.x, y: self.min.y),
        ]
    }

    pub fn to_lines(&self) -> [Line<T, NoValue>; 4] {
        [
            Line::new(
                coord! {
                    x: self.min.x,
                    y: self.min.y,
                },
                coord! {
                    x: self.min.x,
                    y: self.max.y,
                },
            ),
            Line::new(
                coord! {
                    x: self.min.x,
                    y: self.max.y,
                },
                coord! {
                    x: self.max.x,
                    y: self.max.y,
                },
            ),
            Line::new(
                coord! {
                    x: self.max.x,
                    y: self.max.y,
                },
                coord! {
                    x: self.max.x,
                    y: self.min.y,
                },
            ),
            Line::new(
                coord! {
                    x: self.max.x,
                    y: self.min.y,
                },
                coord! {
                    x: self.min.x,
                    y: self.min.y,
                },
            ),
        ]
    }

    fn assert_valid_bounds(&self) {
        if !self.has_valid_bounds() {
            panic!("{}", RECT_INVALID_BOUNDS_ERROR);
        }
    }

    fn has_valid_bounds(&self) -> bool {
        self.min.x <= self.max.x && self.min.y <= self.max.y
    }
}

impl<T: CoordNum> Rectangle<T, NoValue> {
    pub fn split_x(self) -> [Rectangle<T, NoValue>; 2] {
        let two = T::one() + T::one();
        let mid_x = self.min().x + self.width() / two;
        [
            Rectangle::new(self.min(), coord! { x: mid_x, y: self.max().y, }),
            Rectangle::new(coord! { x: mid_x, y: self.min().y }, self.max()),
        ]
    }

    pub fn split_y(self) -> [Rectangle<T, NoValue>; 2] {
        let two = T::one() + T::one();
        let mid_y = self.min().y + self.height() / two;
        [
            Rectangle::new(self.min(), coord! { x: self.max().x, y: mid_y, }),
            Rectangle::new(coord! { x: self.min().x, y: mid_y, }, self.max()),
        ]
    }
}

impl<T, Z> Rectangle<T, Z>
where
    T: CoordNum,
    Z: CoordNum + One + NumOps,
{
    pub fn center(self) -> Coordinate<T, Z> {
        let two = T::one() + T::one();
        coord! {
            x: (self.max.x + self.min.x) / two,
            y: (self.max.y + self.min.y) / two,
            z: (self.max.z + self.min.z) / (Z::one() + Z::one()),
        }
    }
}

static RECT_INVALID_BOUNDS_ERROR: &str = "Failed to create Rect: 'min' coordinate's x/y value must be smaller or equal to the 'max' x/y value";

impl<T> RelativeEq for Rectangle<T, T>
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
        if !self.min.relative_eq(&other.min, epsilon, max_relative) {
            return false;
        }

        if !self.max.relative_eq(&other.max, epsilon, max_relative) {
            return false;
        }

        true
    }
}

impl<T> AbsDiffEq for Rectangle<T, T>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum,
    T::Epsilon: Copy,
{
    type Epsilon = T;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        if !self.min.abs_diff_eq(&other.min, epsilon) {
            return false;
        }

        if !self.max.abs_diff_eq(&other.max, epsilon) {
            return false;
        }

        true
    }
}
