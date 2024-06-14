use crate::{
    algorithm::coords_iter::CoordsIter,
    types::{
        coordinate::Coordinate, coordnum::CoordNum, line::Line, line_string::LineString,
        multi_line_string::MultiLineString, multi_point::MultiPoint, multi_polygon::MultiPolygon,
        point::Point, polygon::Polygon, rect::Rect, triangle::Triangle,
    },
};

use crate::algorithm::dimensions::*;

use super::Contains;

impl<T, Z> Contains<Coordinate<T, Z>> for Point<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    fn contains(&self, coord: &Coordinate<T, Z>) -> bool {
        &self.0 == coord
    }
}

impl<T, Z> Contains<Point<T, Z>> for Point<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    fn contains(&self, p: &Point<T, Z>) -> bool {
        self.contains(&p.0)
    }
}

impl<T, Z> Contains<Line<T, Z>> for Point<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    fn contains(&self, line: &Line<T, Z>) -> bool {
        if line.start == line.end {
            line.start == self.0
        } else {
            false
        }
    }
}

impl<T, Z> Contains<LineString<T, Z>> for Point<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    fn contains(&self, line_string: &LineString<T, Z>) -> bool {
        if line_string.is_empty() {
            return false;
        }
        // only a degenerate LineString could be within a point
        line_string.coords().all(|c| c == &self.0)
    }
}

impl<T, Z> Contains<Polygon<T, Z>> for Point<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    fn contains(&self, polygon: &Polygon<T, Z>) -> bool {
        if polygon.is_empty() {
            return false;
        }
        // only a degenerate Polygon could be within a point
        polygon.coords_iter().all(|coord| coord == self.0)
    }
}

impl<T, Z> Contains<MultiPoint<T, Z>> for Point<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    fn contains(&self, multi_point: &MultiPoint<T, Z>) -> bool {
        if multi_point.is_empty() {
            return false;
        }
        multi_point.iter().all(|point| self.contains(point))
    }
}

impl<T, Z> Contains<MultiLineString<T, Z>> for Point<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    fn contains(&self, multi_line_string: &MultiLineString<T, Z>) -> bool {
        if multi_line_string.is_empty() {
            return false;
        }
        multi_line_string
            .iter()
            .all(|line_string| self.contains(line_string))
    }
}

impl<T, Z> Contains<MultiPolygon<T, Z>> for Point<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    fn contains(&self, multi_polygon: &MultiPolygon<T, Z>) -> bool {
        if multi_polygon.is_empty() {
            return false;
        }
        // only a degenerate MultiPolygon could be within a point
        multi_polygon.iter().all(|polygon| self.contains(polygon))
    }
}

impl<T, Z> Contains<Rect<T, Z>> for Point<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    fn contains(&self, rect: &Rect<T, Z>) -> bool {
        // only a degenerate Rect could be within a point
        rect.min() == rect.max() && rect.min() == self.0
    }
}

impl<T, Z> Contains<Triangle<T, Z>> for Point<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    fn contains(&self, triangle: &Triangle<T, Z>) -> bool {
        // only a degenerate Triangle could be within a point
        triangle.0 == triangle.1 && triangle.0 == triangle.2 && triangle.0 == self.0
    }
}

impl<T, Z> Contains<Coordinate<T, Z>> for MultiPoint<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    fn contains(&self, coord: &Coordinate<T, Z>) -> bool {
        self.iter().any(|c| &c.0 == coord)
    }
}

impl<T, Z> Contains<Point<T, Z>> for MultiPoint<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    fn contains(&self, point: &Point<T, Z>) -> bool {
        self.iter().any(|c| c == point)
    }
}

impl_contains_from_relate!(MultiPoint<T, Z>, [Line<T, Z>, LineString<T, Z>, Polygon<T, Z>, MultiLineString<T, Z>, MultiPolygon<T, Z>, MultiPoint<T, Z>, Rect<T, Z>, Triangle<T, Z>]);
