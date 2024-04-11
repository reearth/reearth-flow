use std::iter::FromIterator;

use approx::{AbsDiffEq, RelativeEq};
use serde::{Deserialize, Serialize};

use nusamai_geometry::{MultiPolygon2 as NMultiPolygon2, MultiPolygon3 as NMultiPolygon3};

use super::coordnum::CoordNum;
use super::no_value::NoValue;
use super::polygon::{Polygon, Polygon2D, Polygon3D};

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug, Hash)]
pub struct MultiPolygon<T: CoordNum = f64, Z: CoordNum = NoValue>(pub Vec<Polygon<T, Z>>);

pub type MultiPolygon2D<T> = MultiPolygon<T>;
pub type MultiPolygon3D<T> = MultiPolygon<T, T>;

impl<T: CoordNum, Z: CoordNum, IP: Into<Polygon<T, Z>>> From<IP> for MultiPolygon<T, Z> {
    fn from(x: IP) -> Self {
        Self(vec![x.into()])
    }
}

impl<T: CoordNum, Z: CoordNum, IP: Into<Polygon<T, Z>>> From<Vec<IP>> for MultiPolygon<T, Z> {
    fn from(x: Vec<IP>) -> Self {
        Self(x.into_iter().map(|p| p.into()).collect())
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

    pub fn iter(&self) -> impl Iterator<Item = &Polygon<T, Z>> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Polygon<T, Z>> {
        self.0.iter_mut()
    }
}

impl<'a> From<NMultiPolygon2<'a>> for MultiPolygon2D<f64> {
    #[inline]
    fn from(mpoly: NMultiPolygon2<'a>) -> Self {
        mpoly.iter().map(Polygon2D::from).collect()
    }
}

impl<'a> From<NMultiPolygon3<'a>> for MultiPolygon3D<f64> {
    #[inline]
    fn from(mpoly: NMultiPolygon3<'a>) -> Self {
        mpoly.iter().map(Polygon3D::from).collect()
    }
}

impl<T> RelativeEq for MultiPolygon<T, T>
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

impl<T> AbsDiffEq for MultiPolygon<T, T>
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
mod test {
    use super::*;
    use crate::polygon;

    #[test]
    fn test_iter() {
        let multi = MultiPolygon::new(vec![
            polygon![(x: 0, y: 0), (x: 2, y: 0), (x: 1, y: 2), (x:0, y:0)],
            polygon![(x: 10, y: 10), (x: 12, y: 10), (x: 11, y: 12), (x:10, y:10)],
        ]);

        let mut first = true;
        for p in &multi {
            if first {
                assert_eq!(
                    p,
                    &polygon![(x: 0, y: 0), (x: 2, y: 0), (x: 1, y: 2), (x:0, y:0)]
                );
                first = false;
            } else {
                assert_eq!(
                    p,
                    &polygon![(x: 10, y: 10), (x: 12, y: 10), (x: 11, y: 12), (x:10, y:10)]
                );
            }
        }

        // Do it again to prove that `multi` wasn't `moved`.
        first = true;
        for p in &multi {
            if first {
                assert_eq!(
                    p,
                    &polygon![(x: 0, y: 0), (x: 2, y: 0), (x: 1, y: 2), (x:0, y:0)]
                );
                first = false;
            } else {
                assert_eq!(
                    p,
                    &polygon![(x: 10, y: 10), (x: 12, y: 10), (x: 11, y: 12), (x:10, y:10)]
                );
            }
        }
    }

    #[test]
    fn test_iter_mut() {
        let mut multi = MultiPolygon::new(vec![
            polygon![(x: 0, y: 0), (x: 2, y: 0), (x: 1, y: 2), (x:0, y:0)],
            polygon![(x: 10, y: 10), (x: 12, y: 10), (x: 11, y: 12), (x:10, y:10)],
        ]);

        for poly in &mut multi {
            poly.exterior_mut(|exterior| {
                for coord in exterior {
                    coord.x += 1;
                    coord.y += 1;
                }
            });
        }

        for poly in multi.iter_mut() {
            poly.exterior_mut(|exterior| {
                for coord in exterior {
                    coord.x += 1;
                    coord.y += 1;
                }
            });
        }

        let mut first = true;
        for p in &multi {
            if first {
                assert_eq!(
                    p,
                    &polygon![(x: 2, y: 2), (x: 4, y: 2), (x: 3, y: 4), (x:2, y:2)]
                );
                first = false;
            } else {
                assert_eq!(
                    p,
                    &polygon![(x: 12, y: 12), (x: 14, y: 12), (x: 13, y: 14), (x:12, y:12)]
                );
            }
        }
    }
}
