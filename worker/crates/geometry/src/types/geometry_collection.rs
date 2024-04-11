use std::iter::FromIterator;
use std::ops::{Index, IndexMut};

use approx::{AbsDiffEq, RelativeEq};
use serde::{Deserialize, Serialize};

use super::coordnum::CoordNum;
use super::geometry::Geometry;
use super::no_value::NoValue;

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug, Hash)]
pub struct GeometryCollection<T: CoordNum = f64, Z: CoordNum = NoValue>(pub Vec<Geometry<T, Z>>);

pub type GeometryCollection2D<T> = GeometryCollection<T>;
pub type GeometryCollection3D<T> = GeometryCollection<T, T>;

impl<T: CoordNum, Z: CoordNum> Default for GeometryCollection<T, Z> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T: CoordNum, Z: CoordNum> GeometryCollection<T, Z> {
    #[inline]
    pub fn new(value: Vec<Geometry<T, Z>>) -> Self {
        Self(value)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T: CoordNum, IG: Into<Geometry<T>>> From<Vec<IG>> for GeometryCollection<T> {
    fn from(geoms: Vec<IG>) -> Self {
        let geoms: Vec<Geometry<_>> = geoms.into_iter().map(Into::into).collect();
        Self(geoms)
    }
}

impl<T: CoordNum, Z: CoordNum, IG: Into<Geometry<T, Z>>> FromIterator<IG>
    for GeometryCollection<T, Z>
{
    fn from_iter<I: IntoIterator<Item = IG>>(iter: I) -> Self {
        Self(iter.into_iter().map(|g| g.into()).collect())
    }
}

impl<T: CoordNum, Z: CoordNum> Index<usize> for GeometryCollection<T, Z> {
    type Output = Geometry<T, Z>;

    fn index(&self, index: usize) -> &Geometry<T, Z> {
        self.0.index(index)
    }
}

impl<T: CoordNum, Z: CoordNum> IndexMut<usize> for GeometryCollection<T, Z> {
    fn index_mut(&mut self, index: usize) -> &mut Geometry<T, Z> {
        self.0.index_mut(index)
    }
}

#[derive(Debug)]
pub struct IntoIteratorHelper<T: CoordNum, Z: CoordNum> {
    iter: ::std::vec::IntoIter<Geometry<T, Z>>,
}

impl<T: CoordNum, Z: CoordNum> IntoIterator for GeometryCollection<T, Z> {
    type Item = Geometry<T, Z>;
    type IntoIter = IntoIteratorHelper<T, Z>;

    // note that into_iter() is consuming self
    fn into_iter(self) -> Self::IntoIter {
        IntoIteratorHelper {
            iter: self.0.into_iter(),
        }
    }
}

impl<T: CoordNum, Z: CoordNum> Iterator for IntoIteratorHelper<T, Z> {
    type Item = Geometry<T, Z>;

    // just return the reference
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[derive(Debug)]
pub struct IterHelper<'a, T: CoordNum, Z: CoordNum> {
    iter: ::std::slice::Iter<'a, Geometry<T, Z>>,
}

impl<'a, T: CoordNum, Z: CoordNum> IntoIterator for &'a GeometryCollection<T, Z> {
    type Item = &'a Geometry<T, Z>;
    type IntoIter = IterHelper<'a, T, Z>;

    // note that into_iter() is consuming self
    fn into_iter(self) -> Self::IntoIter {
        IterHelper {
            iter: self.0.iter(),
        }
    }
}

impl<'a, T: CoordNum, Z: 'a + CoordNum> Iterator for IterHelper<'a, T, Z> {
    type Item = &'a Geometry<T, Z>;

    // just return the str reference
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[derive(Debug)]
pub struct IterMutHelper<'a, T: CoordNum, Z: CoordNum> {
    iter: ::std::slice::IterMut<'a, Geometry<T, Z>>,
}

impl<'a, T: CoordNum, Z: CoordNum> IntoIterator for &'a mut GeometryCollection<T, Z> {
    type Item = &'a mut Geometry<T, Z>;
    type IntoIter = IterMutHelper<'a, T, Z>;

    fn into_iter(self) -> Self::IntoIter {
        IterMutHelper {
            iter: self.0.iter_mut(),
        }
    }
}

impl<'a, T: CoordNum, Z: CoordNum> Iterator for IterMutHelper<'a, T, Z> {
    type Item = &'a mut Geometry<T, Z>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl<'a, T: CoordNum, Z: CoordNum> GeometryCollection<T, Z> {
    pub fn iter(&'a self) -> IterHelper<'a, T, Z> {
        self.into_iter()
    }

    pub fn iter_mut(&'a mut self) -> IterMutHelper<'a, T, Z> {
        self.into_iter()
    }
}

impl<T> RelativeEq for GeometryCollection<T, T>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum + RelativeEq,
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

impl<T> AbsDiffEq for GeometryCollection<T, T>
where
    T: AbsDiffEq<Epsilon = T> + CoordNum,
    T::Epsilon: Copy,
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
mod tests {
    use crate::types::point::Point;

    use super::*;

    #[test]
    fn from_vec() {
        let gc = GeometryCollection::from(vec![Point::new(1i32, 2)]);
        let p = Point::try_from(gc[0].clone()).unwrap();
        assert_eq!(p.y(), 2);
    }
}
