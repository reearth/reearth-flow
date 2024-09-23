use crate::types::{coordinate::Coordinate, coordnum::CoordNum, point::Point};

use super::Intersects;

impl<T, Z> Intersects<Coordinate<T, Z>> for Coordinate<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    fn intersects(&self, rhs: &Coordinate<T, Z>) -> bool {
        self == rhs
    }
}

// The other side of this is handled via a blanket impl.
impl<T, Z> Intersects<Point<T, Z>> for Coordinate<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    fn intersects(&self, rhs: &Point<T, Z>) -> bool {
        self == &rhs.0
    }
}
