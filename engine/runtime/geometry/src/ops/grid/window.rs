//! Clipping to one grid cell: four half-planes, then rings sorted back into
//! faces.

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
///
/// The shoelace is accumulated over each vertex's offset from the ring's
/// first vertex, not over its absolute coordinate. The two are algebraically
/// identical -- a translation does not change an area -- but numerically they
/// are not, and the difference decides whether this op works on real data.
/// On absolute coordinates each cross product scales with the *coordinates*
/// while their sum scales with the *area*, so at a projected CRS (Japan's
/// Plane Rectangular system runs to ~1.5e5 m; a UTM northing to ~4e6) a unit
/// cell's terms are ~1e10 to ~1e13 apart from the 1.0 they sum to, and the
/// rounding left over swamps `COVERAGE_TOLERANCE`. Cells that exactly fill
/// their window then measure short of it, come back `Partial`, and are
/// dropped under `completeCellsOnly`. Translating first makes every term the
/// size of the ring itself, so the precision follows the ring's extent rather
/// than its distance from the coordinate origin.
pub(crate) fn signed_area_xy<const N: usize>(ring: &[Corner<N>]) -> f64 {
    let Some(origin) = ring.first().map(|c| c.pos) else {
        return 0.0;
    };
    let mut acc = 0.0;
    for i in 0..ring.len() {
        let a = ring[i].pos;
        let b = ring[(i + 1) % ring.len()].pos;
        acc += (a[0] - origin[0]) * (b[1] - origin[1]) - (b[0] - origin[0]) * (a[1] - origin[1]);
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
///
/// This is only sound for a `point` strictly inside or strictly outside
/// `ring`. For a `point` exactly on one of `ring`'s edges the result is not
/// well-defined: this is a known property of the even-odd ray-cast rule
/// (PNPOLY), whose answer for an on-boundary point depends on which edges
/// happen to register a crossing, not on any principled inside/outside call.
/// Every caller in this module is responsible for only ever probing with a
/// point already confirmed strictly interior to *some* ring -- see
/// [`ring_probe`], which exists specifically to avoid ever handing this an
/// on-edge point.
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

/// Twice the signed area of the 2D triangle `a, b, c`, matching the sign
/// convention of [`signed_area_xy`]: positive when `a -> b -> c` turns
/// counter-clockwise, negative when clockwise, zero when collinear.
fn tri_cross(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

/// Whether `p` lies inside or on the boundary of triangle `a, b, c` (either
/// winding). Used only to test whether some *other* ring vertex intrudes on a
/// candidate ear triangle, so a boundary touch (a degenerate, e.g. duplicate,
/// vertex) is treated as intrusion -- conservative, since rejecting a
/// borderline ear just means trying the next vertex, not losing correctness.
fn point_in_or_on_triangle(p: [f64; 2], a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> bool {
    let d1 = tri_cross(a, b, p);
    let d2 = tri_cross(b, c, p);
    let d3 = tri_cross(c, a, p);
    let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_neg && has_pos)
}

/// A point strictly inside `ring`, found as the centroid of one of its ears.
///
/// An "ear" is a vertex whose triangle with its two ring neighbours (a) turns
/// the same way the ring itself winds, and (b) contains no other vertex of
/// the ring. For a simple (non-self-intersecting) polygon this guarantees the
/// whole triangle -- not just its three corners -- lies inside the polygon,
/// which is the standard fact ear-clipping triangulation relies on: no other
/// edge can cross into the triangle without a vertex of that edge first
/// landing inside it, which condition (b) already rules out. The triangle's
/// centroid then sits strictly inside the ring, away from every edge -- unlike
/// a probe placed on the ring's own boundary, which is not a case
/// [`contains_point`]'s ray cast handles consistently (see the doc comment on
/// [`contains_point`]'s call sites in `clip_to_window`).
///
/// The two-ears theorem guarantees at least one such vertex exists for every
/// simple polygon with three or more corners, so this returns `Some` for
/// every well-formed ring this module's clipping produces. It returns `None`
/// only for a degenerate ring: fewer than three corners, or one where no
/// vertex's turn is a strict convex turn (e.g. all corners collinear, so
/// every triangle has zero area and no "ear" exists to speak of) --
/// exhaustively checked, not assumed.
fn ring_probe<const N: usize>(ring: &[Corner<N>]) -> Option<[f64; 2]> {
    let n = ring.len();
    if n < 3 {
        return None;
    }
    let ccw = signed_area_xy(ring) > 0.0;

    for i in 0..n {
        let prev = ring[(i + n - 1) % n].pos;
        let cur = ring[i].pos;
        let next = ring[(i + 1) % n].pos;
        let (prev2, cur2, next2) = ([prev[0], prev[1]], [cur[0], cur[1]], [next[0], next[1]]);

        let turn = tri_cross(prev2, cur2, next2);
        let convex = if ccw { turn > 0.0 } else { turn < 0.0 };
        if !convex {
            continue;
        }

        let blocked = ring.iter().enumerate().any(|(j, corner)| {
            let is_triangle_vertex = j == i || j == (i + n - 1) % n || j == (i + 1) % n;
            !is_triangle_vertex
                && point_in_or_on_triangle([corner.pos[0], corner.pos[1]], prev2, cur2, next2)
        });
        if blocked {
            continue;
        }

        return Some([
            (prev2[0] + cur2[0] + next2[0]) / 3.0,
            (prev2[1] + cur2[1] + next2[1]) / 3.0,
        ]);
    }

    None
}

/// The axis-aligned bounding box, `(min, max)`, of a ring's XY-projected
/// vertices.
fn ring_bbox<const N: usize>(ring: &[Corner<N>]) -> ([f64; 2], [f64; 2]) {
    let mut min = [f64::INFINITY, f64::INFINITY];
    let mut max = [f64::NEG_INFINITY, f64::NEG_INFINITY];
    for c in ring {
        min[0] = min[0].min(c.pos[0]);
        min[1] = min[1].min(c.pos[1]);
        max[0] = max[0].max(c.pos[0]);
        max[1] = max[1].max(c.pos[1]);
    }
    (min, max)
}

/// Whether bounding box `outer` contains bounding box `inner` (inclusive of
/// touching edges).
fn bbox_contains(outer: &([f64; 2], [f64; 2]), inner: &([f64; 2], [f64; 2])) -> bool {
    outer.0[0] <= inner.0[0]
        && outer.0[1] <= inner.0[1]
        && outer.1[0] >= inner.1[0]
        && outer.1[1] >= inner.1[1]
}

/// A bounding box's area, for breaking ties between candidate containers.
fn bbox_area(bbox: &([f64; 2], [f64; 2])) -> f64 {
    (bbox.1[0] - bbox.0[0]) * (bbox.1[1] - bbox.0[1])
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
    // being attached to the wrong face.
    for hole in holes {
        if hole.is_empty() {
            continue;
        }
        let owner = match ring_probe(&hole) {
            Some(probe) => faces
                .iter()
                .position(|f| contains_point(&f.rings[0], probe)),
            None => {
                // `hole` is degenerate (see `ring_probe`'s doc comment) and has
                // no point that is safely, verifiably interior to it, so a
                // point-in-ring test against it cannot be trusted either way.
                // Fall back to bounding-box containment instead: a hole's box
                // is always inside its own exterior's box, so this can never
                // produce a false "inside" for the correct owner. It can be
                // ambiguous when candidate exterior boxes overlap, so among
                // every candidate whose box contains the hole's, deterministically
                // pick the one with the smallest box -- the tightest fit, and
                // so the most likely correct one.
                let hole_bbox = ring_bbox(&hole);
                faces
                    .iter()
                    .map(|f| ring_bbox(&f.rings[0]))
                    .enumerate()
                    .filter(|(_, bbox)| bbox_contains(bbox, &hole_bbox))
                    .min_by(|(_, a), (_, b)| {
                        bbox_area(a)
                            .partial_cmp(&bbox_area(b))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(i, _)| i)
            }
        };
        // No exterior -- by point-in-ring test, or by bounding-box fallback --
        // claims this hole. There is nothing to subtract it from, so it is
        // dropped rather than attached to an arbitrary face: the affected
        // face's area is then a knowable overstatement by this hole's own
        // area, rather than a silent, unrelated corruption.
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
    fn concave_c_hole_defeats_a_naive_centroid_probe() {
        // Confirms the fixture is a genuine test of concavity, not an
        // accidentally-convex shape: the plain arithmetic-mean-of-vertices
        // approach `ring_probe` deliberately does not use would misjudge even
        // this hole's own shape, let alone which exterior it belongs to.
        let hole = concave_c_hole();
        assert!(signed_area_xy(&hole) < 0.0, "fixture hole must be CW");
        let mut sum = [0.0, 0.0];
        for c in &hole {
            sum[0] += c.pos[0];
            sum[1] += c.pos[1];
        }
        let n = hole.len() as f64;
        let centroid = [sum[0] / n, sum[1] / n];
        assert!(
            !contains_point(&hole, centroid),
            "centroid was {centroid:?}, expected it outside the hole's own notch"
        );
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

    /// A degenerate "ring": four collinear points, so every triangle any
    /// vertex could form with its neighbours has zero area, and no ear
    /// exists. Every coordinate here is an exact binary fraction (an
    /// integer), so the collinearity is exact in `f64`, not an artifact of
    /// rounding.
    fn collinear_points() -> Vec<Corner<2>> {
        ring2(&[(0.0, 0.0), (1.0, 0.0), (2.0, 0.0), (3.0, 0.0)])
    }

    #[test]
    fn ring_probe_returns_none_for_a_degenerate_collinear_ring() {
        // `clip_rings_halfplane` is not expected to ever hand `clip_to_window`
        // a ring this degenerate, so this exercises `ring_probe`'s `None`
        // branch directly rather than trying to force the clip to produce
        // one.
        assert_eq!(ring_probe(&collinear_points()), None);
    }

    /// A degenerate "hole": three collinear points, all exact binary
    /// fractions (multiples of 0.25) so the zero signed area is exact, not
    /// an artifact of rounding. `clip_to_window`'s exterior/hole partition
    /// sorts anything with signed area not `> 0.0` into `holes`, so this
    /// lands there despite not really being a hole in any meaningful sense
    /// -- which is exactly the case this fixture is for.
    fn collinear_hole() -> Vec<Corner<2>> {
        ring2(&[(0.25, 0.5), (0.5, 0.5), (0.75, 0.5)])
    }

    #[test]
    fn degenerate_hole_with_no_verified_probe_falls_back_to_bounding_box_containment() {
        let exterior = ring2(&[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]);
        let hole = collinear_hole();
        assert_eq!(
            ring_probe(&hole),
            None,
            "fixture must actually defeat ring_probe"
        );

        let faces = clip_to_window(vec![exterior, hole], &unit_window());
        assert_eq!(faces.len(), 1);
        assert_eq!(
            faces[0].rings.len(),
            2,
            "the degenerate ring's bounding box sits inside the exterior's, \
             so the bounding-box fallback must still attribute it"
        );
        // A zero-area ring changes nothing: still the full unit square.
        assert!((faces_area_xy(&faces) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn degenerate_hole_with_no_containing_exterior_is_dropped() {
        // No exterior at all in this batch, so neither the point-in-ring test
        // nor the bounding-box fallback has anything to attribute the
        // degenerate hole to; it must be dropped rather than surface as a
        // face on its own (it has zero area and is not an exterior ring).
        let hole = collinear_hole();
        assert!(clip_to_window(vec![hole], &unit_window()).is_empty());
    }
}
