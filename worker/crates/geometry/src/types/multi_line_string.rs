use std::iter::FromIterator;

use approx::{AbsDiffEq, RelativeEq};
use nalgebra::{Point2 as NaPoint2, Point3 as NaPoint3};
use serde::{Deserialize, Serialize};

use nusamai_geometry::{
    MultiLineString2 as NMultiLineString2, MultiLineString3 as NMultiLineString3,
};

use super::conversion::geojson::{
    create_geo_multi_line_string, create_multi_line_string_type, mismatch_geom_err,
};
use super::coordnum::{CoordFloat, CoordNum};
use super::line::Line;
use super::line_string::LineString;
use super::no_value::NoValue;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug, Hash)]
pub struct MultiLineString<T: CoordNum = f64, Z: CoordNum = f64>(pub Vec<LineString<T, Z>>);

pub type MultiLineString2D<T> = MultiLineString<T, NoValue>;
pub type MultiLineString3D<T> = MultiLineString<T, T>;

impl<T: CoordNum, Z: CoordNum> MultiLineString<T, Z> {
    pub fn new(value: Vec<LineString<T, Z>>) -> Self {
        Self(value)
    }

    pub fn is_closed(&self) -> bool {
        self.iter().all(LineString::is_closed)
    }

    pub fn lines(&'_ self) -> impl ExactSizeIterator<Item = Line<T, Z>> + '_ {
        self.iter()
            .flat_map(|ls| ls.lines())
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl From<MultiLineString2D<f64>> for Vec<NaPoint2<f64>> {
    #[inline]
    fn from(p: MultiLineString2D<f64>) -> Vec<NaPoint2<f64>> {
        let result =
            p.0.into_iter()
                .map(|c| c.into())
                .collect::<Vec<Vec<NaPoint2<f64>>>>();
        result.into_iter().flatten().collect()
    }
}

impl From<MultiLineString3D<f64>> for Vec<NaPoint3<f64>> {
    #[inline]
    fn from(p: MultiLineString3D<f64>) -> Vec<NaPoint3<f64>> {
        let result =
            p.0.into_iter()
                .map(|c| c.into())
                .collect::<Vec<Vec<NaPoint3<f64>>>>();
        result.into_iter().flatten().collect()
    }
}

impl From<MultiLineString3D<f64>> for MultiLineString2D<f64> {
    #[inline]
    fn from(p: MultiLineString3D<f64>) -> MultiLineString2D<f64> {
        MultiLineString2D::new(p.0.into_iter().map(|c| c.into()).collect())
    }
}

impl<T: CoordNum, Z: CoordNum, ILS: Into<LineString<T, Z>>> From<ILS> for MultiLineString<T, Z> {
    fn from(ls: ILS) -> Self {
        Self(vec![ls.into()])
    }
}

impl<T: CoordNum, Z: CoordNum, ILS: Into<LineString<T, Z>>> FromIterator<ILS>
    for MultiLineString<T, Z>
{
    fn from_iter<I: IntoIterator<Item = ILS>>(iter: I) -> Self {
        Self(iter.into_iter().map(|ls| ls.into()).collect())
    }
}

impl<T: CoordNum, Z: CoordNum> IntoIterator for MultiLineString<T, Z> {
    type Item = LineString<T, Z>;
    type IntoIter = ::std::vec::IntoIter<LineString<T, Z>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: CoordNum, Z: CoordNum> IntoIterator for &'a MultiLineString<T, Z> {
    type Item = &'a LineString<T, Z>;
    type IntoIter = ::std::slice::Iter<'a, LineString<T, Z>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a, T: CoordNum, Z: CoordNum> IntoIterator for &'a mut MultiLineString<T, Z> {
    type Item = &'a mut LineString<T, Z>;
    type IntoIter = ::std::slice::IterMut<'a, LineString<T, Z>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl<T: CoordNum, Z: CoordNum> MultiLineString<T, Z> {
    pub fn iter(&self) -> impl Iterator<Item = &LineString<T, Z>> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut LineString<T, Z>> {
        self.0.iter_mut()
    }
}

impl<'a> From<NMultiLineString2<'a>> for MultiLineString2D<f64> {
    #[inline]
    fn from(line_strings: NMultiLineString2<'a>) -> Self {
        MultiLineString2D::new(line_strings.iter().map(|a| a.into()).collect::<Vec<_>>())
    }
}

impl<'a> From<NMultiLineString3<'a>> for MultiLineString3D<f64> {
    #[inline]
    fn from(line_strings: NMultiLineString3<'a>) -> Self {
        MultiLineString3D::new(line_strings.iter().map(|a| a.into()).collect::<Vec<_>>())
    }
}

impl<T: CoordFloat, Z: CoordFloat> From<MultiLineString<T, Z>> for geojson::Value {
    fn from(multi_line_string: MultiLineString<T, Z>) -> Self {
        let coords = create_multi_line_string_type(&multi_line_string);
        geojson::Value::MultiLineString(coords)
    }
}

impl<T, Z> TryFrom<geojson::Value> for MultiLineString<T, Z>
where
    T: CoordFloat,
    Z: CoordFloat,
{
    type Error = crate::error::Error;

    fn try_from(value: geojson::Value) -> crate::error::Result<Self> {
        match value {
            geojson::Value::MultiLineString(multi_line_string_type) => {
                Ok(create_geo_multi_line_string(&multi_line_string_type))
            }
            other => Err(mismatch_geom_err("MultiLineString", &other)),
        }
    }
}

impl<T, Z> RelativeEq for MultiLineString<T, Z>
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

impl<T, Z> AbsDiffEq for MultiLineString<T, Z>
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

#[cfg(test)]
mod test {
    use crate::line_string;

    use super::*;

    #[test]
    fn test_iter() {
        let multi: Vec<LineString<i32, NoValue>> = vec![
            line_string![(x: 0, y: 0), (x: 2, y: 0), (x: 1, y: 2), (x:0, y:0)],
            line_string![(x: 10, y: 10), (x: 12, y: 10), (x: 11, y: 12), (x:10, y:10)],
        ];
        let multi: MultiLineString<i32, NoValue> = MultiLineString::new(multi);

        let mut first = true;
        for p in &multi {
            if first {
                assert_eq!(
                    p,
                    &line_string![(x: 0, y: 0), (x: 2, y: 0), (x: 1, y: 2), (x:0, y:0)]
                );
                first = false;
            } else {
                assert_eq!(
                    p,
                    &line_string![(x: 10, y: 10), (x: 12, y: 10), (x: 11, y: 12), (x:10, y:10)]
                );
            }
        }

        // Do it again to prove that `multi` wasn't `moved`.
        first = true;
        for p in &multi {
            if first {
                assert_eq!(
                    p,
                    &line_string![(x: 0, y: 0), (x: 2, y: 0), (x: 1, y: 2), (x:0, y:0)]
                );
                first = false;
            } else {
                assert_eq!(
                    p,
                    &line_string![(x: 10, y: 10), (x: 12, y: 10), (x: 11, y: 12), (x:10, y:10)]
                );
            }
        }
    }

    #[test]
    fn test_iter_mut() {
        let mut multi = MultiLineString::new(vec![
            line_string![(x: 0, y: 0), (x: 2, y: 0), (x: 1, y: 2), (x:0, y:0)],
            line_string![(x: 10, y: 10), (x: 12, y: 10), (x: 11, y: 12), (x:10, y:10)],
        ]);

        for line_string in &mut multi {
            for coord in line_string {
                coord.x += 1;
                coord.y += 1;
            }
        }

        for line_string in multi.iter_mut() {
            for coord in line_string {
                coord.x += 1;
                coord.y += 1;
            }
        }

        let mut first = true;
        for p in &multi {
            if first {
                assert_eq!(
                    p,
                    &line_string![(x: 2, y: 2), (x: 4, y: 2), (x: 3, y: 4), (x:2, y:2)]
                );
                first = false;
            } else {
                assert_eq!(
                    p,
                    &line_string![(x: 12, y: 12), (x: 14, y: 12), (x: 13, y: 14), (x:12, y:12)]
                );
            }
        }
    }
}
