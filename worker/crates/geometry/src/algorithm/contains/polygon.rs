use crate::{
    algorithm::{
        coordinate_position::{CoordPos, CoordinatePosition},
        GeoNum,
    },
    types::{
        coordinate::Coordinate, multi_point::MultiPoint, multi_polygon::MultiPolygon, point::Point,
        polygon::Polygon,
    },
};

use super::Contains;

impl<T, Z> Contains<Coordinate<T, Z>> for Polygon<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn contains(&self, coord: &Coordinate<T, Z>) -> bool {
        self.coordinate_position(coord) == CoordPos::Inside
    }
}

impl<T, Z> Contains<Point<T, Z>> for Polygon<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn contains(&self, p: &Point<T, Z>) -> bool {
        self.contains(&p.0)
    }
}

impl<T, Z> Contains<Coordinate<T, Z>> for MultiPolygon<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn contains(&self, coord: &Coordinate<T, Z>) -> bool {
        self.iter().any(|poly| poly.contains(coord))
    }
}

impl<T, Z> Contains<Point<T, Z>> for MultiPolygon<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn contains(&self, p: &Point<T, Z>) -> bool {
        self.contains(&p.0)
    }
}

impl<T: GeoNum, Z: GeoNum> Contains<MultiPoint<T, Z>> for MultiPolygon<T, Z> {
    fn contains(&self, rhs: &MultiPoint<T, Z>) -> bool {
        rhs.iter().all(|point| self.contains(point))
    }
}
