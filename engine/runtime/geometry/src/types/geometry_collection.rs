use super::no_value::NoValue;
use super::traits::Elevation;
use super::{coordnum::CoordNum, geometry::Geometry};

use alloc::vec::Vec;
use core::iter::FromIterator;
use core::ops::{Index, IndexMut};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct GeometryCollection<T: CoordNum = f64, Z: CoordNum = f64>(pub Vec<Geometry<T, Z>>);

// Implementing Default by hand because T does not have Default restriction
// todo: consider adding Default as a CoordNum requirement
impl<T: CoordNum, Z: CoordNum> Default for GeometryCollection<T, Z> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T: CoordNum, Z: CoordNum> GeometryCollection<T, Z> {
    pub fn new(items: Vec<Geometry<T, Z>>) -> Self {
        Self(items)
    }
}

pub type GeometryCollection2D<T> = GeometryCollection<T, NoValue>;
pub type GeometryCollection3D<T> = GeometryCollection<T, T>;

impl<T: CoordNum> GeometryCollection<T> {
    /// Number of geometries in this GeometryCollection
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Is this GeometryCollection empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<T: CoordNum, Z: CoordNum, IG: Into<Geometry<T, Z>>> From<Vec<IG>>
    for GeometryCollection<T, Z>
{
    fn from(geoms: Vec<IG>) -> Self {
        let geoms: Vec<Geometry<_, _>> = geoms.into_iter().map(Into::into).collect();
        Self(geoms)
    }
}

/// Collect Geometries (or what can be converted to a Geometry) into a GeometryCollection
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

// structure helper for consuming iterator
#[derive(Debug)]
pub struct IntoIteratorHelper<T: CoordNum, Z: CoordNum> {
    iter: ::alloc::vec::IntoIter<Geometry<T, Z>>,
}

// implement the IntoIterator trait for a consuming iterator. Iteration will
// consume the GeometryCollection
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

// implement Iterator trait for the helper struct, to be used by adapters
impl<T: CoordNum, Z: CoordNum> Iterator for IntoIteratorHelper<T, Z> {
    type Item = Geometry<T, Z>;

    // just return the reference
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

// structure helper for non-consuming iterator
#[derive(Debug)]
pub struct IterHelper<'a, T: CoordNum, Z: CoordNum> {
    iter: ::core::slice::Iter<'a, Geometry<T, Z>>,
}

// implement the IntoIterator trait for a non-consuming iterator. Iteration will
// borrow the GeometryCollection
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

// implement the Iterator trait for the helper struct, to be used by adapters
impl<'a, T: CoordNum, Z: CoordNum> Iterator for IterHelper<'a, T, Z> {
    type Item = &'a Geometry<T, Z>;

    // just return the str reference
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

// structure helper for mutable non-consuming iterator
#[derive(Debug)]
pub struct IterMutHelper<'a, T: CoordNum, Z: CoordNum> {
    iter: ::core::slice::IterMut<'a, Geometry<T, Z>>,
}

// implement the IntoIterator trait for a mutable non-consuming iterator. Iteration will
// mutably borrow the GeometryCollection
impl<'a, T: CoordNum, Z: CoordNum> IntoIterator for &'a mut GeometryCollection<T, Z> {
    type Item = &'a mut Geometry<T, Z>;
    type IntoIter = IterMutHelper<'a, T, Z>;

    // note that into_iter() is consuming self
    fn into_iter(self) -> Self::IntoIter {
        IterMutHelper {
            iter: self.0.iter_mut(),
        }
    }
}

// implement the Iterator trait for the helper struct, to be used by adapters
impl<'a, T: CoordNum, Z: CoordNum> Iterator for IterMutHelper<'a, T, Z> {
    type Item = &'a mut Geometry<T, Z>;

    // just return the str reference
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

impl Elevation for GeometryCollection3D<f64> {
    #[inline]
    fn is_elevation_zero(&self) -> bool {
        self.0.iter().all(|g| g.is_elevation_zero())
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::GeometryCollection;
    use crate::{point, types::point::Point};

    #[test]
    fn from_vec() {
        let gc = GeometryCollection::from(vec![point!(x: 1i32, y: 2)]);
        let p = Point::try_from(gc[0].clone()).unwrap();
        assert_eq!(p.y(), 2);
    }
}
