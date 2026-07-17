//! Chain and ring simplicity: the shared detectors behind
//! [`SelfIntersection`](super::ValidationType::SelfIntersection).

use rstar::AABB;

use super::{open_ring, DuplicateCoord, ValidationReport};
use crate::coordinate::CoordinateFrame;
use crate::predicates::edge_set::{for_each_candidate_pair, Edge2, EdgeSet};
use crate::predicates::kernel::{segment_intersection, SegmentIntersection};
use crate::predicates::kernel3d::{classify_segments_3d, same_direction_3d, SegmentContact3D};

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

/// The 3D twin of [`check_chain_simple_2d`], deciding pairs with the exact 3D
/// kernel. Coordinates must be finite.
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

/// The 3D twin of [`check_ring_pair_2d`], deciding cross-ring edge pairs with
/// the exact 3D kernel.
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
