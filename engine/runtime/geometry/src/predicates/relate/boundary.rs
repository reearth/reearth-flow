//! Union-boundary extraction for mesh leaves.
//!
//! A mesh cannot feed the geometry graph face by face: adjacent faces share
//! whole edges, which the JTS graph would label as boundary even though they
//! are interior to the union (the same fact the shared-edge refinement in
//! [`position`](crate::predicates::position) encodes). This pre-pass reduces a
//! mesh to the directed boundary rings of its face union:
//!
//! 1. collect every directed ring edge of every face (holes included);
//! 2. cancel pairs that occur in both directions — internal shared edges;
//! 3. stitch the survivors into closed loops.
//!
//! Directions are preserved throughout, so with Flow's validated winding
//! (exteriors CCW, holes CW — face interior locally *left* of every directed
//! edge) each output ring has the union interior on its left, and can be fed
//! to the graph with fixed `On = Boundary, Left = Inside, Right = Outside`
//! labels — no winding recomputation.
//!
//! Assumes a valid mesh (non-overlapping face interiors, consistent winding),
//! like the rest of the predicates; on invalid input where a walk cannot
//! close, the chain is force-closed with a synthetic edge back to its start
//! rather than panicking.

use std::collections::BTreeMap;

use crate::predicates::view::AreaView;

/// A coordinate as its bit pattern, usable as an exact, `Ord` map key.
type BitCoord = [u64; 2];

fn bits(c: [f64; 2]) -> BitCoord {
    // Fold -0.0 into 0.0 so the two bit patterns of numeric zero cancel.
    let f = |v: f64| {
        if v == 0.0 {
            0.0f64.to_bits()
        } else {
            v.to_bits()
        }
    };
    [f(c[0]), f(c[1])]
}

/// The union boundary of an areal leaf, as closed directed rings
/// (first == last) with the union interior locally left of every edge.
pub(crate) fn union_boundary_rings(area: &AreaView<'_>) -> Vec<Vec<[f64; 2]>> {
    // Multiset of surviving directed edges, opposite pairs cancelled.
    let mut counts: BTreeMap<(BitCoord, BitCoord), usize> = BTreeMap::new();
    let mut coord_of: BTreeMap<BitCoord, [f64; 2]> = BTreeMap::new();
    for (start, end) in area.edges() {
        if start == end {
            continue;
        }
        let key = (bits(start), bits(end));
        let reverse = (key.1, key.0);
        match counts.get_mut(&reverse) {
            Some(count) => {
                *count -= 1;
                if *count == 0 {
                    counts.remove(&reverse);
                }
            }
            None => {
                *counts.entry(key).or_insert(0) += 1;
                coord_of.insert(key.0, start);
                coord_of.insert(key.1, end);
            }
        }
    }

    // Outgoing-edge multimap over the survivors.
    let mut outgoing: BTreeMap<BitCoord, Vec<BitCoord>> = BTreeMap::new();
    for ((start, end), count) in counts {
        outgoing
            .entry(start)
            .or_default()
            .extend(std::iter::repeat_n(end, count));
    }

    // Stitch loops by walking outgoing edges. Whenever the walk revisits a
    // vertex already on the current path, the piece since that visit is a
    // closed loop: split it off as its own ring. This keeps every output ring
    // simple — a walk through a pinch vertex yields one ring per wedge, never
    // a figure eight.
    let mut rings = Vec::new();
    while let Some((&start, _)) = outgoing.iter().next() {
        let mut path: Vec<BitCoord> = vec![start];
        let mut position_in_path: BTreeMap<BitCoord, usize> = BTreeMap::from([(start, 0)]);
        loop {
            let current = *path.last().expect("path starts non-empty");
            let Some(nexts) = outgoing.get_mut(&current) else {
                // No outgoing edge. A leftover chain means invalid input
                // (unmatched edges): force-close it rather than panic.
                if path.len() > 1 {
                    let mut ring: Vec<[f64; 2]> = path.iter().map(|k| coord_of[k]).collect();
                    ring.push(ring[0]);
                    rings.push(ring);
                }
                break;
            };
            let next = nexts.pop().expect("empty entries are removed eagerly");
            if nexts.is_empty() {
                outgoing.remove(&current);
            }
            match position_in_path.get(&next) {
                Some(&p) => {
                    // Closed a loop back to path[p]: emit path[p..] + closer.
                    let mut ring: Vec<[f64; 2]> = path[p..].iter().map(|k| coord_of[k]).collect();
                    ring.push(coord_of[&next]);
                    rings.push(ring);
                    for popped in path.drain(p + 1..) {
                        position_in_path.remove(&popped);
                    }
                }
                None => {
                    position_in_path.insert(next, path.len());
                    path.push(next);
                }
            }
        }
    }
    rings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use crate::polygon_mesh::PolygonMesh2D;
    use crate::triangular_mesh::TriangularMesh2D;
    use pretty_assertions::assert_eq;

    fn e() -> CoordinateFrame {
        CoordinateFrame::Euclidean
    }

    /// Twice the signed area of a closed ring (positive = CCW).
    fn doubled_signed_area(ring: &[[f64; 2]]) -> f64 {
        ring.windows(2)
            .map(|e| e[0][0] * e[1][1] - e[1][0] * e[0][1])
            .sum()
    }

    fn ring_edges(ring: &[[f64; 2]]) -> Vec<([f64; 2], [f64; 2])> {
        ring.windows(2).map(|e| (e[0], e[1])).collect()
    }

    #[test]
    fn two_quads_dissolve_to_one_ring() {
        // Two quads sharing the edge x = 2.
        let mesh = PolygonMesh2D::from_parts(
            e(),
            vec![
                [0.0, 0.0],
                [2.0, 0.0],
                [2.0, 2.0],
                [0.0, 2.0],
                [4.0, 0.0],
                [4.0, 2.0],
            ],
            vec![vec![0u32, 1, 2, 3], vec![1, 4, 5, 2]],
        )
        .unwrap();
        let rings = union_boundary_rings(&AreaView::from_polygon_mesh(&mesh));

        assert_eq!(rings.len(), 1);
        let ring = &rings[0];
        assert_eq!(ring.first(), ring.last());
        assert_eq!(ring.len(), 7); // 6 boundary vertices, stored closed
        assert!(doubled_signed_area(ring) > 0.0); // interior on the left
        let edges = ring_edges(ring);
        // The shared edge is gone in both directions.
        assert!(!edges.contains(&([2.0, 0.0], [2.0, 2.0])));
        assert!(!edges.contains(&([2.0, 2.0], [2.0, 0.0])));
        // Rim edges keep their face direction.
        assert!(edges.contains(&([0.0, 0.0], [2.0, 0.0])));
        assert!(edges.contains(&([2.0, 0.0], [4.0, 0.0])));
    }

    #[test]
    fn triangulated_square_dissolves_to_square() {
        let mesh = TriangularMesh2D::from_parts(
            e(),
            vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0]],
            [0u32, 1, 2, 0, 2, 3],
        )
        .unwrap();
        let rings = union_boundary_rings(&AreaView::from_triangular_mesh(&mesh));

        assert_eq!(rings.len(), 1);
        assert_eq!(rings[0].len(), 5);
        assert_eq!(doubled_signed_area(&rings[0]), 8.0);
    }

    #[test]
    fn quad_ring_yields_outer_and_hole_rings() {
        // A 3x3 grid of unit quads with the center quad missing: the union is
        // a square annulus whose boundary is an outer CCW and an inner CW ring.
        let mut vertices = Vec::new();
        for y in 0..4 {
            for x in 0..4 {
                vertices.push([x as f64, y as f64]);
            }
        }
        let index = |x: u32, y: u32| y * 4 + x;
        let mut faces = Vec::new();
        for y in 0..3u32 {
            for x in 0..3u32 {
                if x == 1 && y == 1 {
                    continue;
                }
                faces.push(vec![
                    index(x, y),
                    index(x + 1, y),
                    index(x + 1, y + 1),
                    index(x, y + 1),
                ]);
            }
        }
        let mesh = PolygonMesh2D::from_parts(e(), vertices, faces).unwrap();
        let mut rings = union_boundary_rings(&AreaView::from_polygon_mesh(&mesh));
        rings.sort_by_key(|r| r.len());

        assert_eq!(rings.len(), 2);
        // Inner ring: the hole, wound CW (union interior still on the left).
        assert_eq!(rings[0].len(), 5);
        assert_eq!(doubled_signed_area(&rings[0]), -2.0);
        // Outer ring: CCW around the full 3x3 square.
        assert_eq!(rings[1].len(), 13);
        assert_eq!(doubled_signed_area(&rings[1]), 18.0);
    }

    #[test]
    fn pinch_vertex_yields_two_rings() {
        // Two triangles touching only at (1, 1).
        let mesh = TriangularMesh2D::from_parts(
            e(),
            vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [2.0, 2.0], [1.0, 2.0]],
            [0u32, 1, 2, 2, 3, 4],
        )
        .unwrap();
        let rings = union_boundary_rings(&AreaView::from_triangular_mesh(&mesh));

        assert_eq!(rings.len(), 2);
        for ring in &rings {
            assert_eq!(ring.len(), 4);
            assert!(doubled_signed_area(ring) > 0.0);
        }
    }

    #[test]
    fn face_hole_ring_passes_through() {
        // One quad with an unshared hole: both rings survive verbatim.
        let mesh = PolygonMesh2D::from_raw_parts(
            e(),
            vec![
                [0.0, 0.0],
                [6.0, 0.0],
                [6.0, 6.0],
                [0.0, 6.0],
                [2.0, 2.0],
                [2.0, 4.0],
                [4.0, 4.0],
                [4.0, 2.0],
            ],
            vec![0, 1, 2, 3, 0, 4, 5, 6, 7, 4],
            vec![],
            vec![5],
        )
        .unwrap();
        let mut rings = union_boundary_rings(&AreaView::from_polygon_mesh(&mesh));
        rings.sort_by_key(|r| doubled_signed_area(r) as i64);

        assert_eq!(rings.len(), 2);
        assert_eq!(doubled_signed_area(&rings[0]), -8.0); // hole, CW
        assert_eq!(doubled_signed_area(&rings[1]), 72.0); // exterior, CCW
    }
}
