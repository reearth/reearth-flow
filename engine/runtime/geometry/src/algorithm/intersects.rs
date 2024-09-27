use crate::types::coordinate::Coordinate;
use crate::types::coordnum::CoordNum;

pub trait Intersects<Rhs = Self> {
    fn intersects(&self, rhs: &Rhs) -> bool;
}

macro_rules! symmetric_intersects_impl {
    ($t:ty, $k:ty) => {
        impl<T, Z> crate::algorithm::intersects::Intersects<$k> for $t
        where
            $k: crate::algorithm::intersects::Intersects<$t>,
            T: crate::types::coordnum::CoordNum,
            Z: crate::types::coordnum::CoordNum,
        {
            fn intersects(&self, rhs: &$k) -> bool {
                rhs.intersects(self)
            }
        }
    };
}

pub mod coordinate;
pub mod line;
pub mod line_string;
pub mod point;
pub mod polygon;
pub mod rect;

// Helper function to check value lies between min and max.
// Only makes sense if min <= max (or always false)
#[inline]
fn value_in_range<T>(value: T, min: T, max: T) -> bool
where
    T: std::cmp::PartialOrd,
{
    value >= min && value <= max
}

#[inline]
pub(crate) fn value_in_between<T>(value: T, bound_1: T, bound_2: T) -> bool
where
    T: std::cmp::PartialOrd,
{
    if bound_1 < bound_2 {
        value_in_range(value, bound_1, bound_2)
    } else {
        value_in_range(value, bound_2, bound_1)
    }
}

#[inline]
pub(crate) fn point_in_rect<T, Z>(
    value: Coordinate<T, Z>,
    bound_1: Coordinate<T, Z>,
    bound_2: Coordinate<T, Z>,
) -> bool
where
    T: CoordNum,
    Z: CoordNum,
{
    value_in_between(value.x, bound_1.x, bound_2.x)
        && value_in_between(value.y, bound_1.y, bound_2.y)
}
