//! Maps the workflow-level `errorPolicy` (parsed by `reearth_flow_types`)
//! into the diagnostics crate's compile-time input seam (`PolicyInput`),
//! and validates policy-override `node` selectors against the flattened,
//! composed-id DAG once it has been built (spec 4.2's load-time
//! node-matching rule, plus the multiply-instantiated-subgraph ambiguity
//! rule), plus the load-time Reject-routing validation (spec 4.4, Task 5).
//!
//! This is the runner's half of the Task 3 policy-threading contract: the
//! diagnostics crate (`reearth_flow_diagnostics::policy`) deliberately has
//! no dependency on `reearth_flow_types`, so this module is where the two
//! seams meet. `Orchestrator::run_apps` calls `map_error_policy` +
//! `DispositionPolicy::compile` once at load, before DAG construction, and
//! `validate_node_selectors` + `validate_reject_routing` once the DAG
//! exists (`DagExecutor::node_identities` / `DagExecutor::
//! reject_routing_info`).

use std::collections::{HashMap, HashSet};

use reearth_flow_diagnostics::{
    Disposition, DispositionPolicy, ErrorCode, OnFatalInput, OverrideInput, PolicyInput,
};
use reearth_flow_runtime::executor::dag_executor::{NodeKindTag, RejectRoutingInfo};
use reearth_flow_runtime::node::REJECTED_PORT;
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

/// Load-time Reject-routing validation (spec 4.4, Task 5): for every
/// non-source node, if any error code could resolve to `Reject` for it
/// under the compiled policy, the workflow must declare where those
/// rejected features go, or fail to load:
///   - a processor must declare `REJECTED_PORT` among its output ports
///     AND have at least one edge wired from it;
///   - a sink must have `policy.side_file()` set.
///
/// Source nodes are skipped — sources don't report `Reject`. With zero
/// authored-`Reject` registry codes (checked as of Task 5), only an
/// override can trigger this, so a no-policy workflow never hits it —
/// zero behavior change, per this task's binding constraint.
///
/// `rejecting_codes` combines the node-agnostic `may_resolve_to_reject`
/// fast gate with the node-aware `resolve` ladder: `may_resolve_to_reject`
/// is node-agnostic *by design* (T2's accessor contract — it proves a
/// `Reject` is possible *somewhere* in the policy, not at which node), so
/// used alone per-node it would over-flag every node in the DAG the moment
/// any single override anywhere promotes to `Reject`. `resolve` already
/// implements the full node+code/category/default ladder (spec 4.2), so
/// layering it on top is what actually scopes the requirement to the node(s)
/// an override targets — this is the follow-up test the T2 review requested
/// (see `validate_reject_routing_only_flags_the_overridden_node` below),
/// applied at this validation layer rather than to the node-agnostic
/// accessor itself.
pub fn validate_reject_routing(
    policy: &DispositionPolicy,
    nodes: &[RejectRoutingInfo],
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    for node in nodes {
        if node.kind == NodeKindTag::Source {
            continue;
        }
        let codes = rejecting_codes(policy, &node.composed_id);
        if codes.is_empty() {
            continue;
        }
        let code_list = codes
            .iter()
            .map(ErrorCode::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        match node.kind {
            NodeKindTag::Processor => {
                let declares_port = node.output_ports.contains(&REJECTED_PORT);
                if !declares_port || !node.rejected_port_wired {
                    let problem = if !declares_port {
                        "does not declare a `rejected` output port"
                    } else {
                        "declares `rejected` but has no edge wired from it"
                    };
                    errors.push(format!(
                        "node `{}`: a policy override may resolve code(s) [{code_list}] to \
                         `reject` here, but this processor {problem} — wire an edge from \
                         `rejected` to route rejected features somewhere",
                        node.composed_id
                    ));
                }
            }
            NodeKindTag::Sink => {
                if !policy.side_file() {
                    errors.push(format!(
                        "node `{}`: a policy override may resolve code(s) [{code_list}] to \
                         `reject` here, but `errorPolicy.sideFile` is not enabled; set \
                         `errorPolicy.sideFile: true` to capture rejected features",
                        node.composed_id
                    ));
                }
            }
            NodeKindTag::Source => unreachable!("source nodes are skipped above"),
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Every registry `ErrorCode` that could resolve to `Reject` specifically
/// at `composed_id` under `policy`. See `validate_reject_routing`'s doc
/// comment for why this needs both `may_resolve_to_reject` (fast, but
/// node-agnostic) and `resolve` (node-aware, the actual decision).
fn rejecting_codes(policy: &DispositionPolicy, composed_id: &str) -> Vec<ErrorCode> {
    ErrorCode::ALL
        .iter()
        .copied()
        .filter(|&code| {
            policy.may_resolve_to_reject(code)
                && policy.resolve(composed_id, code) == Disposition::Reject
        })
        .collect()
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

    // -----------------------------------------------------------------
    // validate_reject_routing (spec 4.4, Task 5)
    // -----------------------------------------------------------------

    fn diag_override(
        node: Option<&str>,
        code: Option<&str>,
        disposition: Disposition,
    ) -> OverrideInput {
        OverrideInput {
            node: node.map(String::from),
            code: code.map(String::from),
            category: None,
            disposition,
        }
    }

    fn compile_policy(overrides: Vec<OverrideInput>, side_file: bool) -> DispositionPolicy {
        DispositionPolicy::compile(PolicyInput {
            side_file,
            overrides,
            ..Default::default()
        })
        .expect("policy should compile")
    }

    fn processor(
        composed_id: &str,
        output_ports: &[&str],
        rejected_port_wired: bool,
    ) -> RejectRoutingInfo {
        RejectRoutingInfo {
            composed_id: composed_id.to_string(),
            kind: NodeKindTag::Processor,
            output_ports: output_ports
                .iter()
                .map(|p| reearth_flow_runtime::node::Port::new(*p))
                .collect(),
            rejected_port_wired,
        }
    }

    fn sink(composed_id: &str) -> RejectRoutingInfo {
        RejectRoutingInfo {
            composed_id: composed_id.to_string(),
            kind: NodeKindTag::Sink,
            output_ports: vec![],
            rejected_port_wired: false,
        }
    }

    fn source(composed_id: &str) -> RejectRoutingInfo {
        RejectRoutingInfo {
            composed_id: composed_id.to_string(),
            kind: NodeKindTag::Source,
            output_ports: vec![],
            rejected_port_wired: false,
        }
    }

    #[test]
    fn validate_reject_routing_passes_under_the_default_policy() {
        let policy = compile_policy(vec![], false);
        let nodes = vec![
            source("src"),
            processor("proc", &["features"], false),
            sink("sink-a"),
        ];
        assert!(validate_reject_routing(&policy, &nodes).is_ok());
    }

    #[test]
    fn validate_reject_routing_flags_a_sink_without_side_file() {
        let policy = compile_policy(
            vec![diag_override(
                Some("sink-a"),
                Some("cesium3dtiles.empty_geometry"),
                Disposition::Reject,
            )],
            false,
        );
        let nodes = vec![sink("sink-a")];
        let errors = validate_reject_routing(&policy, &nodes).expect_err("should reject");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("sink-a"));
        assert!(errors[0].contains("errorPolicy.sideFile"));
        assert!(errors[0].contains("cesium3dtiles.empty_geometry"));
    }

    #[test]
    fn validate_reject_routing_passes_a_sink_with_side_file() {
        let policy = compile_policy(
            vec![diag_override(
                Some("sink-a"),
                Some("cesium3dtiles.empty_geometry"),
                Disposition::Reject,
            )],
            true,
        );
        let nodes = vec![sink("sink-a")];
        assert!(validate_reject_routing(&policy, &nodes).is_ok());
    }

    #[test]
    fn validate_reject_routing_flags_a_processor_missing_the_rejected_port_declaration() {
        let policy = compile_policy(
            vec![diag_override(
                Some("proc"),
                Some("gltf.zero_face_solid"),
                Disposition::Reject,
            )],
            false,
        );
        let nodes = vec![processor("proc", &["features"], false)];
        let errors = validate_reject_routing(&policy, &nodes).expect_err("should reject");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("proc"));
        assert!(errors[0].contains("does not declare"));
        assert!(errors[0].contains("gltf.zero_face_solid"));
    }

    #[test]
    fn validate_reject_routing_flags_a_processor_with_an_unwired_rejected_port() {
        let policy = compile_policy(
            vec![diag_override(
                Some("proc"),
                Some("gltf.zero_face_solid"),
                Disposition::Reject,
            )],
            false,
        );
        let nodes = vec![processor("proc", &["features", "rejected"], false)];
        let errors = validate_reject_routing(&policy, &nodes).expect_err("should reject");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("proc"));
        assert!(errors[0].contains("no edge wired"));
    }

    #[test]
    fn validate_reject_routing_passes_a_processor_with_a_wired_rejected_port() {
        let policy = compile_policy(
            vec![diag_override(
                Some("proc"),
                Some("gltf.zero_face_solid"),
                Disposition::Reject,
            )],
            false,
        );
        let nodes = vec![processor("proc", &["features", "rejected"], true)];
        assert!(validate_reject_routing(&policy, &nodes).is_ok());
    }

    #[test]
    fn validate_reject_routing_skips_source_nodes_even_under_a_matching_override() {
        let policy = compile_policy(
            vec![diag_override(
                Some("src"),
                Some("gltf.zero_face_solid"),
                Disposition::Reject,
            )],
            false,
        );
        let nodes = vec![source("src")];
        assert!(validate_reject_routing(&policy, &nodes).is_ok());
    }

    /// The T2-review follow-up (see `validate_reject_routing`'s doc
    /// comment): `may_resolve_to_reject` is node-agnostic by design, so this
    /// exercises the *validation layer's* node scoping instead — a
    /// Reject-promoting override that targets one sink must not require
    /// side-file routing on an unrelated sink in the same workflow.
    #[test]
    fn validate_reject_routing_only_flags_the_overridden_node() {
        let policy = compile_policy(
            vec![diag_override(
                Some("sink-a"),
                Some("cesium3dtiles.empty_geometry"),
                Disposition::Reject,
            )],
            false,
        );
        let nodes = vec![sink("sink-a"), sink("sink-b")];
        let errors = validate_reject_routing(&policy, &nodes).expect_err("should reject");
        assert_eq!(
            errors.len(),
            1,
            "only the overridden node should be flagged: {errors:?}"
        );
        assert!(errors[0].contains("sink-a"));
        assert!(!errors[0].contains("sink-b"));
    }

    #[test]
    fn validate_reject_routing_collects_every_problem_without_short_circuiting() {
        let policy = compile_policy(
            vec![diag_override(
                None,
                Some("gltf.zero_face_solid"),
                Disposition::Reject,
            )],
            false,
        );
        let nodes = vec![
            sink("sink-a"),
            processor("proc", &["features", "rejected"], false),
        ];
        let errors = validate_reject_routing(&policy, &nodes).expect_err("should reject");
        assert_eq!(errors.len(), 2);
    }
}
