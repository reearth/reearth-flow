use kiddo::{KdTree, SquaredEuclidean};

use crate::types::{coordinate::Coordinate, coordnum::CoordFloat};

/// Glue vertices that are closer than the given tolerance.
/// No points will be moved more than the tolerance distance.
/// Each pair of vertices will be at least the tolerance distance apart after this operation.
pub fn glue_vertices_closer_than<T: CoordFloat + From<Z>, Z: CoordFloat>(
    tolerance: T,
    mut vertices: Vec<&mut Coordinate<T, Z>>,
) {
    let n = vertices.len();
    if n <= 1 {
        return;
    }

    let tol_f64 = tolerance.to_f64().unwrap();
    let sq_tol = tol_f64 * tol_f64;

    // Build a k-d tree over the vertex positions (2D: x, y)
    let mut tree: KdTree<f64, 2> = KdTree::new();
    for (i, v) in vertices.iter().enumerate() {
        let pt = [v.x.to_f64().unwrap(), v.y.to_f64().unwrap()];
        tree.add(&pt, i as u64);
    }

    let mut updated = vec![false; n];

    for i in 0..n {
        if updated[i] {
            continue;
        }
        let vi = *vertices[i];
        let query = [vi.x.to_f64().unwrap(), vi.y.to_f64().unwrap()];

        // Find all points within tolerance (squared euclidean distance < sq_tol)
        let neighbors = tree.within::<SquaredEuclidean>(&query, sq_tol);

        for nb in neighbors {
            let j = nb.item as usize;
            if j <= i || updated[j] {
                continue;
            }
            // Verify with full norm (includes z) to match original semantics
            let vj = *vertices[j];
            if (vi - vj).norm() < tolerance {
                *vertices[j] = vi;
                updated[j] = true;
            }
        }
    }
}
