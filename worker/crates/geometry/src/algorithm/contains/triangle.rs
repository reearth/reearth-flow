use crate::{
    algorithm::{
        kernels::{Orientation, RobustKernel},
        GeoNum,
    },
    types::{
        coordinate::Coordinate, line::Line, line_string::LineString,
        multi_line_string::MultiLineString, multi_point::MultiPoint, multi_polygon::MultiPolygon,
        point::Point, polygon::Polygon, rect::Rect, triangle::Triangle,
    },
};

use super::Contains;

impl<T, Z> Contains<Coordinate<T, Z>> for Triangle<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn contains(&self, coord: &Coordinate<T, Z>) -> bool {
        // leverageing robust predicates
        self.to_lines()
            .map(|l| RobustKernel::orient(l.start, l.end, *coord, None))
            .windows(2)
            .all(|win| win[0] == win[1] && win[0] != Orientation::Collinear)
    }
}

impl<T, Z> Contains<Point<T, Z>> for Triangle<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn contains(&self, point: &Point<T, Z>) -> bool {
        self.contains(&point.0)
    }
}

impl_contains_from_relate!(Triangle<T, Z>, [Line<T, Z>, LineString<T, Z>, Polygon<T, Z>, MultiPoint<T, Z>, MultiLineString<T, Z>, MultiPolygon<T, Z>, Rect<T, Z>, Triangle<T, Z>]);
