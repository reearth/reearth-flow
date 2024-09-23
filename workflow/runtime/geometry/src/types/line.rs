use std::fmt::Debug;

use approx::{AbsDiffEq, RelativeEq};
use num_traits::Zero;
use serde::{Deserialize, Serialize};

use crate::utils::{line_bounding_rect, point_line_euclidean_distance};

use super::conversion::geojson::create_from_line_type;
use super::coordinate::Coordinate;
use super::coordnum::{CoordFloat, CoordNum};
use super::traits::Elevation;
use super::{no_value::NoValue, point::Point};

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub struct Line<T: CoordNum = f64, Z: CoordNum = f64> {
    pub start: Coordinate<T, Z>,
    pub end: Coordinate<T, Z>,
}

pub type Line2D<T> = Line<T, NoValue>;
pub type Line3D<T> = Line<T, T>;

impl<T: CoordNum> Line<T, NoValue> {
    pub fn new<C>(start: C, end: C) -> Self
    where
        C: Into<Coordinate<T, NoValue>>,
    {
        Self {
            start: start.into(),
            end: end.into(),
        }
    }
}

impl<T: CoordNum, Z: CoordNum> Line<T, Z> {
    pub fn new_<C>(start: C, end: C) -> Self
    where
        C: Into<Coordinate<T, Z>>,
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

    pub fn slope(&self) -> Coordinate<T, Z> {
        Coordinate {
            x: self.end.x - self.start.x,
            y: self.end.y - self.start.y,
            z: self.end.z - self.start.z,
        }
    }

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

impl<T: CoordNum> Line3D<T> {
    pub fn determinant3d(&self) -> T {
        self.start.x * (self.end.y * self.start.z - self.end.z * self.start.y)
            - self.start.y * (self.end.x * self.start.z - self.end.z * self.start.x)
            + self.start.z * (self.end.x * self.start.y - self.end.y * self.start.x)
    }
}

impl<T: CoordNum> Line2D<T> {
    pub fn determinant2d(&self) -> T {
        self.start.x * self.end.y - self.start.y * self.end.x
    }
}

impl From<Line3D<f64>> for Line2D<f64> {
    fn from(line: Line3D<f64>) -> Self {
        Line::new(line.start.x_y(), line.end.x_y())
    }
}

impl<T: CoordNum> From<[(T, T); 2]> for Line<T, NoValue> {
    fn from(coord: [(T, T); 2]) -> Self {
        Line::new(coord[0], coord[1])
    }
}

impl<T: CoordNum> From<[(T, T, T); 2]> for Line<T, T> {
    fn from(coord: [(T, T, T); 2]) -> Self {
        Line::new_(coord[0], coord[1])
    }
}

impl<T: CoordFloat, Z: CoordFloat> From<Line<T, Z>> for geojson::Value {
    fn from(line: Line<T, Z>) -> Self {
        let coords = create_from_line_type(&line);
        geojson::Value::LineString(coords)
    }
}

impl<T, Z> RelativeEq for Line<T, Z>
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
        self.start.relative_eq(&other.start, epsilon, max_relative)
            && self.end.relative_eq(&other.end, epsilon, max_relative)
    }
}

impl<T: AbsDiffEq<Epsilon = T> + CoordNum, Z: AbsDiffEq<Epsilon = Z> + CoordNum> AbsDiffEq
    for Line<T, Z>
{
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

impl<T, Z> rstar::RTreeObject for Line<T, Z>
where
    T: num_traits::Float + rstar::RTreeNum + CoordNum,
    Z: num_traits::Float + rstar::RTreeNum + CoordNum,
{
    type Envelope = rstar::AABB<Point<T, Z>>;

    fn envelope(&self) -> Self::Envelope {
        let bounding_rect = line_bounding_rect(*self);
        rstar::AABB::from_corners(bounding_rect.min().into(), bounding_rect.max().into())
    }
}

impl<T, Z> rstar::PointDistance for Line<T, Z>
where
    T: num_traits::Float + rstar::RTreeNum + CoordNum,
    Z: num_traits::Float + rstar::RTreeNum + CoordNum,
{
    fn distance_2(&self, point: &Point<T, Z>) -> T {
        let d = point_line_euclidean_distance(*point, *self);
        d.powi(2)
    }
}

impl<T, Z> Elevation for Line<T, Z>
where
    T: CoordNum + Zero,
    Z: CoordNum + Zero,
{
    #[inline]
    fn is_elevation_zero(&self) -> bool {
        self.start.is_elevation_zero() && self.end.is_elevation_zero()
    }
}
