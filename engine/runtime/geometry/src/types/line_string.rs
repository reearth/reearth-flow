use nalgebra::{Point2 as NaPoint2, Point3 as NaPoint3};
use num_traits::Bounded;
use num_traits::Zero;
use nusamai_projection::vshift::Jgd2011ToWgs84;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};

use flatgeom::{LineString2 as NLineString2, LineString3 as NLineString3};
use geo_types::LineString as GeoLineString;

use crate::algorithm::utils::denormalize_vertices_2d;
use crate::types::coordinate::Coordinate2D;
use crate::types::coordinate::Coordinate3D;
use crate::types::face::Face;
use crate::utils::line_string_bounding_rect;

use super::conversion::geojson::create_geo_line_string_2d;
use super::conversion::geojson::create_geo_line_string_3d;
use super::conversion::geojson::{create_line_string_type, mismatch_geom_err};
use super::coordinate::{self, Coordinate};
use super::coordnum::{CoordFloat, CoordNum};
use super::line::Line;
use super::point::Point2D;
use super::point::Point3D;
use super::traits::Elevation;
use super::triangle::Triangle;
use super::{no_value::NoValue, point::Point};

use crate::point;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug, Hash)]
pub struct LineString<T: CoordNum = f64, Z: CoordNum = f64>(pub Vec<Coordinate<T, Z>>);

pub type LineString2D<T> = LineString<T, NoValue>;
pub type LineString3D<T> = LineString<T, T>;

impl From<LineString<f64, f64>> for LineString<f64, NoValue> {
    #[inline]
    fn from(coords: LineString<f64, f64>) -> Self {
        let new_coords = coords
            .0
            .into_iter()
            .map(|c| c.into())
            .collect::<Vec<Coordinate<f64, NoValue>>>();
        LineString(new_coords)
    }
}

impl<T: CoordNum, Z: CoordNum> From<Face<T, Z>> for LineString<T, Z> {
    #[inline]
    fn from(face: Face<T, Z>) -> Self {
        LineString(face.0)
    }
}

impl<T: CoordNum, Z: CoordNum> LineString<T, Z> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Coordinate<T, Z>> {
        self.0.iter()
    }
}

impl LineString3D<f64> {
    pub fn transform_inplace(&mut self, jgd2wgs: &Jgd2011ToWgs84) {
        for coord in &mut self.0 {
            coord.transform_inplace(jgd2wgs);
        }
    }

    pub fn transform_offset(&mut self, x: f64, y: f64, z: f64) {
        for coord in &mut self.0 {
            coord.transform_offset(x, y, z);
        }
    }

    /// Calculates the exterior angle sum of the LineString, assuming that the line is closed and that the line is planar.
    /// The sign of the angle sum is determined by the normal vector `n`. If `n` is not provided, it is estimated from the cross products of the first segments.
    pub fn exterior_angle_sum(&self, n: Option<Coordinate3D<f64>>) -> f64 {
        assert!(
            self.0.len() >= 3,
            "LineString must have at least 3 vertices"
        );
        assert!(self.0.first() == self.0.last(), "LineString must be closed");
        let n = n.unwrap_or(
            self.0
                .windows(3)
                .map(|w| {
                    let a = w[0] - w[1];
                    let b = w[2] - w[1];
                    a.cross(&b)
                })
                .max_by(|a, b| a.norm().partial_cmp(&b.norm()).unwrap())
                .unwrap()
                .normalize(),
        );
        let num_vertices = self.0.len() - 1;
        (0..num_vertices)
            .map(|i| {
                let prev = (i + num_vertices - 1) % num_vertices;
                let next = (i + 1) % num_vertices;
                let a = self.0[prev] - self.0[i];
                let b = self.0[next] - self.0[i];
                let cross = a.cross(&b);
                let angle = a.angle(&b);
                if cross.dot(&n) > 0.0 {
                    PI - angle
                } else {
                    angle - PI
                }
            })
            .sum::<f64>()
    }
}

#[derive(Debug)]
pub struct PointsIter<'a, T, Z>(::std::slice::Iter<'a, Coordinate<T, Z>>)
where
    T: CoordNum + 'a,
    Z: CoordNum + 'a;

pub type PointsIter2D<'a, T> = PointsIter<'a, T, NoValue>;
pub type PointsIter3D<'a, T> = PointsIter<'a, T, T>;

impl From<LineString2D<f64>> for Vec<NaPoint2<f64>> {
    #[inline]
    fn from(p: LineString2D<f64>) -> Vec<NaPoint2<f64>> {
        p.0.into_iter().map(|c| c.into()).collect()
    }
}

impl From<LineString3D<f64>> for Vec<NaPoint3<f64>> {
    #[inline]
    fn from(p: LineString3D<f64>) -> Vec<NaPoint3<f64>> {
        p.0.into_iter().map(|c| c.into()).collect()
    }
}

impl<'a, T, Z> Iterator for PointsIter<'a, T, Z>
where
    T: CoordNum + 'a,
    Z: CoordNum + 'a,
{
    type Item = Point<T, Z>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|c| Point::from(*c))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<T: CoordNum, Z: CoordNum> ExactSizeIterator for PointsIter<'_, T, Z> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: CoordNum, Z: CoordNum> DoubleEndedIterator for PointsIter<'_, T, Z> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|c| Point::from(*c))
    }
}

pub struct CoordinatesIter<'a, T: CoordNum + 'a, Z: CoordNum + 'a>(
    ::std::slice::Iter<'a, Coordinate<T, Z>>,
);

impl<'a, T: CoordNum, Z: CoordNum> Iterator for CoordinatesIter<'a, T, Z> {
    type Item = &'a Coordinate<T, Z>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<T: CoordNum, Z: CoordNum> ExactSizeIterator for CoordinatesIter<'_, T, Z> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: CoordNum, Z: CoordNum> DoubleEndedIterator for CoordinatesIter<'_, T, Z> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl<T: CoordNum, Z: CoordNum> LineString<T, Z> {
    /// Instantiate Self from the raw content value
    pub fn new(value: Vec<Coordinate<T, Z>>) -> Self {
        Self(value)
    }

    pub fn points(&self) -> PointsIter<T, Z> {
        PointsIter(self.0.iter())
    }

    pub fn coords(&self) -> impl Iterator<Item = &Coordinate<T, Z>> {
        self.0.iter()
    }

    pub fn coords_mut(&mut self) -> impl Iterator<Item = &mut Coordinate<T, Z>> {
        self.0.iter_mut()
    }

    pub fn into_points(self) -> Vec<Point<T, Z>> {
        self.0.into_iter().map(Point::from).collect()
    }

    pub fn into_inner(self) -> Vec<Coordinate<T, Z>> {
        self.0
    }

    pub fn lines(&'_ self) -> impl ExactSizeIterator<Item = Line<T, Z>> + '_ {
        self.0.windows(2).map(|w| {
            // slice::windows(N) is guaranteed to yield a slice with exactly N elements
            unsafe { Line::new_(*w.get_unchecked(0), *w.get_unchecked(1)) }
        })
    }

    pub fn triangles(&'_ self) -> impl ExactSizeIterator<Item = Triangle<T, Z>> + '_ {
        self.0.windows(3).map(|w| {
            // slice::windows(N) is guaranteed to yield a slice with exactly N elements
            unsafe {
                Triangle::new(
                    *w.get_unchecked(0),
                    *w.get_unchecked(1),
                    *w.get_unchecked(2),
                )
            }
        })
    }

    pub fn translate_z(&mut self, height: Z) {
        for coordinate in &mut self.0 {
            coordinate.z = coordinate.z.add(height);
        }
    }

    pub fn close(&mut self) {
        if !self.is_closed() {
            self.0.push(self.0[0]);
        }
    }

    pub fn is_closed(&self) -> bool {
        self.0.first() == self.0.last()
    }

    /// Reverses the coordinates in the LineString.
    pub fn reverse_inplace(&mut self) {
        let len = self.0.len();
        if len > 0 {
            let data = self.0.as_mut_slice();
            for i in 0..data.len() / 2 {
                data.swap(i, len - (i + 1));
            }
        }
    }

    pub fn ring_area(&self) -> f64 {
        self.signed_ring_area().abs()
    }

    pub fn signed_ring_area(&self) -> f64 {
        if self.is_empty() {
            return 0.0;
        }
        let mut area = 0.0;
        let mut ring_iter = self.iter();
        let mut prev = ring_iter.next().unwrap().x_y();
        // shoelace formula
        for coord in ring_iter {
            let xy = coord.x_y();
            area += (prev.0.to_f64().unwrap() * xy.1.to_f64().unwrap())
                - (prev.1.to_f64().unwrap() * xy.0.to_f64().unwrap());
            prev = xy;
        }
        area / 2.0
    }

    /// Splits the LineString at the given index.
    pub fn split_at(self, index: usize) -> (Self, Self) {
        assert!(index < self.len());
        let first = LineString(self.0[..=index].to_vec());
        let second = LineString(self.0[index..].to_vec());
        (first, second)
    }
}

impl<T: CoordNum, Z: CoordNum, IC: Into<Coordinate<T, Z>>> From<Vec<IC>> for LineString<T, Z> {
    fn from(v: Vec<IC>) -> Self {
        Self(v.into_iter().map(|c| c.into()).collect())
    }
}

impl<T: CoordNum, Z: CoordNum> From<Line<T, Z>> for LineString<T, Z> {
    fn from(line: Line<T, Z>) -> Self {
        Self(vec![line.start, line.end])
    }
}

impl<T: CoordNum, Z: CoordNum, IC: Into<Coordinate<T, Z>>> FromIterator<IC> for LineString<T, Z> {
    fn from_iter<I: IntoIterator<Item = IC>>(iter: I) -> Self {
        Self(iter.into_iter().map(|c| c.into()).collect())
    }
}

impl<T: CoordNum, Z: CoordNum> IntoIterator for LineString<T, Z> {
    type Item = Coordinate<T, Z>;
    type IntoIter = ::std::vec::IntoIter<Coordinate<T, Z>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: CoordNum, Z: CoordNum> IntoIterator for &'a LineString<T, Z> {
    type Item = &'a Coordinate<T, Z>;
    type IntoIter = CoordinatesIter<'a, T, Z>;

    fn into_iter(self) -> Self::IntoIter {
        CoordinatesIter(self.0.iter())
    }
}

impl<'a, T: CoordNum, Z: CoordNum> IntoIterator for &'a mut LineString<T, Z> {
    type Item = &'a mut Coordinate<T, Z>;
    type IntoIter = ::std::slice::IterMut<'a, Coordinate<T, Z>>;

    fn into_iter(self) -> ::std::slice::IterMut<'a, Coordinate<T, Z>> {
        self.0.iter_mut()
    }
}

impl<T: CoordNum, Z: CoordNum> Index<usize> for LineString<T, Z> {
    type Output = Coordinate<T, Z>;

    fn index(&self, index: usize) -> &Coordinate<T, Z> {
        self.0.index(index)
    }
}

impl<T: CoordNum, Z: CoordNum> IndexMut<usize> for LineString<T, Z> {
    fn index_mut(&mut self, index: usize) -> &mut Coordinate<T, Z> {
        self.0.index_mut(index)
    }
}

impl<'a> From<NLineString2<'a>> for LineString2D<f64> {
    #[inline]
    fn from(coords: NLineString2<'a>) -> Self {
        LineString2D::new(
            coords
                .iter_closed()
                .map(|a| coordinate::Coordinate2D::new_(a[0], a[1]))
                .collect::<Vec<_>>(),
        )
    }
}

impl From<LineString3D<f64>> for NLineString2<'_> {
    #[inline]
    fn from(coords: LineString3D<f64>) -> Self {
        let mut line_string = NLineString2::new();
        for coord in coords.iter() {
            line_string.push([coord.x, coord.y]);
        }
        line_string
    }
}

impl<'a> From<NLineString3<'a>> for LineString3D<f64> {
    #[inline]
    fn from(coords: NLineString3<'a>) -> Self {
        LineString3D::new(
            coords
                .iter_closed()
                .map(|a| coordinate::Coordinate3D::new__(a[0], a[1], a[2]))
                .collect::<Vec<_>>(),
        )
    }
}

impl From<LineString2D<f64>> for NLineString2<'_> {
    #[inline]
    fn from(coords: LineString2D<f64>) -> Self {
        let mut line_string = NLineString2::new();
        for coord in coords.iter() {
            line_string.push([coord.x, coord.y]);
        }
        line_string
    }
}

impl From<LineString3D<f64>> for NLineString3<'_> {
    #[inline]
    fn from(coords: LineString3D<f64>) -> Self {
        let mut line_string = NLineString3::new();
        for coord in coords.iter() {
            line_string.push([coord.x, coord.y, coord.z]);
        }
        line_string
    }
}

pub fn from_line_string_5d(
    line_strings: flatgeom::LineString<[f64; 5]>,
) -> (LineString3D<f64>, LineString2D<f64>) {
    let targets = line_strings
        .iter_closed()
        .map(|line| {
            (
                coordinate::Coordinate3D::new__(line[0], line[1], line[2]),
                coordinate::Coordinate2D::new_(line[3], line[4]),
            )
        })
        .collect::<Vec<_>>();
    let line_string_3d = LineString3D::new(targets.iter().map(|(a, _)| *a).collect());
    let line_string_2d = LineString2D::new(targets.iter().map(|(_, b)| *b).collect());
    (line_string_3d, line_string_2d)
}

impl<T: CoordFloat, Z: CoordFloat> From<LineString<T, Z>> for geojson::Value {
    fn from(line_string: LineString<T, Z>) -> Self {
        let coords = create_line_string_type(&line_string);
        geojson::Value::LineString(coords)
    }
}

impl TryFrom<geojson::Value> for LineString2D<f64> {
    type Error = crate::error::Error;

    fn try_from(value: geojson::Value) -> crate::error::Result<Self> {
        match value {
            geojson::Value::LineString(multi_point_type) => {
                Ok(create_geo_line_string_2d(&multi_point_type))
            }
            other => Err(mismatch_geom_err("LineString", &other)),
        }
    }
}

impl TryFrom<geojson::Value> for LineString3D<f64> {
    type Error = crate::error::Error;

    fn try_from(value: geojson::Value) -> crate::error::Result<Self> {
        match value {
            geojson::Value::LineString(multi_point_type) => {
                Ok(create_geo_line_string_3d(&multi_point_type))
            }
            other => Err(mismatch_geom_err("LineString", &other)),
        }
    }
}

impl<T, Z> approx::RelativeEq for LineString<T, Z>
where
    T: approx::AbsDiffEq<Epsilon = T> + CoordNum + approx::RelativeEq,
    Z: approx::AbsDiffEq<Epsilon = Z> + CoordNum + approx::RelativeEq,
{
    #[inline]
    fn default_max_relative() -> Self::Epsilon {
        T::default_max_relative()
    }

    fn relative_eq(
        &self,
        other: &Self,
        epsilon: Self::Epsilon,
        max_relative: Self::Epsilon,
    ) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        let points_zipper = self.points().zip(other.points());
        for (lhs, rhs) in points_zipper {
            if lhs.relative_ne(&rhs, epsilon, max_relative) {
                return false;
            }
        }
        true
    }
}

impl<
        T: approx::AbsDiffEq<Epsilon = T> + CoordNum,
        Z: approx::AbsDiffEq<Epsilon = Z> + CoordNum,
    > approx::AbsDiffEq for LineString<T, Z>
{
    type Epsilon = T;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        let mut points_zipper = self.points().zip(other.points());
        points_zipper.all(|(lhs, rhs)| lhs.abs_diff_eq(&rhs, epsilon))
    }
}

impl rstar::RTreeObject for LineString2D<f64> {
    type Envelope = rstar::AABB<Point2D<f64>>;

    fn envelope(&self) -> Self::Envelope {
        let bounding_rect = line_string_bounding_rect(self);
        match bounding_rect {
            None => rstar::AABB::from_corners(
                point!(
                    x: Bounded::min_value(),
                    y: Bounded::min_value(),
                ),
                point!(x: Bounded::max_value(), y: Bounded::max_value()),
            ),
            Some(b) => rstar::AABB::from_corners(
                point!(x: b.min().x, y: b.min().y),
                point!(x: b.max().x, y: b.max().y),
            ),
        }
    }
}

impl rstar::RTreeObject for LineString3D<f64> {
    type Envelope = rstar::AABB<Point3D<f64>>;

    fn envelope(&self) -> Self::Envelope {
        let bounding_rect = line_string_bounding_rect(self);
        match bounding_rect {
            None => rstar::AABB::from_corners(
                point!(
                    x: Bounded::min_value(),
                    y: Bounded::min_value(),
                    z: Bounded::min_value(),
                ),
                point!(x: Bounded::max_value(), y: Bounded::max_value(), z: Bounded::max_value()),
            ),
            Some(b) => rstar::AABB::from_corners(
                point!(x: b.min().x, y: b.min().y, z: b.min().z),
                point!(x: b.max().x, y: b.max().y, z: b.max().z),
            ),
        }
    }
}

impl<T, Z> Elevation for LineString<T, Z>
where
    T: CoordNum + Zero,
    Z: CoordNum + Zero,
{
    #[inline]
    fn is_elevation_zero(&self) -> bool {
        self.0.iter().all(|c| c.is_elevation_zero())
    }
}

impl<Z: CoordFloat> LineString<f64, Z> {
    pub fn approx_eq(&self, other: &LineString<f64, Z>, epsilon: f64) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        let points_zipper = self.points().zip(other.points());
        for (lhs, rhs) in points_zipper {
            if !lhs.approx_eq(&rhs, epsilon) {
                return false;
            }
        }
        true
    }
}

impl<T: CoordNum> From<LineString2D<T>> for GeoLineString<T> {
    fn from(line_string: LineString2D<T>) -> Self {
        GeoLineString(line_string.0.into_iter().map(|c| c.x_y().into()).collect())
    }
}

impl<T: CoordNum> From<GeoLineString<T>> for LineString2D<T> {
    fn from(line_string: GeoLineString<T>) -> Self {
        LineString2D::new(line_string.0.into_iter().map(Into::into).collect())
    }
}

impl<T: CoordFloat> LineString2D<T> {
    pub fn denormalize_vertices_2d(&mut self, avg: Coordinate2D<T>, norm: Coordinate2D<T>) {
        denormalize_vertices_2d(&mut self.0, avg, norm);
    }
}

impl<T: CoordFloat + From<Z>, Z: CoordFloat> LineString<T, Z> {
    pub fn get_vertices(&self) -> Vec<&Coordinate<T, Z>> {
        self.0.iter().collect()
    }

    pub fn get_vertices_mut(&mut self) -> Vec<&mut Coordinate<T, Z>> {
        self.0.iter_mut().collect()
    }
}
