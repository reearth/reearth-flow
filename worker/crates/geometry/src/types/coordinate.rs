use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::coord;
use approx::{AbsDiffEq, RelativeEq, UlpsEq};
use num_traits::Zero;
use serde::{Deserialize, Serialize};

use super::coordnum::CoordNum;
use super::no_value::NoValue;
use super::point::Point;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Debug, Hash, Default)]
pub struct Coordinate<T: CoordNum = f64, Z: CoordNum = NoValue> {
    pub x: T,
    pub y: T,
    pub z: Z,
}

impl<T: CoordNum> Coordinate<T> {
    #[inline]
    pub fn new_(x: T, y: T) -> Self {
        Self { x, y, z: NoValue }
    }
}

impl<T: CoordNum, Z: CoordNum> Coordinate<T, Z> {
    #[inline]
    pub fn new__(x: T, y: T, z: Z) -> Self {
        Self { x, y, z }
    }
}

pub type Coordinate2D<T> = Coordinate<T, NoValue>;
pub type Coordinate3D<T> = Coordinate<T, T>;

impl<T: CoordNum> From<(T, T)> for Coordinate<T> {
    #[inline]
    fn from(coords: (T, T)) -> Self {
        coord! {
            x: coords.0,
            y: coords.1,
        }
    }
}

impl<T: CoordNum> From<(T, T, T)> for Coordinate<T, T> {
    #[inline]
    fn from(coords: (T, T, T)) -> Self {
        coord! {
            x: coords.0,
            y: coords.1,
            z: coords.2,
        }
    }
}

impl<T: CoordNum> From<[T; 2]> for Coordinate<T> {
    #[inline]
    fn from(coords: [T; 2]) -> Self {
        coord! {
            x: coords[0],
            y: coords[1],
        }
    }
}

impl<T: CoordNum> From<[T; 3]> for Coordinate<T, T> {
    #[inline]
    fn from(coords: [T; 3]) -> Self {
        coord! {
            x: coords[0],
            y: coords[1],
            z: coords[2],
        }
    }
}

impl<T: CoordNum, Z: CoordNum> From<Point<T, Z>> for Coordinate<T, Z> {
    #[inline]
    fn from(point: Point<T, Z>) -> Self {
        point.0
    }
}

impl<T: CoordNum, Z: CoordNum> From<Coordinate<T, Z>> for (T, T) {
    #[inline]
    fn from(coord: Coordinate<T, Z>) -> Self {
        (coord.x, coord.y)
    }
}

impl<T: CoordNum> From<Coordinate<T, T>> for (T, T, T) {
    #[inline]
    fn from(coord: Coordinate<T, T>) -> Self {
        (coord.x, coord.y, coord.z)
    }
}

impl<T: CoordNum, Z: CoordNum> From<Coordinate<T, Z>> for [T; 2] {
    #[inline]
    fn from(coord: Coordinate<T, Z>) -> Self {
        [coord.x, coord.y]
    }
}

impl<T: CoordNum> From<Coordinate<T, T>> for [T; 3] {
    #[inline]
    fn from(coord: Coordinate<T, T>) -> Self {
        [coord.x, coord.y, coord.z]
    }
}

impl<T: CoordNum, Z: CoordNum> Coordinate<T, Z> {
    #[inline]
    pub fn x_y(&self) -> (T, T) {
        (self.x, self.y)
    }
}

impl<T: CoordNum> Coordinate<T, T> {
    #[inline]
    pub fn x_y_z(&self) -> (T, T, T) {
        (self.x, self.y, self.z)
    }
}

impl<T, Z> Neg for Coordinate<T, Z>
where
    T: CoordNum + Neg<Output = T>,
    Z: CoordNum + Neg<Output = Z>,
{
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        coord! {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl<T: CoordNum, Z: CoordNum> Add for Coordinate<T, Z> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        coord! {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl<T: CoordNum, Z: CoordNum> Sub for Coordinate<T, Z> {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        coord! {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl<T, Z> Mul<T> for Coordinate<T, Z>
where
    T: CoordNum,
    Z: CoordNum + Mul<T, Output = Z>,
{
    type Output = Self;

    #[inline]
    fn mul(self, rhs: T) -> Self {
        coord! {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl<T, Z> Div<T> for Coordinate<T, Z>
where
    T: CoordNum,
    Z: CoordNum + Div<T, Output = Z>,
{
    type Output = Self;

    #[inline]
    fn div(self, rhs: T) -> Self {
        coord! {
            x: self.x / rhs,
            y: self.y / rhs,
            z: self.z / rhs,
        }
    }
}

impl<T: CoordNum, Z: CoordNum> Coordinate<T, Z> {
    #[inline]
    pub fn zero() -> Self {
        coord! {
            x: T::zero(),
            y: T::zero(),
            z: Z::zero(),
        }
    }
}

impl<T: CoordNum, Z: CoordNum> Zero for Coordinate<T, Z> {
    #[inline]
    fn zero() -> Self {
        Self::zero()
    }
    #[inline]
    fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y.is_zero() && self.z.is_zero()
    }
}

impl<T: CoordNum + AbsDiffEq> AbsDiffEq for Coordinate<T, T>
where
    T::Epsilon: Copy,
{
    type Epsilon = T::Epsilon;

    #[inline]
    fn default_epsilon() -> T::Epsilon {
        T::default_epsilon()
    }

    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: T::Epsilon) -> bool {
        T::abs_diff_eq(&self.x, &other.x, epsilon)
            && T::abs_diff_eq(&self.y, &other.y, epsilon)
            && T::abs_diff_eq(&self.z, &other.z, epsilon)
    }
}

impl<T: CoordNum + RelativeEq> RelativeEq for Coordinate<T, T>
where
    T::Epsilon: Copy,
{
    #[inline]
    fn default_max_relative() -> T::Epsilon {
        T::default_max_relative()
    }

    #[inline]
    fn relative_eq(&self, other: &Self, epsilon: T::Epsilon, max_relative: T::Epsilon) -> bool {
        T::relative_eq(&self.x, &other.x, epsilon, max_relative)
            && T::relative_eq(&self.y, &other.y, epsilon, max_relative)
            && T::relative_eq(&self.z, &other.z, epsilon, max_relative)
    }
}

impl<T: CoordNum + UlpsEq> UlpsEq for Coordinate<T, T>
where
    T::Epsilon: Copy,
{
    #[inline]
    fn default_max_ulps() -> u32 {
        T::default_max_ulps()
    }

    #[inline]
    fn ulps_eq(&self, other: &Self, epsilon: T::Epsilon, max_ulps: u32) -> bool {
        T::ulps_eq(&self.x, &other.x, epsilon, max_ulps)
            && T::ulps_eq(&self.y, &other.y, epsilon, max_ulps)
            && T::ulps_eq(&self.z, &other.z, epsilon, max_ulps)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use approx::*;

    #[test]
    fn test_coordinates() {
        let p = coord! { x: 1.0, y: 2.0 };
        assert_relative_eq!(p.x, 1.0);
        assert_relative_eq!(p.y, 2.0);
        assert_eq!(p.z, NoValue);

        let p = coord! { x: 1.0, y: 2.0, z: 3.0 };
        assert_relative_eq!(p.x, 1.0);
        assert_relative_eq!(p.y, 2.0);
        assert_relative_eq!(p.z, 3.0);
    }
}
