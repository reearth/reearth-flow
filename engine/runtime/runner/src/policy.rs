//! Maps the workflow-level `errorPolicy` (parsed by `reearth_flow_types`)
//! into the diagnostics crate's compile-time input seam (`PolicyInput`),
//! and validates policy-override `node` selectors against the flattened,
//! composed-id DAG once it has been built (spec 4.2's load-time
//! node-matching rule, plus the multiply-instantiated-subgraph ambiguity
//! rule).
//!
//! This is the runner's half of the Task 3 policy-threading contract: the
//! diagnostics crate (`reearth_flow_diagnostics::policy`) deliberately has
//! no dependency on `reearth_flow_types`, so this module is where the two
//! seams meet. `Orchestrator::run_apps` calls `map_error_policy` +
//! `DispositionPolicy::compile` once at load, before DAG construction, and
//! `validate_node_selectors` once the DAG exists (`DagExecutor::
//! node_identities`).

use std::collections::{HashMap, HashSet};

use reearth_flow_diagnostics::{Disposition, OnFatalInput, OverrideInput, PolicyInput};
use reearth_flow_types::{ErrorPolicy, OnFatal, PolicyDisposition, PolicyOverride};

/// Pure field-by-field mapping from the workflow-level `ErrorPolicy` (types
/// crate) to the diagnostics crate's compile-time input seam. No behavior
/// of its own — `DispositionPolicy::compile` does all the registry-aware
/// validation once this seam is populated.
pub fn map_error_policy(policy: &ErrorPolicy) -> PolicyInput {
    PolicyInput {
        on_fatal: map_on_fatal(&policy.on_fatal),
        treat_all_as_fatal: policy.treat_all_as_fatal,
        allow_relax_internal: policy.allow_relax_internal,
        side_file: policy.side_file,
        overrides: policy.overrides.iter().map(map_override).collect(),
    }
}

fn map_on_fatal(on_fatal: &OnFatal) -> OnFatalInput {
    match on_fatal {
        OnFatal::Terminate => OnFatalInput::Terminate,
        OnFatal::Continue => OnFatalInput::Continue,
    }
}

fn map_disposition(disposition: &PolicyDisposition) -> Disposition {
    match disposition {
        PolicyDisposition::WarnDrop => Disposition::WarnDrop,
        PolicyDisposition::Reject => Disposition::Reject,
        PolicyDisposition::Fatal => Disposition::Fatal,
    }
}

fn map_override(o: &PolicyOverride) -> OverrideInput {
    OverrideInput {
        node: o.node.clone(),
        code: o.code.clone(),
        category: o.category.clone(),
        disposition: map_disposition(&o.disposition),
    }
}

/// Load-time node-matching check (spec 4.2): every override's `node` value
/// must equal a composed id in the flattened DAG built from this workflow.
/// Additionally, if an override's `node` value equals the raw (un-prefixed)
/// id of a node that was instantiated more than once (a subgraph used at
/// multiple call sites, each producing a distinct composed id), that's
/// flagged as ambiguous rather than "not found" — the author almost
/// certainly meant one specific instance and needs to supply its composed
/// id.
///
/// `node_identities` is `(composed_id, raw_id)` for every node in the built
/// DAG (`DagExecutor::node_identities`). Returns every problem found, not
/// just the first (mirrors `ErrorPolicy::validate`'s collect-everything
/// convention).
pub fn validate_node_selectors(
    policy: &ErrorPolicy,
    node_identities: &[(String, String)],
) -> Result<(), Vec<String>> {
    let composed_ids: HashSet<&str> = node_identities.iter().map(|(c, _)| c.as_str()).collect();
    let mut raw_id_counts: HashMap<&str, usize> = HashMap::new();
    for (_, raw) in node_identities {
        *raw_id_counts.entry(raw.as_str()).or_insert(0) += 1;
    }

    let mut errors = Vec::new();
    for (index, o) in policy.overrides.iter().enumerate() {
        let Some(node) = o.node.as_deref() else {
            continue;
        };
        if composed_ids.contains(node) {
            continue;
        }
        if raw_id_counts.get(node).copied().unwrap_or(0) > 1 {
            errors.push(format!(
                "overrides[{index}]: node id `{node}` is ambiguous — it names the \
                 un-prefixed id of a subgraph instantiated {} times; use the \
                 fully-qualified composed id instead",
                raw_id_counts[node]
            ));
            continue;
        }
        let nearest = nearest_composed_ids(node, &composed_ids, 3);
        errors.push(format!(
            "overrides[{index}]: node id `{node}` not found in the workflow graph; \
             nearest matches: {}",
            nearest.join(", ")
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Simple Levenshtein edit distance, used only to surface "nearest match"
/// suggestions in unmatched-node-id errors — mirrors
/// `reearth_flow_diagnostics::policy`'s private helper of the same shape
/// (duplicated rather than shared: that helper is private to the
/// diagnostics crate and this crate has no other reason to depend on it).
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, &ca) in a.iter().enumerate() {
        let mut prev_diag = row[0];
        row[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            let above_left = prev_diag;
            prev_diag = row[j + 1];
            row[j + 1] = (row[j + 1] + 1).min(row[j] + 1).min(above_left + cost);
        }
    }
    row[b.len()]
}

fn nearest_composed_ids(raw: &str, composed_ids: &HashSet<&str>, limit: usize) -> Vec<String> {
    let mut scored: Vec<(usize, &str)> = composed_ids
        .iter()
        .map(|c| (levenshtein_distance(raw, c), *c))
        .collect();
    scored.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(b.1)));
    scored
        .into_iter()
        .take(limit)
        .map(|(_, s)| s.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_policy() -> ErrorPolicy {
        ErrorPolicy {
            on_fatal: OnFatal::Continue,
            treat_all_as_fatal: true,
            allow_relax_internal: true,
            side_file: true,
            overrides: vec![
                PolicyOverride {
                    node: Some("node-a".to_string()),
                    code: None,
                    category: Some("geometry".to_string()),
                    disposition: PolicyDisposition::Reject,
                },
                PolicyOverride {
                    node: None,
                    code: Some("gltf.zero_face_solid".to_string()),
                    category: None,
                    disposition: PolicyDisposition::Fatal,
                },
            ],
        }
    }

    #[test]
    fn map_error_policy_maps_every_field() {
        let policy = sample_policy();
        let input = map_error_policy(&policy);

        assert_eq!(input.on_fatal, OnFatalInput::Continue);
        assert!(input.treat_all_as_fatal);
        assert!(input.allow_relax_internal);
        assert!(input.side_file);
        assert_eq!(input.overrides.len(), 2);

        assert_eq!(input.overrides[0].node.as_deref(), Some("node-a"));
        assert_eq!(input.overrides[0].code, None);
        assert_eq!(input.overrides[0].category.as_deref(), Some("geometry"));
        assert_eq!(input.overrides[0].disposition, Disposition::Reject);

        assert_eq!(input.overrides[1].node, None);
        assert_eq!(
            input.overrides[1].code.as_deref(),
            Some("gltf.zero_face_solid")
        );
        assert_eq!(input.overrides[1].category, None);
        assert_eq!(input.overrides[1].disposition, Disposition::Fatal);
    }

    #[test]
    fn map_error_policy_default_maps_to_default_input() {
        let input = map_error_policy(&ErrorPolicy::default());
        assert_eq!(input.on_fatal, OnFatalInput::Terminate);
        assert!(!input.treat_all_as_fatal);
        assert!(!input.allow_relax_internal);
        assert!(!input.side_file);
        assert!(input.overrides.is_empty());
    }

    #[test]
    fn map_on_fatal_maps_both_variants() {
        assert_eq!(map_on_fatal(&OnFatal::Terminate), OnFatalInput::Terminate);
        assert_eq!(map_on_fatal(&OnFatal::Continue), OnFatalInput::Continue);
    }

    #[test]
    fn map_disposition_maps_all_three_variants() {
        assert_eq!(
            map_disposition(&PolicyDisposition::WarnDrop),
            Disposition::WarnDrop
        );
        assert_eq!(
            map_disposition(&PolicyDisposition::Reject),
            Disposition::Reject
        );
        assert_eq!(
            map_disposition(&PolicyDisposition::Fatal),
            Disposition::Fatal
        );
    }

    fn identities(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
        pairs
            .iter()
            .map(|(c, r)| (c.to_string(), r.to_string()))
            .collect()
    }

    #[test]
    fn validate_node_selectors_passes_when_every_override_names_a_composed_id() {
        let policy = ErrorPolicy {
            overrides: vec![PolicyOverride {
                node: Some("prefix.node-a".to_string()),
                code: None,
                category: None,
                disposition: PolicyDisposition::Reject,
            }],
            ..Default::default()
        };
        let ids = identities(&[("prefix.node-a", "node-a")]);
        assert!(validate_node_selectors(&policy, &ids).is_ok());
    }

    #[test]
    fn validate_node_selectors_ignores_overrides_with_no_node_selector() {
        let policy = ErrorPolicy {
            overrides: vec![PolicyOverride {
                node: None,
                code: Some("gltf.zero_face_solid".to_string()),
                category: None,
                disposition: PolicyDisposition::Reject,
            }],
            ..Default::default()
        };
        assert!(validate_node_selectors(&policy, &[]).is_ok());
    }

    #[test]
    fn validate_node_selectors_reports_unmatched_node_with_near_misses() {
        let policy = ErrorPolicy {
            overrides: vec![PolicyOverride {
                node: Some("node-ax".to_string()),
                code: None,
                category: None,
                disposition: PolicyDisposition::Reject,
            }],
            ..Default::default()
        };
        let ids = identities(&[("node-a", "node-a"), ("node-b", "node-b")]);
        let errors = validate_node_selectors(&policy, &ids).expect_err("should reject");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("node-ax"));
        assert!(errors[0].contains("node-a"));
    }

    #[test]
    fn validate_node_selectors_flags_ambiguous_raw_id_across_subgraph_instances() {
        // The same subgraph instantiated twice: same raw id, two composed ids.
        let policy = ErrorPolicy {
            overrides: vec![PolicyOverride {
                node: Some("writer".to_string()),
                code: None,
                category: None,
                disposition: PolicyDisposition::Reject,
            }],
            ..Default::default()
        };
        let ids = identities(&[
            ("instance-1.writer", "writer"),
            ("instance-2.writer", "writer"),
        ]);
        let errors = validate_node_selectors(&policy, &ids).expect_err("should reject");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("ambiguous"));
        assert!(errors[0].contains("writer"));
    }

    #[test]
    fn validate_node_selectors_collects_every_problem_without_short_circuiting() {
        let policy = ErrorPolicy {
            overrides: vec![
                PolicyOverride {
                    node: Some("missing-a".to_string()),
                    code: None,
                    category: None,
                    disposition: PolicyDisposition::Reject,
                },
                PolicyOverride {
                    node: Some("missing-b".to_string()),
                    code: None,
                    category: None,
                    disposition: PolicyDisposition::Reject,
                },
            ],
            ..Default::default()
        };
        let ids = identities(&[("node-a", "node-a")]);
        let errors = validate_node_selectors(&policy, &ids).expect_err("should reject");
        assert_eq!(errors.len(), 2);
    }
}
