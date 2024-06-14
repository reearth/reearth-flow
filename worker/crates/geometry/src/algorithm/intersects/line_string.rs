use crate::{
    algorithm::{bounding_rect::BoundingRect, utils::has_disjoint_bboxes},
    types::{
        coordinate::Coordinate, coordnum::CoordNum, line::Line, line_string::LineString,
        multi_line_string::MultiLineString, point::Point, rect::Rect, triangle::Triangle,
    },
};

use super::Intersects;

impl<T, Z, G> Intersects<G> for LineString<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
    Line<T, Z>: Intersects<G>,
    G: BoundingRect<T, Z>,
{
    fn intersects(&self, geom: &G) -> bool {
        if has_disjoint_bboxes(self, geom) {
            return false;
        }
        self.lines().any(|l| l.intersects(geom))
    }
}
symmetric_intersects_impl!(Coordinate<T, Z>, LineString<T, Z>);
symmetric_intersects_impl!(Line<T, Z>, LineString<T, Z>);
symmetric_intersects_impl!(Rect<T, Z>, LineString<T, Z>);
symmetric_intersects_impl!(Triangle<T, Z>, LineString<T, Z>);

// Blanket implementation from LineString<T, Z>
impl<T, Z, G> Intersects<G> for MultiLineString<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
    LineString<T, Z>: Intersects<G>,
    G: BoundingRect<T, Z>,
{
    fn intersects(&self, rhs: &G) -> bool {
        if has_disjoint_bboxes(self, rhs) {
            return false;
        }
        self.iter().any(|p| p.intersects(rhs))
    }
}

symmetric_intersects_impl!(Point<T, Z>, MultiLineString<T, Z>);
symmetric_intersects_impl!(Line<T, Z>, MultiLineString<T, Z>);
symmetric_intersects_impl!(Rect<T, Z>, MultiLineString<T, Z>);
symmetric_intersects_impl!(Triangle<T, Z>, MultiLineString<T, Z>);
