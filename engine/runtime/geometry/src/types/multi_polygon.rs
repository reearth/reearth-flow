use std::iter::FromIterator;
use std::ops::Range;

use approx::{AbsDiffEq, RelativeEq};
use flatgeom::{MultiPolygon2 as NMultiPolygon2, MultiPolygon3 as NMultiPolygon3};
use geo_types::{MultiPolygon as GeoMultiPolygon, Polygon as GeoPolygon};
use nalgebra::{Point2 as NaPoint2, Point3 as NaPoint3};
use num_traits::Zero;
use nusamai_projection::vshift::Jgd2011ToWgs84;
use serde::{Deserialize, Serialize};

use crate::types::coordinate::{Coordinate, Coordinate2D};

use super::conversion::geojson::{
    create_geo_multi_polygon_2d, create_geo_multi_polygon_3d, create_multi_polygon_type,
    mismatch_geom_err,
};
use super::coordnum::{CoordFloat, CoordNum};
use super::line_string::LineString;
use super::no_value::NoValue;
use super::polygon::{Polygon, Polygon2D};
use super::rect::Rect;
use super::traits::Elevation;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug, Hash)]
pub struct MultiPolygon<T: CoordNum = f64, Z: CoordNum = f64>(pub Vec<Polygon<T, Z>>);

pub type MultiPolygon2D<T> = MultiPolygon<T, NoValue>;
pub type MultiPolygon3D<T> = MultiPolygon<T, T>;

impl From<Vec<Polygon<f64, NoValue>>> for MultiPolygon<f64, NoValue> {
    fn from(x: Vec<Polygon<f64, NoValue>>) -> Self {
        Self(x)
    }
}

impl<T: CoordNum, Z: CoordNum, IP: Into<Polygon<T, Z>>> From<IP> for MultiPolygon<T, Z> {
    fn from(x: IP) -> Self {
        Self(vec![x.into()])
    }
}

impl<T: CoordNum, Z: CoordNum, IP: Into<Polygon<T, Z>>> FromIterator<IP> for MultiPolygon<T, Z> {
    fn from_iter<I: IntoIterator<Item = IP>>(iter: I) -> Self {
        Self(iter.into_iter().map(|p| p.into()).collect())
    }
}

impl<T: CoordNum, Z: CoordNum> IntoIterator for MultiPolygon<T, Z> {
    type Item = Polygon<T, Z>;
    type IntoIter = ::std::vec::IntoIter<Polygon<T, Z>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: CoordNum, Z: CoordNum> IntoIterator for &'a MultiPolygon<T, Z> {
    type Item = &'a Polygon<T, Z>;
    type IntoIter = ::std::slice::Iter<'a, Polygon<T, Z>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, T: CoordNum, Z: CoordNum> IntoIterator for &'a mut MultiPolygon<T, Z> {
    type Item = &'a mut Polygon<T, Z>;
    type IntoIter = ::std::slice::IterMut<'a, Polygon<T, Z>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<T: CoordNum, Z: CoordNum> MultiPolygon<T, Z> {
    pub fn new(value: Vec<Polygon<T, Z>>) -> Self {
        Self(value)
    }

    pub fn push(&mut self, value: Polygon<T, Z>) {
        self.0.push(value);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Polygon<T, Z>> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Polygon<T, Z>> {
        self.0.iter_mut()
    }

    pub fn add_exterior(&mut self, exterior: LineString<T, Z>) {
        self.0.push(Polygon::new(exterior, Vec::new()));
    }

    pub fn add_interior(&mut self, iter: LineString<T, Z>) {
        self.0
            .last_mut()
            .unwrap()
            .interiors_mut(|interiors| interiors.push(iter));
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn range(&self, range: Range<usize>) -> Vec<Polygon<T, Z>> {
        self.0[range].to_vec()
    }

    pub fn bounding_box(&self) -> Option<Rect<T, Z>> {
        let rects = self.0.iter().map(|p| p.bounding_box());
        let mut rects = rects.flatten();
        let first = rects.next()?;

        Some(rects.fold(first, |acc, r| acc.merge(r)))
    }
}

impl<T: CoordNum, Z: CoordNum> Default for MultiPolygon<T, Z> {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl<'a> From<NMultiPolygon2<'a>> for MultiPolygon2D<f64> {
    #[inline]
    fn from(mpoly: NMultiPolygon2<'a>) -> Self {
        mpoly.iter().map(Polygon2D::from).collect()
    }
}

impl<'a> From<NMultiPolygon3<'a>> for MultiPolygon<f64> {
    #[inline]
    fn from(mpoly: NMultiPolygon3<'a>) -> Self {
        mpoly.iter().map(Polygon::from).collect()
    }
}

impl From<MultiPolygon3D<f64>> for MultiPolygon2D<f64> {
    #[inline]
    fn from(mpoly: MultiPolygon3D<f64>) -> Self {
        MultiPolygon2D::new(mpoly.0.into_iter().map(Polygon2D::from).collect())
    }
}

impl<T: CoordFloat, Z: CoordFloat> From<MultiPolygon<T, Z>> for geojson::Value {
    fn from(multi_polygon: MultiPolygon<T, Z>) -> Self {
        let coords = create_multi_polygon_type(&multi_polygon);
        geojson::Value::MultiPolygon(coords)
    }
}

impl TryFrom<geojson::Value> for MultiPolygon2D<f64> {
    type Error = crate::error::Error;

    fn try_from(value: geojson::Value) -> crate::error::Result<Self> {
        match value {
            geojson::Value::MultiPolygon(multi_polygon_type) => {
                Ok(create_geo_multi_polygon_2d(&multi_polygon_type))
            }
            other => Err(mismatch_geom_err("MultiPolygon", &other)),
        }
    }
}

impl TryFrom<geojson::Value> for MultiPolygon3D<f64> {
    type Error = crate::error::Error;

    fn try_from(value: geojson::Value) -> crate::error::Result<Self> {
        match value {
            geojson::Value::MultiPolygon(multi_polygon_type) => {
                Ok(create_geo_multi_polygon_3d(&multi_polygon_type))
            }
            other => Err(mismatch_geom_err("MultiPolygon", &other)),
        }
    }
}

#[allow(dead_code)]
pub struct Iter<'a, T: CoordNum> {
    mpoly: &'a MultiPolygon<T>,
    pos: usize,
    end: usize,
}

impl<T: CoordNum> Iterator for Iter<'_, T> {
    type Item = Polygon<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.end {
            let poly = self.mpoly.0.get(self.pos);
            self.pos += 1;
            poly.cloned()
        } else {
            None
        }
    }
}

impl<T, Z> RelativeEq for MultiPolygon<T, Z>
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

impl<T, Z> AbsDiffEq for MultiPolygon<T, Z>
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

impl From<MultiPolygon2D<f64>> for Vec<NaPoint2<f64>> {
    #[inline]
    fn from(p: MultiPolygon2D<f64>) -> Vec<NaPoint2<f64>> {
        let result =
            p.0.into_iter()
                .map(|p| p.rings())
                .map(|c| {
                    let result = c
                        .into_iter()
                        .map(|c| c.into())
                        .collect::<Vec<Vec<NaPoint2<f64>>>>();
                    result.into_iter().flatten().collect()
                })
                .collect::<Vec<Vec<NaPoint2<f64>>>>();
        result.into_iter().flatten().collect()
    }
}

impl From<MultiPolygon3D<f64>> for Vec<NaPoint3<f64>> {
    #[inline]
    fn from(p: MultiPolygon3D<f64>) -> Vec<NaPoint3<f64>> {
        let result =
            p.0.into_iter()
                .map(|p| p.rings())
                .map(|c| {
                    let result = c
                        .into_iter()
                        .map(|c| c.into())
                        .collect::<Vec<Vec<NaPoint3<f64>>>>();
                    result.into_iter().flatten().collect()
                })
                .collect::<Vec<Vec<NaPoint3<f64>>>>();
        result.into_iter().flatten().collect()
    }
}

impl<T, Z> Elevation for MultiPolygon<T, Z>
where
    T: CoordNum + Zero,
    Z: CoordNum + Zero,
{
    #[inline]
    fn is_elevation_zero(&self) -> bool {
        self.0.iter().all(|p| p.is_elevation_zero())
    }
}

impl<T: CoordNum> From<MultiPolygon2D<T>> for GeoMultiPolygon<T> {
    fn from(mpolygon: MultiPolygon2D<T>) -> Self {
        GeoMultiPolygon(
            mpolygon
                .0
                .into_iter()
                .map(GeoPolygon::from)
                .collect::<Vec<_>>(),
        )
    }
}

impl<T: CoordNum> From<GeoMultiPolygon<T>> for MultiPolygon2D<T> {
    fn from(mpolygon: GeoMultiPolygon<T>) -> Self {
        let polygons = mpolygon
            .0
            .into_iter()
            .map(Polygon2D::from)
            .collect::<Vec<_>>();
        MultiPolygon2D::new(polygons)
    }
}

impl MultiPolygon3D<f64> {
    pub fn transform_inplace(&mut self, jgd2wgs: &Jgd2011ToWgs84) {
        for poly in self.0.iter_mut() {
            poly.transform_inplace(jgd2wgs);
        }
    }

    pub fn transform_offset(&mut self, x: f64, y: f64, z: f64) {
        for poly in self.0.iter_mut() {
            poly.transform_offset(x, y, z);
        }
    }
}

impl<T: CoordFloat> MultiPolygon2D<T> {
    pub fn denormalize_vertices_2d(&mut self, avg: Coordinate2D<T>, norm_avg: Coordinate2D<T>) {
        for polygon in self.0.iter_mut() {
            polygon.denormalize_vertices_2d(avg, norm_avg);
        }
    }
}

impl<T: CoordFloat + From<Z>, Z: CoordFloat> MultiPolygon<T, Z> {
    pub fn get_vertices(&self) -> Vec<&Coordinate<T, Z>> {
        let mut vertices = Vec::new();
        for polygon in self.0.iter() {
            for coord in polygon.get_vertices() {
                vertices.push(coord);
            }
        }
        vertices
    }

    pub fn get_vertices_mut(&mut self) -> Vec<&mut Coordinate<T, Z>> {
        let mut vertices = Vec::new();
        for polygon in self.0.iter_mut() {
            for coord in polygon.get_vertices_mut() {
                vertices.push(coord);
            }
        }
        vertices
    }
}
