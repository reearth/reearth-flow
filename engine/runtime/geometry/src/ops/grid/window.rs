//! Clipping to one grid cell: four half-planes, then rings sorted back into
//! faces.

// Nothing outside this module's own tests calls these yet: the grid-division
// op that wires `Window` and `clip_to_window` into cell iteration lands in a
// later task. Drop this once that caller exists.
#![allow(dead_code)]

use super::halfplane::{clip_rings_halfplane, Corner, Edge};

/// One grid cell, as an axis-aligned box in the geometry's own frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Window {
    pub min: [f64; 2],
    pub max: [f64; 2],
}

impl Window {
    pub(crate) fn area(&self) -> f64 {
        (self.max[0] - self.min[0]) * (self.max[1] - self.min[1])
    }
}

/// A clipped face: its exterior ring first, then any hole rings.
#[derive(Clone, Debug)]
pub(crate) struct Face<const N: usize> {
    pub rings: Vec<Vec<Corner<N>>>,
}

/// Twice the signed area of a ring projected on XY, halved. Positive is
/// counter-clockwise, which is Flow's exterior convention.
pub(crate) fn signed_area_xy<const N: usize>(ring: &[Corner<N>]) -> f64 {
    let mut acc = 0.0;
    for i in 0..ring.len() {
        let a = ring[i].pos;
        let b = ring[(i + 1) % ring.len()].pos;
        acc += a[0] * b[1] - b[0] * a[1];
    }
    acc / 2.0
}

/// Net XY area of a set of faces: exteriors minus their holes.
pub(crate) fn faces_area_xy<const N: usize>(faces: &[Face<N>]) -> f64 {
    faces
        .iter()
        .flat_map(|f| f.rings.iter())
        .map(|r| signed_area_xy(r))
        .sum::<f64>()
        .abs()
}

/// Whether `point` lies inside `ring`, by the even-odd ray rule on XY.
fn contains_point<const N: usize>(ring: &[Corner<N>], point: [f64; 2]) -> bool {
    let mut inside = false;
    for i in 0..ring.len() {
        let a = ring[i].pos;
        let b = ring[(i + 1) % ring.len()].pos;
        let crosses = (a[1] > point[1]) != (b[1] > point[1]);
        if crosses {
            let x = a[0] + (point[1] - a[1]) / (b[1] - a[1]) * (b[0] - a[0]);
            if point[0] < x {
                inside = !inside;
            }
        }
    }
    inside
}

/// The arithmetic mean of a ring's vertices.
///
/// This is confined to the ring's bounding box but, for a non-convex ring, is
/// not confined to the ring's interior: a C- or L-shaped ring's centroid can
/// fall in its own notch, outside the ring entirely. It is a candidate probe
/// point, not a guaranteed-interior one -- see [`ring_probe`], which verifies
/// it before use and falls back when it fails.
fn ring_centroid<const N: usize>(ring: &[Corner<N>]) -> [f64; 2] {
    let mut sum = [0.0, 0.0];
    for c in ring {
        sum[0] += c.pos[0];
        sum[1] += c.pos[1];
    }
    let n = ring.len() as f64;
    [sum[0] / n, sum[1] / n]
}

/// A point just inside one edge of `ring`: that edge's midpoint, nudged along
/// its inward normal by `shrink` times the edge's own length.
///
/// `ccw` must be the winding of `ring` (from [`signed_area_xy`]), since the
/// inward side is the left of travel for a counter-clockwise ring and the
/// right for a clockwise one. Returns `None` for a degenerate (zero-length)
/// edge, which contributes no usable normal.
fn edge_inward_offset<const N: usize>(
    ring: &[Corner<N>],
    edge_index: usize,
    shrink: f64,
    ccw: bool,
) -> Option<[f64; 2]> {
    let n = ring.len();
    let a = ring[edge_index].pos;
    let b = ring[(edge_index + 1) % n].pos;
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = (dx * dx + dy * dy).sqrt();
    if len == 0.0 {
        return None;
    }
    let (nx, ny) = if ccw {
        (-dy / len, dx / len)
    } else {
        (dy / len, -dx / len)
    };
    let mid = [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
    let offset = len * shrink;
    Some([mid[0] + nx * offset, mid[1] + ny * offset])
}

/// A point confirmed, by [`contains_point`] against `ring` itself, to lie
/// strictly inside `ring`. Used as the probe in a point-in-ring test against
/// a *different* ring (which exterior a hole belongs to).
///
/// The centroid ([`ring_centroid`]) is tried first: it is exact and cheap,
/// and correct whenever `ring` is convex. It is wrong in general, so it is
/// verified rather than trusted -- when it fails (a concave ring whose
/// centroid lands in its own notch), this falls back to an inward offset from
/// each edge in turn ([`edge_inward_offset`]), at shrinking fractions of that
/// edge's length, until one verifies. Every candidate this function can
/// return has passed the same [`contains_point`] check the caller will apply
/// it with, so "this point is inside `ring`" is not an assumption here, it is
/// checked.
fn ring_probe<const N: usize>(ring: &[Corner<N>]) -> [f64; 2] {
    let centroid = ring_centroid(ring);
    if contains_point(ring, centroid) {
        return centroid;
    }

    let ccw = signed_area_xy(ring) > 0.0;
    for i in 0..ring.len() {
        for &shrink in &[0.25, 0.1, 0.01, 0.001] {
            if let Some(candidate) = edge_inward_offset(ring, i, shrink, ccw) {
                if contains_point(ring, candidate) {
                    return candidate;
                }
            }
        }
    }

    // Every well-formed simple ring this module produces has at least one
    // edge whose inward-offset midpoint verifies as interior, so this is not
    // expected to be reached. Falling back to the (unverified) centroid
    // rather than panicking leaves ownership assignment no worse than it
    // would be without this fallback chain, for whatever degenerate ring got
    // this far.
    centroid
}

/// Clip a face's rings to `window`, returning the surviving faces.
///
/// Rings arrive exterior-first; winding is preserved through the four half-plane
/// passes, so exteriors come out counter-clockwise and holes clockwise, and the
/// two are told apart by the sign of their area rather than by their input
/// position. A hole that ran off the cell edge has already merged into the
/// exterior boundary during clipping and does not reappear here.
pub(crate) fn clip_to_window<const N: usize>(
    rings: Vec<Vec<Corner<N>>>,
    window: &Window,
) -> Vec<Face<N>> {
    let mut current = rings;
    for edge in [
        Edge::MinX(window.min[0]),
        Edge::MaxX(window.max[0]),
        Edge::MinY(window.min[1]),
        Edge::MaxY(window.max[1]),
    ] {
        current = clip_rings_halfplane(current, edge);
        if current.is_empty() {
            return Vec::new();
        }
    }

    let (exteriors, holes): (Vec<_>, Vec<_>) = current
        .into_iter()
        .filter(|r| r.len() >= 3)
        .partition(|r| signed_area_xy(r) > 0.0);

    let mut faces: Vec<Face<N>> = exteriors
        .into_iter()
        .map(|r| Face { rings: vec![r] })
        .collect();

    // Each hole belongs to the exterior that contains it: a point-in-ring test
    // per hole. This batch may hold rings from more than one original face, so
    // "exactly one exterior survived" does not mean "this hole belongs to it" --
    // that exterior could belong to a different original face than the hole,
    // while the hole's own exterior was filtered out above (e.g. clipped to a
    // degenerate sliver under 3 corners). Always testing containment, rather
    // than short-circuiting on a single surviving exterior, keeps a hole from
    // being attached to the wrong face. A hole matching no exterior in this
    // batch contributes nothing and is dropped.
    for hole in holes {
        if hole.is_empty() {
            continue;
        }
        let probe = ring_probe(&hole);
        let owner = faces
            .iter()
            .position(|f| contains_point(&f.rings[0], probe));
        if let Some(i) = owner {
            faces[i].rings.push(hole);
        }
    }

    faces.retain(|f| signed_area_xy(&f.rings[0]).abs() > 0.0);
    faces
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::grid::halfplane::Corner;

    fn ring2(pts: &[(f64, f64)]) -> Vec<Corner<2>> {
        pts.iter()
            .map(|&(x, y)| Corner {
                pos: [x, y],
                uv: None,
            })
            .collect()
    }

    fn unit_window() -> Window {
        Window {
            min: [0.0, 0.0],
            max: [1.0, 1.0],
        }
    }

    #[test]
    fn face_covering_the_window_yields_exactly_the_window() {
        let big = ring2(&[(-5.0, -5.0), (5.0, -5.0), (5.0, 5.0), (-5.0, 5.0)]);
        let faces = clip_to_window(vec![big], &unit_window());
        assert_eq!(faces.len(), 1);
        assert_eq!(faces[0].rings.len(), 1);
        assert!((faces_area_xy(&faces) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn face_outside_the_window_yields_nothing() {
        let away = ring2(&[(10.0, 10.0), (11.0, 10.0), (11.0, 11.0), (10.0, 11.0)]);
        assert!(clip_to_window(vec![away], &unit_window()).is_empty());
    }

    #[test]
    fn half_covering_face_yields_half_the_area() {
        let half = ring2(&[(0.0, 0.0), (0.5, 0.0), (0.5, 1.0), (0.0, 1.0)]);
        let faces = clip_to_window(vec![half], &unit_window());
        assert!((faces_area_xy(&faces) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn hole_wholly_inside_the_window_is_kept_as_a_hole() {
        // Exterior CCW, hole CW, both inside the window.
        let exterior = ring2(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
        let hole = ring2(&[(0.25, 0.25), (0.25, 0.75), (0.75, 0.75), (0.75, 0.25)]);
        let faces = clip_to_window(vec![exterior, hole], &unit_window());
        assert_eq!(faces.len(), 1);
        assert_eq!(faces[0].rings.len(), 2, "exterior plus one hole");
        // 1.0 total minus a 0.5 x 0.5 hole.
        assert!((faces_area_xy(&faces) - 0.75).abs() < 1e-12);
    }

    #[test]
    fn hole_straddling_the_window_edge_stays_related_to_its_exterior() {
        // The hole runs off the left edge, so after clipping it merges into the
        // exterior boundary rather than surviving as a free-floating ring.
        let exterior = ring2(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
        let hole = ring2(&[(-0.5, 0.25), (-0.5, 0.75), (0.5, 0.75), (0.5, 0.25)]);
        let faces = clip_to_window(vec![exterior, hole], &unit_window());
        // Area is the unit square minus the part of the hole inside it (0.5 x 0.5).
        assert!(
            (faces_area_xy(&faces) - 0.75).abs() < 1e-9,
            "area was {}",
            faces_area_xy(&faces)
        );
    }

    /// A C-shaped hole (spine on the left, arms top and bottom, notch opening
    /// right), wound clockwise as a hole must be. Its centroid is `(0.55, 0.5)`,
    /// which sits in the notch -- outside the shape -- so this is the fixture
    /// that defeats a plain-centroid probe and exercises the inward-offset
    /// fallback in `ring_probe`.
    ///
    /// Bounding box area is 0.6 x 0.6 = 0.36; the notch removed is 0.4 x 0.2 =
    /// 0.08; shape area is 0.36 - 0.08 = 0.28.
    fn concave_c_hole() -> Vec<Corner<2>> {
        ring2(&[
            (0.8, 0.8),
            (0.8, 0.6),
            (0.4, 0.6),
            (0.4, 0.4),
            (0.8, 0.4),
            (0.8, 0.2),
            (0.2, 0.2),
            (0.2, 0.8),
        ])
    }

    #[test]
    fn concave_c_hole_centroid_is_outside_the_hole() {
        // Confirms the fixture actually exercises the fallback: a plain
        // centroid probe would misjudge this hole's own shape, let alone which
        // exterior it belongs to.
        let hole = concave_c_hole();
        assert!(signed_area_xy(&hole) < 0.0, "fixture hole must be CW");
        assert!(!contains_point(&hole, ring_centroid(&hole)));
    }

    #[test]
    fn concave_hole_nested_in_one_exterior_is_attributed_to_it() {
        let exterior = ring2(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
        let hole = concave_c_hole();
        let faces = clip_to_window(vec![exterior, hole], &unit_window());
        assert_eq!(faces.len(), 1);
        assert_eq!(faces[0].rings.len(), 2, "exterior plus the concave hole");
        // 1.0 total minus the 0.28 hole.
        assert!(
            (faces_area_xy(&faces) - 0.72).abs() < 1e-12,
            "area was {}",
            faces_area_xy(&faces)
        );
    }

    #[test]
    fn concave_hole_is_not_attributed_to_an_unrelated_exterior() {
        // Two disjoint exteriors in one batch: A holds the concave hole, B is
        // a plain square far away. A robust probe must not attach the hole to
        // B just because B also happens to be a surviving exterior.
        let exterior_a = ring2(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
        let hole = concave_c_hole();
        let exterior_b = ring2(&[(2.0, 0.0), (3.0, 0.0), (3.0, 1.0), (2.0, 1.0)]);
        let window = Window {
            min: [-1.0, -1.0],
            max: [4.0, 2.0],
        };
        let faces = clip_to_window(vec![exterior_a, hole, exterior_b], &window);
        assert_eq!(faces.len(), 2);

        let with_hole = faces
            .iter()
            .find(|f| f.rings.len() == 2)
            .expect("one face must carry the hole");
        let without_hole = faces
            .iter()
            .find(|f| f.rings.len() == 1)
            .expect("the other face must carry no hole");
        assert!(
            with_hole.rings[0]
                .iter()
                .all(|c| c.pos[0] >= 0.0 && c.pos[0] <= 1.0),
            "the hole must land on exterior A, not B"
        );
        assert!((signed_area_xy(&without_hole.rings[0]) - 1.0).abs() < 1e-12);

        // A's net area (1.0 - 0.28) plus B's full 1.0.
        assert!(
            (faces_area_xy(&faces) - 1.72).abs() < 1e-12,
            "area was {}",
            faces_area_xy(&faces)
        );
    }

    #[test]
    fn concave_face_cut_into_two_pieces_yields_two_faces() {
        // A "U" opening to the right: a spine at x in [0, 0.2] joining two arms
        // at y in [0, 0.2] and y in [0.8, 1.0], each reaching to x = 1.
        let u = ring2(&[
            (0.0, 0.0),
            (1.0, 0.0),
            (1.0, 0.2),
            (0.2, 0.2),
            (0.2, 0.8),
            (1.0, 0.8),
            (1.0, 1.0),
            (0.0, 1.0),
        ]);

        // A window containing the spine keeps the arms joined.
        let joined = Window {
            min: [0.0, 0.0],
            max: [0.5, 1.0],
        };
        assert_eq!(
            clip_to_window(vec![u.clone()], &joined).len(),
            1,
            "the spine still joins the arms"
        );

        // A window to the right of the spine severs them.
        let severed = Window {
            min: [0.5, 0.0],
            max: [1.0, 1.0],
        };
        let faces = clip_to_window(vec![u], &severed);
        assert_eq!(faces.len(), 2, "severed arms must be two faces");
        // Each arm is 0.5 wide by 0.2 tall.
        assert!((faces_area_xy(&faces) - 0.2).abs() < 1e-12);
    }

    #[test]
    fn signed_area_sign_distinguishes_exterior_from_hole() {
        let ccw = ring2(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
        let cw = ring2(&[(0.0, 0.0), (0.0, 1.0), (1.0, 1.0), (1.0, 0.0)]);
        assert!(signed_area_xy(&ccw) > 0.0);
        assert!(signed_area_xy(&cw) < 0.0);
    }
}
