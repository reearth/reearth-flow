//! Chain and ring simplicity: the shared detectors behind
//! [`SelfIntersection`](super::ValidationType::SelfIntersection), and the
//! ring-touch detector behind
//! [`InteriorConnectivity`](super::ValidationType::InteriorConnectivity).

use std::collections::HashSet;

use reearth_flow_common::union_find::UnionFind;
use rstar::AABB;

use super::{DuplicateCoord, IssuePart, ValidationReport, open_ring};
use crate::coordinate::CoordinateFrame;
use crate::predicates::edge_set::{Edge2, EdgeSet, for_each_candidate_pair};
use crate::predicates::kernel::{SegmentIntersection, segment_intersection};
use crate::predicates::kernel3d::{SegmentContact3D, classify_segments_3d, same_direction_3d};

/// Record a problem at a coordinate, as a point leaf of its dimension.
fn push_point<const N: usize>(
    frame: &CoordinateFrame,
    coord: [f64; N],
    report: &mut ValidationReport,
) where
    [f64; N]: DuplicateCoord,
{
    report.push(coord.into_point(frame));
}

/// The chain's consecutive-distinct vertices and whether it closes. A closed
/// chain (first == last) has its closing duplicate dropped and is treated
/// cyclically by the pairing below.
fn distinct_vertices<const N: usize>(coords: &[[f64; N]]) -> (Vec<[f64; N]>, bool) {
    let closed = coords.len() >= 3 && coords.first() == coords.last();
    let src = if closed { open_ring(coords) } else { coords };
    let mut vertices: Vec<[f64; N]> = Vec::with_capacity(src.len());
    for &c in src {
        if vertices.last() != Some(&c) {
            vertices.push(c);
        }
    }
    if closed && vertices.len() > 1 && vertices.first() == vertices.last() {
        vertices.pop();
    }
    (vertices, closed)
}

/// Run the candidate-pair sweep over a chain's edges, classifying each pair as
/// adjacent (sharing one endpoint by construction) or not, and let `decide`
/// record any problem. `edge(i)` is `(vertices[i], vertices[(i + 1) % n])`.
fn sweep_chain_pairs<const N: usize>(
    vertices: &[[f64; N]],
    closed: bool,
    mut decide: impl FnMut(([f64; N], [f64; N]), ([f64; N], [f64; N]), Option<[f64; N]>),
) where
    [f64; N]: rstar::Point,
{
    let n = vertices.len();
    let n_edges = if closed { n } else { n.saturating_sub(1) };
    if n_edges < 2 {
        return;
    }
    let edge = |i: usize| (vertices[i], vertices[(i + 1) % n]);
    let envelopes: Vec<AABB<[f64; N]>> = (0..n_edges)
        .map(|i| {
            let (a, b) = edge(i);
            AABB::from_corners(a, b)
        })
        .collect();
    for_each_candidate_pair(&envelopes, |i, j| {
        let (e1, e2) = (edge(i), edge(j));
        let shared = if j == i + 1 {
            Some(e1.1)
        } else if closed && i == 0 && j == n_edges - 1 {
            Some(e1.0)
        } else {
            None
        };
        decide(e1, e2, shared);
    });
}

/// Report a [`SelfIntersection`](super::ValidationType::SelfIntersection)
/// problem at each point where a 2D chain intersects itself. A closed chain
/// (first == last) is treated cyclically, so its closure contact is allowed;
/// rings are stored closed and need no separate entry point. Adjacent edges
/// may meet only at their shared vertex (anything more is a spike or fold);
/// non-adjacent edges may not meet at all. Consecutive duplicate vertices are
/// collapsed first, so a repeated point alone is not a self-intersection.
/// Coordinates must be finite.
pub(crate) fn check_chain_simple_2d(
    frame: &CoordinateFrame,
    coords: &[[f64; 2]],
    report: &mut ValidationReport,
) {
    let (vertices, closed) = distinct_vertices(coords);
    sweep_chain_pairs(&vertices, closed, |(a1, b1), (a2, b2), shared| {
        match (segment_intersection(a1, b1, a2, b2), shared) {
            (None, _) => {}
            (Some(SegmentIntersection::SinglePoint { intersection, .. }), Some(shared)) => {
                if intersection != shared {
                    push_point(frame, intersection, report);
                }
            }
            (Some(SegmentIntersection::Collinear { .. }), Some(shared)) => {
                push_point(frame, shared, report);
            }
            (Some(SegmentIntersection::SinglePoint { intersection, .. }), None) => {
                push_point(frame, intersection, report);
            }
            (Some(SegmentIntersection::Collinear { start, .. }), None) => {
                push_point(frame, start, report);
            }
        }
    });
}

/// Report a [`SelfIntersection`](super::ValidationType::SelfIntersection)
/// problem at each point where a 3D chain meets itself, decided with the exact
/// 3D kernel. Coordinates must be finite.
pub(crate) fn check_chain_simple_3d(
    frame: &CoordinateFrame,
    coords: &[[f64; 3]],
    report: &mut ValidationReport,
) {
    let (vertices, closed) = distinct_vertices(coords);
    sweep_chain_pairs(&vertices, closed, |(a1, b1), (a2, b2), shared| {
        match shared {
            // Adjacent edges share exactly one endpoint, so they meet beyond
            // it only by folding back along it.
            Some(shared) => {
                let u = if shared == b1 { a1 } else { b1 };
                let w = if shared == a2 { b2 } else { a2 };
                if same_direction_3d(shared, u, w) {
                    push_point(frame, shared, report);
                }
            }
            None => match classify_segments_3d(a1, b1, a2, b2) {
                None => {}
                Some(SegmentContact3D::Proper(p) | SegmentContact3D::Touch(p)) => {
                    push_point(frame, p, report);
                }
                Some(SegmentContact3D::Overlap { start, .. }) => {
                    push_point(frame, start, report);
                }
            },
        }
    });
}

/// Report a [`SelfIntersection`](super::ValidationType::SelfIntersection)
/// problem where two different rings of one face cross: a proper crossing or
/// a collinear overlap between their edges; touching at isolated points is
/// allowed. Both rings must be stored closed (first == last) with finite
/// coordinates. Positions are points at each crossing (overlap start for
/// collinear).
pub(crate) fn check_ring_pair_2d(
    frame: &CoordinateFrame,
    ring_a: &[[f64; 2]],
    ring_b: &[[f64; 2]],
    report: &mut ValidationReport,
) {
    let edges_a: Vec<Edge2> = ring_a.windows(2).map(|w| (w[0], w[1])).collect();
    let probes = ring_b.len().saturating_sub(1);
    let set = EdgeSet::new(edges_a, probes);
    for w in ring_b.windows(2) {
        let (u, v) = (w[0], w[1]);
        set.probe(u, v, |s, t| {
            match segment_intersection(u, v, s, t) {
                Some(SegmentIntersection::SinglePoint {
                    intersection,
                    is_proper: true,
                }) => push_point(frame, intersection, report),
                Some(SegmentIntersection::Collinear { start, .. }) => {
                    push_point(frame, start, report)
                }
                _ => {}
            }
            false
        });
    }
}

/// Report a [`SelfIntersection`](super::ValidationType::SelfIntersection)
/// problem where two rings of one face cross, deciding edge pairs with the
/// exact 3D kernel.
pub(crate) fn check_ring_pair_3d(
    frame: &CoordinateFrame,
    ring_a: &[[f64; 3]],
    ring_b: &[[f64; 3]],
    report: &mut ValidationReport,
) {
    let mut edges: Vec<([f64; 3], [f64; 3])> = ring_a.windows(2).map(|w| (w[0], w[1])).collect();
    let n_a = edges.len();
    edges.extend(ring_b.windows(2).map(|w| (w[0], w[1])));
    let envelopes: Vec<AABB<[f64; 3]>> = edges
        .iter()
        .map(|&(a, b)| AABB::from_corners(a, b))
        .collect();
    for_each_candidate_pair(&envelopes, |i, j| {
        if (i < n_a) == (j < n_a) {
            return;
        }
        let (a1, b1) = edges[i];
        let (a2, b2) = edges[j];
        match classify_segments_3d(a1, b1, a2, b2) {
            Some(SegmentContact3D::Proper(p)) => push_point(frame, p, report),
            Some(SegmentContact3D::Overlap { start, .. }) => push_point(frame, start, report),
            _ => {}
        }
    });
}

/// The points where two closed rings touch without crossing, each once, in the
/// order found. `touch` decides one edge pair: the shared point of a touch, or
/// `None` for a miss, a proper crossing or an overlap.
fn ring_pair_touches<const N: usize>(
    ring_a: &[[f64; N]],
    ring_b: &[[f64; N]],
    touch: &impl Fn([f64; N], [f64; N], [f64; N], [f64; N]) -> Option<[f64; N]>,
) -> Vec<[f64; N]>
where
    [f64; N]: rstar::Point,
{
    let mut edges: Vec<([f64; N], [f64; N])> = ring_a.windows(2).map(|w| (w[0], w[1])).collect();
    let n_a = edges.len();
    edges.extend(ring_b.windows(2).map(|w| (w[0], w[1])));
    let envelopes: Vec<AABB<[f64; N]>> = edges
        .iter()
        .map(|&(a, b)| AABB::from_corners(a, b))
        .collect();
    let mut points: Vec<[f64; N]> = Vec::new();
    for_each_candidate_pair(&envelopes, |i, j| {
        if (i < n_a) == (j < n_a) {
            return;
        }
        let ((a1, b1), (a2, b2)) = (edges[i], edges[j]);
        if let Some(p) = touch(a1, b1, a2, b2)
            && !points.contains(&p)
        {
            points.push(p);
        }
    });
    points
}

/// Report an [`InteriorConnectivity`](super::ValidationType::InteriorConnectivity)
/// problem at each point where the rings of one face touch so that its
/// interior falls apart, attributed to the touching ring pair. `rings[0]` is the
/// exterior and the rest are holes, all closed and crossing neither themselves
/// nor each other.
///
/// Rings and touch points form a graph with an edge from each ring to each
/// point it touches. The interior is connected exactly when that graph has no
/// cycle, so every touch that closes one is reported: a hole touching the
/// exterior twice, or a chain of holes touching back to where it started. A
/// point where three or more rings meet is a single node, so it closes no cycle
/// on its own.
fn check_interior_connected<const N: usize>(
    frame: &CoordinateFrame,
    rings: &[&[[f64; N]]],
    touch: impl Fn([f64; N], [f64; N], [f64; N], [f64; N]) -> Option<[f64; N]>,
    report: &mut ValidationReport,
) where
    [f64; N]: rstar::Point + DuplicateCoord,
{
    let mut points: Vec<[f64; N]> = Vec::new();
    let mut links: Vec<(usize, usize, usize)> = Vec::new();
    for i in 0..rings.len() {
        for j in (i + 1)..rings.len() {
            for p in ring_pair_touches(rings[i], rings[j], &touch) {
                let k = points.iter().position(|q| *q == p).unwrap_or_else(|| {
                    points.push(p);
                    points.len() - 1
                });
                links.push((i, j, k));
            }
        }
    }
    let mut sets = UnionFind::<()>::new(rings.len() + points.len());
    let mut linked: HashSet<(usize, usize)> = HashSet::new();
    for (i, j, k) in links {
        let point = rings.len() + k;
        let mut cuts = false;
        for ring in [i, j] {
            if !linked.insert((ring, k)) {
                continue;
            }
            if sets.root(ring) == sets.root(point) {
                cuts = true;
            } else {
                sets.merge(ring, point);
            }
        }
        if cuts {
            report.record_part(IssuePart::RingPair(i, j), |r| {
                push_point(frame, points[k], r)
            });
        }
    }
}

/// [`check_interior_connected`] for the rings of a 2D face.
pub(crate) fn check_interior_connected_2d(
    frame: &CoordinateFrame,
    rings: &[&[[f64; 2]]],
    report: &mut ValidationReport,
) {
    check_interior_connected(
        frame,
        rings,
        |a1, b1, a2, b2| match segment_intersection(a1, b1, a2, b2) {
            Some(SegmentIntersection::SinglePoint {
                intersection,
                is_proper: false,
            }) => Some(intersection),
            _ => None,
        },
        report,
    );
}

/// [`check_interior_connected`] for the rings of a 3D face, deciding edge pairs
/// with the exact 3D kernel.
pub(crate) fn check_interior_connected_3d(
    frame: &CoordinateFrame,
    rings: &[&[[f64; 3]]],
    report: &mut ValidationReport,
) {
    check_interior_connected(
        frame,
        rings,
        |a1, b1, a2, b2| match classify_segments_3d(a1, b1, a2, b2) {
            Some(SegmentContact3D::Touch(p)) => Some(p),
            _ => None,
        },
        report,
    );
}
