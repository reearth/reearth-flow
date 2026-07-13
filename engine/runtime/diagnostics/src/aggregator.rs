use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use uuid::Uuid;

use crate::{AggregateInfo, Diagnostic, DiagnosticDraft, Disposition, ErrorCode, Severity};

/// Sample-id cap per bucket (spec 4.4: "capped ≈10").
pub const SAMPLE_FEATURE_ID_CAP: usize = 10;

/// Which aggregation lane a report lands in. Fatal is never aggregated —
/// it goes to the per-node fatal slot and fails the node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DiagnosticKind {
    WarnContinue,
    WarnDrop,
    Reject,
}

/// Run-scoped dedup set for `warn_once` (shared by every node of a run).
pub type WarnOnceSet = Arc<Mutex<HashSet<ErrorCode>>>;

#[derive(Debug, Default)]
struct Bucket {
    count: u64,
    sample_feature_ids: Vec<Uuid>,
}

/// Per-node aggregator: counts per (code, kind) with sampled feature ids,
/// plus the first-wins fatal slot that backstops swallowed `report()` errors.
#[derive(Debug)]
pub struct NodeDiagnostics {
    node_id: String,
    action_type: String,
    buckets: Mutex<HashMap<(ErrorCode, DiagnosticKind), Bucket>>,
    fatal: Mutex<Option<Diagnostic>>,
    warn_once: WarnOnceSet,
}

impl NodeDiagnostics {
    pub fn new(node_id: String, action_type: String, warn_once: WarnOnceSet) -> Self {
        Self {
            node_id,
            action_type,
            buckets: Mutex::new(HashMap::new()),
            fatal: Mutex::new(None),
            warn_once,
        }
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    pub fn action_type(&self) -> &str {
        &self.action_type
    }

    /// O(1): increment + bounded sample push.
    pub fn record(&self, kind: DiagnosticKind, code: ErrorCode, feature_id: Option<Uuid>) {
        let mut buckets = self.buckets.lock().unwrap();
        let bucket = buckets.entry((code, kind)).or_default();
        bucket.count += 1;
        if let Some(id) = feature_id {
            if bucket.sample_feature_ids.len() < SAMPLE_FEATURE_ID_CAP {
                bucket.sample_feature_ids.push(id);
            }
        }
    }

    /// First-wins: the executor fails the node with the first recorded fatal
    /// even if the action swallowed `report()`'s Err.
    pub fn record_fatal(&self, diagnostic: Diagnostic) {
        let mut slot = self.fatal.lock().unwrap();
        if slot.is_none() {
            *slot = Some(diagnostic);
        }
    }

    pub fn take_fatal(&self) -> Option<Diagnostic> {
        self.fatal.lock().unwrap().take()
    }

    /// Returns true exactly once per run per code (run-scoped set).
    pub fn try_mark_warn_once(&self, code: ErrorCode) -> bool {
        self.warn_once.lock().unwrap().insert(code)
    }

    /// One summary Diagnostic per (code, kind), deterministic order, buckets drained.
    /// Consumers read `aggregated.count` — the message is only the human rendering.
    pub fn drain_summaries(&self) -> Vec<Diagnostic> {
        let mut drained: Vec<((ErrorCode, DiagnosticKind), Bucket)> =
            self.buckets.lock().unwrap().drain().collect();
        drained
            .sort_by(|((ca, ka), _), ((cb, kb), _)| ca.as_str().cmp(cb.as_str()).then(ka.cmp(kb)));
        drained
            .into_iter()
            .map(|((code, kind), bucket)| self.summarize(code, kind, bucket))
            .collect()
    }

    fn summarize(&self, code: ErrorCode, kind: DiagnosticKind, bucket: Bucket) -> Diagnostic {
        let verb = match kind {
            DiagnosticKind::WarnContinue => "warned about",
            DiagnosticKind::WarnDrop => "dropped",
            DiagnosticKind::Reject => "rejected",
        };
        let mut message = format!(
            "{} (node {}): {verb} {} feature(s) ({}).",
            self.action_type, self.node_id, bucket.count, code
        );
        if !bucket.sample_feature_ids.is_empty() {
            let ids: Vec<String> = bucket
                .sample_feature_ids
                .iter()
                .map(Uuid::to_string)
                .collect();
            message.push_str(&format!(" Sample ids: {}", ids.join(", ")));
            let overflow = bucket
                .count
                .saturating_sub(bucket.sample_feature_ids.len() as u64);
            if overflow > 0 {
                message.push_str(&format!(" (+{overflow} more)"));
            }
            message.push('.');
        }
        let mut diagnostic = Diagnostic::from_draft(
            DiagnosticDraft::new(code)
                .with_message(message)
                .with_severity(Severity::Warn),
            Some(self.node_id.clone()),
            Some(self.action_type.clone()),
            None,
        );
        diagnostic.effective_disposition = match kind {
            // warn-and-continue skips resolve(); consumers must treat None as non-fatal
            DiagnosticKind::WarnContinue => None,
            DiagnosticKind::WarnDrop => Some(Disposition::WarnDrop),
            DiagnosticKind::Reject => Some(Disposition::Reject),
        };
        diagnostic.aggregated = Some(AggregateInfo {
            count: bucket.count,
            sample_feature_ids: bucket.sample_feature_ids,
        });
        diagnostic
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Disposition, ErrorCode};
    use pretty_assertions::assert_eq;
    use std::sync::Arc;

    fn make() -> NodeDiagnostics {
        NodeDiagnostics::new(
            "node-1".to_string(),
            "Cesium 3D Tiles Writer".to_string(),
            Arc::default(),
        )
    }

    #[test]
    fn n_drops_produce_one_summary_with_count_and_capped_samples() {
        let agg = make();
        let ids: Vec<uuid::Uuid> = (0..25).map(|_| uuid::Uuid::new_v4()).collect();
        for id in &ids {
            agg.record(
                DiagnosticKind::WarnDrop,
                ErrorCode::Cesium3dtilesEmptyGeometry,
                Some(*id),
            );
        }
        let summaries = agg.drain_summaries();
        assert_eq!(summaries.len(), 1);
        let s = &summaries[0];
        let info = s.aggregated.as_ref().unwrap();
        assert_eq!(info.count, 25);
        assert_eq!(info.sample_feature_ids.len(), SAMPLE_FEATURE_ID_CAP);
        assert_eq!(info.sample_feature_ids, ids[..SAMPLE_FEATURE_ID_CAP]);
        assert_eq!(s.effective_disposition, Some(Disposition::WarnDrop));
        assert_eq!(s.node_id.as_deref(), Some("node-1"));
        assert_eq!(s.action_type.as_deref(), Some("Cesium 3D Tiles Writer"));
        assert!(s.message.contains("dropped 25 feature(s)"));
        assert!(s.message.contains("cesium3dtiles.empty_geometry"));
        assert!(s.message.contains("(+15 more)"));
        // draining empties the buckets
        assert!(agg.drain_summaries().is_empty());
    }

    #[test]
    fn distinct_codes_and_kinds_get_distinct_summaries() {
        let agg = make();
        agg.record(
            DiagnosticKind::WarnDrop,
            ErrorCode::Cesium3dtilesEmptyGeometry,
            None,
        );
        agg.record(
            DiagnosticKind::WarnDrop,
            ErrorCode::Cesium3dtilesNonCitygmlGeometry,
            None,
        );
        agg.record(
            DiagnosticKind::WarnContinue,
            ErrorCode::Cesium3dtilesEmptyGeometry,
            None,
        );
        let summaries = agg.drain_summaries();
        assert_eq!(summaries.len(), 3);
        // warn-and-continue summaries never carry an effective disposition (they skip resolve())
        let warn_continue = summaries
            .iter()
            .find(|s| s.aggregated.is_some() && s.effective_disposition.is_none())
            .unwrap();
        assert!(warn_continue.message.contains("warned"));
    }

    #[test]
    fn fatal_slot_is_first_wins_and_take_clears_it() {
        let agg = make();
        let first = crate::Diagnostic::from_draft(
            crate::DiagnosticDraft::new(ErrorCode::InternalInvariantViolation)
                .with_message("first"),
            None,
            None,
            None,
        );
        let second = crate::Diagnostic::from_draft(
            crate::DiagnosticDraft::new(ErrorCode::InternalInvariantViolation)
                .with_message("second"),
            None,
            None,
            None,
        );
        agg.record_fatal(first);
        agg.record_fatal(second);
        assert_eq!(agg.take_fatal().unwrap().message, "first");
        assert!(agg.take_fatal().is_none());
    }

    #[test]
    fn warn_once_marks_exactly_once_per_run_across_nodes() {
        let shared: WarnOnceSet = Arc::default();
        let a = NodeDiagnostics::new("a".into(), "X".into(), shared.clone());
        let b = NodeDiagnostics::new("b".into(), "Y".into(), shared);
        assert!(a.try_mark_warn_once(ErrorCode::GltfZeroFaceSolid));
        assert!(!a.try_mark_warn_once(ErrorCode::GltfZeroFaceSolid));
        assert!(!b.try_mark_warn_once(ErrorCode::GltfZeroFaceSolid)); // run-scoped, not node-scoped
    }

    #[test]
    fn record_is_correct_under_concurrency() {
        let agg = Arc::new(make());
        std::thread::scope(|scope| {
            for _ in 0..16 {
                let agg = Arc::clone(&agg);
                scope.spawn(move || {
                    for _ in 0..1000 {
                        agg.record(
                            DiagnosticKind::WarnDrop,
                            ErrorCode::Cesium3dtilesEmptyGeometry,
                            Some(uuid::Uuid::new_v4()),
                        );
                    }
                });
            }
        });
        let summaries = agg.drain_summaries();
        assert_eq!(summaries[0].aggregated.as_ref().unwrap().count, 16_000);
        assert_eq!(
            summaries[0]
                .aggregated
                .as_ref()
                .unwrap()
                .sample_feature_ids
                .len(),
            SAMPLE_FEATURE_ID_CAP
        );
    }
}
