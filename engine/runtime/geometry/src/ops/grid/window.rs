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

/// A point used to test which exterior a hole belongs to.
///
/// This is the centroid of the hole's own vertices, not any single vertex.
/// A hole that survived clipping against the window has, in the common case,
/// at least one vertex sitting exactly on the window boundary (a cut vertex
/// from [`clip_rings_halfplane`]) -- often the very same boundary line an
/// exterior face shares. A point-in-ring test against a boundary point is a
/// degenerate case for the ray-cast rule in [`contains_point`]: it can go
/// either way depending on exactly how the boundary is walked, so probing
/// with a vertex is fragile in exactly the cases clipping produces most
/// often. The centroid is not on the window boundary except by symmetric
/// coincidence, and for the roughly rectilinear pieces this module deals in
/// it lands inside the hole itself.
fn ring_centroid<const N: usize>(ring: &[Corner<N>]) -> [f64; 2] {
    let mut sum = [0.0, 0.0];
    for c in ring {
        sum[0] += c.pos[0];
        sum[1] += c.pos[1];
    }
    let n = ring.len() as f64;
    [sum[0] / n, sum[1] / n]
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
        let probe = ring_centroid(&hole);
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
