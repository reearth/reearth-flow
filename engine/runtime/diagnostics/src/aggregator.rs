use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use uuid::Uuid;

use crate::{AggregateInfo, Diagnostic, DiagnosticDraft, Disposition, ErrorCode, Severity};

pub const SAMPLE_FEATURE_ID_CAP: usize = 10;

/// Fatal has no kind here — it goes to the per-node fatal slot, which counts its own occurrences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DiagnosticKind {
    WarnContinue,
    WarnDrop,
    Reject,
}

pub type WarnOnceSet = Arc<Mutex<HashSet<ErrorCode>>>;

#[derive(Debug, Default)]
struct Bucket {
    count: u64,
    sample_feature_ids: Vec<Uuid>,
}

impl Bucket {
    fn record(&mut self, feature_id: Option<Uuid>) {
        self.count += 1;
        if let Some(id) = feature_id {
            if self.sample_feature_ids.len() < SAMPLE_FEATURE_ID_CAP {
                self.sample_feature_ids.push(id);
            }
        }
    }

    fn into_aggregate(self) -> AggregateInfo {
        AggregateInfo {
            count: self.count,
            sample_feature_ids: self.sample_feature_ids,
        }
    }
}

/// The stored diagnostic is first-wins; the buckets keep counting, so the one that is
/// eventually taken can say how many times its code fired.
///
/// Counting is keyed by code rather than pooled per node because a single failing feature
/// can record twice: `report()` records the classified fatal, and the executor then records
/// an `internal.unclassified` one for the same `Err` on its way out. Those are distinct
/// codes, so they land in distinct buckets and neither inflates the other. Pooling them
/// would double every per-feature failure that came through `report()`.
#[derive(Debug, Default)]
struct FatalState {
    first: Option<Diagnostic>,
    buckets: HashMap<ErrorCode, Bucket>,
}

#[derive(Debug)]
pub struct NodeDiagnostics {
    node_id: String,
    action_type: String,
    buckets: Mutex<HashMap<(ErrorCode, DiagnosticKind), Bucket>>,
    fatal: Mutex<FatalState>,
    warn_once: WarnOnceSet,
}

impl NodeDiagnostics {
    pub fn new(node_id: String, action_type: String, warn_once: WarnOnceSet) -> Self {
        Self {
            node_id,
            action_type,
            buckets: Mutex::new(HashMap::new()),
            fatal: Mutex::new(FatalState::default()),
            warn_once,
        }
    }

    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    pub fn action_type(&self) -> &str {
        &self.action_type
    }

    pub fn record(&self, kind: DiagnosticKind, code: ErrorCode, feature_id: Option<Uuid>) {
        self.buckets
            .lock()
            .unwrap()
            .entry((code, kind))
            .or_default()
            .record(feature_id);
    }

    /// First-wins: the node fails with the first recorded fatal, even if the action swallowed `report()`'s Err.
    /// Every fatal is counted, though, so the stored one can carry the total for its code.
    pub fn record_fatal(&self, diagnostic: Diagnostic) {
        let mut state = self.fatal.lock().unwrap();
        state
            .buckets
            .entry(diagnostic.code)
            .or_default()
            .record(diagnostic.feature_id);
        if state.first.is_none() {
            state.first = Some(diagnostic);
        }
    }

    /// Draining. The returned diagnostic carries `aggregated` for **its own code only** — a
    /// fatal recorded under a different code is counted but never reported, exactly as its
    /// `Diagnostic` is dropped by first-wins.
    pub fn take_fatal(&self) -> Option<Diagnostic> {
        let mut state = std::mem::take(&mut *self.fatal.lock().unwrap());
        let mut diagnostic = state.first?;
        diagnostic.aggregated = state
            .buckets
            .remove(&diagnostic.code)
            .map(Bucket::into_aggregate);
        Some(diagnostic)
    }

    /// Returns true exactly once per run per code — the set is shared across all nodes in a run.
    pub fn try_mark_warn_once(&self, code: ErrorCode) -> bool {
        self.warn_once.lock().unwrap().insert(code)
    }

    /// Draining — buckets are emptied, so a second call returns nothing new.
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
        diagnostic.aggregated = Some(bucket.into_aggregate());
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
        let warn_continue = summaries
            .iter()
            .find(|s| s.aggregated.is_some() && s.effective_disposition.is_none())
            .unwrap();
        assert!(warn_continue.message.contains("warned"));
    }

    fn fatal(code: ErrorCode, message: &str, feature_id: Option<uuid::Uuid>) -> Diagnostic {
        Diagnostic::from_draft(
            crate::DiagnosticDraft::new(code).with_message(message),
            None,
            None,
            feature_id,
        )
    }

    #[test]
    fn fatal_slot_is_first_wins_and_take_clears_it() {
        let agg = make();
        agg.record_fatal(fatal(ErrorCode::InternalInvariantViolation, "first", None));
        agg.record_fatal(fatal(ErrorCode::InternalInvariantViolation, "second", None));
        assert_eq!(agg.take_fatal().unwrap().message, "first");
        assert!(agg.take_fatal().is_none());
    }

    #[test]
    fn repeat_fatals_of_one_code_report_a_count_and_capped_samples() {
        // The symptom this fixes: N features fail, the UI is told about 1.
        let agg = make();
        let ids: Vec<uuid::Uuid> = (0..25).map(|_| uuid::Uuid::new_v4()).collect();
        for id in &ids {
            agg.record_fatal(fatal(
                ErrorCode::InternalUnclassified,
                "expression failed",
                Some(*id),
            ));
        }
        let taken = agg.take_fatal().expect("fatal slot should be set");
        let info = taken
            .aggregated
            .as_ref()
            .expect("fatal should carry a count");
        assert_eq!(info.count, 25);
        assert_eq!(info.sample_feature_ids.len(), SAMPLE_FEATURE_ID_CAP);
        assert_eq!(info.sample_feature_ids, ids[..SAMPLE_FEATURE_ID_CAP]);
        // First-wins is unchanged — only the count is new.
        assert_eq!(taken.feature_id, Some(ids[0]));
    }

    #[test]
    fn a_single_fatal_reports_a_count_of_one() {
        // A build()-time fatal really did happen once; saying so is the point of the field.
        let agg = make();
        agg.record_fatal(fatal(ErrorCode::InternalUnclassified, "bad param", None));
        let info = agg.take_fatal().unwrap().aggregated.expect("count");
        assert_eq!(info.count, 1);
        assert!(info.sample_feature_ids.is_empty());
    }

    #[test]
    fn a_fatal_counts_only_its_own_code() {
        // One failing feature records twice in the real executor: report() records the
        // classified fatal, then the node records internal.unclassified for the same Err.
        // Pooling per node would report 2 features failed when 1 did.
        let agg = make();
        let feature = uuid::Uuid::new_v4();
        agg.record_fatal(fatal(
            ErrorCode::ExprAttributeOperationFailed,
            "classified",
            Some(feature),
        ));
        agg.record_fatal(fatal(
            ErrorCode::InternalUnclassified,
            "same Err, on the way out",
            Some(feature),
        ));
        let taken = agg.take_fatal().unwrap();
        assert_eq!(taken.code, ErrorCode::ExprAttributeOperationFailed);
        assert_eq!(taken.aggregated.unwrap().count, 1);
    }

    #[test]
    fn record_fatal_is_correct_under_concurrency() {
        let agg = Arc::new(make());
        std::thread::scope(|scope| {
            for _ in 0..16 {
                let agg = Arc::clone(&agg);
                scope.spawn(move || {
                    for _ in 0..1000 {
                        agg.record_fatal(fatal(
                            ErrorCode::InternalUnclassified,
                            "concurrent",
                            Some(uuid::Uuid::new_v4()),
                        ));
                    }
                });
            }
        });
        let info = agg.take_fatal().unwrap().aggregated.expect("count");
        assert_eq!(info.count, 16_000);
        assert_eq!(info.sample_feature_ids.len(), SAMPLE_FEATURE_ID_CAP);
    }

    #[test]
    fn warn_once_marks_exactly_once_per_run_across_nodes() {
        let shared: WarnOnceSet = Arc::default();
        let a = NodeDiagnostics::new("a".into(), "X".into(), shared.clone());
        let b = NodeDiagnostics::new("b".into(), "Y".into(), shared);
        assert!(a.try_mark_warn_once(ErrorCode::GltfZeroFaceSolid));
        assert!(!a.try_mark_warn_once(ErrorCode::GltfZeroFaceSolid));
        assert!(!b.try_mark_warn_once(ErrorCode::GltfZeroFaceSolid));
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
