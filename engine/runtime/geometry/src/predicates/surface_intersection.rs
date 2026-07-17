//! Global self-intersection of face sets: the face-vs-face walkers behind
//! surface and solid validation, kept here so overlay and CSG work can reuse
//! them. The 2D walker asks whether any two faces of one areal view overlap;
//! the 3D walker asks which faces of one or more triangulated surfaces
//! intersect beyond their shared corners and edges.

use std::collections::{BTreeSet, HashMap};

use rstar::AABB;

use super::edge_set::for_each_candidate_pair;
use super::kernel::{segment_intersection, CoordPos, SegmentIntersection};
use super::kernel3d::{
    classify_triangle, triangles_overlap_beyond_contact, TriangleContact, TriangleShape,
};
use super::position::{face_interior_point, face_position};
use super::view::AreaView;
use super::view3d::TriangleSet;

/// A conflict between two different faces of one 2D areal view.
#[derive(PartialEq, Clone, Copy, Debug)]
pub(crate) enum FaceConflict2D {
    /// Boundaries of two faces cross properly at this constructed point.
    Crossing([f64; 2]),
    /// This face's interior sample lies strictly inside another face.
    Contained {
        /// The contained face's index in the view.
        face: usize,
    },
}

/// Scan an areal view's faces pairwise for interior overlaps, emitting each
/// conflict found: a proper edge crossing between two different faces, or a
/// face whose interior sample lies strictly inside another face. Boundary
/// touches (shared edges, vertex touches, collinear overlaps) are allowed.
///
/// # Precondition
///
/// Coordinates are finite.
pub(crate) fn face_overlap_conflicts_2d(view: &AreaView<'_>, mut emit: impl FnMut(FaceConflict2D)) {
    let mut edges: Vec<(usize, [f64; 2], [f64; 2])> = Vec::new();
    for (f, face) in view.faces().enumerate() {
        for (a, b) in face.edges() {
            edges.push((f, a, b));
        }
    }
    let envelopes: Vec<AABB<[f64; 2]>> = edges
        .iter()
        .map(|&(_, a, b)| AABB::from_corners(a, b))
        .collect();
    for_each_candidate_pair(&envelopes, |i, j| {
        let (fi, a1, b1) = edges[i];
        let (fj, a2, b2) = edges[j];
        if fi == fj {
            return;
        }
        if let Some(SegmentIntersection::SinglePoint {
            intersection,
            is_proper: true,
        }) = segment_intersection(a1, b1, a2, b2)
        {
            emit(FaceConflict2D::Crossing(intersection));
        }
    });

    let n_faces = view.num_faces();
    let face_boxes: Vec<AABB<[f64; 2]>> = (0..n_faces)
        .map(|f| {
            let mut lo = [f64::INFINITY; 2];
            let mut hi = [f64::NEG_INFINITY; 2];
            for ring in view.face(f).rings() {
                for c in ring.coords() {
                    for k in 0..2 {
                        lo[k] = lo[k].min(c[k]);
                        hi[k] = hi[k].max(c[k]);
                    }
                }
            }
            if lo[0] > hi[0] {
                AABB::from_corners([0.0; 2], [0.0; 2])
            } else {
                AABB::from_corners(lo, hi)
            }
        })
        .collect();
    // One interior sample per face, computed on first use.
    let mut samples: Vec<Option<Option<[f64; 2]>>> = vec![None; n_faces];
    for_each_candidate_pair(&face_boxes, |i, j| {
        for (inner, outer) in [(i, j), (j, i)] {
            let sample =
                *samples[inner].get_or_insert_with(|| face_interior_point(view.face(inner)));
            if let Some(p) = sample {
                if face_position(p, view.face(outer)) == CoordPos::Inside {
                    emit(FaceConflict2D::Contained { face: inner });
                }
            }
        }
    });
}

/// One proper triangle of one surface's triangulation, with its provenance
/// and canonicalized corner ids.
struct SurfaceTriangle {
    surface: usize,
    face: usize,
    corners: [[f64; 3]; 3],
    canon: [u32; 3],
}

/// The `(surface, face)` pairs whose interiors intersect across one or more
/// triangulated surfaces: a mesh is one surface; a solid passes one set per
/// shell, so cross-shell pairs are covered. Faces touching only at shared
/// corners or shared edges (by coordinate) do not conflict; anything more
/// does, including a face touching another away from any shared corner.
/// Non-planar faces are compared through their best-fit-plane triangulation,
/// the same approximation the triangulating operations make. Degenerate
/// triangles are skipped (degeneracy is its own check).
///
/// Triangle corners are canonicalized by coordinate bit pattern (`-0.0`
/// normalized to `+0.0`) across all surfaces' pools, so duplicate pool
/// vertices and separate shell pools share ids.
///
/// # Precondition
///
/// Coordinates are finite.
pub(crate) fn intersecting_faces_3d(surfaces: &[&TriangleSet<'_>]) -> BTreeSet<(usize, usize)> {
    let mut canon_ids: HashMap<[u64; 3], u32> = HashMap::new();
    let mut canon_of = |p: [f64; 3]| -> u32 {
        let key = p.map(|x| (x + 0.0).to_bits());
        let next = canon_ids.len() as u32;
        *canon_ids.entry(key).or_insert(next)
    };
    let mut tris: Vec<SurfaceTriangle> = Vec::new();
    for (si, set) in surfaces.iter().enumerate() {
        for i in 0..set.len() {
            let corners = set.triangle(i);
            if !matches!(classify_triangle(corners), TriangleShape::Proper) {
                continue;
            }
            tris.push(SurfaceTriangle {
                surface: si,
                face: set.face_of(i),
                corners,
                canon: corners.map(&mut canon_of),
            });
        }
    }
    let envelopes: Vec<AABB<[f64; 3]>> = tris
        .iter()
        .map(|t| {
            let mut lo = t.corners[0];
            let mut hi = t.corners[0];
            for p in &t.corners[1..] {
                for k in 0..3 {
                    lo[k] = lo[k].min(p[k]);
                    hi[k] = hi[k].max(p[k]);
                }
            }
            AABB::from_corners(lo, hi)
        })
        .collect();

    let mut conflicts: BTreeSet<(usize, usize)> = BTreeSet::new();
    for_each_candidate_pair(&envelopes, |i, j| {
        let (t, s) = (&tris[i], &tris[j]);
        if (t.surface, t.face) == (s.surface, s.face) {
            return;
        }
        if triangles_conflict(t, s) {
            conflicts.insert((t.surface, t.face));
            conflicts.insert((s.surface, s.face));
        }
    });
    conflicts
}

/// Whether two proper triangles from different faces intersect beyond their
/// shared corners (identical triangles always conflict).
fn triangles_conflict(t: &SurfaceTriangle, s: &SurfaceTriangle) -> bool {
    let mut shared_t: Vec<usize> = Vec::with_capacity(3);
    let mut shared_s: Vec<usize> = Vec::with_capacity(3);
    for k in 0..3 {
        if let Some(l) = (0..3).find(|&l| s.canon[l] == t.canon[k]) {
            shared_t.push(k);
            shared_s.push(l);
        }
    }
    let other =
        |shared: &[usize]| -> Vec<usize> { (0..3).filter(|k| !shared.contains(k)).collect() };
    let contact = match shared_t.len() {
        0 => TriangleContact::None,
        1 => {
            let (tr, sr) = (other(&shared_t), other(&shared_s));
            TriangleContact::Vertex {
                v: t.corners[shared_t[0]],
                t_rest: [t.corners[tr[0]], t.corners[tr[1]]],
                s_rest: [s.corners[sr[0]], s.corners[sr[1]]],
            }
        }
        2 => TriangleContact::Edge {
            a: t.corners[shared_t[0]],
            b: t.corners[shared_t[1]],
            t_far: t.corners[other(&shared_t)[0]],
            s_far: s.corners[other(&shared_s)[0]],
        },
        _ => return true,
    };
    triangles_overlap_beyond_contact(t.corners, s.corners, &contact)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use crate::polygon_mesh::PolygonMesh2D;
    use crate::triangular_mesh::TriangularMesh3D;

    fn e() -> CoordinateFrame {
        CoordinateFrame::Euclidean
    }

    fn conflicts_of(mesh: &PolygonMesh2D) -> Vec<FaceConflict2D> {
        let view = AreaView::from_polygon_mesh(mesh);
        let mut out = Vec::new();
        face_overlap_conflicts_2d(&view, |c| out.push(c));
        out
    }

    #[test]
    fn adjacent_2d_faces_do_not_conflict() {
        // Two unit quads sharing an edge.
        let mesh = PolygonMesh2D::from_parts(
            e(),
            vec![
                [0.0, 0.0],
                [1.0, 0.0],
                [1.0, 1.0],
                [0.0, 1.0],
                [2.0, 0.0],
                [2.0, 1.0],
            ],
            [vec![0u32, 1, 2, 3], vec![1, 4, 5, 2]],
        )
        .unwrap();
        assert_eq!(conflicts_of(&mesh), Vec::new());
    }

    #[test]
    fn crossing_2d_faces_conflict() {
        // Two quads whose boundaries cross.
        let mesh = PolygonMesh2D::from_parts(
            e(),
            vec![
                [0.0, 0.0],
                [2.0, 0.0],
                [2.0, 2.0],
                [0.0, 2.0],
                [1.0, 1.0],
                [3.0, 1.0],
                [3.0, 3.0],
                [1.0, 3.0],
            ],
            [vec![0u32, 1, 2, 3], vec![4, 5, 6, 7]],
        )
        .unwrap();
        let out = conflicts_of(&mesh);
        assert!(out.iter().any(|c| matches!(c, FaceConflict2D::Crossing(_))));
    }

    #[test]
    fn contained_2d_face_conflicts_without_crossing() {
        // A small quad strictly inside a big one.
        let mesh = PolygonMesh2D::from_parts(
            e(),
            vec![
                [0.0, 0.0],
                [4.0, 0.0],
                [4.0, 4.0],
                [0.0, 4.0],
                [1.0, 1.0],
                [2.0, 1.0],
                [2.0, 2.0],
                [1.0, 2.0],
            ],
            [vec![0u32, 1, 2, 3], vec![4, 5, 6, 7]],
        )
        .unwrap();
        let out = conflicts_of(&mesh);
        assert!(out
            .iter()
            .any(|c| matches!(c, FaceConflict2D::Contained { face: 1 })));
        assert!(!out.iter().any(|c| matches!(c, FaceConflict2D::Crossing(_))));
    }

    fn tri_mesh(vertices: Vec<[f64; 3]>, indices: Vec<u32>) -> TriangularMesh3D {
        TriangularMesh3D::from_parts(e(), vertices, indices).unwrap()
    }

    #[test]
    fn piercing_shells_conflict_across_surfaces() {
        // One triangle in the plane z = 0, another passing through it.
        let flat = tri_mesh(
            vec![[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [0.0, 4.0, 0.0]],
            vec![0, 1, 2],
        );
        let pierce = tri_mesh(
            vec![[1.0, 1.0, -1.0], [1.0, 1.0, 1.0], [3.0, 3.0, 1.0]],
            vec![0, 1, 2],
        );
        let sets = [
            TriangleSet::from_triangular_data(flat.data()),
            TriangleSet::from_triangular_data(pierce.data()),
        ];
        let refs: Vec<&TriangleSet> = sets.iter().collect();
        let conflicts = intersecting_faces_3d(&refs);
        assert_eq!(
            conflicts.into_iter().collect::<Vec<_>>(),
            vec![(0, 0), (1, 0)]
        );
    }

    #[test]
    fn shells_sharing_a_corner_coordinate_do_not_conflict() {
        // Separate pools; the shared corner matches only by coordinate.
        let a = tri_mesh(
            vec![[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [0.0, 4.0, 0.0]],
            vec![0, 1, 2],
        );
        let b = tri_mesh(
            vec![[0.0, 0.0, 0.0], [-4.0, 0.0, 1.0], [0.0, -4.0, 1.0]],
            vec![0, 1, 2],
        );
        let sets = [
            TriangleSet::from_triangular_data(a.data()),
            TriangleSet::from_triangular_data(b.data()),
        ];
        let refs: Vec<&TriangleSet> = sets.iter().collect();
        assert!(intersecting_faces_3d(&refs).is_empty());
    }

    #[test]
    fn coherent_closed_surface_has_no_conflicts() {
        // A tetrahedron: every face pair shares an edge, none overlap.
        let tetra = tri_mesh(
            vec![
                [0.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [0.0, 4.0, 0.0],
                [0.0, 0.0, 4.0],
            ],
            vec![0, 2, 1, 0, 1, 3, 1, 2, 3, 2, 0, 3],
        );
        let set = TriangleSet::from_triangular_data(tetra.data());
        assert!(intersecting_faces_3d(&[&set]).is_empty());
    }
}
