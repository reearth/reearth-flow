use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::{Div, Mul};

use approx::{AbsDiffEq, RelativeEq};
use geo_types::Line as GeoLine;
use num_traits::{NumCast, Zero};
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

impl<T: CoordFloat + From<Z>, Z: CoordFloat + Mul<T, Output = Z>> Line<T, Z> {
    /// Smallest distance between segments `self` and `other`.
    pub fn distance(&self, other: &Self) -> T {
        let (p, q) = self.closest_points(other);
        (p - q).norm()
    }

    pub fn closest_points(&self, other: &Self) -> (Coordinate<T, Z>, Coordinate<T, Z>) {
        #[inline]
        fn clamp<T: PartialOrd>(x: T, lo: T, hi: T) -> T {
            if x < lo {
                lo
            } else if x > hi {
                hi
            } else {
                x
            }
        }

        let epsilon = <T as NumCast>::from(1e-5).unwrap_or_default();

        let u = self.end - self.start;
        let v = other.end - other.start;
        let w0 = self.start - other.start;

        let a = u.dot(&u);
        let b = u.dot(&v);
        let c = v.dot(&v);
        let d = u.dot(&w0);
        let e = v.dot(&w0);
        let dnm = a * c - b * b;

        // Degenerate cases (point vs point/segment)
        if a <= epsilon && c <= epsilon {
            return (self.start, other.start);
        }
        if a <= epsilon {
            let t = clamp(e / c, T::zero(), T::one());
            return (self.start, (other.start + v * t));
        }
        if c <= epsilon {
            let s = clamp(-d / a, T::zero(), T::one());
            return (self.start + u * s, other.start);
        }

        // General case
        let mut s_n = b * e - c * d;
        let mut t_n = a * e - b * d;
        let mut s_d = dnm;
        let mut t_d = dnm;

        // Nearly parallel
        if dnm <= epsilon {
            s_n = T::zero();
            s_d = T::one();
            t_n = e;
            t_d = c;
        }

        // Clamp s to [0,1] with coupling to t
        if s_n < T::zero() {
            s_n = T::zero();
            t_n = e;
            t_d = c;
        } else if s_n > s_d {
            s_n = s_d;
            t_n = e + b;
            t_d = c;
        }

        // Clamp t to [0,1], possibly re-clamping s
        if t_n < T::zero() {
            t_n = T::zero();
            s_n = -d;
            s_d = a;
            if s_n < T::zero() {
                s_n = T::zero();
            } else if s_n > s_d {
                s_n = s_d;
            }
        } else if t_n > t_d {
            t_n = t_d;
            s_n = -d + b;
            s_d = a;
            if s_n < T::zero() {
                s_n = T::zero();
            } else if s_n > s_d {
                s_n = s_d;
            }
        }

        let s = if s_d.abs() > epsilon {
            s_n / s_d
        } else {
            T::zero()
        };
        let t = if t_d.abs() > epsilon {
            t_n / t_d
        } else {
            T::zero()
        };

        let cp = self.start + u * s;
        let cq = other.start + v * t;
        (cp, cq)
    }
}

impl<T, Z> Line<T, Z>
where
    T: CoordFloat + From<Z>,
    Z: CoordFloat + Mul<T, Output = Z> + Div<T, Output = Z>,
{
    pub fn intersection(&self, other: &Self, epsilon: Option<T>) -> Option<Coordinate<T, Z>> {
        let (cp, cq) = self.closest_points(other);
        let d = (cp - cq).norm();
        if d < epsilon.unwrap_or(<T as NumCast>::from(1e-5).unwrap_or_default()) {
            Some((cp + cq) / <T as NumCast>::from(2.0).unwrap_or_default())
        } else {
            None
        }
    }

    pub fn contains(&self, mut p: Coordinate<T, Z>) -> bool {
        let epsilon = <T as NumCast>::from(1e-5).unwrap_or_default();
        let mut line = *self;
        p = p - self.start;
        line.end = line.end - line.start;
        line.start = Coordinate::zero();

        let line_len = line.end.norm();
        let p_norm = p.norm();
        let norm = if line_len < p_norm { p_norm } else { line_len };
        line.end = line.end / norm;
        p = p / norm;

        let dot = line.end.dot(&p);
        let cross_norm = line.end.cross(&p).norm();
        cross_norm < epsilon && dot > -epsilon && p_norm < line_len + epsilon
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_distance() {
        let line1 = Line3D::new_((0.0, 0.0, 0.0), (1.0, 0.0, 0.0));
        let line2 = Line3D::new_((0.0, 1.0, 0.0), (1.0, 1.0, 0.0));
        assert!((line1.distance(&line2) - 1_f64).abs() < 1e-6);

        let line3 = Line3D::new_((2.0, 1.0, 0.0), (2.0, 2.0, 0.0));
        assert!((line1.distance(&line3) - (2_f64).sqrt()).abs() < 1e-6);

        let line4 = Line3D::new_((2.0, 0.0, 0.0), (3.0, 0.0, 0.0));
        assert!((line1.distance(&line4) - 1_f64).abs() < 1e-6);

        let line5 = Line3D::new_((0.0, 0.0, 0.0), (1.0, 1.0, 1.0));
        let line6 = Line3D::new_((1.0, 0.0, 0.0), (0.0, 1.0, 1.0));
        assert!(line5.distance(&line6) < 1e-6);
    }

    #[test]
    fn test_line_contains() {
        let line = Line3D::new_((0.0, 0.0, 0.0), (1.0, 1.0, 1.0));
        let points_contained = [
            Coordinate::new__(0.0, 0.0, 0.0),
            Coordinate::new__(1e-12, 0.0, 1e-12),
            Coordinate::new__(0.5 + 1e-12, 0.5, 0.5),
            Coordinate::new__(1.0, 1.0, 1.0),
        ];
        for p in &points_contained {
            assert!(line.contains(*p), "Point {p:?} should be contained");
        }
        let points_not_contained = [
            Coordinate::new__(1.0, 0.0, 0.0),
            Coordinate::new__(0.5, 0.5, 0.6),
            Coordinate::new__(1.0, 1.1, 1.1),
        ];
        for p in &points_not_contained {
            assert!(!line.contains(*p), "Point {p:?} should not be contained");
        }
    }
}
