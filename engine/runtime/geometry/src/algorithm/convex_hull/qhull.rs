use crate::{
    algorithm::{
        convex_hull::trivial_hull,
        kernels::{Orientation, RobustKernel},
        utils::{least_and_greatest_index, partition_slice},
        GeoNum,
    },
    types::{coordinate::Coordinate, line_string::LineString},
};

use super::swap_with_first_and_remove;

#[inline]
fn is_ccw<T, Z>(p_a: Coordinate<T, Z>, p_b: Coordinate<T, Z>, p_c: Coordinate<T, Z>) -> bool
where
    T: GeoNum,
    Z: GeoNum,
{
    RobustKernel::orient(p_a, p_b, p_c, None) == Orientation::CounterClockwise
}

pub fn quick_hull<T, Z>(mut points: &mut [Coordinate<T, Z>]) -> LineString<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    // can't build a hull from fewer than four points
    if points.len() < 4 {
        return trivial_hull(points, false);
    }
    let mut hull = vec![];

    let (min, max) = {
        let (min_idx, mut max_idx) = least_and_greatest_index(points);
        let min = swap_with_first_and_remove(&mut points, min_idx);

        // Two special cases to consider:
        // (1) max_idx = 0, and got swapped
        if max_idx == 0 {
            max_idx = min_idx;
        }

        // (2) max_idx = min_idx: then any point could be
        // chosen as max. But from case (1), it could now be
        // 0, and we should not decrement it.
        max_idx = max_idx.saturating_sub(1);

        let max = swap_with_first_and_remove(&mut points, max_idx);
        (min, max)
    };

    {
        let (points, _) = partition_slice(points, |p| is_ccw(*max, *min, *p));
        hull_set(*max, *min, points, &mut hull);
    }
    hull.push(*max);
    let (points, _) = partition_slice(points, |p| is_ccw(*min, *max, *p));
    hull_set(*min, *max, points, &mut hull);
    hull.push(*min);
    // close the polygon
    let mut hull: LineString<_, _> = hull.into();
    hull.close();
    hull
}

/// Recursively calculate the convex hull of a subset of points
fn hull_set<T, Z>(
    p_a: Coordinate<T, Z>,
    p_b: Coordinate<T, Z>,
    mut set: &mut [Coordinate<T, Z>],
    hull: &mut Vec<Coordinate<T, Z>>,
) where
    T: GeoNum,
    Z: GeoNum,
{
    if set.is_empty() {
        return;
    }
    if set.len() == 1 {
        hull.push(set[0]);
        return;
    }

    // Construct orthogonal vector to `p_b` - `p_a` We
    // compute inner product of this with `v` - `p_a` to
    // find the farthest point from the line segment a-b.
    let p_orth = Coordinate::new__(p_a.y - p_b.y, p_b.x - p_a.x, p_b.z - p_a.z);

    let furthest_idx = set
        .iter()
        .map(|pt| {
            let p_diff = Coordinate::new__(pt.x - p_a.x, pt.y - p_a.y, pt.z - p_a.z);
            p_orth.x * p_diff.x + p_orth.y * p_diff.y
        })
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .unwrap()
        .0;

    // move Coord at furthest_point from set into hull
    let furthest_point = swap_with_first_and_remove(&mut set, furthest_idx);
    // points over PB
    {
        let (points, _) = partition_slice(set, |p| is_ccw(*furthest_point, p_b, *p));
        hull_set(*furthest_point, p_b, points, hull);
    }
    hull.push(*furthest_point);
    // points over AP
    let (points, _) = partition_slice(set, |p| is_ccw(p_a, *furthest_point, *p));
    hull_set(p_a, *furthest_point, points, hull);
}

#[cfg(test)]
mod test {
    use crate::coord;

    use super::*;

    #[test]
    fn quick_hull_test2() {
        let mut v = vec![
            coord! { x: 0., y: 10. },
            coord! { x: 1., y: 1. },
            coord! { x: 10., y: 0. },
            coord! { x: 1., y: -1. },
            coord! { x: 0., y: -10. },
            coord! { x: -1., y: -1. },
            coord! { x: -10., y: 0. },
            coord! { x: -1., y: 1. },
            coord! { x: 0., y: 10. },
        ];
        let correct = vec![
            coord! { x: 0., y: -10. },
            coord! { x: 10., y: 0. },
            coord! { x: 0., y: 10. },
            coord! { x: -10., y: 0. },
            coord! { x: 0., y: -10. },
        ];
        let res = quick_hull(&mut v);
        assert_eq!(res.0, correct);
    }
}
