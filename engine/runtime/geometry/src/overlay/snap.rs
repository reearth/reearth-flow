//! Vertex snapping for overlay operands.
//!
//! Boundaries that were meant to coincide often do not, by a hair: a shared
//! edge digitized twice, coordinates round-tripped through a lower precision,
//! neighbouring tiles cut from slightly different sources. The overlay backend
//! reads those as real geometry and leaves a sliver gap where the caller wanted
//! one face. Pulling near-coincident vertices onto one position first closes
//! the gap before the boolean ever runs.
//!
//! The adjustment is anchored: a vertex claims every unclaimed vertex within
//! the tolerance and they take *its* position, so no vertex moves further than
//! the tolerance and existing coordinates stay put rather than drifting to a
//! computed average.

use kiddo::{ImmutableKdTree, SquaredEuclidean};

use super::shapes::Shape;

/// Pull the vertices of `shapes` that lie closer together than `tolerance` onto
/// one shared position, in place. A non-positive tolerance snaps nothing.
///
/// Vertex to vertex only: a gap between two edges that have no vertices facing
/// each other is left open, however narrow.
pub(super) fn snap_shapes(shapes: &mut [Shape], tolerance: f64) {
    if tolerance <= 0.0 {
        return;
    }
    let points: Vec<[f64; 2]> = shapes
        .iter()
        .flat_map(|shape| shape.iter().flat_map(|path| path.iter().copied()))
        .collect();
    let Some(snapped) = snapped_positions(&points, tolerance) else {
        return;
    };
    let mut i = 0;
    for shape in shapes {
        for path in shape {
            for coord in path {
                *coord = snapped[i];
                i += 1;
            }
        }
    }
}

/// The position each of `points` snaps to, or `None` when there is nothing to
/// snap. Scanning in index order, a point not yet claimed becomes an anchor and
/// claims every later unclaimed point within `tolerance` of it.
fn snapped_positions(points: &[[f64; 2]], tolerance: f64) -> Option<Vec<[f64; 2]>> {
    let n = points.len();
    if n <= 1 {
        return None;
    }
    // ImmutableKdTree handles degenerate point distributions (many points
    // sharing one axis coordinate, as rectilinear rings do) without panicking,
    // unlike the mutable KdTree.
    let tree: ImmutableKdTree<f64, 2> = ImmutableKdTree::new_from_slice(points);
    let squared_tolerance = tolerance * tolerance;

    let mut snapped = points.to_vec();
    let mut claimed = vec![false; n];
    for i in 0..n {
        if claimed[i] {
            continue;
        }
        for neighbour in tree.within::<SquaredEuclidean>(&points[i], squared_tolerance) {
            let j = neighbour.item as usize;
            if j <= i || claimed[j] || neighbour.distance >= squared_tolerance {
                continue;
            }
            snapped[j] = points[i];
            claimed[j] = true;
        }
    }
    Some(snapped)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn near_coincident_vertices_take_the_anchors_position() {
        let mut shapes = vec![
            vec![vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]],
            vec![vec![[1.0005, 0.0], [2.0, 0.0], [1.0005, 1.0004]]],
        ];
        snap_shapes(&mut shapes, 0.01);

        assert_eq!(shapes[1][0][0], [1.0, 0.0]);
        assert_eq!(shapes[1][0][2], [1.0, 1.0]);
        // Far vertices keep their own position.
        assert_eq!(shapes[1][0][1], [2.0, 0.0]);
    }

    #[test]
    fn vertices_further_apart_than_the_tolerance_stay_put() {
        let original = vec![vec![vec![[0.0, 0.0], [1.0, 0.0]]], vec![vec![[1.5, 0.0]]]];
        let mut shapes = original.clone();
        snap_shapes(&mut shapes, 0.1);
        assert_eq!(shapes, original);
    }

    #[test]
    fn a_non_positive_tolerance_snaps_nothing() {
        let original = vec![vec![vec![
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0 + f64::EPSILON, 0.0],
        ]]];
        let mut shapes = original.clone();
        snap_shapes(&mut shapes, 0.0);
        assert_eq!(shapes, original);
        snap_shapes(&mut shapes, -1.0);
        assert_eq!(shapes, original);
    }

    #[test]
    fn no_vertex_moves_further_than_the_tolerance() {
        // A chain of steps each under the tolerance: anchoring must not walk a
        // vertex along the chain by more than the tolerance.
        let mut shapes = vec![vec![(0..10).map(|i| [i as f64 * 0.004, 0.0]).collect()]];
        let original = shapes.clone();
        snap_shapes(&mut shapes, 0.01);

        for (moved, before) in shapes[0][0].iter().zip(&original[0][0]) {
            let d = ((moved[0] - before[0]).powi(2) + (moved[1] - before[1]).powi(2)).sqrt();
            assert!(d < 0.01, "vertex moved {d} > tolerance");
        }
    }
}
