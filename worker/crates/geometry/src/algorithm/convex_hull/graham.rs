use std::cmp::Ordering;

use crate::{
    algorithm::{
        kernels::{Orientation, RobustKernel},
        utils::least_index,
        GeoNum,
    },
    types::{coordinate::Coordinate, line_string::LineString},
};

use super::{swap_with_first_and_remove, trivial_hull};

pub fn graham_hull<T, Z>(
    mut points: &mut [Coordinate<T, Z>],
    include_on_hull: bool,
) -> LineString<T, Z>
where
    T: GeoNum,
    Z: GeoNum,
{
    if points.len() < 4 {
        // Nothing to build with fewer than four points.
        return trivial_hull(points, include_on_hull);
    }

    // Allocate output vector
    let mut output = Vec::with_capacity(points.len());

    let min_idx = least_index(points);
    let head = swap_with_first_and_remove(&mut points, min_idx);
    output.push(*head);

    // Sort rest of the points by angle it makes with head
    // point. If two points are collinear with head, we sort
    // by distance. We use kernel predicates here.
    let cmp = |q: &Coordinate<T, Z>, r: &Coordinate<T, Z>| match RobustKernel::orient(
        *q, *head, *r, None,
    ) {
        Orientation::CounterClockwise => Ordering::Greater,
        Orientation::Clockwise => Ordering::Less,
        Orientation::Collinear => {
            let dist1 = RobustKernel::square_euclidean_distance(*head, *q);
            let dist2 = RobustKernel::square_euclidean_distance(*head, *r);
            dist1.partial_cmp(&dist2).unwrap()
        }
    };
    points.sort_unstable_by(cmp);

    for pt in points.iter() {
        while output.len() > 1 {
            let len = output.len();
            match RobustKernel::orient(output[len - 2], output[len - 1], *pt, None) {
                Orientation::CounterClockwise => {
                    break;
                }
                Orientation::Clockwise => {
                    output.pop();
                }
                Orientation::Collinear => {
                    if include_on_hull {
                        break;
                    } else {
                        output.pop();
                    }
                }
            }
        }
        // Corner case: if the lex. least point added before
        // this loop is repeated, then we should not end up
        // adding it here (because output.len() == 1 in the
        // first iteration)
        if include_on_hull || pt != output.last().unwrap() {
            output.push(*pt);
        }
    }

    // Close and output the line string
    let mut output = LineString::new(output);
    output.close();
    output
}

#[cfg(test)]
mod test {
    use crate::{
        algorithm::{convex_hull::graham_hull, is_convex::IsConvex, GeoNum},
        types::coordinate::Coordinate,
    };

    fn test_convexity<T: GeoNum, Z: GeoNum>(mut initial: Vec<Coordinate<T, Z>>) {
        let hull = graham_hull(&mut initial, false);
        assert!(hull.is_strictly_ccw_convex());
        let hull = graham_hull(&mut initial, true);
        assert!(hull.is_ccw_convex());
    }

    #[test]
    fn test_graham_hull_ccw() {
        let initial = [
            (1.0, 0.0),
            (2.0, 1.0),
            (1.75, 1.1),
            (1.0, 2.0),
            (0.0, 1.0),
            (1.0, 0.0),
        ];
        let initial = initial
            .iter()
            .map(|e| Coordinate::from((e.0, e.1)))
            .collect();
        test_convexity(initial);
    }

    #[test]
    fn graham_hull_test1() {
        let v: Vec<_> = vec![
            (0., 0.),
            (4., 0.),
            (4., 1.),
            (1., 1.),
            (1., 4.),
            (0., 4.),
            (0., 0.),
        ];
        let initial = v.iter().map(|e| Coordinate::from((e.0, e.1))).collect();
        test_convexity(initial);
    }

    #[test]
    fn graham_hull_test2() {
        let v = [
            (0., 10.),
            (1., 1.),
            (10., 0.),
            (1., -1.),
            (0., -10.),
            (-1., -1.),
            (-10., 0.),
            (-1., 1.),
            (0., 10.),
        ];
        let initial = v.iter().map(|e| Coordinate::from((e.0, e.1))).collect();
        test_convexity(initial);
    }
}
