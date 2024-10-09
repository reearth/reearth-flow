use std::iter::FromIterator;

use approx::{AbsDiffEq, RelativeEq};
use flatgeom::{MultiPoint2 as NMultiPoint2, MultiPoint3 as NMultiPoint3};
use geo_types::MultiPoint as GeoMultiPoint;
use nalgebra::{Point2 as NaPoint2, Point3 as NaPoint3};
use num_traits::Zero;
use serde::{Deserialize, Serialize};

use super::conversion::geojson::{create_geo_point, create_point_type, mismatch_geom_err};
use super::coordnum::{CoordFloat, CoordNum};
use super::no_value::NoValue;
use super::point::Point;
use super::traits::Elevation;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug, Hash)]
pub struct MultiPoint<T: CoordNum = f64, Z: CoordNum = f64>(pub Vec<Point<T, Z>>);

pub type MultiPoint2D<T> = MultiPoint<T, NoValue>;
pub type MultiPoint3D<T> = MultiPoint<T, T>;

impl<T: CoordNum, Z: CoordNum, IP: Into<Point<T, Z>>> From<IP> for MultiPoint<T, Z> {
    fn from(x: IP) -> Self {
        Self(vec![x.into()])
    }
}

impl<T: CoordNum, Z: CoordNum, IP: Into<Point<T, Z>>> From<Vec<IP>> for MultiPoint<T, Z> {
    fn from(v: Vec<IP>) -> Self {
        Self(v.into_iter().map(|p| p.into()).collect())
    }
}

impl<T: CoordNum, Z: CoordNum, IP: Into<Point<T, Z>>> FromIterator<IP> for MultiPoint<T, Z> {
    fn from_iter<I: IntoIterator<Item = IP>>(iter: I) -> Self {
        Self(iter.into_iter().map(|p| p.into()).collect())
    }
}

/// Iterate over the `Point`s in this `MultiPoint`.
impl<T: CoordNum, Z: CoordNum> IntoIterator for MultiPoint<T, Z> {
    type Item = Point<T, Z>;
    type IntoIter = ::std::vec::IntoIter<Point<T, Z>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: CoordNum, Z: CoordNum> IntoIterator for &'a MultiPoint<T, Z> {
    type Item = &'a Point<T, Z>;
    type IntoIter = ::std::slice::Iter<'a, Point<T, Z>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, T: CoordNum, Z: CoordNum> IntoIterator for &'a mut MultiPoint<T, Z> {
    type Item = &'a mut Point<T, Z>;
    type IntoIter = ::std::slice::IterMut<'a, Point<T, Z>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<T: CoordNum, Z: CoordNum> MultiPoint<T, Z> {
    pub fn new(value: Vec<Point<T, Z>>) -> Self {
        Self(value)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Point<T, Z>> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Point<T, Z>> {
        self.0.iter_mut()
    }
}

impl From<MultiPoint3D<f64>> for MultiPoint2D<f64> {
    #[inline]
    fn from(mp: MultiPoint3D<f64>) -> Self {
        MultiPoint2D::new(mp.0.into_iter().map(|p| p.into()).collect())
    }
}

impl<'a> From<NMultiPoint2<'a>> for MultiPoint2D<f64> {
    #[inline]
    fn from(line_strings: NMultiPoint2<'a>) -> Self {
        MultiPoint2D::new(line_strings.iter().map(|a| a.into()).collect::<Vec<_>>())
    }
}

impl<'a> From<NMultiPoint3<'a>> for MultiPoint3D<f64> {
    #[inline]
    fn from(line_strings: NMultiPoint3<'a>) -> Self {
        MultiPoint3D::new(line_strings.iter().map(|a| a.into()).collect::<Vec<_>>())
    }
}

impl From<MultiPoint2D<f64>> for Vec<NaPoint2<f64>> {
    #[inline]
    fn from(p: MultiPoint2D<f64>) -> Vec<NaPoint2<f64>> {
        p.0.into_iter().map(|c| c.into()).collect()
    }
}

impl From<MultiPoint3D<f64>> for Vec<NaPoint3<f64>> {
    #[inline]
    fn from(p: MultiPoint3D<f64>) -> Vec<NaPoint3<f64>> {
        p.0.into_iter().map(|c| c.into()).collect()
    }
}

impl<T: CoordFloat, Z: CoordFloat> From<MultiPoint<T, Z>> for geojson::Value {
    fn from(multi_point: MultiPoint<T, Z>) -> Self {
        let coords = multi_point
            .0
            .iter()
            .map(|point| create_point_type(point))
            .collect();

        geojson::Value::MultiPoint(coords)
    }
}

impl<T, Z> TryFrom<geojson::Value> for MultiPoint<T, Z>
where
    T: CoordFloat,
    Z: CoordFloat,
{
    type Error = crate::error::Error;

    fn try_from(value: geojson::Value) -> crate::error::Result<Self> {
        match value {
            geojson::Value::MultiPoint(multi_point_type) => Ok(MultiPoint::new(
                multi_point_type
                    .iter()
                    .map(|point_type| create_geo_point(point_type))
                    .collect(),
            )),
            other => Err(mismatch_geom_err("MultiPoint", &other)),
        }
    }
}

impl<T, Z> RelativeEq for MultiPoint<T, Z>
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
        if self.0.len() != other.0.len() {
            return false;
        }

        let mut mp_zipper = self.iter().zip(other.iter());
        mp_zipper.all(|(lhs, rhs)| lhs.relative_eq(rhs, epsilon, max_relative))
    }
}

impl<T, Z> AbsDiffEq for MultiPoint<T, Z>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum,
    Z: AbsDiffEq<Epsilon = Z> + CoordNum,
    T::Epsilon: Copy,
    Z::Epsilon: Copy,
{
    type Epsilon = T;

    #[inline]
    fn default_epsilon() -> Self::Epsilon {
        T::default_epsilon()
    }

    #[inline]
    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        let mut mp_zipper = self.into_iter().zip(other);
        mp_zipper.all(|(lhs, rhs)| lhs.abs_diff_eq(rhs, epsilon))
    }
}

impl<T, Z> Elevation for MultiPoint<T, Z>
where
    T: CoordNum + Zero,
    Z: CoordNum + Zero,
{
    #[inline]
    fn is_elevation_zero(&self) -> bool {
        self.0.iter().all(|p| p.is_elevation_zero())
    }
}

impl<T: CoordNum> From<GeoMultiPoint<T>> for MultiPoint2D<T> {
    fn from(coord: GeoMultiPoint<T>) -> Self {
        MultiPoint::new(coord.0.into_iter().map(|p| p.into()).collect())
    }
}

impl<T: CoordNum> From<MultiPoint2D<T>> for GeoMultiPoint<T> {
    fn from(coord: MultiPoint2D<T>) -> Self {
        GeoMultiPoint(coord.0.into_iter().map(|p| p.into()).collect())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::point;

    #[test]
    fn test_iter() {
        let multi = MultiPoint::new(vec![point![x: 0, y: 0], point![x: 10, y: 10]]);

        let mut first = true;
        for p in &multi {
            if first {
                assert_eq!(p, &point![x: 0, y: 0]);
                first = false;
            } else {
                assert_eq!(p, &point![x: 10, y: 10]);
            }
        }

        // Do it again to prove that `multi` wasn't `moved`.
        first = true;
        for p in &multi {
            if first {
                assert_eq!(p, &point![x: 0, y: 0]);
                first = false;
            } else {
                assert_eq!(p, &point![x: 10, y: 10]);
            }
        }
    }

    #[test]
    fn test_iter_mut() {
        let mut multi = MultiPoint::new(vec![point![x: 0, y: 0], point![x: 10, y: 10]]);

        for point in &mut multi {
            point.0.x += 1;
            point.0.y += 1;
        }

        for point in multi.iter_mut() {
            point.0.x += 1;
            point.0.y += 1;
        }

        let mut first = true;
        for p in &multi {
            if first {
                assert_eq!(p, &point![x: 2, y: 2]);
                first = false;
            } else {
                assert_eq!(p, &point![x: 12, y: 12]);
            }
        }
    }
}
