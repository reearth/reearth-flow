use crate::types::{
    coordinate::Coordinate, coordnum::CoordNum, line::Line, line_string::LineString,
    multi_line_string::MultiLineString, multi_point::MultiPoint, multi_polygon::MultiPolygon,
    point::Point, polygon::Polygon, rect::Rect, triangle::Triangle,
};

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

impl_contains_from_relate!(Rect<T, Z>, [Line<T, Z>, LineString<T, Z>, Polygon<T, Z>, MultiPoint<T, Z>, MultiLineString<T, Z>, MultiPolygon<T, Z>, Triangle<T, Z>]);
