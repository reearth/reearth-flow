use std::fmt::Debug;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use approx::{AbsDiffEq, RelativeEq};
use geo_types::Point as GeoPoint;
use nalgebra::{Point2 as NaPoint2, Point3 as NaPoint3};
use num_traits::Zero;
use nusamai_projection::vshift::Jgd2011ToWgs84;
use serde::{Deserialize, Serialize};

use super::conversion::geojson::{
    create_geo_point_2d, create_geo_point_3d, create_point_type, mismatch_geom_err,
};
use super::traits::Elevation;
use crate::{coord, point};

use super::coordinate::Coordinate;
use super::coordnum::{CoordFloat, CoordNum, CoordNumT};
use super::no_value::NoValue;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Debug, Hash, Default)]
pub struct Point<T: CoordNum = f64, Z: CoordNum = f64>(pub Coordinate<T, Z>);

pub type Point2D<T> = Point<T, NoValue>;
pub type Point3D<T> = Point<T, T>;

impl From<Point3D<f64>> for Point2D<f64> {
    fn from(p: Point3D<f64>) -> Point2D<f64> {
        point! { x: p.x(), y: p.y() }
    }
}

impl<T: CoordNum, Z: CoordNum> From<Coordinate<T, Z>> for Point<T, Z> {
    fn from(coords: Coordinate<T, Z>) -> Self {
        Self(coords)
    }
}

impl<T: CoordNum> From<(T, T)> for Point2D<T> {
    fn from(coords: (T, T)) -> Self {
        point!(x: coords.0, y: coords.1)
    }
}

impl<T: CoordNum> From<[T; 2]> for Point2D<T> {
    fn from(coords: [T; 2]) -> Self {
        point!(x: coords[0], y:coords[1])
    }
}

impl<T: CoordNum> From<[T; 3]> for Point3D<T> {
    fn from(coords: [T; 3]) -> Self {
        Point::new_(coords[0], coords[1], coords[2])
    }
}

impl<T: CoordNum> From<Point2D<T>> for (T, T) {
    fn from(point: Point2D<T>) -> Self {
        point.0.into()
    }
}

impl<T: CoordNum, Z: CoordNum> From<Point<T, Z>> for (T, T, Z) {
    fn from(point: Point<T, Z>) -> Self {
        point.0.into()
    }
}

impl<T: CoordNum> From<Point2D<T>> for [T; 2] {
    fn from(point: Point2D<T>) -> Self {
        point.0.into()
    }
}

impl<T: CoordNum> From<Point3D<T>> for [T; 3] {
    fn from(point: Point3D<T>) -> Self {
        point.0.into()
    }
}

impl<T: CoordNum, Z: CoordNum> Point<T, Z> {
    pub fn new(x: T, y: T, z: Z) -> Self {
        point! { x: x, y: y, z: z }
    }
}

impl<T: CoordFloat, Z: CoordFloat> From<Point<T, Z>> for geojson::Value {
    fn from(point: Point<T, Z>) -> Self {
        let coords = create_point_type(&point);
        geojson::Value::Point(coords)
    }
}

impl TryFrom<geojson::Value> for Point2D<f64> {
    type Error = crate::error::Error;

    fn try_from(value: geojson::Value) -> crate::error::Result<Self> {
        match value {
            geojson::Value::Point(point_type) => Ok(create_geo_point_2d(&point_type)),
            other => Err(mismatch_geom_err("Point", &other)),
        }
    }
}

impl TryFrom<geojson::Value> for Point3D<f64> {
    type Error = crate::error::Error;

    fn try_from(value: geojson::Value) -> crate::error::Result<Self> {
        match value {
            geojson::Value::Point(point_type) => Ok(create_geo_point_3d(&point_type)),
            other => Err(mismatch_geom_err("Point", &other)),
        }
    }
}

impl<T: CoordNum, Z: CoordNum> Point<T, Z> {
    pub fn new_(x: T, y: T, z: Z) -> Self {
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

    pub fn z(self) -> Z {
        self.0.z
    }

    pub fn set_z(&mut self, z: Z) -> &mut Self {
        self.0.z = z;
        self
    }
}

impl<T: CoordNum> Point3D<T> {
    pub fn x_y_z(self) -> (T, T, T) {
        (self.0.x, self.0.y, self.0.z)
    }
}
impl<T: CoordNum> Point2D<T> {
    pub fn dot(self, other: Self) -> T {
        self.x() * other.x() + self.y() * other.y()
    }

    pub fn cross_prod(self, point_b: Self, point_c: Self) -> T {
        (point_b.x() - self.x()) * (point_c.y() - self.y())
            - (point_b.y() - self.y()) * (point_c.x() - self.x())
    }
}

impl<T: CoordFloat> Point2D<T> {
    pub fn to_degrees(self) -> Self {
        let (x, y) = self.x_y();
        let x = x.to_degrees();
        let y = y.to_degrees();
        point!(x: x, y: y)
    }

    pub fn to_radians(self) -> Self {
        let (x, y) = self.x_y();
        let x = x.to_radians();
        let y = y.to_radians();
        point!(x: x, y: y)
    }
}

impl From<Point2D<f64>> for NaPoint2<f64> {
    #[inline]
    fn from(p: Point2D<f64>) -> NaPoint2<f64> {
        NaPoint2::new(p.0.x, p.0.y)
    }
}

impl From<Point3D<f64>> for NaPoint3<f64> {
    #[inline]
    fn from(p: Point3D<f64>) -> NaPoint3<f64> {
        NaPoint3::new(p.0.x, p.0.y, p.0.z)
    }
}

impl<T, Z> Neg for Point<T, Z>
where
    T: CoordNum + Neg<Output = T>,
    Z: CoordNum + Neg<Output = Z>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Point::from(-self.0)
    }
}

impl<T: CoordNum, Z: CoordNum> Add for Point<T, Z> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Point::from(self.0 + rhs.0)
    }
}

impl<T: CoordNum, Z: CoordNum> AddAssign for Point<T, Z> {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0 + rhs.0;
    }
}

impl<T: CoordNum, Z: CoordNum> Sub for Point<T, Z> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Point::from(self.0 - rhs.0)
    }
}

impl<T: CoordNum, Z: CoordNum> SubAssign for Point<T, Z> {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0 - rhs.0;
    }
}

impl<T: CoordNumT> Mul<T> for Point2D<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        let x = self.x() * rhs;
        let y = self.y() * rhs;
        Point::from(coord! { x: x, y: y })
    }
}

impl<T: CoordNumT> Mul<T> for Point3D<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        let x = self.x() * rhs;
        let y = self.y() * rhs;
        let z = self.z() * rhs;
        Point::from(coord! { x: x, y: y, z: z })
    }
}

impl<T: CoordNumT> MulAssign<T> for Point3D<T> {
    fn mul_assign(&mut self, rhs: T) {
        let x = self.x() * rhs;
        let y = self.y() * rhs;
        let z = self.z() * rhs;
        self.0 = coord! {
            x: x,
            y: y,
            z: z,
        }
    }
}

impl<T: CoordNum, Z: CoordNum> Mul for Point<T, Z> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let x = self.x() * rhs.x();
        let y = self.y() * rhs.y();
        let z = self.z() * rhs.z();
        Point::from(coord! { x: x, y: y, z: z })
    }
}

impl<T: CoordNum, Z: CoordNum> MulAssign for Point<T, Z> {
    fn mul_assign(&mut self, rhs: Self) {
        let x = self.x() * rhs.x();
        let y = self.y() * rhs.y();
        let z = self.z() * rhs.z();
        self.0 = coord! {
            x: x,
            y: y,
            z: z,
        }
    }
}

impl<T: CoordNum> Div<T> for Point2D<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        let x = self.x() / rhs;
        let y = self.y() / rhs;
        Point::from(coord! { x: x, y: y})
    }
}

impl<T: CoordNumT> Div<T> for Point3D<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        let x = self.x() / rhs;
        let y = self.y() / rhs;
        let z = self.z() / rhs;
        Point::from(coord! { x: x, y: y, z: z})
    }
}

impl<T: CoordNum, Z: CoordNum> Div for Point<T, Z> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        let x = self.x() / rhs.x();
        let y = self.y() / rhs.y();
        let z = self.z() / rhs.z();
        Point::from(coord! { x: x, y: y, z: z })
    }
}

impl<T: CoordNum, Z: CoordNum> DivAssign for Point<T, Z> {
    fn div_assign(&mut self, rhs: Self) {
        let x = self.x() / rhs.x();
        let y = self.y() / rhs.y();
        let z = self.z() / rhs.z();
        self.0 = coord! { x: x, y: y, z: z };
    }
}

impl<T, Z> RelativeEq for Point<T, Z>
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
        self.0.relative_eq(&other.0, epsilon, max_relative)
    }
}

impl<T, Z> AbsDiffEq for Point<T, Z>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum,
    Z: AbsDiffEq<Epsilon = Z> + CoordNum,
    T::Epsilon: Copy,
    Z::Epsilon: Copy,
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

impl rstar::Point for Point2D<f64> {
    type Scalar = f64;

    const DIMENSIONS: usize = 2;

    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        point!(x: generator(0), y:generator(1))
    }

    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.0.x,
            1 => self.0.y,
            _ => unreachable!(),
        }
    }
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.0.x,
            1 => &mut self.0.y,
            _ => unreachable!(),
        }
    }
}

impl rstar::Point for Point3D<f64> {
    type Scalar = f64;

    const DIMENSIONS: usize = 3;

    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        Point::new_(generator(0), generator(1), generator(2))
    }

    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.0.x,
            1 => self.0.y,
            2 => self.0.z,
            _ => unreachable!(),
        }
    }
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.0.x,
            1 => &mut self.0.y,
            2 => &mut self.0.z,
            _ => unreachable!(),
        }
    }
}

impl<T, Z> Elevation for Point<T, Z>
where
    T: CoordNum + Zero,
    Z: CoordNum + Zero,
{
    #[inline]
    fn is_elevation_zero(&self) -> bool {
        self.0.is_elevation_zero()
    }
}

impl<Z: CoordFloat> Point<f64, Z> {
    pub fn approx_eq(&self, other: &Point<f64, Z>, epsilon: f64) -> bool {
        self.0.approx_eq(&other.0, epsilon)
    }
}

impl<T: CoordNum> From<GeoPoint<T>> for Point2D<T> {
    fn from(coord: GeoPoint<T>) -> Self {
        Point2D::from((coord.x(), coord.y()))
    }
}

impl<T: CoordNum> From<Point2D<T>> for GeoPoint<T> {
    fn from(coord: Point2D<T>) -> Self {
        GeoPoint::new(coord.x(), coord.y())
    }
}

impl Point3D<f64> {
    pub fn transform_inplace(&mut self, jgd2wgs: &Jgd2011ToWgs84) {
        self.0.transform_inplace(jgd2wgs);
    }

    pub fn transform_offset(&mut self, x: f64, y: f64, z: f64) {
        self.0.transform_offset(x, y, z);
    }
}
