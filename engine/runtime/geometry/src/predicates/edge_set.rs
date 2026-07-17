//! A queryable set of 2D segments with a size-gated search strategy.
//!
//! [`EdgeSet`] answers "visit every stored segment that may touch this probe
//! segment" either by a linear scan or through an rstar index. The strategy is
//! chosen at construction from the expected total number of kernel calls, so
//! small inputs skip the index build and large inputs avoid the quadratic
//! scan. Both strategies visit a superset of the truly intersecting segments;
//! callers decide each candidate with the exact kernel, so the strategy never
//! changes an answer.

use rstar::RTree;

use super::view::{Leaf2D, Operand2D};

/// One directed 2D segment as an endpoint pair.
pub(crate) type Edge2 = ([f64; 2], [f64; 2]);

/// Kernel-call budget above which a probe sweep builds an rstar index instead
/// of scanning linearly.
// TODO: tune with a benchmark on representative datasets.
pub(crate) const DIRECT_WORK_LIMIT: usize = 4096;

/// Whether probing `edges` stored segments `probes` times warrants an index.
pub(crate) fn should_index(edges: usize, probes: usize) -> bool {
    edges.saturating_mul(probes) > DIRECT_WORK_LIMIT
}

/// One stored segment with its precomputed rstar envelope.
pub(crate) struct IndexedEdge {
    start: [f64; 2],
    end: [f64; 2],
    envelope: rstar::AABB<[f64; 2]>,
}

impl rstar::RTreeObject for IndexedEdge {
    type Envelope = rstar::AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.envelope
    }
}

/// A set of 2D segments queryable by probe segment, behind one of two
/// strategies: a plain vector scanned linearly, or an rstar tree queried by
/// the probe's bounding box.
pub(crate) enum EdgeSet {
    /// Scan every stored segment per probe.
    Linear(Vec<Edge2>),
    /// Query segments whose box overlaps the probe's box.
    Indexed(RTree<IndexedEdge>),
}

impl EdgeSet {
    /// Build the set, picking the strategy from the stored segment count and
    /// the expected number of probes.
    pub(crate) fn new(edges: Vec<Edge2>, probe_hint: usize) -> Self {
        let indexed = should_index(edges.len(), probe_hint);
        Self::with_strategy(edges, indexed)
    }

    /// Build the set with an explicit strategy.
    pub(crate) fn with_strategy(edges: Vec<Edge2>, indexed: bool) -> Self {
        if indexed {
            EdgeSet::Indexed(RTree::bulk_load(
                edges
                    .into_iter()
                    .map(|(start, end)| IndexedEdge {
                        start,
                        end,
                        envelope: rstar::AABB::from_corners(start, end),
                    })
                    .collect(),
            ))
        } else {
            EdgeSet::Linear(edges)
        }
    }

    /// Visit stored segments that may touch the probe `u -> v`, stopping early
    /// when `visit` returns `true` and reporting whether it did. The linear
    /// strategy visits every segment; the indexed one only those whose box
    /// overlaps the probe's box, which never skips an intersecting segment.
    pub(crate) fn probe(
        &self,
        u: [f64; 2],
        v: [f64; 2],
        mut visit: impl FnMut([f64; 2], [f64; 2]) -> bool,
    ) -> bool {
        match self {
            EdgeSet::Linear(edges) => {
                for &(s, t) in edges {
                    if visit(s, t) {
                        return true;
                    }
                }
                false
            }
            EdgeSet::Indexed(tree) => {
                let envelope = rstar::AABB::from_corners(u, v);
                for edge in tree.locate_in_envelope_intersecting(&envelope) {
                    if visit(edge.start, edge.end) {
                        return true;
                    }
                }
                false
            }
        }
    }
}

/// Every segment of the operand: line chain segments and areal ring edges
/// (points contribute nothing). Zero-length segments are kept verbatim.
pub(crate) fn operand_edges(operand: &Operand2D<'_>) -> Vec<Edge2> {
    let mut edges = Vec::new();
    for prepared in &operand.leaves {
        match prepared.leaf {
            Leaf2D::Point(_) => {}
            Leaf2D::Line(l) => {
                edges.extend(l.coords().windows(2).map(|s| (s[0], s[1])));
            }
            _ => {
                edges.extend(prepared.area.as_ref().expect("leaf is areal").edges());
            }
        }
    }
    edges
}

/// The operand's segment count (the length [`operand_edges`] would have),
/// computed structurally without touching coordinates.
pub(crate) fn operand_segment_count(operand: &Operand2D<'_>) -> usize {
    operand
        .leaves
        .iter()
        .map(|prepared| match prepared.leaf {
            Leaf2D::Point(_) => 0,
            Leaf2D::Line(l) => l.coords().len().saturating_sub(1),
            _ => {
                let view = prepared.area.as_ref().expect("leaf is areal");
                view.faces()
                    .flat_map(|f| f.rings())
                    .map(|r| r.open_len())
                    .sum()
            }
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid_edges(n: usize) -> Vec<Edge2> {
        (0..n)
            .map(|i| {
                let x = i as f64;
                ([x, 0.0], [x, 1.0])
            })
            .collect()
    }

    #[test]
    fn strategies_visit_the_same_intersecting_segments() {
        let edges = grid_edges(20);
        let linear = EdgeSet::with_strategy(edges.clone(), false);
        let indexed = EdgeSet::with_strategy(edges, true);

        let collect = |set: &EdgeSet, u: [f64; 2], v: [f64; 2]| {
            let mut hits: Vec<Edge2> = Vec::new();
            set.probe(u, v, |s, t| {
                if crate::predicates::kernel::segment_intersection(u, v, s, t).is_some() {
                    hits.push((s, t));
                }
                false
            });
            hits.sort_by(|a, b| a.partial_cmp(b).unwrap());
            hits
        };

        let probes = [
            ([2.5, 0.5], [7.5, 0.5]),
            ([0.0, 0.0], [19.0, 1.0]),
            ([-5.0, 0.5], [-1.0, 0.5]),
            ([3.0, 0.0], [3.0, 1.0]),
        ];
        for (u, v) in probes {
            assert_eq!(collect(&linear, u, v), collect(&indexed, u, v));
        }
    }

    #[test]
    fn probe_early_exit_reports_a_hit() {
        for indexed in [false, true] {
            let set = EdgeSet::with_strategy(grid_edges(10), indexed);
            assert!(set.probe([4.5, 0.5], [5.5, 0.5], |_, _| true));
            assert!(!set.probe([100.0, 0.5], [101.0, 0.5], |s, t| {
                crate::predicates::kernel::segment_intersection([100.0, 0.5], [101.0, 0.5], s, t)
                    .is_some()
            }));
        }
    }

    #[test]
    fn should_index_gates_on_the_work_product() {
        assert!(!should_index(10, 10));
        assert!(!should_index(0, usize::MAX));
        assert!(should_index(100, 100));
        assert!(should_index(usize::MAX, 2));
    }
}
