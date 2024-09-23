use crate::types::{
    coordinate::Coordinate, coordnum::CoordNum, line::Line, multi_point::MultiPoint, point::Point,
    polygon::Polygon, triangle::Triangle,
};

use super::Intersects;

// Blanket implementation from Coord<T>
impl<T, Z, G> Intersects<G> for Point<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
    Coordinate<T, Z>: Intersects<G>,
{
    fn intersects(&self, rhs: &G) -> bool {
        self.0.intersects(rhs)
    }
}

// Blanket implementation from Point<T>
impl<T, Z, G> Intersects<G> for MultiPoint<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
    Point<T, Z>: Intersects<G>,
{
    fn intersects(&self, rhs: &G) -> bool {
        self.iter().any(|p| p.intersects(rhs))
    }
}

symmetric_intersects_impl!(Coordinate<T, Z>, MultiPoint<T, Z>);
symmetric_intersects_impl!(Line<T, Z>, MultiPoint<T, Z>);
symmetric_intersects_impl!(Triangle<T, Z>, MultiPoint<T, Z>);
symmetric_intersects_impl!(Polygon<T, Z>, MultiPoint<T, Z>);
