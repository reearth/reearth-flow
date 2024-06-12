use std::cmp::Ordering;

use num_traits::Float;

use crate::types::{coordnum::CoordNum, point::Point};

pub mod bounding_rect;
pub mod contains;
pub mod coordinate_position;
pub mod coords_iter;
pub mod dimensions;
pub mod intersects;
pub mod kernels;
pub mod line_intersection;
pub mod utils;

pub trait GeoFloat:
    GeoNum + num_traits::Float + num_traits::Signed + num_traits::Bounded + float_next_after::NextAfter
{
}
impl<T> GeoFloat for T where
    T: GeoNum
        + num_traits::Float
        + num_traits::Signed
        + num_traits::Bounded
        + float_next_after::NextAfter
{
}

pub trait GeoNum: CoordNum + Float {
    fn total_cmp(&self, other: &Self) -> Ordering;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Closest<F: GeoFloat> {
    Intersection(Point<F>),
    SinglePoint(Point<F>),
    Indeterminate,
}
