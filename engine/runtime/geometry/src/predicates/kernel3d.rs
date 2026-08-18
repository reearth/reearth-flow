//! Exact 3D intersection tests over the robust [`kernel`](super::kernel).
//!
//! Boolean primitives for the 3D predicates: point × segment / triangle,
//! segment × segment / triangle, and triangle × triangle, all **exact**: every
//! decision is a robust [`orient2d`] / [`orient3d`] sign, with no epsilon
//! thresholds. These are closed point-set tests: shared vertices, edge
//! touches, and coplanar overlaps all intersect.
//!
//! Degenerate configurations reduce exactly instead of erroring: a collinear
//! "triangle" is tested as the segment it spans, coplanar cases drop to 2D
//! through an axis projection that is injective on the carrying plane (an axis
//! projection of coplanar points is degenerate exactly when it maps them all
//! onto one line, which is detectable with `orient2d` alone).
//!
//! The *constructed* counterpart (ray casting with hit coordinates) lives in
//! [`ray`](super::ray) and is deliberately not exact; see there.

use super::kernel::{orient2d, orient3d, segment_intersection, Orientation};
use super::kernel::{segment_intersection_3d, SegmentIntersection};
use super::view::point_in_triangle_2d;

/// Whether the three points are collinear, exactly.
///
/// The three components of the cross product `(b - a) × (p - a)` are the signed
/// areas of the three axis projections, so the points are collinear in 3D iff
/// every axis projection is collinear in 2D: three robust [`orient2d`] calls.
pub fn collinear_3d(a: [f64; 3], b: [f64; 3], p: [f64; 3]) -> bool {
    (0..3).all(|axis| {
        orient2d(drop_axis(a, axis), drop_axis(b, axis), drop_axis(p, axis))
            == Orientation::Collinear
    })
}

/// Whether `p` lies on the closed segment `[a, b]`, exactly.
pub fn point_on_segment_3d(p: [f64; 3], a: [f64; 3], b: [f64; 3]) -> bool {
    collinear_3d(a, b, p) && (0..3).all(|k| value_in_between(p[k], a[k], b[k]))
}

/// Whether the four points lie in one plane, exactly ([`orient3d`]).
#[inline]
pub fn coplanar(a: [f64; 3], b: [f64; 3], c: [f64; 3], d: [f64; 3]) -> bool {
    orient3d(a, b, c, d) == Orientation::Collinear
}

/// Whether `p` lies on the closed triangle `t` (interior, edge, or vertex),
/// exactly. A degenerate triangle is tested as the segment or point it spans.
pub fn point_in_triangle_3d(p: [f64; 3], t: [[f64; 3]; 3]) -> bool {
    match classify_triangle(t) {
        TriangleShape::Proper => {
            if !coplanar(t[0], t[1], t[2], p) {
                return false;
            }
            let axis = projection_axis(t);
            point_in_triangle_2d(drop_axis(p, axis), project_triangle(t, axis))
        }
        TriangleShape::Segment([a, b]) => point_on_segment_3d(p, a, b),
        TriangleShape::Point(q) => p == q,
    }
}

/// Whether the closed segments `[p1, p2]` and `[q1, q2]` share at least one
/// point, exactly. Collinear overlaps and endpoint touches intersect; skew and
/// parallel-disjoint segments do not.
pub fn segments_intersect_3d(p1: [f64; 3], p2: [f64; 3], q1: [f64; 3], q2: [f64; 3]) -> bool {
    if p1 == p2 {
        return if q1 == q2 {
            p1 == q1
        } else {
            point_on_segment_3d(p1, q1, q2)
        };
    }
    if q1 == q2 {
        return point_on_segment_3d(q1, p1, p2);
    }
    // Two proper segments only meet if all four endpoints are coplanar.
    if !coplanar(p1, p2, q1, q2) {
        return false;
    }
    // Drop to 2D through an axis projection that is injective on the carrying
    // plane: one where the four projected points do not all collapse onto a
    // line. If every projection collapses, the points were collinear in 3D and
    // the question is a 1D overlap.
    for axis in [2, 1, 0] {
        let (a, b, c, d) = (
            drop_axis(p1, axis),
            drop_axis(p2, axis),
            drop_axis(q1, axis),
            drop_axis(q2, axis),
        );
        let collapsed = orient2d(a, b, c) == Orientation::Collinear
            && orient2d(a, b, d) == Orientation::Collinear
            && orient2d(a, c, d) == Orientation::Collinear;
        if !collapsed {
            return segment_intersection(a, b, c, d).is_some();
        }
    }
    // All four points collinear in 3D: closed intervals on one line overlap
    // iff some endpoint of either segment lies on the other.
    point_on_segment_3d(q1, p1, p2)
        || point_on_segment_3d(q2, p1, p2)
        || point_on_segment_3d(p1, q1, q2)
        || point_on_segment_3d(p2, q1, q2)
}

/// Whether the rays `origin -> u` and `origin -> w` point in the same
/// direction: exactly collinear with matching per-axis difference signs. The
/// sign of an IEEE subtraction is exact, so this is exact for distinct
/// endpoints.
#[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
pub(crate) fn same_direction_3d(origin: [f64; 3], u: [f64; 3], w: [f64; 3]) -> bool {
    fn sign(v: f64) -> i8 {
        if v > 0.0 {
            1
        } else if v < 0.0 {
            -1
        } else {
            0
        }
    }
    collinear_3d(origin, u, w) && (0..3).all(|k| sign(u[k] - origin[k]) == sign(w[k] - origin[k]))
}

/// The classified intersection of two closed 3D segments.
#[derive(PartialEq, Clone, Copy, Debug)]
#[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
pub(crate) enum SegmentContact3D {
    /// A single crossing point interior to both segments.
    Proper([f64; 3]),
    /// A single shared point involving an endpoint of at least one segment
    /// (an endpoint touch or a T-junction).
    Touch([f64; 3]),
    /// A collinear overlap along the sub-segment `[start, end]`.
    Overlap {
        /// One end of the overlap.
        start: [f64; 3],
        /// The other end of the overlap.
        end: [f64; 3],
    },
}

/// Classify the intersection of the closed segments `[p1, p2]` and `[q1, q2]`,
/// exactly: `None` when disjoint (including skew and parallel-disjoint pairs),
/// otherwise the contact class. The class decision is robust; the `Proper`
/// coordinate is a constructed f64 witness. Coordinates must be finite.
#[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
pub(crate) fn classify_segments_3d(
    p1: [f64; 3],
    p2: [f64; 3],
    q1: [f64; 3],
    q2: [f64; 3],
) -> Option<SegmentContact3D> {
    if p1 == p2 {
        let touching = if q1 == q2 {
            p1 == q1
        } else {
            point_on_segment_3d(p1, q1, q2)
        };
        return touching.then_some(SegmentContact3D::Touch(p1));
    }
    if q1 == q2 {
        return point_on_segment_3d(q1, p1, p2).then_some(SegmentContact3D::Touch(q1));
    }
    if !coplanar(p1, p2, q1, q2) {
        return None;
    }
    // Mirror `segments_intersect_3d`: drop to an axis projection injective on
    // the carrying plane, falling through to the 1D case when every
    // projection collapses the four points onto a line.
    for axis in [2, 1, 0] {
        let (a, b, c, d) = (
            drop_axis(p1, axis),
            drop_axis(p2, axis),
            drop_axis(q1, axis),
            drop_axis(q2, axis),
        );
        let collapsed = orient2d(a, b, c) == Orientation::Collinear
            && orient2d(a, b, d) == Orientation::Collinear
            && orient2d(a, c, d) == Orientation::Collinear;
        if collapsed {
            continue;
        }
        return match segment_intersection(a, b, c, d)? {
            SegmentIntersection::SinglePoint {
                intersection,
                is_proper: true,
            } => Some(SegmentContact3D::Proper(
                segment_intersection_3d(p1, p2, q1, q2)
                    .unwrap_or_else(|| lift_onto_segment(intersection, axis, p1, p2)),
            )),
            SegmentIntersection::SinglePoint { intersection, .. } => Some(SegmentContact3D::Touch(
                touch_witness_3d(intersection, axis, [p1, p2, q1, q2]),
            )),
            SegmentIntersection::Collinear { .. } => {
                unreachable!("a collinear 2D overlap requires a collapsed projection")
            }
        };
    }
    collinear_segments_contact_3d(p1, p2, q1, q2)
}

/// The contact of two proper segments whose four endpoints are collinear in
/// 3D: a 1D interval question decided by which endpoints lie on the other
/// segment.
#[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
fn collinear_segments_contact_3d(
    p1: [f64; 3],
    p2: [f64; 3],
    q1: [f64; 3],
    q2: [f64; 3],
) -> Option<SegmentContact3D> {
    let mut shared: Vec<[f64; 3]> = Vec::with_capacity(4);
    for e in [q1, q2] {
        if point_on_segment_3d(e, p1, p2) {
            shared.push(e);
        }
    }
    for e in [p1, p2] {
        if point_on_segment_3d(e, q1, q2) {
            shared.push(e);
        }
    }
    shared.sort_by(|x, y| x.partial_cmp(y).expect("finite coordinates"));
    shared.dedup();
    match shared.as_slice() {
        [] => None,
        [point] => Some(SegmentContact3D::Touch(*point)),
        // Lexicographic order is monotone along a line, so the extremes are
        // the overlap's ends.
        [start, .., end] => Some(SegmentContact3D::Overlap {
            start: *start,
            end: *end,
        }),
    }
}

/// The 3D endpoint behind an improper single-point 2D contact: such a contact
/// always includes a segment endpoint, and the projection is injective on the
/// carrying plane, so the endpoint projecting onto it is the witness.
#[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
fn touch_witness_3d(projected: [f64; 2], axis: usize, endpoints: [[f64; 3]; 4]) -> [f64; 3] {
    for e in endpoints {
        if drop_axis(e, axis) == projected {
            return e;
        }
    }
    lift_onto_segment(projected, axis, endpoints[0], endpoints[1])
}

/// Lift a projected point back onto the 3D segment `[a, b]` by interpolating
/// along the dominant projected direction. A constructed fallback witness, not
/// a decision input.
#[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
fn lift_onto_segment(w: [f64; 2], axis: usize, a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    let (a2, b2) = (drop_axis(a, axis), drop_axis(b, axis));
    let d = [b2[0] - a2[0], b2[1] - a2[1]];
    let t = if d[0].abs() >= d[1].abs() {
        if d[0] == 0.0 {
            0.0
        } else {
            (w[0] - a2[0]) / d[0]
        }
    } else {
        (w[1] - a2[1]) / d[1]
    };
    [
        a[0] + t * (b[0] - a[0]),
        a[1] + t * (b[1] - a[1]),
        a[2] + t * (b[2] - a[2]),
    ]
}

/// Whether the closed segment `[p, q]` shares at least one point with the
/// closed triangle `t`, exactly. Endpoint touches, edge grazes, and coplanar
/// crossings all intersect; degenerate inputs reduce to the lower-dimensional
/// tests.
pub fn segment_intersects_triangle_3d(p: [f64; 3], q: [f64; 3], t: [[f64; 3]; 3]) -> bool {
    match classify_triangle(t) {
        TriangleShape::Proper => {}
        TriangleShape::Segment([a, b]) => return segments_intersect_3d(p, q, a, b),
        TriangleShape::Point(v) => return point_on_segment_3d(v, p, q),
    }
    if p == q {
        return point_in_triangle_3d(p, t);
    }

    let side_p = orient3d(t[0], t[1], t[2], p);
    let side_q = orient3d(t[0], t[1], t[2], q);
    match (side_p, side_q) {
        // Both endpoints strictly on one side of the triangle's plane.
        (Orientation::Clockwise, Orientation::Clockwise)
        | (Orientation::CounterClockwise, Orientation::CounterClockwise) => false,
        // The segment lies in the plane: a 2D segment × triangle problem.
        (Orientation::Collinear, Orientation::Collinear) => {
            let axis = projection_axis(t);
            let tri = project_triangle(t, axis);
            let (a, b) = (drop_axis(p, axis), drop_axis(q, axis));
            point_in_triangle_2d(a, tri)
                || point_in_triangle_2d(b, tri)
                || triangle_edges_2d(tri)
                    .iter()
                    .any(|&(u, v)| segment_intersection(a, b, u, v).is_some())
        }
        // One endpoint in the plane: the plane intersection is that endpoint.
        (Orientation::Collinear, _) => point_in_triangle_3d(p, t),
        (_, Orientation::Collinear) => point_in_triangle_3d(q, t),
        // The segment straddles the plane: it meets the closed triangle iff
        // the line through `p, q` does not pass strictly outside any edge,
        // i.e. no two of the three edge orientations strictly disagree.
        _ => {
            let u = orient3d(p, q, t[0], t[1]);
            let v = orient3d(p, q, t[1], t[2]);
            let w = orient3d(p, q, t[2], t[0]);
            !has_strict_disagreement(u, v, w)
        }
    }
}

/// Whether the two closed triangles share at least one point, exactly.
/// Coplanar overlaps (including full containment) and boundary contacts all
/// intersect; degenerate triangles reduce to segment / point tests.
pub fn triangles_intersect_3d(t: [[f64; 3]; 3], s: [[f64; 3]; 3]) -> bool {
    match classify_triangle(t) {
        TriangleShape::Proper => {}
        TriangleShape::Segment([a, b]) => return segment_intersects_triangle_3d(a, b, s),
        TriangleShape::Point(p) => return point_in_triangle_3d(p, s),
    }
    match classify_triangle(s) {
        TriangleShape::Proper => {}
        TriangleShape::Segment([a, b]) => return segment_intersects_triangle_3d(a, b, t),
        TriangleShape::Point(p) => return point_in_triangle_3d(p, t),
    }

    // Early reject: all of `s` strictly on one side of `t`'s plane (or vice
    // versa) cannot intersect.
    let sides = s.map(|p| orient3d(t[0], t[1], t[2], p));
    if sides.iter().all(|&o| o == Orientation::CounterClockwise)
        || sides.iter().all(|&o| o == Orientation::Clockwise)
    {
        return false;
    }

    if sides.iter().all(|&o| o == Orientation::Collinear) {
        // Coplanar triangles: a 2D overlap test in a projection injective on
        // the shared plane (`t` is proper, so its own projection axis works).
        let axis = projection_axis(t);
        let (pt, ps) = (project_triangle(t, axis), project_triangle(s, axis));
        return ps.iter().any(|&p| point_in_triangle_2d(p, pt))
            || pt.iter().any(|&p| point_in_triangle_2d(p, ps))
            || triangle_edges_2d(pt).iter().any(|&(a, b)| {
                triangle_edges_2d(ps)
                    .iter()
                    .any(|&(c, d)| segment_intersection(a, b, c, d).is_some())
            });
    }

    // Non-coplanar: a non-empty intersection is a segment whose endpoints lie
    // where one triangle's boundary crosses the other's closed region, so an
    // edge of one intersects the other.
    triangle_edges_3d(t)
        .iter()
        .any(|&(a, b)| segment_intersects_triangle_3d(a, b, s))
        || triangle_edges_3d(s)
            .iter()
            .any(|&(a, b)| segment_intersects_triangle_3d(a, b, t))
}

/// How two triangles from different faces share corners (by coordinate).
#[derive(PartialEq, Clone, Copy, Debug)]
#[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
pub(crate) enum TriangleContact {
    /// No equal corners.
    None,
    /// Exactly one equal corner `v`; `t_rest` / `s_rest` are each triangle's
    /// other two corners.
    Vertex {
        /// The shared corner.
        v: [f64; 3],
        /// The first triangle's other two corners.
        t_rest: [[f64; 3]; 2],
        /// The second triangle's other two corners.
        s_rest: [[f64; 3]; 2],
    },
    /// Two equal corners `[a, b]`, necessarily an edge of both triangles;
    /// `t_far` / `s_far` are the opposite corners.
    Edge {
        /// One shared corner.
        a: [f64; 3],
        /// The other shared corner.
        b: [f64; 3],
        /// The first triangle's opposite corner.
        t_far: [f64; 3],
        /// The second triangle's opposite corner.
        s_far: [f64; 3],
    },
}

/// Whether two triangles known to share the given contact intersect beyond
/// it, exactly: the adjacency-aware refinement of [`triangles_intersect_3d`]
/// for face-vs-face self-intersection, where a contact confined to the shared
/// corners or shared edge is legitimate. Both triangles must be proper
/// (non-degenerate) and `contact` must describe their actual shared corners.
#[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
pub(crate) fn triangles_overlap_beyond_contact(
    t: [[f64; 3]; 3],
    s: [[f64; 3]; 3],
    contact: &TriangleContact,
) -> bool {
    match *contact {
        TriangleContact::None => triangles_intersect_3d(t, s),
        // Non-coplanar wings sharing an edge meet exactly at that edge: a
        // proper triangle meets the line through one of its edges in exactly
        // that edge. Coplanar wings overlap iff the far corners lie on the
        // same side of the shared edge.
        TriangleContact::Edge { a, b, t_far, s_far } => {
            if orient3d(a, b, t_far, s_far) != Orientation::Collinear {
                return false;
            }
            let axis = projection_axis(t);
            orient2d(
                drop_axis(a, axis),
                drop_axis(b, axis),
                drop_axis(t_far, axis),
            ) == orient2d(
                drop_axis(a, axis),
                drop_axis(b, axis),
                drop_axis(s_far, axis),
            )
        }
        // Triangles sharing one corner meet beyond it iff an opposite edge
        // reaches the other triangle, or an edge from the corner lies in the
        // other triangle's plane and runs into it (a non-coplanar edge
        // through the corner meets that plane only at the corner).
        TriangleContact::Vertex { v, t_rest, s_rest } => {
            segment_intersects_triangle_3d(t_rest[0], t_rest[1], s)
                || segment_intersects_triangle_3d(s_rest[0], s_rest[1], t)
                || wing_reaches_triangle(v, t_rest, s)
                || wing_reaches_triangle(v, s_rest, t)
        }
    }
}

/// Whether an edge from the shared corner `v` to either of `rest` lies in
/// `other`'s plane and meets `other` at some point besides `v`. `other` must
/// be a proper triangle with `v` as one of its corners.
#[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
fn wing_reaches_triangle(v: [f64; 3], rest: [[f64; 3]; 2], other: [[f64; 3]; 3]) -> bool {
    let axis = projection_axis(other);
    let tri = project_triangle(other, axis);
    let v2 = drop_axis(v, axis);
    rest.into_iter().any(|x| {
        if orient3d(other[0], other[1], other[2], x) != Orientation::Collinear {
            return false;
        }
        let x2 = drop_axis(x, axis);
        point_in_triangle_2d(x2, tri)
            || triangle_edges_2d(tri).iter().any(|&(u, w)| {
                match segment_intersection(v2, x2, u, w) {
                    None => false,
                    Some(SegmentIntersection::SinglePoint { intersection, .. }) => {
                        intersection != v2
                    }
                    Some(SegmentIntersection::Collinear { start, end }) => start != v2 || end != v2,
                }
            })
    })
}

/// The true point-set shape of a possibly-degenerate triangle.
pub(crate) enum TriangleShape {
    /// Three non-collinear corners: a genuine triangle.
    Proper,
    /// Collinear but not coincident corners: the spanned segment.
    Segment([[f64; 3]; 2]),
    /// All corners coincident.
    Point([f64; 3]),
}

/// Classify a triangle by the point set its corners span, exactly.
pub(crate) fn classify_triangle(t: [[f64; 3]; 3]) -> TriangleShape {
    if !collinear_3d(t[0], t[1], t[2]) {
        return TriangleShape::Proper;
    }
    if t[0] == t[1] && t[1] == t[2] {
        return TriangleShape::Point(t[0]);
    }
    // Collinear: the spanned segment is the pair containing the third point.
    for (a, b, c) in [(0, 1, 2), (0, 2, 1), (1, 2, 0)] {
        if point_on_segment_3d(t[c], t[a], t[b]) {
            return TriangleShape::Segment([t[a], t[b]]);
        }
    }
    // Unreachable for exact arithmetic; be safe rather than panic.
    TriangleShape::Segment([t[0], t[1]])
}

/// An axis whose drop projection keeps a **proper** triangle non-degenerate
/// (injective on the triangle's plane). At least one exists: the cross
/// product's components are the three projected signed areas, and a proper
/// triangle has a non-zero cross product.
pub(crate) fn projection_axis(t: [[f64; 3]; 3]) -> usize {
    for axis in [2, 1, 0] {
        if orient2d(
            drop_axis(t[0], axis),
            drop_axis(t[1], axis),
            drop_axis(t[2], axis),
        ) != Orientation::Collinear
        {
            return axis;
        }
    }
    unreachable!("a proper triangle projects non-degenerately along some axis")
}

/// Drop the given axis: the remaining two coordinates, in cyclic order.
#[inline]
pub(crate) fn drop_axis(p: [f64; 3], axis: usize) -> [f64; 2] {
    [p[(axis + 1) % 3], p[(axis + 2) % 3]]
}

/// The triangle's corners with the given axis dropped.
#[inline]
pub(crate) fn project_triangle(t: [[f64; 3]; 3], axis: usize) -> [[f64; 2]; 3] {
    t.map(|p| drop_axis(p, axis))
}

/// The three directed edges of a 3D triangle.
#[inline]
fn triangle_edges_3d(t: [[f64; 3]; 3]) -> [([f64; 3], [f64; 3]); 3] {
    [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])]
}

/// The three directed edges of a 2D triangle.
#[inline]
fn triangle_edges_2d(t: [[f64; 2]; 3]) -> [([f64; 2], [f64; 2]); 3] {
    [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])]
}

/// Whether two of the three orientations strictly disagree (one clockwise and
/// one counter-clockwise); collinear entries never disagree.
fn has_strict_disagreement(u: Orientation, v: Orientation, w: Orientation) -> bool {
    let has_cw = [u, v, w].contains(&Orientation::Clockwise);
    let has_ccw = [u, v, w].contains(&Orientation::CounterClockwise);
    has_cw && has_ccw
}

/// Whether `value` lies within `[min(a, b), max(a, b)]` (inclusive).
#[inline]
fn value_in_between(value: f64, a: f64, b: f64) -> bool {
    let (lo, hi) = if a < b { (a, b) } else { (b, a) };
    value >= lo && value <= hi
}

#[cfg(test)]
mod tests {
    use super::*;

    const TRI: [[f64; 3]; 3] = [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [0.0, 4.0, 0.0]];
    /// The same triangle in a tilted plane (z = x).
    const TILTED: [[f64; 3]; 3] = [[0.0, 0.0, 0.0], [4.0, 0.0, 4.0], [0.0, 4.0, 0.0]];

    #[test]
    fn collinear_and_on_segment() {
        assert!(collinear_3d(
            [0.0, 0.0, 0.0],
            [2.0, 2.0, 2.0],
            [5.0, 5.0, 5.0]
        ));
        assert!(!collinear_3d(
            [0.0, 0.0, 0.0],
            [2.0, 2.0, 2.0],
            [5.0, 5.0, 5.1]
        ));
        assert!(point_on_segment_3d(
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            [2.0, 2.0, 2.0]
        ));
        // Collinear but beyond the endpoint.
        assert!(!point_on_segment_3d(
            [3.0, 3.0, 3.0],
            [0.0, 0.0, 0.0],
            [2.0, 2.0, 2.0]
        ));
        // Degenerate segment.
        assert!(point_on_segment_3d(
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0]
        ));
    }

    #[test]
    fn point_in_triangle_cases() {
        assert!(point_in_triangle_3d([1.0, 1.0, 0.0], TRI));
        assert!(point_in_triangle_3d([2.0, 0.0, 0.0], TRI)); // edge
        assert!(point_in_triangle_3d([0.0, 4.0, 0.0], TRI)); // vertex
        assert!(!point_in_triangle_3d([1.0, 1.0, 0.5], TRI)); // off plane
        assert!(!point_in_triangle_3d([3.0, 3.0, 0.0], TRI)); // in plane, outside
        assert!(point_in_triangle_3d([1.0, 1.0, 1.0], TILTED));
        assert!(!point_in_triangle_3d([1.0, 1.0, 0.0], TILTED));

        // A vertical triangle (constant y) exercises the projection choice.
        let vertical = [[0.0, 1.0, 0.0], [4.0, 1.0, 0.0], [0.0, 1.0, 4.0]];
        assert!(point_in_triangle_3d([1.0, 1.0, 1.0], vertical));
        assert!(!point_in_triangle_3d([1.0, 1.5, 1.0], vertical));

        // A degenerate (collinear) triangle is its spanned segment.
        let flat = [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        assert!(point_in_triangle_3d([3.0, 0.0, 0.0], flat));
        assert!(!point_in_triangle_3d([3.0, 0.1, 0.0], flat));
    }

    #[test]
    fn segment_pairs() {
        // Proper coplanar crossing.
        assert!(segments_intersect_3d(
            [0.0, 0.0, 0.0],
            [2.0, 2.0, 2.0],
            [2.0, 0.0, 0.0],
            [0.0, 2.0, 2.0]
        ));
        // Skew.
        assert!(!segments_intersect_3d(
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 1.0],
            [0.0, -1.0, 1.0]
        ));
        // Endpoint touch.
        assert!(segments_intersect_3d(
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [2.0, 0.0, 5.0]
        ));
        // Collinear overlap.
        assert!(segments_intersect_3d(
            [0.0, 0.0, 0.0],
            [4.0, 4.0, 4.0],
            [2.0, 2.0, 2.0],
            [6.0, 6.0, 6.0]
        ));
        // Collinear disjoint.
        assert!(!segments_intersect_3d(
            [0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0],
            [2.0, 2.0, 2.0],
            [3.0, 3.0, 3.0]
        ));
        // Parallel in one plane.
        assert!(!segments_intersect_3d(
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [4.0, 1.0, 0.0]
        ));
        // A vertical plane (all x equal) exercises the projection fallback.
        assert!(segments_intersect_3d(
            [1.0, 0.0, 0.0],
            [1.0, 2.0, 2.0],
            [1.0, 0.0, 2.0],
            [1.0, 2.0, 0.0]
        ));
    }

    #[test]
    fn segment_triangle_cases() {
        // Straddling pierce through the interior.
        assert!(segment_intersects_triangle_3d(
            [1.0, 1.0, -1.0],
            [1.0, 1.0, 1.0],
            TRI
        ));
        // Straddling the plane but passing outside.
        assert!(!segment_intersects_triangle_3d(
            [5.0, 5.0, -1.0],
            [5.0, 5.0, 1.0],
            TRI
        ));
        // Pierce exactly through a vertex.
        assert!(segment_intersects_triangle_3d(
            [0.0, 0.0, -1.0],
            [0.0, 0.0, 1.0],
            TRI
        ));
        // Pierce exactly through an edge.
        assert!(segment_intersects_triangle_3d(
            [2.0, 0.0, -1.0],
            [2.0, 0.0, 1.0],
            TRI
        ));
        // Through the edge's extension (outside the triangle).
        assert!(!segment_intersects_triangle_3d(
            [5.0, 0.0, -1.0],
            [5.0, 0.0, 1.0],
            TRI
        ));
        // Endpoint resting on the triangle.
        assert!(segment_intersects_triangle_3d(
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 5.0],
            TRI
        ));
        // Entirely above.
        assert!(!segment_intersects_triangle_3d(
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 5.0],
            TRI
        ));
        // Coplanar: crossing, contained, and disjoint.
        assert!(segment_intersects_triangle_3d(
            [-1.0, 1.0, 0.0],
            [5.0, 1.0, 0.0],
            TRI
        ));
        assert!(segment_intersects_triangle_3d(
            [0.5, 0.5, 0.0],
            [1.0, 1.0, 0.0],
            TRI
        ));
        assert!(!segment_intersects_triangle_3d(
            [5.0, 5.0, 0.0],
            [6.0, 5.0, 0.0],
            TRI
        ));
    }

    #[test]
    fn triangle_pairs() {
        // Piercing cross.
        let pierce = [[1.0, 1.0, -1.0], [1.0, 1.0, 1.0], [3.0, 3.0, 1.0]];
        assert!(triangles_intersect_3d(TRI, pierce));
        assert!(triangles_intersect_3d(pierce, TRI));
        // Strictly above.
        let above = TRI.map(|[x, y, _]| [x, y, 1.0]);
        assert!(!triangles_intersect_3d(TRI, above));
        // Shared edge only.
        let folded = [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [0.0, -4.0, 4.0]];
        assert!(triangles_intersect_3d(TRI, folded));
        // Shared vertex only.
        let corner = [[0.0, 0.0, 0.0], [-4.0, 0.0, 1.0], [0.0, -4.0, 1.0]];
        assert!(triangles_intersect_3d(TRI, corner));
        // Coplanar overlap.
        let coplanar_overlap = [[1.0, 1.0, 0.0], [5.0, 1.0, 0.0], [1.0, 5.0, 0.0]];
        assert!(triangles_intersect_3d(TRI, coplanar_overlap));
        // Coplanar containment (small triangle strictly inside).
        let inside = [[0.5, 0.5, 0.0], [1.5, 0.5, 0.0], [0.5, 1.5, 0.0]];
        assert!(triangles_intersect_3d(TRI, inside));
        assert!(triangles_intersect_3d(inside, TRI));
        // Coplanar disjoint.
        let coplanar_far = [[10.0, 10.0, 0.0], [14.0, 10.0, 0.0], [10.0, 14.0, 0.0]];
        assert!(!triangles_intersect_3d(TRI, coplanar_far));
        // Degenerate triangle acting as a segment.
        let needle = [[1.0, 1.0, -1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 0.5]];
        assert!(triangles_intersect_3d(TRI, needle));
    }

    #[test]
    fn same_direction_3d_cases() {
        let o = [1.0, 1.0, 1.0];
        assert!(same_direction_3d(o, [2.0, 2.0, 2.0], [3.0, 3.0, 3.0]));
        // Opposite direction.
        assert!(!same_direction_3d(o, [2.0, 2.0, 2.0], [0.0, 0.0, 0.0]));
        // Not collinear.
        assert!(!same_direction_3d(o, [2.0, 2.0, 2.0], [3.0, 3.0, 4.0]));
    }

    #[test]
    fn classify_segments_proper_crossing() {
        let contact = classify_segments_3d(
            [0.0, 0.0, 0.0],
            [2.0, 2.0, 2.0],
            [2.0, 0.0, 0.0],
            [0.0, 2.0, 2.0],
        );
        assert_eq!(contact, Some(SegmentContact3D::Proper([1.0, 1.0, 1.0])));
    }

    #[test]
    fn classify_segments_touches() {
        // Endpoint on endpoint.
        assert_eq!(
            classify_segments_3d(
                [0.0, 0.0, 0.0],
                [1.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [2.0, 0.0, 5.0],
            ),
            Some(SegmentContact3D::Touch([1.0, 1.0, 1.0]))
        );
        // T-junction: endpoint interior to the other segment.
        assert_eq!(
            classify_segments_3d(
                [0.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [2.0, 0.0, 0.0],
                [2.0, 3.0, 0.0],
            ),
            Some(SegmentContact3D::Touch([2.0, 0.0, 0.0]))
        );
        // Collinear single-point touch.
        assert_eq!(
            classify_segments_3d(
                [0.0, 0.0, 0.0],
                [2.0, 2.0, 2.0],
                [2.0, 2.0, 2.0],
                [4.0, 4.0, 4.0],
            ),
            Some(SegmentContact3D::Touch([2.0, 2.0, 2.0]))
        );
    }

    #[test]
    fn classify_segments_overlap_and_disjoint() {
        assert_eq!(
            classify_segments_3d(
                [0.0, 0.0, 0.0],
                [4.0, 4.0, 4.0],
                [2.0, 2.0, 2.0],
                [6.0, 6.0, 6.0],
            ),
            Some(SegmentContact3D::Overlap {
                start: [2.0, 2.0, 2.0],
                end: [4.0, 4.0, 4.0],
            })
        );
        // Identical segments overlap end to end.
        assert_eq!(
            classify_segments_3d(
                [0.0, 0.0, 0.0],
                [4.0, 4.0, 4.0],
                [0.0, 0.0, 0.0],
                [4.0, 4.0, 4.0],
            ),
            Some(SegmentContact3D::Overlap {
                start: [0.0, 0.0, 0.0],
                end: [4.0, 4.0, 4.0],
            })
        );
        // Skew.
        assert_eq!(
            classify_segments_3d(
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 1.0],
                [0.0, -1.0, 1.0],
            ),
            None
        );
        // Parallel in one plane.
        assert_eq!(
            classify_segments_3d(
                [0.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [4.0, 1.0, 0.0],
            ),
            None
        );
        // Collinear disjoint.
        assert_eq!(
            classify_segments_3d(
                [0.0, 0.0, 0.0],
                [1.0, 1.0, 1.0],
                [2.0, 2.0, 2.0],
                [3.0, 3.0, 3.0],
            ),
            None
        );
        // Degenerate segment resting on a segment.
        assert_eq!(
            classify_segments_3d(
                [1.0, 1.0, 1.0],
                [1.0, 1.0, 1.0],
                [0.0, 0.0, 0.0],
                [2.0, 2.0, 2.0],
            ),
            Some(SegmentContact3D::Touch([1.0, 1.0, 1.0]))
        );
    }

    /// Contact for `TRI` and a triangle sharing `TRI`'s edge `[0,0,0]-[4,0,0]`.
    fn edge_contact(t_far: [f64; 3], s_far: [f64; 3]) -> TriangleContact {
        TriangleContact::Edge {
            a: [0.0, 0.0, 0.0],
            b: [4.0, 0.0, 0.0],
            t_far,
            s_far,
        }
    }

    #[test]
    fn shared_edge_wings() {
        let t_far = [0.0, 4.0, 0.0];
        // Folded (non-coplanar) wings meet exactly at the shared edge.
        let folded_far = [0.0, -4.0, 4.0];
        let folded = [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], folded_far];
        assert!(!triangles_overlap_beyond_contact(
            TRI,
            folded,
            &edge_contact(t_far, folded_far)
        ));
        // Coplanar, far corners on opposite sides: a clean quad.
        let opposite_far = [0.0, -4.0, 0.0];
        let opposite = [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], opposite_far];
        assert!(!triangles_overlap_beyond_contact(
            TRI,
            opposite,
            &edge_contact(t_far, opposite_far)
        ));
        // Coplanar, far corners on the same side: interiors overlap.
        let same_side_far = [2.0, 4.0, 0.0];
        let same_side = [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], same_side_far];
        assert!(triangles_overlap_beyond_contact(
            TRI,
            same_side,
            &edge_contact(t_far, same_side_far)
        ));
    }

    /// Contact for two triangles sharing only `TRI`'s corner at the origin.
    fn vertex_contact(t_rest: [[f64; 3]; 2], s_rest: [[f64; 3]; 2]) -> TriangleContact {
        TriangleContact::Vertex {
            v: [0.0, 0.0, 0.0],
            t_rest,
            s_rest,
        }
    }

    #[test]
    fn shared_vertex_fans() {
        let t_rest = [[4.0, 0.0, 0.0], [0.0, 4.0, 0.0]];
        // A clean fan: the second triangle bends away from TRI's plane.
        let fan_rest = [[-4.0, 0.0, 1.0], [0.0, -4.0, 1.0]];
        let fan = [[0.0, 0.0, 0.0], fan_rest[0], fan_rest[1]];
        assert!(!triangles_overlap_beyond_contact(
            TRI,
            fan,
            &vertex_contact(t_rest, fan_rest)
        ));
        // The second triangle's opposite edge pierces TRI's interior.
        let pierce_rest = [[1.0, 1.0, -1.0], [1.0, 1.0, 1.0]];
        let pierce = [[0.0, 0.0, 0.0], pierce_rest[0], pierce_rest[1]];
        assert!(triangles_overlap_beyond_contact(
            TRI,
            pierce,
            &vertex_contact(t_rest, pierce_rest)
        ));
        // A coplanar wedge poking into TRI: its edges from the shared corner
        // run through TRI's interior, but its opposite edge stays outside.
        let wedge_rest = [[4.0, 1.0, 0.0], [1.0, 4.0, 0.0]];
        let wedge = [[0.0, 0.0, 0.0], wedge_rest[0], wedge_rest[1]];
        assert!(triangles_overlap_beyond_contact(
            TRI,
            wedge,
            &vertex_contact(t_rest, wedge_rest)
        ));
        // A coplanar wedge opening away from TRI touches only at the corner.
        let away_rest = [[-4.0, -1.0, 0.0], [-1.0, -4.0, 0.0]];
        let away = [[0.0, 0.0, 0.0], away_rest[0], away_rest[1]];
        assert!(!triangles_overlap_beyond_contact(
            TRI,
            away,
            &vertex_contact(t_rest, away_rest)
        ));
        // A wing running along TRI's edge shares a whole sub-segment.
        let along_rest = [[2.0, 0.0, 0.0], [0.0, -4.0, 1.0]];
        let along = [[0.0, 0.0, 0.0], along_rest[0], along_rest[1]];
        assert!(triangles_overlap_beyond_contact(
            TRI,
            along,
            &vertex_contact(t_rest, along_rest)
        ));
    }

    #[test]
    fn no_contact_pairs() {
        let pierce = [[1.0, 1.0, -1.0], [1.0, 1.0, 1.0], [3.0, 3.0, 1.0]];
        assert!(triangles_overlap_beyond_contact(
            TRI,
            pierce,
            &TriangleContact::None
        ));
        let above = TRI.map(|[x, y, _]| [x, y, 1.0]);
        assert!(!triangles_overlap_beyond_contact(
            TRI,
            above,
            &TriangleContact::None
        ));
    }
}
