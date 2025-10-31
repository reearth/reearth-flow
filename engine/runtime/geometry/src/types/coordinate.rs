use std::fmt::Debug;
use std::ops::{Add, Div, Mul, Neg, Sub};

use approx::{AbsDiffEq, RelativeEq, UlpsEq};
use geo_types::Coord as GeoCoord;
use nalgebra::{Point2 as NaPoint2, Point3 as NaPoint3};
use num_traits::{Float, NumCast, Zero};
use nusamai_projection::vshift::Jgd2011ToWgs84;
use serde::{Deserialize, Serialize};
use std::ops::Index;

use super::coordnum::{CoordFloat, CoordNum};
use super::no_value::NoValue;
use super::point::Point;
use super::traits::Elevation;
use crate::algorithm::GeoFloat;
use crate::coord;
use crate::utils::{are_points_coplanar, PointsCoplanar};

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Debug, Hash, Default)]
pub struct Coordinate<T: CoordNum = f64, Z: CoordNum = f64> {
    pub x: T,
    pub y: T,
    pub z: Z,
}

impl<T: CoordNum> Coordinate<T, NoValue> {
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

    #[inline]
    pub fn is_2d(&self) -> bool {
        self.z.to_f64().is_none()
    }

    #[inline]
    pub fn is_3d(&self) -> bool {
        !self.is_2d()
    }

    #[inline]
    pub fn is_z_zero(&self) -> bool {
        self.is_3d() && self.z.is_zero()
    }
}

pub type Coordinate2D<T> = Coordinate<T, NoValue>;
pub type Coordinate3D<T> = Coordinate<T, T>;

impl<T: CoordNum> From<(T, T)> for Coordinate<T, NoValue> {
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

impl<T: CoordNum> From<[T; 2]> for Coordinate<T, NoValue> {
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

impl From<Coordinate<f64, f64>> for Coordinate<f64, NoValue> {
    #[inline]
    fn from(coords: Coordinate<f64, f64>) -> Self {
        Coordinate::new__(coords.x, coords.y, NoValue)
    }
}

impl Coordinate3D<f64> {
    pub fn transform_inplace(&mut self, jgd2wgs: &Jgd2011ToWgs84) {
        let (x, y, z) = jgd2wgs.convert(self.x, self.y, self.z);
        self.x = x;
        self.y = y;
        self.z = z;
    }

    pub fn transform_offset(&mut self, x: f64, y: f64, z: f64) {
        if x.is_nan() || y.is_nan() || z.is_nan() {
            return;
        }
        self.x += x;
        self.y += y;
        self.z += z;
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

impl<T: CoordNum, Z: CoordNum> From<Coordinate<T, Z>> for (T, T, Z) {
    #[inline]
    fn from(coord: Coordinate<T, Z>) -> Self {
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

impl<T: GeoFloat + 'static> From<NaPoint2<T>> for Coordinate2D<T> {
    #[inline]
    fn from(p: NaPoint2<T>) -> Self {
        Self::new_(p.x, p.y)
    }
}

impl<T: GeoFloat + 'static> From<Coordinate2D<T>> for NaPoint2<T> {
    #[inline]
    fn from(p: Coordinate2D<T>) -> Self {
        Self::new(p.x, p.y)
    }
}

impl<T: GeoFloat + 'static> From<NaPoint3<T>> for Coordinate3D<T> {
    #[inline]
    fn from(p: NaPoint3<T>) -> Self {
        Self::new__(p.x, p.y, p.z)
    }
}

impl<T: GeoFloat + 'static> From<Coordinate3D<T>> for NaPoint3<T> {
    #[inline]
    fn from(p: Coordinate3D<T>) -> Self {
        Self::new(p.x, p.y, p.z)
    }
}

impl<T: CoordNum, Z: CoordNum> Coordinate<T, Z> {
    #[inline]
    pub fn x_y(&self) -> (T, T) {
        (self.x, self.y)
    }

    pub fn x_y_z(&self) -> (T, T, Z) {
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

impl<T, Z> Mul for Coordinate<T, Z>
where
    T: CoordNum,
    Z: CoordNum + Mul<T, Output = Z>,
{
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        coord! {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
            z: self.z * rhs.z,
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

impl<T, Z> Div for Coordinate<T, Z>
where
    T: CoordNum,
    Z: CoordNum + Div<T, Output = Z>,
{
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self {
        coord! {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
            z: self.z / rhs.z,
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

impl<T: CoordNum + Copy + From<Z>, Z: CoordNum> Coordinate<T, Z> {
    pub fn dot(&self, other: &Self) -> T {
        self.x * other.x + self.y * other.y + (self.z * other.z).into()
    }
}

impl<T: CoordNum + Float + From<Z>, Z: CoordNum> Coordinate<T, Z> {
    pub fn norm(&self) -> T {
        (self.x * self.x + self.y * self.y + (self.z * self.z).into()).sqrt()
    }
}

impl<T, Z> Coordinate<T, Z>
where
    T: CoordNum + Float + From<Z>,
    Z: CoordNum + Div<T, Output = Z>,
{
    pub fn normalize(&self) -> Self {
        let norm = self.norm();
        if norm.is_zero() {
            *self
        } else {
            *self / norm
        }
    }
}

impl<T: CoordNum + Float + From<Z>, Z: CoordNum> Coordinate<T, Z> {
    pub fn cross(&self, other: &Self) -> Self {
        coord! {
            x: self.y * other.z.into() - other.y * self.z.into(),
            y: other.x * self.z.into() - self.x * other.z.into(),
            z: Z::from(self.x * other.y - self.y * other.x).unwrap(),
        }
    }
}

impl<T: CoordNum + Float + From<Z>, Z: CoordNum> Coordinate<T, Z> {
    /// Returns the smaller angle (in radians) between this vector and another.
    pub fn angle(&self, other: &Self) -> T {
        let norms = self.norm() * other.norm();
        if norms.is_zero() {
            T::zero()
        } else {
            let dot = self.dot(other);
            let cos_theta = (dot / norms).clamp(-T::one(), T::one());
            let out = cos_theta.to_f64().unwrap().acos();
            <T as NumCast>::from(out).unwrap()
        }
    }
}

impl<T: CoordNum> Index<usize> for Coordinate3D<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &T {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            _ => panic!("Index out of bounds for Coordinate"),
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

impl<T: CoordNum + AbsDiffEq, Z: CoordNum + AbsDiffEq> AbsDiffEq for Coordinate<T, Z>
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
            && Z::abs_diff_eq(&self.z, &other.z, Z::default_epsilon())
    }
}

impl<T: CoordNum + RelativeEq, Z: CoordNum + RelativeEq> RelativeEq for Coordinate<T, Z>
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
            && Z::relative_eq(
                &self.z,
                &other.z,
                Z::default_epsilon(),
                Z::default_max_relative(),
            )
    }
}

impl<T: CoordNum + UlpsEq, Z: CoordNum + UlpsEq> UlpsEq for Coordinate<T, Z>
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
            && Z::ulps_eq(
                &self.z,
                &other.z,
                Z::default_epsilon(),
                Z::default_max_ulps(),
            )
    }
}

impl<T, Z> ::rstar::Point for Coordinate<T, Z>
where
    T: ::num_traits::Float + ::rstar::RTreeNum + Default,
    Z: ::num_traits::Float + ::rstar::RTreeNum + Default,
{
    type Scalar = T;

    const DIMENSIONS: usize = 3;

    fn generate(mut generator: impl FnMut(usize) -> Self::Scalar) -> Self {
        Coordinate::new__(generator(0), generator(1), Z::zero())
    }

    #[inline]
    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.x,
            1 => self.y,
            2 => T::zero(),
            _ => unreachable!(),
        }
    }

    #[inline]
    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => unreachable!(),
        }
    }
}

impl<T, Z> Elevation for Coordinate<T, Z>
where
    T: CoordNum + Zero,
    Z: CoordNum + Zero,
{
    #[inline]
    fn is_elevation_zero(&self) -> bool {
        self.z.is_zero()
    }
}

impl<Z: CoordFloat> Coordinate<f64, Z> {
    pub fn approx_eq(&self, other: &Coordinate<f64, Z>, epsilon: f64) -> bool {
        let result = (self.x - other.x).abs() <= epsilon && (self.y - other.y).abs() <= epsilon;
        if self.is_3d() {
            result && (self.z.to_f64().unwrap() - other.z.to_f64().unwrap()).abs() <= epsilon
        } else {
            result
        }
    }
}

impl<T: CoordNum> From<GeoCoord<T>> for Coordinate2D<T> {
    fn from(coord: GeoCoord<T>) -> Self {
        Coordinate::new_(coord.x, coord.y)
    }
}

impl<T: CoordNum> From<Coordinate2D<T>> for GeoCoord<T> {
    fn from(coord: Coordinate2D<T>) -> Self {
        GeoCoord {
            x: coord.x,
            y: coord.y,
        }
    }
}

pub fn are_coplanar(points: &[Coordinate3D<f64>]) -> Option<PointsCoplanar> {
    let points = points
        .iter()
        .map(|c| NaPoint3::new(c.x, c.y, c.z))
        .collect();
    are_points_coplanar(points, 1e-6)
}
