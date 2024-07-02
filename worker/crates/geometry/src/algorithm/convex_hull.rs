pub trait ConvexHull<'a, T, Z> {
    type ScalarXY: GeoNum;
    type ScalarZ: GeoNum;
    fn convex_hull(&'a self) -> Polygon<Self::ScalarXY, Self::ScalarZ>;
}

impl<'a, T, Z, G> ConvexHull<'a, T, Z> for G
where
    T: GeoNum,
    Z: GeoNum,
    G: CoordsIter<ScalarXY = T, ScalarZ = Z>,
{
    type ScalarXY = T;
    type ScalarZ = Z;

    fn convex_hull(&'a self) -> Polygon<T, Z> {
        let mut exterior: Vec<_> = self.exterior_coords_iter().collect();
        Polygon::new(quick_hull(&mut exterior), vec![])
    }
}

pub mod qhull;
pub use qhull::quick_hull;

pub mod graham;
pub use graham::graham_hull;

use crate::{
    algorithm::{
        kernels::{Orientation, RobustKernel},
        utils::lex_cmp,
    },
    types::{coordinate::Coordinate, line_string::LineString, polygon::Polygon},
};

use super::{coords_iter::CoordsIter, GeoNum};

// Helper function that outputs the convex hull in the
// trivial case: input with at most 3 points. It ensures the
// output is ccw, and does not repeat points unless
// required.
fn trivial_hull<T, Z>(points: &mut [Coordinate<T, Z>], include_on_hull: bool) -> LineString<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    assert!(points.len() < 4);

    // Remove repeated points unless collinear points
    // are to be included.
    let mut ls: Vec<Coordinate<T, Z>> = points.to_vec();
    if !include_on_hull {
        ls.sort_unstable_by(lex_cmp);
        if ls.len() == 3
            && RobustKernel::orient(ls[0], ls[1], ls[2], None) == Orientation::Collinear
        {
            ls.remove(1);
        }
    }

    // A linestring with a single point is invalid.
    if ls.len() == 1 {
        ls.push(ls[0]);
    }

    let mut ls = LineString::new(ls);
    ls.close();

    // Maintain the CCW invariance
    use super::winding_order::Winding;
    ls.make_ccw_winding();
    ls
}

/// Utility function for convex hull ops
///
/// 1. _swap_ the element at `idx` with the element at `head` (0th position)
/// 2. remove the _new_ `head` element (modifying the slice)
/// 3. return a _mutable ref_ to the removed head element
fn swap_with_first_and_remove<'a, T>(slice: &mut &'a mut [T], idx: usize) -> &'a mut T {
    // temporarily replace `slice` with an empty value
    let tmp = std::mem::take(slice);
    tmp.swap(0, idx);
    let (h, t) = tmp.split_first_mut().unwrap();
    *slice = t;
    h
}
