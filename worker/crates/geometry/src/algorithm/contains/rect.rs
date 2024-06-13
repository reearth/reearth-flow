use crate::types::{coordinate::Coordinate, coordnum::CoordNum, point::Point, rect::Rect};

use super::Contains;

impl<T, Z> Contains<Coordinate<T, Z>> for Rect<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    fn contains(&self, coord: &Coordinate<T, Z>) -> bool {
        coord.x > self.min().x
            && coord.x < self.max().x
            && coord.y > self.min().y
            && coord.y < self.max().y
            && coord.z > self.min().z
            && coord.z < self.max().z
    }
}

impl<T, Z> Contains<Point<T, Z>> for Rect<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    fn contains(&self, p: &Point<T, Z>) -> bool {
        self.contains(&p.0)
    }
}

impl<T, Z> Contains<Rect<T, Z>> for Rect<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    fn contains(&self, other: &Rect<T, Z>) -> bool {
        // TODO: check for degenerate rectangle (which is a line or a point)
        // All points of LineString must be in the polygon ?
        self.min().x <= other.min().x
            && self.max().x >= other.max().x
            && self.min().y <= other.min().y
            && self.max().y >= other.max().y
            && self.min().z <= other.min().z
            && self.max().z >= other.max().z
    }
}
