use nalgebra::{Point2 as NaPoint2, Point3 as NaPoint3};
use num_traits::Zero;
use serde::{Deserialize, Serialize};
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};

use nusamai_geometry::{LineString2 as NLineString2, LineString3 as NLineString3};

use crate::utils::line_string_bounding_rect;

use super::conversion::geojson::{
    create_geo_line_string, create_line_string_type, mismatch_geom_err,
};
use super::coordinate::{self, Coordinate};
use super::coordnum::{CoordFloat, CoordNum};
use super::line::Line;
use super::traits::Elevation;
use super::triangle::Triangle;
use super::{no_value::NoValue, point::Point};

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

impl<'a, T: CoordNum, Z: CoordNum> ExactSizeIterator for PointsIter<'a, T, Z> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, T: CoordNum, Z: CoordNum> DoubleEndedIterator for PointsIter<'a, T, Z> {
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

impl<'a, T: CoordNum, Z: CoordNum> ExactSizeIterator for CoordinatesIter<'a, T, Z> {
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a, T: CoordNum, Z: CoordNum> DoubleEndedIterator for CoordinatesIter<'a, T, Z> {
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

impl<'a> From<NLineString2<'a>> for LineString<f64, NoValue> {
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

impl<'a> From<NLineString3<'a>> for LineString<f64> {
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

impl<T: CoordFloat, Z: CoordFloat> From<LineString<T, Z>> for geojson::Value {
    fn from(line_string: LineString<T, Z>) -> Self {
        let coords = create_line_string_type(&line_string);
        geojson::Value::LineString(coords)
    }
}

impl<T, Z> TryFrom<geojson::Value> for LineString<T, Z>
where
    T: CoordFloat,
    Z: CoordFloat,
{
    type Error = crate::error::Error;

    fn try_from(value: geojson::Value) -> crate::error::Result<Self> {
        match value {
            geojson::Value::LineString(multi_point_type) => {
                Ok(create_geo_line_string(&multi_point_type))
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

impl<T, Z> rstar::RTreeObject for LineString<T, Z>
where
    T: num_traits::Float + rstar::RTreeNum + CoordNum,
    Z: num_traits::Float + rstar::RTreeNum + CoordNum,
{
    type Envelope = rstar::AABB<Point<T, Z>>;

    fn envelope(&self) -> Self::Envelope {
        use num_traits::Bounded;
        let bounding_rect = line_string_bounding_rect(self);
        match bounding_rect {
            None => rstar::AABB::from_corners(
                Point::new_(
                    Bounded::min_value(),
                    Bounded::min_value(),
                    Bounded::min_value(),
                ),
                Point::new_(
                    Bounded::max_value(),
                    Bounded::max_value(),
                    Bounded::max_value(),
                ),
            ),
            Some(b) => rstar::AABB::from_corners(
                Point::new_(b.min().x, b.min().y, b.min().z),
                Point::new_(b.max().x, b.max().y, b.max().z),
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
