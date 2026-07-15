//! Robust geometric kernel for the new geometry predicates.
//!
//! Pure-Rust ports of the load-bearing legacy primitives, re-hosted over the new
//! flat `[f64; 2]` / `[f64; 3]` coordinate layout instead of the generic
//! `Coordinate<T, Z>`:
//!
//! - [`orient2d`] / [`orient3d`] — robust orientation signs (wrapping the
//!   `robust` crate's adaptive-precision predicates).
//! - [`segment_intersection`] — robust 2D segment × segment intersection,
//!   preserving the JTS central-endpoint conditioning of the legacy
//!   `line_intersection`.
//! - [`segment_intersection_3d`] — 3D segment × segment (coplanar) intersection.
//! - [`coord_pos_relative_to_ring`] — winding-number point-in-ring with an
//!   on-boundary short-circuit.
//!
//! All sign decisions go through the robust predicates; only *constructed* points
//! (the intersection coordinates) are f64-rounded, matching legacy behavior.

use robust::{orient2d as robust_orient2d, orient3d as robust_orient3d, Coord, Coord3D};

/// The orientation of an ordered point triple, from the robust sign of the
/// signed area (2D) or signed volume (3D).
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Orientation {
    /// Positive orientation (left turn / above the plane).
    CounterClockwise,
    /// Negative orientation (right turn / below the plane).
    Clockwise,
    /// Zero orientation: collinear (2D) or coplanar (3D).
    Collinear,
}

impl Orientation {
    /// Map the robust predicate's signed value to an [`Orientation`], using the
    /// same sign convention as the legacy kernel: positive is counter-clockwise.
    #[inline]
    fn from_sign(value: f64) -> Orientation {
        if value < 0.0 {
            Orientation::Clockwise
        } else if value > 0.0 {
            Orientation::CounterClockwise
        } else {
            Orientation::Collinear
        }
    }
}

/// Robust orientation of the 2D triple `(p, q, r)`: counter-clockwise if `r` is
/// left of the directed line `p -> q`, clockwise if right, collinear if on it.
#[inline]
pub fn orient2d(p: [f64; 2], q: [f64; 2], r: [f64; 2]) -> Orientation {
    Orientation::from_sign(robust_orient2d(
        Coord { x: p[0], y: p[1] },
        Coord { x: q[0], y: q[1] },
        Coord { x: r[0], y: r[1] },
    ))
}

/// Robust orientation of the 3D quadruple `(p, q, r, s)`: the sign of the signed
/// volume of the tetrahedron, i.e. which side of the plane through `p, q, r` the
/// point `s` lies on. Collinear here means `s` is coplanar with `p, q, r`.
#[inline]
pub fn orient3d(p: [f64; 3], q: [f64; 3], r: [f64; 3], s: [f64; 3]) -> Orientation {
    Orientation::from_sign(robust_orient3d(
        Coord3D {
            x: p[0],
            y: p[1],
            z: p[2],
        },
        Coord3D {
            x: q[0],
            y: q[1],
            z: q[2],
        },
        Coord3D {
            x: r[0],
            y: r[1],
            z: r[2],
        },
        Coord3D {
            x: s[0],
            y: s[1],
            z: s[2],
        },
    ))
}

/// The intersection of two 2D segments.
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum SegmentIntersection {
    /// The segments meet at a single point. `is_proper` is true when the
    /// crossing is in the interior of both segments (not at a shared endpoint or
    /// a T-junction).
    SinglePoint {
        /// The intersection point.
        intersection: [f64; 2],
        /// Whether the crossing is proper (interior to both segments).
        is_proper: bool,
    },
    /// The segments overlap along a collinear sub-segment `[start, end]`.
    Collinear {
        /// One end of the overlap.
        start: [f64; 2],
        /// The other end of the overlap.
        end: [f64; 2],
    },
}

/// Robust 2D segment × segment intersection.
///
/// A direct port of the legacy `line_intersection` (a JTS `RobustLineIntersector`
/// port): robust orientation short-circuits, exact endpoint copying when the
/// crossing is at an endpoint (for coordinate exactness), and central-endpoint
/// conditioning for the proper-crossing coordinate. Returns `None` when the
/// segments are disjoint.
pub fn segment_intersection(
    p_start: [f64; 2],
    p_end: [f64; 2],
    q_start: [f64; 2],
    q_end: [f64; 2],
) -> Option<SegmentIntersection> {
    // Bounding-box quick reject.
    if !segments_bbox_overlap(p_start, p_end, q_start, q_end) {
        return None;
    }

    let p_q1 = orient2d(p_start, p_end, q_start);
    let p_q2 = orient2d(p_start, p_end, q_end);
    if matches!(
        (p_q1, p_q2),
        (Orientation::Clockwise, Orientation::Clockwise)
            | (Orientation::CounterClockwise, Orientation::CounterClockwise)
    ) {
        return None;
    }

    let q_p1 = orient2d(q_start, q_end, p_start);
    let q_p2 = orient2d(q_start, q_end, p_end);
    if matches!(
        (q_p1, q_p2),
        (Orientation::Clockwise, Orientation::Clockwise)
            | (Orientation::CounterClockwise, Orientation::CounterClockwise)
    ) {
        return None;
    }

    if matches!(
        (p_q1, p_q2, q_p1, q_p2),
        (
            Orientation::Collinear,
            Orientation::Collinear,
            Orientation::Collinear,
            Orientation::Collinear
        )
    ) {
        return collinear_intersection(p_start, p_end, q_start, q_end);
    }

    // A single, non-collinear intersection. When it lands on an endpoint, copy
    // that endpoint verbatim so the returned coordinate is exact; otherwise
    // compute the crossing.
    if p_q1 == Orientation::Collinear
        || p_q2 == Orientation::Collinear
        || q_p1 == Orientation::Collinear
        || q_p2 == Orientation::Collinear
    {
        let intersection: [f64; 2];
        #[allow(clippy::suspicious_operation_groupings)]
        if p_start == q_start || p_start == q_end {
            intersection = p_start;
        } else if p_end == q_start || p_end == q_end {
            intersection = p_end;
        } else if p_q1 == Orientation::Collinear {
            intersection = q_start;
        } else if p_q2 == Orientation::Collinear {
            intersection = q_end;
        } else if q_p1 == Orientation::Collinear {
            intersection = p_start;
        } else {
            debug_assert_eq!(q_p2, Orientation::Collinear);
            intersection = p_end;
        }
        Some(SegmentIntersection::SinglePoint {
            intersection,
            is_proper: false,
        })
    } else {
        let intersection = proper_intersection(p_start, p_end, q_start, q_end);
        Some(SegmentIntersection::SinglePoint {
            intersection,
            is_proper: true,
        })
    }
}

/// Overlap of two collinear segments; a port of the legacy `collinear_intersection`.
fn collinear_intersection(
    p_start: [f64; 2],
    p_end: [f64; 2],
    q_start: [f64; 2],
    q_end: [f64; 2],
) -> Option<SegmentIntersection> {
    let collinear = |start: [f64; 2], end: [f64; 2]| SegmentIntersection::Collinear { start, end };
    let improper = |intersection: [f64; 2]| SegmentIntersection::SinglePoint {
        intersection,
        is_proper: false,
    };

    // `p_contains_qx`: does p's bounding box contain q's endpoint (inclusive)?
    let p_has_qs = point_in_bbox(q_start, p_start, p_end);
    let p_has_qe = point_in_bbox(q_end, p_start, p_end);
    let q_has_ps = point_in_bbox(p_start, q_start, q_end);
    let q_has_pe = point_in_bbox(p_end, q_start, q_end);

    Some(match (p_has_qs, p_has_qe, q_has_ps, q_has_pe) {
        (true, true, _, _) => collinear(q_start, q_end),
        (_, _, true, true) => collinear(p_start, p_end),
        (true, false, true, false) if q_start == p_start => improper(q_start),
        (true, _, true, _) => collinear(q_start, p_start),
        (true, false, false, true) if q_start == p_end => improper(q_start),
        (true, _, _, true) => collinear(q_start, p_end),
        (false, true, true, false) if q_end == p_start => improper(q_end),
        (_, true, true, _) => collinear(q_end, p_start),
        (false, true, false, true) if q_end == p_end => improper(q_end),
        (_, true, _, true) => collinear(q_end, p_end),
        _ => return None,
    })
}

/// The homogeneous-coordinate segment crossing with central-endpoint
/// conditioning; a port of the legacy `raw_line_intersection` (2D projection).
fn raw_line_intersection(
    p_start: [f64; 2],
    p_end: [f64; 2],
    q_start: [f64; 2],
    q_end: [f64; 2],
) -> Option<[f64; 2]> {
    let p_min_x = p_start[0].min(p_end[0]);
    let p_min_y = p_start[1].min(p_end[1]);
    let p_max_x = p_start[0].max(p_end[0]);
    let p_max_y = p_start[1].max(p_end[1]);

    let q_min_x = q_start[0].min(q_end[0]);
    let q_min_y = q_start[1].min(q_end[1]);
    let q_max_x = q_start[0].max(q_end[0]);
    let q_max_y = q_start[1].max(q_end[1]);

    let int_min_x = p_min_x.max(q_min_x);
    let int_max_x = p_max_x.min(q_max_x);
    let int_min_y = p_min_y.max(q_min_y);
    let int_max_y = p_max_y.min(q_max_y);

    let mid_x = (int_min_x + int_max_x) / 2.0;
    let mid_y = (int_min_y + int_max_y) / 2.0;

    // Condition ordinates by subtracting the midpoint of the overlap box.
    let p1x = p_start[0] - mid_x;
    let p1y = p_start[1] - mid_y;
    let p2x = p_end[0] - mid_x;
    let p2y = p_end[1] - mid_y;
    let q1x = q_start[0] - mid_x;
    let q1y = q_start[1] - mid_y;
    let q2x = q_end[0] - mid_x;
    let q2y = q_end[1] - mid_y;

    // Unrolled homogeneous-coordinate line intersection.
    let px = p1y - p2y;
    let py = p2x - p1x;
    let pw = p1x * p2y - p2x * p1y;

    let qx = q1y - q2y;
    let qy = q2x - q1x;
    let qw = q1x * q2y - q2x * q1y;

    let xw = py * qw - qy * pw;
    let yw = qx * pw - px * qw;
    let w = px * qy - qx * py;

    let x_int = xw / w;
    let y_int = yw / w;

    if x_int.is_nan() || x_int.is_infinite() || y_int.is_nan() || y_int.is_infinite() {
        None
    } else {
        Some([x_int + mid_x, y_int + mid_y])
    }
}

/// The endpoint of either segment nearest to the other segment; the legacy
/// `nearest_endpoint` fallback for the central-endpoint heuristic.
fn nearest_endpoint(
    p_start: [f64; 2],
    p_end: [f64; 2],
    q_start: [f64; 2],
    q_end: [f64; 2],
) -> [f64; 2] {
    let mut nearest = p_start;
    let mut min_dist = point_segment_distance(p_start, q_start, q_end);

    let dist = point_segment_distance(p_end, q_start, q_end);
    if dist < min_dist {
        min_dist = dist;
        nearest = p_end;
    }
    let dist = point_segment_distance(q_start, p_start, p_end);
    if dist < min_dist {
        min_dist = dist;
        nearest = q_start;
    }
    let dist = point_segment_distance(q_end, p_start, p_end);
    if dist < min_dist {
        nearest = q_end;
    }
    nearest
}

/// The proper (interior) crossing point, falling back to the nearest endpoint
/// when the raw computation lands outside both segments' boxes; a port of the
/// legacy `proper_intersection`.
fn proper_intersection(
    p_start: [f64; 2],
    p_end: [f64; 2],
    q_start: [f64; 2],
    q_end: [f64; 2],
) -> [f64; 2] {
    let mut pt = raw_line_intersection(p_start, p_end, q_start, q_end)
        .unwrap_or_else(|| nearest_endpoint(p_start, p_end, q_start, q_end));
    if !(point_in_bbox(pt, p_start, p_end) && point_in_bbox(pt, q_start, q_end)) {
        pt = nearest_endpoint(p_start, p_end, q_start, q_end);
    }
    pt
}

/// 3D segment × segment intersection, for coplanar, non-parallel segments; a
/// port of the legacy `line_intersection3d`. Returns the crossing point when it
/// lies within both segments, `None` for skew, parallel, or non-overlapping
/// segments.
pub fn segment_intersection_3d(
    p_start: [f64; 3],
    p_end: [f64; 3],
    q_start: [f64; 3],
    q_end: [f64; 3],
) -> Option<[f64; 3]> {
    let d1 = sub3(p_end, p_start);
    let d2 = sub3(q_end, q_start);

    let cross_d1_d2 = cross3(d1, d2);
    // Parallel or collinear.
    if norm_sq3(cross_d1_d2) == 0.0 {
        return None;
    }

    // Reject skew (non-coplanar) segments.
    let v1 = sub3(q_start, p_start);
    let plane_normal = cross3(d1, v1);
    if dot3(plane_normal, d2) != 0.0 {
        return None;
    }

    let denom = norm_sq3(cross_d1_d2);
    let t = dot3(cross3(v1, d2), cross_d1_d2) / denom;
    let u = dot3(cross3(v1, d1), cross_d1_d2) / denom;

    if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
        Some([
            p_start[0] + t * d1[0],
            p_start[1] + t * d1[1],
            p_start[2] + t * d1[2],
        ])
    } else {
        None
    }
}

/// Where a coordinate lies relative to a single closed ring.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum CoordPos {
    /// On the ring boundary.
    OnBoundary,
    /// Strictly inside the ring.
    Inside,
    /// Strictly outside the ring.
    Outside,
}

/// Point-in-ring by winding number, with an on-boundary short-circuit; a port of
/// the legacy `coord_pos_relative_to_ring`.
///
/// `ring` is a closed ring (first vertex == last), the storage convention of the
/// new [`Polygon`](crate::polygon) leaves. The `z` of any 2.5D ring is ignored:
/// the test is the XY-projection algorithm, matching legacy 2D semantics.
pub fn coord_pos_relative_to_ring(coord: [f64; 2], ring: &[[f64; 2]]) -> CoordPos {
    if ring.is_empty() {
        return CoordPos::Outside;
    }
    if ring.len() == 1 {
        return if coord == ring[0] {
            CoordPos::OnBoundary
        } else {
            CoordPos::Outside
        };
    }

    // Winding number with the standard edge-crossing rules:
    //   1. an upward edge includes its start, excludes its end;
    //   2. a downward edge excludes its start, includes its end;
    //   3. horizontal edges are ignored;
    //   4. the crossing must be strictly right of `coord`.
    let mut winding_number: i32 = 0;
    for edge in ring.windows(2) {
        let start = edge[0];
        let end = edge[1];
        if start[1] <= coord[1] {
            if end[1] >= coord[1] {
                let o = orient2d(start, end, coord);
                if o == Orientation::CounterClockwise && end[1] != coord[1] {
                    winding_number += 1;
                } else if o == Orientation::Collinear
                    && value_in_between(coord[0], start[0], end[0])
                {
                    return CoordPos::OnBoundary;
                }
            }
        } else if end[1] <= coord[1] {
            let o = orient2d(start, end, coord);
            if o == Orientation::Clockwise {
                winding_number -= 1;
            } else if o == Orientation::Collinear && value_in_between(coord[0], start[0], end[0]) {
                return CoordPos::OnBoundary;
            }
        }
    }

    if winding_number == 0 {
        CoordPos::Outside
    } else {
        CoordPos::Inside
    }
}

// --- small numeric helpers (ports of legacy `intersects`/`utils`) -----------

/// Whether `value` lies within `[min(a, b), max(a, b)]` (inclusive).
#[inline]
fn value_in_between(value: f64, a: f64, b: f64) -> bool {
    let (lo, hi) = if a < b { (a, b) } else { (b, a) };
    value >= lo && value <= hi
}

/// Whether `p` lies inside the axis-aligned box spanned by `a` and `b`
/// (inclusive on every axis).
#[inline]
fn point_in_bbox(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> bool {
    value_in_between(p[0], a[0], b[0]) && value_in_between(p[1], a[1], b[1])
}

/// Whether the bounding boxes of two 2D segments overlap (inclusive).
#[inline]
fn segments_bbox_overlap(
    p_start: [f64; 2],
    p_end: [f64; 2],
    q_start: [f64; 2],
    q_end: [f64; 2],
) -> bool {
    let p_min_x = p_start[0].min(p_end[0]);
    let p_max_x = p_start[0].max(p_end[0]);
    let p_min_y = p_start[1].min(p_end[1]);
    let p_max_y = p_start[1].max(p_end[1]);
    let q_min_x = q_start[0].min(q_end[0]);
    let q_max_x = q_start[0].max(q_end[0]);
    let q_min_y = q_start[1].min(q_end[1]);
    let q_max_y = q_start[1].max(q_end[1]);

    p_min_x <= q_max_x && q_min_x <= p_max_x && p_min_y <= q_max_y && q_min_y <= p_max_y
}

/// Euclidean distance from a 2D point to a segment; a port of the legacy
/// `line_segment_distance`.
fn point_segment_distance(point: [f64; 2], start: [f64; 2], end: [f64; 2]) -> f64 {
    if start == end {
        return (point[0] - start[0]).hypot(point[1] - start[1]);
    }
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let r = ((point[0] - start[0]) * dx + (point[1] - start[1]) * dy) / (dx * dx + dy * dy);
    if r <= 0.0 {
        return (point[0] - start[0]).hypot(point[1] - start[1]);
    }
    if r >= 1.0 {
        return (point[0] - end[0]).hypot(point[1] - end[1]);
    }
    let s = ((start[1] - point[1]) * dx - (start[0] - point[0]) * dy) / (dx * dx + dy * dy);
    s.abs() * dx.hypot(dy)
}

// --- 3D vector helpers ------------------------------------------------------

#[inline]
fn sub3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn cross3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

#[inline]
fn norm_sq3(a: [f64; 3]) -> f64 {
    dot3(a, a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orient2d_basic_signs() {
        assert_eq!(
            orient2d([0.0, 0.0], [1.0, 0.0], [0.0, 1.0]),
            Orientation::CounterClockwise
        );
        assert_eq!(
            orient2d([0.0, 0.0], [1.0, 0.0], [0.0, -1.0]),
            Orientation::Clockwise
        );
        assert_eq!(
            orient2d([0.0, 0.0], [1.0, 0.0], [2.0, 0.0]),
            Orientation::Collinear
        );
    }

    #[test]
    fn orient3d_basic_signs() {
        // z above / below the z=0 plane spanned by the first three points.
        let a = [0.0, 0.0, 0.0];
        let b = [1.0, 0.0, 0.0];
        let c = [0.0, 1.0, 0.0];
        assert_ne!(
            orient3d(a, b, c, [0.0, 0.0, 1.0]),
            orient3d(a, b, c, [0.0, 0.0, -1.0])
        );
        assert_eq!(orient3d(a, b, c, [5.0, 5.0, 0.0]), Orientation::Collinear);
    }

    #[test]
    fn proper_crossing() {
        let x = segment_intersection([0.0, 0.0], [2.0, 2.0], [0.0, 2.0], [2.0, 0.0]);
        assert_eq!(
            x,
            Some(SegmentIntersection::SinglePoint {
                intersection: [1.0, 1.0],
                is_proper: true,
            })
        );
    }

    #[test]
    fn endpoint_touch_is_improper_and_exact() {
        // q starts exactly on p's interior: a T-junction, improper, exact copy.
        let x = segment_intersection([0.0, 0.0], [4.0, 0.0], [2.0, 0.0], [2.0, 3.0]);
        assert_eq!(
            x,
            Some(SegmentIntersection::SinglePoint {
                intersection: [2.0, 0.0],
                is_proper: false,
            })
        );
    }

    #[test]
    fn disjoint_segments_do_not_intersect() {
        assert_eq!(
            segment_intersection([0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]),
            None
        );
    }

    #[test]
    fn collinear_overlap() {
        let x = segment_intersection([0.0, 0.0], [4.0, 0.0], [2.0, 0.0], [6.0, 0.0]);
        match x {
            Some(SegmentIntersection::Collinear { start, end }) => {
                let mut pts = [start, end];
                pts.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());
                assert_eq!(pts, [[2.0, 0.0], [4.0, 0.0]]);
            }
            other => panic!("expected collinear overlap, got {other:?}"),
        }
    }

    // JTS/GEOS central-endpoint heuristic cases carried over from the legacy
    // `line_intersection` test suite (the intersection coordinate must match
    // exactly to preserve robustness parity).
    #[test]
    fn jts_central_endpoint_heuristic_failure_1() {
        let x = segment_intersection(
            [163.81867067, -211.31840378],
            [165.9174252, -214.1665075],
            [2.84139601, -57.95412726],
            [469.59990601, -502.63851732],
        );
        assert_eq!(
            x,
            Some(SegmentIntersection::SinglePoint {
                intersection: [163.81867067, -211.31840378],
                is_proper: true,
            })
        );
    }

    #[test]
    fn jts_leduc_1() {
        let x = segment_intersection(
            [305690.0434123494, 254176.46578338774],
            [305601.9999843455, 254243.19999846347],
            [305689.6153764265, 254177.33102743194],
            [305692.4999844298, 254171.4999983967],
        );
        assert_eq!(
            x,
            Some(SegmentIntersection::SinglePoint {
                intersection: [305690.0434123494, 254176.46578338774],
                is_proper: true,
            })
        );
    }

    #[test]
    fn geos_1() {
        let x = segment_intersection(
            [588750.7429703881, 4518950.493668233],
            [588748.2060409798, 4518933.9452804085],
            [588745.824857241, 4518940.742239175],
            [588748.2060437313, 4518933.9452791475],
        );
        assert_eq!(
            x,
            Some(SegmentIntersection::SinglePoint {
                intersection: [588748.2060416829, 4518933.945284994],
                is_proper: true,
            })
        );
    }

    #[test]
    fn segment_intersection_3d_coplanar_crossing() {
        let x = segment_intersection_3d(
            [0.0, 0.0, 0.0],
            [2.0, 2.0, 2.0],
            [2.0, 0.0, 0.0],
            [0.0, 2.0, 2.0],
        );
        assert_eq!(x, Some([1.0, 1.0, 1.0]));
    }

    #[test]
    fn segment_intersection_3d_skew_is_none() {
        // Skew lines (different planes) do not meet.
        assert_eq!(
            segment_intersection_3d(
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 1.0],
                [0.0, -1.0, 1.0],
            ),
            None
        );
    }

    #[test]
    fn point_in_ring_winding() {
        // CCW unit square, stored closed.
        let sq: [[f64; 2]; 5] = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];
        assert_eq!(
            coord_pos_relative_to_ring([2.0, 2.0], &sq),
            CoordPos::Inside
        );
        assert_eq!(
            coord_pos_relative_to_ring([5.0, 2.0], &sq),
            CoordPos::Outside
        );
        assert_eq!(
            coord_pos_relative_to_ring([0.0, 2.0], &sq),
            CoordPos::OnBoundary
        );
        // A vertex is on the boundary.
        assert_eq!(
            coord_pos_relative_to_ring([4.0, 4.0], &sq),
            CoordPos::OnBoundary
        );
    }

    #[test]
    fn point_in_ring_orientation_independent() {
        // CW winding gives the same inside/outside answer.
        let cw: [[f64; 2]; 5] = [[0.0, 0.0], [0.0, 4.0], [4.0, 4.0], [4.0, 0.0], [0.0, 0.0]];
        assert_eq!(
            coord_pos_relative_to_ring([2.0, 2.0], &cw),
            CoordPos::Inside
        );
        assert_eq!(
            coord_pos_relative_to_ring([9.0, 9.0], &cw),
            CoordPos::Outside
        );
    }

    #[test]
    fn degenerate_rings() {
        assert_eq!(
            coord_pos_relative_to_ring([0.0, 0.0], &[]),
            CoordPos::Outside
        );
        assert_eq!(
            coord_pos_relative_to_ring([1.0, 1.0], &[[1.0, 1.0]]),
            CoordPos::OnBoundary
        );
    }
}
