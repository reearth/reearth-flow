use crate::{
    algorithm::{
        bounding_rect::BoundingRect,
        coordinate_position::{CoordPos, CoordinatePosition},
        utils::has_disjoint_bboxes,
        GeoNum,
    },
    types::{
        coordinate::Coordinate, line::Line, line_string::LineString,
        multi_line_string::MultiLineString, multi_polygon::MultiPolygon, point::Point,
        polygon::Polygon, rect::Rect, triangle::Triangle,
    },
};

use super::Intersects;

impl<T, Z> Intersects<Coordinate<T, Z>> for Polygon<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn intersects(&self, p: &Coordinate<T, Z>) -> bool {
        self.coordinate_position(p) != CoordPos::Outside
    }
}
symmetric_intersects_impl!(Coordinate<T, Z>, Polygon<T, Z>);
symmetric_intersects_impl!(Polygon<T, Z>, Point<T, Z>);

impl<T, Z> Intersects<Line<T, Z>> for Polygon<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn intersects(&self, line: &Line<T, Z>) -> bool {
        self.exterior().intersects(line)
            || self.interiors().iter().any(|inner| inner.intersects(line))
            || self.intersects(&line.start)
            || self.intersects(&line.end)
    }
}
symmetric_intersects_impl!(Line<T, Z>, Polygon<T, Z>);
symmetric_intersects_impl!(Polygon<T, Z>, LineString<T, Z>);
symmetric_intersects_impl!(Polygon<T, Z>, MultiLineString<T, Z>);

impl<T, Z> Intersects<Rect<T, Z>> for Polygon<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn intersects(&self, rect: &Rect<T, Z>) -> bool {
        self.intersects(&rect.to_polygon())
    }
}
symmetric_intersects_impl!(Rect<T, Z>, Polygon<T, Z>);

impl<T, Z> Intersects<Triangle<T, Z>> for Polygon<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn intersects(&self, rect: &Triangle<T, Z>) -> bool {
        self.intersects(&rect.to_polygon())
    }
}
symmetric_intersects_impl!(Triangle<T, Z>, Polygon<T, Z>);

impl<T, Z> Intersects<Polygon<T, Z>> for Polygon<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn intersects(&self, polygon: &Polygon<T, Z>) -> bool {
        if has_disjoint_bboxes(self, polygon) {
            return false;
        }

        // self intersects (or contains) any line in polygon
        self.intersects(polygon.exterior()) ||
            polygon.interiors().iter().any(|inner_line_string| self.intersects(inner_line_string)) ||
            // self is contained inside polygon
            polygon.intersects(self.exterior())
    }
}

// Implementations for MultiPolygon

impl<G, T, Z> Intersects<G> for MultiPolygon<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
    Polygon<T, Z>: Intersects<G>,
    G: BoundingRect<T, Z>,
{
    fn intersects(&self, rhs: &G) -> bool {
        if has_disjoint_bboxes(self, rhs) {
            return false;
        }
        self.iter().any(|p| p.intersects(rhs))
    }
}

symmetric_intersects_impl!(Point<T, Z>, MultiPolygon<T, Z>);
symmetric_intersects_impl!(Line<T, Z>, MultiPolygon<T, Z>);
symmetric_intersects_impl!(Rect<T, Z>, MultiPolygon<T, Z>);
symmetric_intersects_impl!(Triangle<T, Z>, MultiPolygon<T, Z>);
symmetric_intersects_impl!(Polygon<T, Z>, MultiPolygon<T, Z>);
