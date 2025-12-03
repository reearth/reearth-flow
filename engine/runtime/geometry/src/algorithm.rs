use std::cmp::Ordering;

use num_traits::Float;

use crate::types::{coordnum::CoordNum, point::Point};

pub mod area2d;
pub mod area3d;
pub mod bool_ops;
// the old bool_ops module is kept for reference
// pub mod bool_ops_old;
pub mod bounding_rect;
pub mod bufferable;
pub mod centroid;
pub mod clipper;
pub mod contains;
pub mod convex_hull;
pub mod coordinate_position;
pub mod coords_iter;
pub mod dimensions;
pub mod euclidean_distance;
pub mod euclidean_length;
pub mod geo_distance_converter;
pub mod geometry_cow;
pub mod hole;
pub mod intersects;
pub mod is_convex;
pub mod kernels;
pub mod line_intersection;
pub mod line_ops;
pub mod line_string_ops;
pub mod map_coords;
pub mod normal_3d;
pub mod relate;
pub mod remove_repeated_points;
pub mod rotate;
pub mod segment_triangle_intersection;
pub mod sweep;
pub mod tolerance;
pub mod transverse_mercator_proj;
pub mod triangle_intersection;
pub mod utils;
pub mod winding_order;

pub use relate::Relate;

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

macro_rules! impl_geo_num_for_float {
    ($t: ident) => {
        impl GeoNum for $t {
            fn total_cmp(&self, other: &Self) -> Ordering {
                self.total_cmp(other)
            }
        }
    };
}

impl_geo_num_for_float!(f32);
impl_geo_num_for_float!(f64);
