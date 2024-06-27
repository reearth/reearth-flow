use std::{cmp::Ordering, ops::Deref};

use crate::{algorithm::GeoNum, types::coordinate::Coordinate};

/// A lexicographically ordered point.
///
/// A wrapper around [`Coord`] to order the point by `x`, and then by `y`.
/// Implements `Ord` and `Eq`, allowing usage in ordered collections such as
/// `BinaryHeap`.
///
/// Note that the scalar type `T` is only required to implement `PartialOrd`.
/// Thus, it is a logical error to construct this struct unless the coords are
/// guaranteed to be orderable.
#[derive(PartialEq, Clone, Copy)]
pub struct SweepPoint<T: GeoNum, Z: GeoNum>(Coordinate<T, Z>);

impl<T: GeoNum, Z: GeoNum> std::fmt::Debug for SweepPoint<T, Z> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("SPt")
            .field(&self.0.x)
            .field(&self.0.y)
            .field(&self.0.z)
            .finish()
    }
}

/// Implement lexicographic ordering by `x` and then by `y`
/// coordinate.
impl<T: GeoNum, Z: GeoNum> PartialOrd for SweepPoint<T, Z> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Derive `Ord` from `PartialOrd` and expect to not fail.
impl<T: GeoNum, Z: GeoNum> Ord for SweepPoint<T, Z> {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.x.total_cmp(&other.0.x) {
            Ordering::Equal => self.0.y.total_cmp(&other.0.y),
            o => o,
        }
    }
}

/// We derive `Eq` manually to not require `T: Eq`.
impl<T: GeoNum, Z: GeoNum> Eq for SweepPoint<T, Z> {}

/// Conversion from type that can be converted to a `Coord`.
impl<T: GeoNum, Z: GeoNum, X: Into<Coordinate<T, Z>>> From<X> for SweepPoint<T, Z> {
    fn from(pt: X) -> Self {
        SweepPoint(pt.into())
    }
}

impl<T: GeoNum, Z: GeoNum> Deref for SweepPoint<T, Z> {
    type Target = Coordinate<T, Z>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::types::no_value::NoValue;

    use super::*;

    #[test]
    fn test_sweep_point_ordering() {
        let p1 = SweepPoint::from(Coordinate::new__(0., 0., NoValue));
        let p2 = SweepPoint::from(Coordinate::new__(1., 0., NoValue));
        let p3 = SweepPoint::from(Coordinate::new__(1., 1., NoValue));
        let p4 = SweepPoint::from(Coordinate::new__(1., 1., NoValue));

        assert!(p1 < p2);
        assert!(p1 < p3);
        assert!(p2 < p3);
        assert!(p3 <= p4);
    }
}
