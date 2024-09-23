use crate::{
    algorithm::GeoNum,
    types::{
        coordinate::Coordinate, coordnum::CoordNum, line::Line, multi_point::MultiPoint,
        point::Point, rect::Rect, triangle::Triangle,
    },
};

use super::Intersects;

impl<T, Z> Intersects<Coordinate<T, Z>> for Rect<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    fn intersects(&self, rhs: &Coordinate<T, Z>) -> bool {
        rhs.x >= self.min().x
            && rhs.y >= self.min().y
            && rhs.x <= self.max().x
            && rhs.y <= self.max().y
    }
}
symmetric_intersects_impl!(Coordinate<T, Z>, Rect<T, Z>);
symmetric_intersects_impl!(Rect<T, Z>, Point<T, Z>);
symmetric_intersects_impl!(Rect<T, Z>, MultiPoint<T, Z>);

impl<T, Z> Intersects<Rect<T, Z>> for Rect<T, Z>
where
    T: CoordNum,
    Z: CoordNum,
{
    fn intersects(&self, other: &Rect<T, Z>) -> bool {
        if self.max().x < other.min().x {
            return false;
        }

        if self.max().y < other.min().y {
            return false;
        }

        if self.max().z < other.min().z {
            return false;
        }

        if self.min().x > other.max().x {
            return false;
        }

        if self.min().y > other.max().y {
            return false;
        }

        if self.min().z > other.max().z {
            return false;
        }

        true
    }
}

// Same logic as Polygon<T, Z>: Intersects<Line<T, Z>>, but avoid
// an allocation.
impl<T, Z> Intersects<Line<T, Z>> for Rect<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn intersects(&self, rhs: &Line<T, Z>) -> bool {
        let lt = self.min();
        let rb = self.max();
        let lb = Coordinate::new__(lt.x, rb.y, lt.z);
        let rt = Coordinate::new__(rb.x, lt.y, rb.z);
        // If either rhs.{start,end} lies inside Rect, then true
        self.intersects(&rhs.start)
            || self.intersects(&rhs.end)
            || Line::new_(lt, rt).intersects(rhs)
            || Line::new_(rt, rb).intersects(rhs)
            || Line::new_(lb, rb).intersects(rhs)
            || Line::new_(lt, lb).intersects(rhs)
    }
}
symmetric_intersects_impl!(Line<T, Z>, Rect<T, Z>);

impl<T, Z> Intersects<Triangle<T, Z>> for Rect<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    fn intersects(&self, rhs: &Triangle<T, Z>) -> bool {
        self.intersects(&rhs.to_polygon())
    }
}
symmetric_intersects_impl!(Triangle<T, Z>, Rect<T, Z>);
