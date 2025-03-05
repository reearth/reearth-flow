use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use approx::{AbsDiffEq, RelativeEq};
use geo_types::Line as GeoLine;
use num_traits::Zero;
use nusamai_projection::vshift::Jgd2011ToWgs84;
use serde::{Deserialize, Serialize};

use crate::utils::{line_bounding_rect, point_line_euclidean_distance};

use super::conversion::geojson::create_from_line_type;
use super::coordinate::Coordinate;
use super::coordnum::{CoordFloat, CoordNum};
use super::line_string::{LineString2D, LineString3D};
use super::point::{Point2D, Point3D};
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

impl<T: CoordFloat> Line2D<T> {
    pub fn length(&self) -> T {
        let dx = self.end.x - self.start.x;
        let dy = self.end.y - self.start.y;
        (dx * dx + dy * dy).sqrt()
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

impl rstar::RTreeObject for Line2D<f64> {
    type Envelope = rstar::AABB<Point2D<f64>>;

    fn envelope(&self) -> Self::Envelope {
        let bounding_rect = line_bounding_rect(*self);
        rstar::AABB::from_corners(bounding_rect.min().into(), bounding_rect.max().into())
    }
}

impl rstar::RTreeObject for Line3D<f64> {
    type Envelope = rstar::AABB<Point3D<f64>>;

    fn envelope(&self) -> Self::Envelope {
        let bounding_rect = line_bounding_rect(*self);
        rstar::AABB::from_corners(bounding_rect.min().into(), bounding_rect.max().into())
    }
}

impl rstar::PointDistance for Line2D<f64> {
    fn distance_2(&self, point: &Point2D<f64>) -> f64 {
        let d = point_line_euclidean_distance(*point, *self);
        d.powi(2)
    }
}

impl rstar::PointDistance for Line3D<f64> {
    fn distance_2(&self, point: &Point3D<f64>) -> f64 {
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

impl<Z: CoordFloat> Line<f64, Z> {
    pub fn approx_eq(&self, other: &Line<f64, Z>, epsilon: f64) -> bool {
        self.start.approx_eq(&other.start, epsilon) && self.end.approx_eq(&other.end, epsilon)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Line2DFloat(pub Line2D<f64>);

impl Eq for Line2DFloat {}

impl PartialEq for Line2DFloat {
    fn eq(&self, other: &Self) -> bool {
        let epsilon = 0.001;
        (self.0.start.approx_eq(&other.0.start, epsilon)
            && self.0.end.approx_eq(&other.0.end, epsilon))
            || (self.0.start.approx_eq(&other.0.end, epsilon)
                && self.0.end.approx_eq(&other.0.start, epsilon))
    }
}

impl Hash for Line2DFloat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let precision_inverse = 1000.0; // Inverse of epsilon used in PartialEq
        let mut coords = [
            (self.0.start.x * precision_inverse).round() as i64,
            (self.0.start.y * precision_inverse).round() as i64,
            (self.0.end.x * precision_inverse).round() as i64,
            (self.0.end.y * precision_inverse).round() as i64,
        ];
        coords.sort(); // Ensure direction-independence
        for coord in &coords {
            coord.hash(state);
        }
    }
}

impl From<Line2DFloat> for LineString2D<f64> {
    fn from(line: Line2DFloat) -> Self {
        LineString2D::new(vec![line.0.start, line.0.end])
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Line3DFloat(pub Line3D<f64>);

impl Eq for Line3DFloat {}

impl PartialEq for Line3DFloat {
    fn eq(&self, other: &Self) -> bool {
        let epsilon = 0.001;
        (self.0.start.approx_eq(&other.0.start, epsilon)
            && self.0.end.approx_eq(&other.0.end, epsilon))
            || (self.0.start.approx_eq(&other.0.end, epsilon)
                && self.0.end.approx_eq(&other.0.start, epsilon))
    }
}

impl Hash for Line3DFloat {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let precision_inverse = 1000.0; // Inverse of epsilon used in PartialEq
        let mut coords = [
            (self.0.start.x * precision_inverse).round() as i64,
            (self.0.start.y * precision_inverse).round() as i64,
            (self.0.start.z * precision_inverse).round() as i64,
            (self.0.end.x * precision_inverse).round() as i64,
            (self.0.end.y * precision_inverse).round() as i64,
            (self.0.end.z * precision_inverse).round() as i64,
        ];
        coords.sort(); // Ensure direction-independence
        for coord in &coords {
            coord.hash(state);
        }
    }
}

impl From<Line3DFloat> for LineString3D<f64> {
    fn from(line: Line3DFloat) -> Self {
        LineString3D::new(vec![line.0.start, line.0.end])
    }
}

impl<T: CoordNum> From<GeoLine<T>> for Line2D<T> {
    fn from(line: GeoLine<T>) -> Self {
        Line2D::new(line.start, line.end)
    }
}

impl<T: CoordNum> From<Line2D<T>> for GeoLine<T> {
    fn from(line: Line2D<T>) -> Self {
        GeoLine::new(line.start, line.end)
    }
}

impl Line3D<f64> {
    pub fn transform_inplace(&mut self, jgd2wgs: &Jgd2011ToWgs84) {
        self.start.transform_inplace(jgd2wgs);
        self.end.transform_inplace(jgd2wgs);
    }

    pub fn transform_offset(&mut self, x: f64, y: f64, z: f64) {
        self.start.transform_offset(x, y, z);
        self.end.transform_offset(x, y, z);
    }
}
