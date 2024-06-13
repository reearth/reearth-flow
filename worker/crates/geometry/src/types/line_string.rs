use nusamai_projection::etmerc::ExtendedTransverseMercatorProjection;
use serde::{Deserialize, Serialize};
use std::iter::FromIterator;
use std::ops::{Index, IndexMut};

use nusamai_geometry::{LineString2 as NLineString2, LineString3 as NLineString3};

use crate::error::Error;

use super::coordinate::{self, Coordinate};
use super::coordnum::CoordNum;
use super::line::Line;
use super::triangle::Triangle;
use super::{no_value::NoValue, point::Point};

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug, Hash)]
pub struct LineString<T: CoordNum = f64, Z: CoordNum = f64>(pub Vec<Coordinate<T, Z>>);

pub type LineString2D<T> = LineString<T, NoValue>;
pub type LineString3D<T> = LineString<T, T>;

#[derive(Debug)]
pub struct PointsIter<'a, T, Z>(::std::slice::Iter<'a, Coordinate<T, Z>>)
where
    T: CoordNum + 'a,
    Z: CoordNum + 'a;

pub type PointsIter2D<'a, T> = PointsIter<'a, T, NoValue>;
pub type PointsIter3D<'a, T> = PointsIter<'a, T, T>;

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

impl LineString3D<f64> {
    pub fn projection(
        &mut self,
        projection: &ExtendedTransverseMercatorProjection,
    ) -> Result<(), Error> {
        for coord in &mut self.0 {
            coord.projection(projection)?;
        }
        Ok(())
    }
}

impl<T> approx::RelativeEq for LineString<T, T>
where
    T: approx::AbsDiffEq<Epsilon = T> + CoordNum + approx::RelativeEq,
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

impl<T: approx::AbsDiffEq<Epsilon = T> + CoordNum> approx::AbsDiffEq for LineString<T, T> {
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
