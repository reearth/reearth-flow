//! The disposition-policy resolver (spec 4.2): compiles a workflow's `ErrorPolicy`
//! into a [`DispositionPolicy`] and resolves [`Disposition`] per `(node, code)`. No dependency on `types` by design; not yet wired into production.

use crate::{Disposition, ErrorCategory, ErrorCode};

/// What to do when a resolved `Fatal` diagnostic is unhandled. Mirrors
/// `reearth_flow_types::workflow::OnFatal` one-for-one (no dependency on `types`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OnFatalInput {
    /// Stop the run. This is the safe default.
    #[default]
    Terminate,
    /// Keep running; applied at the join level (spec 4.2), never as a disposition demotion.
    Continue,
}

/// One raw override selector as parsed from workflow YAML. `code`/`category`
/// are raw strings until [`DispositionPolicy::compile`] resolves them against the registry; structural validation happens upstream in `ErrorPolicy::validate`.
#[derive(Debug, Clone)]
pub struct OverrideInput {
    pub node: Option<String>,
    pub code: Option<String>,
    pub category: Option<String>,
    pub disposition: Disposition,
}

/// Crate-local mirror of `reearth_flow_types::workflow::ErrorPolicy`. See
/// module docs for why this seam exists instead of depending on `types`.
#[derive(Debug, Clone, Default)]
pub struct PolicyInput {
    pub on_fatal: OnFatalInput,
    pub treat_all_as_fatal: bool,
    pub allow_relax_internal: bool,
    pub side_file: bool,
    pub overrides: Vec<OverrideInput>,
}

/// `code`/`category` already resolved against the registry. Private —
/// only observable from outside via `resolve` and the accessors below.
#[derive(Debug, Clone)]
struct CompiledOverride {
    node: Option<String>,
    code: Option<ErrorCode>,
    category: Option<ErrorCategory>,
    disposition: Disposition,
}

impl CompiledOverride {
    fn matches_node(&self, composed_node_id: &str) -> bool {
        self.node.as_deref() == Some(composed_node_id)
    }

    fn matches_code(&self, code: ErrorCode) -> bool {
        self.code == Some(code)
    }

    fn matches_category(&self, category: ErrorCategory) -> bool {
        self.category == Some(category)
    }

    fn is_codeless(&self) -> bool {
        self.code.is_none()
    }
}

/// A compiled, validated disposition policy (spec 4.2). Construct via
/// [`DispositionPolicy::compile`] or [`Default::default`] (empty policy; `resolve` always returns the registry default).
#[derive(Debug, Clone, Default)]
pub struct DispositionPolicy {
    on_fatal: OnFatalInput,
    treat_all_as_fatal: bool,
    allow_relax_internal: bool,
    side_file: bool,
    overrides: Vec<CompiledOverride>,
}

const CATEGORY_NAMES: &[(&str, ErrorCategory)] = &[
    ("io", ErrorCategory::Io),
    ("parse", ErrorCategory::Parse),
    ("validation", ErrorCategory::Validation),
    ("geometry", ErrorCategory::Geometry),
    ("schema", ErrorCategory::Schema),
    ("expression", ErrorCategory::Expression),
    ("config", ErrorCategory::Config),
    ("network", ErrorCategory::Network),
    ("resource", ErrorCategory::Resource),
    ("internal", ErrorCategory::Internal),
];

fn parse_category(raw: &str) -> Option<ErrorCategory> {
    CATEGORY_NAMES
        .iter()
        .find(|(name, _)| *name == raw)
        .map(|(_, category)| *category)
}

fn parse_code(raw: &str) -> Option<ErrorCode> {
    ErrorCode::ALL.iter().copied().find(|c| c.as_str() == raw)
}

/// Simple Levenshtein edit distance, used only to surface "nearest match"
/// suggestions in unknown-error-code compile errors (spec 4.2 rule 1).
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

fn nearest_codes(raw: &str, limit: usize) -> Vec<&'static str> {
    let mut scored: Vec<(usize, &'static str)> = ErrorCode::ALL
        .iter()
        .map(|c| (levenshtein_distance(raw, c.as_str()), c.as_str()))
        .collect();
    scored.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(b.1)));
    scored.into_iter().take(limit).map(|(_, s)| s).collect()
}

/// Total order used for promote/demote comparisons: `Fatal > Reject >
/// WarnDrop` (spec 4.2). Higher rank = more severe / more terminal.
fn disposition_rank(disposition: Disposition) -> u8 {
    match disposition {
        Disposition::WarnDrop => 0,
        Disposition::Reject => 1,
        Disposition::Fatal => 2,
    }
}

/// The more severe of the two dispositions per [`disposition_rank`].
fn max_disposition(a: Disposition, b: Disposition) -> Disposition {
    if disposition_rank(a) >= disposition_rank(b) {
        a
    } else {
        b
    }
}

impl DispositionPolicy {
    /// Compiles a [`PolicyInput`] against the registry (spec 4.2): resolves
    /// `code`/`category`, rejecting unknown values (with nearest-match suggestions) and per-code overrides that demote an `Internal` code below its registry default unless `allow_relax_internal`. Violations are collected, not short-circuited.
    pub fn compile(input: PolicyInput) -> Result<DispositionPolicy, Vec<String>> {
        let mut errors = Vec::new();
        let mut overrides = Vec::with_capacity(input.overrides.len());

        for (index, raw) in input.overrides.iter().enumerate() {
            debug_assert!(
                !(raw.code.is_some() && raw.category.is_some()),
                "overrides[{index}]: code+category exclusivity should have been \
                 validated upstream by ErrorPolicy::validate"
            );
            debug_assert!(
                raw.node.is_some() || raw.code.is_some() || raw.category.is_some(),
                "overrides[{index}]: at-least-one-selector should have been \
                 validated upstream by ErrorPolicy::validate"
            );

            let code = match raw.code.as_deref() {
                Some(raw_code) => match parse_code(raw_code) {
                    Some(code) => Some(code),
                    None => {
                        let nearest = nearest_codes(raw_code, 3).join(", ");
                        errors.push(format!(
                            "overrides[{index}]: unknown error code `{raw_code}`; \
                             nearest matches: {nearest}"
                        ));
                        None
                    }
                },
                None => None,
            };

            let category = match raw.category.as_deref() {
                Some(raw_category) => match parse_category(raw_category) {
                    Some(category) => Some(category),
                    None => {
                        errors.push(format!(
                            "overrides[{index}]: unknown error category `{raw_category}`; \
                             valid categories: io, parse, validation, geometry, schema, \
                             expression, config, network, resource, internal"
                        ));
                        None
                    }
                },
                None => None,
            };

            // Floor: a per-code Internal override may never demote below that code's
            // own registry default unless allow_relax_internal — relative to the code's own default, not an unconditional "force Fatal" (e.g. diagnostics_overflow defaults to WarnDrop).
            if let Some(code) = code {
                if code.category() == ErrorCategory::Internal {
                    let default = code.default_disposition();
                    let demotes_below_default =
                        disposition_rank(raw.disposition) < disposition_rank(default);
                    if demotes_below_default && !input.allow_relax_internal {
                        errors.push(format!(
                            "overrides[{index}]: code `{}` is category `internal` and \
                             cannot resolve below its registry default `{default:?}` \
                             (attempted `{:?}`); set `allow_relax_internal: true` to permit this",
                            code.as_str(),
                            raw.disposition
                        ));
                    }
                }
            }

            overrides.push(CompiledOverride {
                node: raw.node.clone(),
                code,
                category,
                disposition: raw.disposition,
            });
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(DispositionPolicy {
            on_fatal: input.on_fatal,
            treat_all_as_fatal: input.treat_all_as_fatal,
            allow_relax_internal: input.allow_relax_internal,
            side_file: input.side_file,
            overrides,
        })
    }

    /// Resolves the effective disposition per spec 4.2's ladder: `node+code >
    /// node+category > node > code > category > default`, then the Internal floor clamp, then `treat_all_as_fatal`. A codeless rung only wins if it doesn't demote the code below its own registry default; otherwise resolution falls through to the next rung.
    pub fn resolve(&self, composed_node_id: &str, code: ErrorCode) -> Disposition {
        let category = code.category();
        let default = code.default_disposition();

        let codeless_wins =
            |o: &CompiledOverride| disposition_rank(o.disposition) >= disposition_rank(default);

        let rung_node_code = self
            .overrides
            .iter()
            .find(|o| o.matches_node(composed_node_id) && o.matches_code(code))
            .map(|o| o.disposition);

        let rung_node_category = self
            .overrides
            .iter()
            .find(|o| {
                o.matches_node(composed_node_id)
                    && o.is_codeless()
                    && o.matches_category(category)
                    && codeless_wins(o)
            })
            .map(|o| o.disposition);

        let rung_node_only = self
            .overrides
            .iter()
            .find(|o| {
                o.matches_node(composed_node_id)
                    && o.is_codeless()
                    && o.category.is_none()
                    && codeless_wins(o)
            })
            .map(|o| o.disposition);

        let rung_code = self
            .overrides
            .iter()
            .find(|o| o.node.is_none() && o.matches_code(code))
            .map(|o| o.disposition);

        let rung_category = self
            .overrides
            .iter()
            .find(|o| {
                o.node.is_none()
                    && o.is_codeless()
                    && o.matches_category(category)
                    && codeless_wins(o)
            })
            .map(|o| o.disposition);

        let mut result = rung_node_code
            .or(rung_node_category)
            .or(rung_node_only)
            .or(rung_code)
            .or(rung_category)
            .unwrap_or(default);

        if category == ErrorCategory::Internal && !self.allow_relax_internal {
            result = max_disposition(result, default);
        }

        if self.treat_all_as_fatal {
            result = Disposition::Fatal;
        }

        result
    }

    /// What to do at the join level when a resolved `Fatal` diagnostic
    /// occurs (spec 4.2). Never touches disposition resolution itself.
    pub fn on_fatal(&self) -> OnFatalInput {
        self.on_fatal
    }

    /// D7: whether rejected features are additionally written to a side file.
    pub fn side_file(&self) -> bool {
        self.side_file
    }

    /// Whether any override in this policy names `composed_node_id` specifically —
    /// used for load-time checks that a referenced node id actually exists in the composed graph.
    pub fn overrides_touching_node(&self, composed_node_id: &str) -> bool {
        self.overrides
            .iter()
            .any(|o| o.matches_node(composed_node_id))
    }

    /// Conservative (over-approximating) check: could `resolve` ever return
    /// `Disposition::Reject` for `code`, under some node? Node-agnostic, but honors `treat_all_as_fatal`, the Internal floor, and the codeless-demotion guard.
    pub fn may_resolve_to_reject(&self, code: ErrorCode) -> bool {
        if self.treat_all_as_fatal {
            return false;
        }
        let category = code.category();
        let default = code.default_disposition();
        let floor_active = category == ErrorCategory::Internal && !self.allow_relax_internal;
        if floor_active && disposition_rank(Disposition::Reject) < disposition_rank(default) {
            return false;
        }
        if default == Disposition::Reject {
            return true;
        }
        self.overrides.iter().any(|o| {
            if o.disposition != Disposition::Reject {
                return false;
            }
            let selector_matches = if let Some(selector_code) = o.code {
                selector_code == code
            } else if let Some(selector_category) = o.category {
                selector_category == category
            } else {
                o.node.is_some()
            };
            if !selector_matches {
                return false;
            }
            if o.is_codeless() && disposition_rank(Disposition::Reject) < disposition_rank(default)
            {
                return false;
            }
            true
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn compile(input: PolicyInput) -> DispositionPolicy {
        DispositionPolicy::compile(input).expect("policy should compile")
    }

    fn override_all(
        node: Option<&str>,
        code: Option<&str>,
        category: Option<&str>,
        disposition: Disposition,
    ) -> OverrideInput {
        OverrideInput {
            node: node.map(String::from),
            code: code.map(String::from),
            category: category.map(String::from),
            disposition,
        }
    }

    // Ladder rungs: each must beat every less-specific rung (fixture: GltfZeroFaceSolid, geometry, default WarnDrop).

    const NODE: &str = "node-a";

    #[test]
    fn rung_node_code_beats_all_less_specific_rungs() {
        let policy = compile(PolicyInput {
            overrides: vec![
                override_all(
                    Some(NODE),
                    Some("gltf.zero_face_solid"),
                    None,
                    Disposition::Fatal,
                ),
                override_all(Some(NODE), None, Some("geometry"), Disposition::Reject),
                override_all(Some(NODE), None, None, Disposition::Reject),
                override_all(
                    None,
                    Some("gltf.zero_face_solid"),
                    None,
                    Disposition::Reject,
                ),
                override_all(None, None, Some("geometry"), Disposition::Reject),
            ],
            ..Default::default()
        });
        assert_eq!(
            policy.resolve(NODE, ErrorCode::GltfZeroFaceSolid),
            Disposition::Fatal
        );
    }

    #[test]
    fn rung_node_category_beats_less_specific_rungs() {
        let policy = compile(PolicyInput {
            overrides: vec![
                override_all(Some(NODE), None, Some("geometry"), Disposition::Fatal),
                override_all(Some(NODE), None, None, Disposition::Reject),
                override_all(
                    None,
                    Some("gltf.zero_face_solid"),
                    None,
                    Disposition::Reject,
                ),
                override_all(None, None, Some("geometry"), Disposition::Reject),
            ],
            ..Default::default()
        });
        assert_eq!(
            policy.resolve(NODE, ErrorCode::GltfZeroFaceSolid),
            Disposition::Fatal
        );
    }

    #[test]
    fn rung_node_only_beats_less_specific_rungs() {
        let policy = compile(PolicyInput {
            overrides: vec![
                override_all(Some(NODE), None, None, Disposition::Fatal),
                override_all(
                    None,
                    Some("gltf.zero_face_solid"),
                    None,
                    Disposition::Reject,
                ),
                override_all(None, None, Some("geometry"), Disposition::Reject),
            ],
            ..Default::default()
        });
        assert_eq!(
            policy.resolve(NODE, ErrorCode::GltfZeroFaceSolid),
            Disposition::Fatal
        );
    }

    #[test]
    fn rung_code_beats_category_rung() {
        let policy = compile(PolicyInput {
            overrides: vec![
                override_all(None, Some("gltf.zero_face_solid"), None, Disposition::Fatal),
                override_all(None, None, Some("geometry"), Disposition::Reject),
            ],
            ..Default::default()
        });
        // NODE is irrelevant here: neither override names a node.
        assert_eq!(
            policy.resolve(NODE, ErrorCode::GltfZeroFaceSolid),
            Disposition::Fatal
        );
    }

    #[test]
    fn rung_category_beats_registry_default() {
        let policy = compile(PolicyInput {
            overrides: vec![override_all(
                None,
                None,
                Some("geometry"),
                Disposition::Fatal,
            )],
            ..Default::default()
        });
        assert_eq!(
            policy.resolve(NODE, ErrorCode::GltfZeroFaceSolid),
            Disposition::Fatal
        );
    }

    #[test]
    fn no_matching_override_falls_back_to_registry_default() {
        let policy = compile(PolicyInput {
            overrides: vec![override_all(
                Some("some-other-node"),
                None,
                None,
                Disposition::Fatal,
            )],
            ..Default::default()
        });
        assert_eq!(
            policy.resolve(NODE, ErrorCode::GltfZeroFaceSolid),
            Disposition::WarnDrop
        );
    }

    // Codeless-demotion guard, exercised via the floor-exempt path (allow_relax_internal: true) to prove it's independent of the Internal floor.

    #[test]
    fn codeless_category_override_never_demotes_authored_fatal_code() {
        let policy = compile(PolicyInput {
            allow_relax_internal: true,
            overrides: vec![override_all(
                None,
                None,
                Some("internal"),
                Disposition::WarnDrop,
            )],
            ..Default::default()
        });
        assert_eq!(
            policy.resolve(NODE, ErrorCode::InternalUnclassified),
            Disposition::Fatal,
            "codeless category override must never demote an authored-Fatal code"
        );
    }

    #[test]
    fn code_bearing_override_demotes_authored_fatal_code_when_codeless_could_not() {
        let policy = compile(PolicyInput {
            allow_relax_internal: true,
            overrides: vec![override_all(
                None,
                Some("internal.unclassified"),
                None,
                Disposition::WarnDrop,
            )],
            ..Default::default()
        });
        assert_eq!(
            policy.resolve(NODE, ErrorCode::InternalUnclassified),
            Disposition::WarnDrop,
            "a per-code override may demote, unlike the codeless rungs"
        );
    }

    // Internal floor: resolve-time clamp, independent of compile-time rejection.
    // Uses a direct struct literal (private-field access) to prove the clamp holds even when compile() would have rejected this policy.

    #[test]
    fn floor_forces_fatal_at_resolve_time_even_if_a_demoting_override_is_present() {
        let policy = DispositionPolicy {
            allow_relax_internal: false,
            overrides: vec![CompiledOverride {
                node: None,
                code: Some(ErrorCode::InternalInvariantViolation),
                category: None,
                disposition: Disposition::WarnDrop,
            }],
            ..Default::default()
        };
        assert_eq!(
            policy.resolve(NODE, ErrorCode::InternalInvariantViolation),
            Disposition::Fatal
        );
    }

    #[test]
    fn floor_is_a_no_op_for_an_internal_code_whose_default_is_not_fatal() {
        // diagnostics_overflow is Internal but defaults to WarnDrop, so the floor (which raises to at least the code's own default) is a no-op here.
        let policy = compile(PolicyInput {
            overrides: vec![override_all(
                None,
                None,
                Some("internal"),
                Disposition::Reject,
            )],
            ..Default::default()
        });
        assert_eq!(
            policy.resolve(NODE, ErrorCode::InternalDiagnosticsOverflow),
            Disposition::Reject,
            "a promotion above the WarnDrop default is honored normally, floor or not"
        );
    }

    #[test]
    fn per_code_demotion_of_authored_fatal_code_allowed_when_floor_permits() {
        let policy = compile(PolicyInput {
            allow_relax_internal: true,
            overrides: vec![override_all(
                None,
                Some("internal.invariant_violation"),
                None,
                Disposition::Reject,
            )],
            ..Default::default()
        });
        assert_eq!(
            policy.resolve(NODE, ErrorCode::InternalInvariantViolation),
            Disposition::Reject
        );
    }

    #[test]
    fn compile_rejects_per_code_demotion_of_authored_fatal_internal_code_without_flag() {
        let errors = DispositionPolicy::compile(PolicyInput {
            allow_relax_internal: false,
            overrides: vec![override_all(
                None,
                Some("internal.invariant_violation"),
                None,
                Disposition::WarnDrop,
            )],
            ..Default::default()
        })
        .expect_err("should reject");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("internal.invariant_violation"));
        assert!(errors[0].contains("allow_relax_internal"));
    }

    #[test]
    fn compile_allows_codeless_internal_override_with_non_fatal_disposition_without_flag() {
        // Rule 4's rejection is scoped to code-bearing overrides; a codeless
        // override can never demote an authored-Fatal code anyway (the codeless-demotion guard handles that at resolve time).
        let policy = DispositionPolicy::compile(PolicyInput {
            allow_relax_internal: false,
            overrides: vec![override_all(
                None,
                None,
                Some("internal"),
                Disposition::WarnDrop,
            )],
            ..Default::default()
        });
        assert!(policy.is_ok());
    }

    // treat_all_as_fatal: post-clamp, promotes an effective WarnDrop or Reject to Fatal.

    #[test]
    fn treat_all_as_fatal_promotes_an_effective_warn_drop() {
        let policy = compile(PolicyInput {
            treat_all_as_fatal: true,
            ..Default::default()
        });
        assert_eq!(
            policy.resolve(NODE, ErrorCode::GltfZeroFaceSolid),
            Disposition::Fatal
        );
    }

    #[test]
    fn treat_all_as_fatal_promotes_an_effective_reject() {
        let policy = compile(PolicyInput {
            treat_all_as_fatal: true,
            overrides: vec![override_all(
                None,
                Some("gltf.zero_face_solid"),
                None,
                Disposition::Reject,
            )],
            ..Default::default()
        });
        assert_eq!(
            policy.resolve(NODE, ErrorCode::GltfZeroFaceSolid),
            Disposition::Fatal
        );
    }

    #[test]
    fn compile_rejects_unknown_code_and_lists_nearest_matches() {
        let errors = DispositionPolicy::compile(PolicyInput {
            overrides: vec![override_all(
                None,
                Some("gltf.zero_face_solidx"),
                None,
                Disposition::Fatal,
            )],
            ..Default::default()
        })
        .expect_err("should reject");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("gltf.zero_face_solidx"));
        assert!(errors[0].contains("gltf.zero_face_solid"));
    }

    #[test]
    fn compile_rejects_unknown_category() {
        let errors = DispositionPolicy::compile(PolicyInput {
            overrides: vec![override_all(
                None,
                None,
                Some("not_a_real_category"),
                Disposition::Fatal,
            )],
            ..Default::default()
        })
        .expect_err("should reject");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("not_a_real_category"));
    }

    #[test]
    fn compile_collects_multiple_errors_without_short_circuiting() {
        let errors = DispositionPolicy::compile(PolicyInput {
            overrides: vec![
                override_all(None, Some("bogus.code"), None, Disposition::Fatal),
                override_all(None, None, Some("bogus_category"), Disposition::Fatal),
            ],
            ..Default::default()
        })
        .expect_err("should reject");
        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn default_policy_resolves_every_code_to_its_registry_default() {
        let policy = DispositionPolicy::default();
        for code in ErrorCode::ALL {
            assert_eq!(
                policy.resolve(NODE, *code),
                code.default_disposition(),
                "code {} should resolve to its registry default under the empty policy",
                code.as_str()
            );
        }
    }

    #[test]
    fn default_policy_has_terminate_and_all_flags_false() {
        let policy = DispositionPolicy::default();
        assert_eq!(policy.on_fatal(), OnFatalInput::Terminate);
        assert!(!policy.side_file());
        assert!(!policy.overrides_touching_node(NODE));
    }

    #[test]
    fn on_fatal_and_side_file_reflect_compiled_input() {
        let policy = compile(PolicyInput {
            on_fatal: OnFatalInput::Continue,
            side_file: true,
            ..Default::default()
        });
        assert_eq!(policy.on_fatal(), OnFatalInput::Continue);
        assert!(policy.side_file());
    }

    #[test]
    fn overrides_touching_node_is_true_only_for_referenced_nodes() {
        let policy = compile(PolicyInput {
            overrides: vec![override_all(Some(NODE), None, None, Disposition::Fatal)],
            ..Default::default()
        });
        assert!(policy.overrides_touching_node(NODE));
        assert!(!policy.overrides_touching_node("some-other-node"));
    }

    #[test]
    fn may_resolve_to_reject_true_when_a_code_override_rejects() {
        let policy = compile(PolicyInput {
            overrides: vec![override_all(
                None,
                Some("gltf.zero_face_solid"),
                None,
                Disposition::Reject,
            )],
            ..Default::default()
        });
        assert!(policy.may_resolve_to_reject(ErrorCode::GltfZeroFaceSolid));
        assert!(!policy.may_resolve_to_reject(ErrorCode::Cesium3dtilesEmptyGeometry));
    }

    #[test]
    fn may_resolve_to_reject_false_when_treat_all_as_fatal() {
        let policy = compile(PolicyInput {
            treat_all_as_fatal: true,
            overrides: vec![override_all(
                None,
                Some("gltf.zero_face_solid"),
                None,
                Disposition::Reject,
            )],
            ..Default::default()
        });
        assert!(!policy.may_resolve_to_reject(ErrorCode::GltfZeroFaceSolid));
    }

    #[test]
    fn may_resolve_to_reject_false_for_floor_protected_internal_code() {
        let policy = compile(PolicyInput {
            allow_relax_internal: true,
            overrides: vec![override_all(
                None,
                Some("internal.invariant_violation"),
                None,
                Disposition::Reject,
            )],
            ..Default::default()
        });
        assert!(policy.may_resolve_to_reject(ErrorCode::InternalInvariantViolation));

        let floored = compile(PolicyInput {
            allow_relax_internal: false,
            ..Default::default()
        });
        assert!(!floored.may_resolve_to_reject(ErrorCode::InternalInvariantViolation));
    }
}
