use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use approx::{AbsDiffEq, RelativeEq};
use serde::{Deserialize, Serialize};

use crate::point;

use super::coordinate::Coordinate;
use super::coordnum::{CoordFloat, CoordNum};
use super::no_value::NoValue;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Debug, Hash, Default)]
pub struct Point<T: CoordNum = f64, Z: CoordNum = NoValue>(pub Coordinate<T, Z>);

pub type Point2D<T> = Point<T, NoValue>;
pub type Point3D<T> = Point<T, T>;

impl<T: CoordNum, Z: CoordNum> From<Coordinate<T, Z>> for Point<T, Z> {
    fn from(coords: Coordinate<T, Z>) -> Self {
        Self(coords)
    }
}

impl<T: CoordNum> From<(T, T)> for Point<T> {
    fn from(coords: (T, T)) -> Self {
        Point::new(coords.0, coords.1)
    }
}

impl<T: CoordNum> From<[T; 2]> for Point<T> {
    fn from(coords: [T; 2]) -> Self {
        Point::new(coords[0], coords[1])
    }
}

impl<T: CoordNum> From<[T; 3]> for Point<T, T> {
    fn from(coords: [T; 3]) -> Self {
        Point::new_(coords[0], coords[1], coords[2])
    }
}

impl<T: CoordNum> From<Point<T>> for (T, T) {
    fn from(point: Point<T>) -> Self {
        point.0.into()
    }
}

impl<T: CoordNum> From<Point<T, T>> for (T, T, T) {
    fn from(point: Point<T, T>) -> Self {
        point.0.into()
    }
}

impl<T: CoordNum> From<Point<T>> for [T; 2] {
    fn from(point: Point<T>) -> Self {
        point.0.into()
    }
}

impl<T: CoordNum> From<Point<T, T>> for [T; 3] {
    fn from(point: Point<T, T>) -> Self {
        point.0.into()
    }
}

impl<T: CoordNum> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        point! { x: x, y: y }
    }
}

impl<T: CoordNum> Point<T, T> {
    pub fn new_(x: T, y: T, z: T) -> Self {
        point! { x: x, y: y, z: z }
    }
}

impl<T: CoordNum, Z: CoordNum> Point<T, Z> {
    pub fn x(self) -> T {
        self.0.x
    }

    pub fn set_x(&mut self, x: T) -> &mut Self {
        self.0.x = x;
        self
    }

    pub fn y(self) -> T {
        self.0.y
    }

    pub fn set_y(&mut self, y: T) -> &mut Self {
        self.0.y = y;
        self
    }

    pub fn x_y(self) -> (T, T) {
        (self.0.x, self.0.y)
    }
}

impl<T: CoordNum> Point<T, T> {
    pub fn z(self) -> T {
        self.0.z
    }

    pub fn set_z(&mut self, z: T) -> &mut Self {
        self.0.z = z;
        self
    }

    pub fn x_y_z(self) -> (T, T, T) {
        (self.0.x, self.0.y, self.0.z)
    }
}

impl<T: CoordNum> Point<T> {
    pub fn dot(self, other: Self) -> T {
        self.x() * other.x() + self.y() * other.y()
    }

    pub fn cross_prod(self, point_b: Self, point_c: Self) -> T {
        (point_b.x() - self.x()) * (point_c.y() - self.y())
            - (point_b.y() - self.y()) * (point_c.x() - self.x())
    }
}

impl<T: CoordFloat> Point<T> {
    pub fn to_degrees(self) -> Self {
        let (x, y) = self.x_y();
        let x = x.to_degrees();
        let y = y.to_degrees();
        Point::new(x, y)
    }

    pub fn to_radians(self) -> Self {
        let (x, y) = self.x_y();
        let x = x.to_radians();
        let y = y.to_radians();
        Point::new(x, y)
    }
}

impl<T> Neg for Point<T>
where
    T: CoordNum + Neg<Output = T>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Point::from(-self.0)
    }
}

impl<T: CoordNum> Add for Point<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Point::from(self.0 + rhs.0)
    }
}

impl<T: CoordNum> AddAssign for Point<T> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0 + rhs.0;
    }
}

impl<T: CoordNum> Sub for Point<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Point::from(self.0 - rhs.0)
    }
}

impl<T: CoordNum> SubAssign for Point<T> {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0 - rhs.0;
    }
}

impl<T: CoordNum> Mul<T> for Point<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Point::from(self.0 * rhs)
    }
}

impl<T: CoordNum> MulAssign<T> for Point<T> {
    fn mul_assign(&mut self, rhs: T) {
        self.0 = self.0 * rhs
    }
}

impl<T: CoordNum> Div<T> for Point<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Point::from(self.0 / rhs)
    }
}

impl<T: CoordNum> DivAssign<T> for Point<T> {
    fn div_assign(&mut self, rhs: T) {
        self.0 = self.0 / rhs
    }
}

impl<T> RelativeEq for Point<T, T>
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
        self.0.relative_eq(&other.0, epsilon, max_relative)
    }
}

impl<T> AbsDiffEq for Point<T, T>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum,
    T::Epsilon: Copy,
{
    type Epsilon = T::Epsilon;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.0.abs_diff_eq(&other.0, epsilon)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use approx::*;

    #[test]
    fn test_point() {
        let p: Point2D<_> = point! { x: 1.0, y: 2.0 };
        assert_relative_eq!(p.x(), 1.0);
        assert_relative_eq!(p.y(), 2.0);

        let p: Point3D<_> = point! { x: 1.0, y: 2.0, z: 3.0 };
        assert_relative_eq!(p.x(), 1.0);
        assert_relative_eq!(p.y(), 2.0);
        assert_relative_eq!(p.z(), 3.0);
    }
}
