//! Pairwise segment × segment intersections between two polyline sets.
//!
//! Every segment of one operand is tested against every bbox-overlapping
//! segment of the other via an rstar-indexed candidate sweep, and each hit
//! goes through the robust
//! [`kernel::segment_intersection`](segment_intersection).

use std::cmp::Ordering;

use rstar::RTree;

use crate::predicates::kernel::{segment_intersection, SegmentIntersection};
use crate::predicates::view::Leaf2D;

use super::leaf_type_name;

/// One directed segment of a polyline, with its precomputed rstar envelope.
pub(super) struct Segment {
    start: [f64; 2],
    end: [f64; 2],
    envelope: rstar::AABB<[f64; 2]>,
}

impl rstar::RTreeObject for Segment {
    type Envelope = rstar::AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.envelope
    }
}

/// The segments of the line leaves. Zero-length segments (repeated vertices)
/// are dropped: they carry no direction and no crossing. Errs with the leaf's
/// type name on the first non-line leaf.
pub(super) fn leaf_segments(leaves: &[Leaf2D<'_>]) -> Result<Vec<Segment>, &'static str> {
    let mut segments = Vec::new();
    for leaf in leaves {
        match leaf {
            Leaf2D::Line(l) => segments.extend(l.coords().windows(2).filter(|w| w[0] != w[1]).map(
                |w| Segment {
                    start: w[0],
                    end: w[1],
                    envelope: rstar::AABB::from_corners(w[0], w[1]),
                },
            )),
            _ => return Err(leaf_type_name(leaf)),
        }
    }
    Ok(segments)
}

/// All intersections between the two segment sets, deduplicated and in a
/// deterministic (lexicographic) order. Collinear overlaps are canonicalized
/// to lexicographically ordered endpoints so the same span found from
/// different segment pairs deduplicates.
pub(super) fn intersections(a: Vec<Segment>, b: Vec<Segment>) -> Vec<SegmentIntersection> {
    let tree_a = RTree::bulk_load(a);
    let tree_b = RTree::bulk_load(b);

    let mut hits: Vec<SegmentIntersection> = tree_a
        .intersection_candidates_with_other_tree(&tree_b)
        .filter_map(|(sa, sb)| segment_intersection(sa.start, sa.end, sb.start, sb.end))
        .map(canonicalize)
        .collect();
    hits.sort_by(cmp_hits);
    hits.dedup();
    hits
}

/// Order a collinear overlap's endpoints lexicographically.
fn canonicalize(hit: SegmentIntersection) -> SegmentIntersection {
    match hit {
        SegmentIntersection::Collinear { start, end } if cmp_coords(&end, &start).is_lt() => {
            SegmentIntersection::Collinear {
                start: end,
                end: start,
            }
        }
        other => other,
    }
}

fn cmp_coords(a: &[f64; 2], b: &[f64; 2]) -> Ordering {
    a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1]))
}

/// Total order over hits: points before overlaps, then by coordinates, with
/// proper crossings before improper ones at the same point.
fn cmp_hits(a: &SegmentIntersection, b: &SegmentIntersection) -> Ordering {
    use SegmentIntersection::*;
    match (a, b) {
        (SinglePoint { .. }, Collinear { .. }) => Ordering::Less,
        (Collinear { .. }, SinglePoint { .. }) => Ordering::Greater,
        (
            SinglePoint {
                intersection: pa,
                is_proper: qa,
            },
            SinglePoint {
                intersection: pb,
                is_proper: qb,
            },
        ) => cmp_coords(pa, pb).then(qb.cmp(qa)),
        (Collinear { start: sa, end: ea }, Collinear { start: sb, end: eb }) => {
            cmp_coords(sa, sb).then(cmp_coords(ea, eb))
        }
    }
}
