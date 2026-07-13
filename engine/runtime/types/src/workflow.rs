use std::{collections::HashMap, env};

use reearth_flow_common::serde::SerdeFormat;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

use reearth_flow_common::serde::determine_format;
use reearth_flow_common::serde::from_str;

pub type Id = Uuid;
pub type NodeProperty = Map<String, Value>;
pub type NodeAction = String;
pub type Parameter = Map<String, Value>;

static ENVIRONMENT_PREFIX: &str = "FLOW_VAR_";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowParameter {
    pub global: Option<Parameter>,
    pub node: Option<NodeProperty>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Workflow {
    pub id: Id,
    pub name: String,
    pub entry_graph_id: Id,
    pub with: Option<Parameter>,
    pub graphs: Vec<Graph>,
    pub error_policy: Option<ErrorPolicy>,
}

impl TryFrom<&str> for Workflow {
    type Error = crate::error::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut workflow: Self = from_str(value).map_err(crate::error::Error::input)?;
        workflow.load_variables_from_environment()?;
        Ok(workflow)
    }
}

impl Workflow {
    fn load_variables_from_environment(&mut self) -> Result<(), crate::error::Error> {
        let environment_vars: Vec<(String, String)> = env::vars()
            .filter(|(key, _)| key.starts_with(ENVIRONMENT_PREFIX))
            .map(|(key, value)| (key[ENVIRONMENT_PREFIX.len()..].to_string(), value))
            .filter(|(key, _)| {
                self.with
                    .as_ref()
                    .unwrap_or(&serde_json::Map::new())
                    .contains_key(key)
            })
            .collect();
        if environment_vars.is_empty() {
            return Ok(());
        }
        let mut with = if let Some(with) = self.with.clone() {
            with
        } else {
            serde_json::Map::<String, Value>::new()
        };
        with.extend(
            environment_vars
                .into_iter()
                .map(|(key, value)| {
                    tracing::info!("Loading environment variable: {}", key);
                    let value = match determine_format(value.as_str()) {
                        SerdeFormat::Json | SerdeFormat::Yaml => {
                            from_str(value.as_str()).map_err(crate::error::Error::input)?
                        }
                        SerdeFormat::Unknown => {
                            serde_json::to_value(value).map_err(crate::error::Error::input)?
                        }
                    };
                    Ok((key, value))
                })
                .collect::<Result<Vec<_>, crate::error::Error>>()?,
        );
        self.with = Some(with);
        Ok(())
    }

    fn process_params(
        &self,
        params: HashMap<String, String>,
    ) -> Result<HashMap<String, Value>, crate::error::Error> {
        params
            .into_iter()
            .map(|(key, value)| {
                let value = match determine_format(value.as_str()) {
                    SerdeFormat::Json | SerdeFormat::Yaml => {
                        from_str(value.as_str()).map_err(crate::error::Error::input)?
                    }
                    SerdeFormat::Unknown => {
                        serde_json::to_value(value).map_err(crate::error::Error::input)?
                    }
                };
                Ok((key, value))
            })
            .collect()
    }

    pub fn extend_with(
        &mut self,
        params: HashMap<String, String>,
    ) -> Result<(), crate::error::Error> {
        if params.is_empty() {
            return Ok(());
        }
        let processed_params = self.process_params(params)?;
        let with = self.with.get_or_insert_with(Map::new);
        with.extend(processed_params);
        Ok(())
    }

    pub fn merge_with(
        &mut self,
        params: HashMap<String, String>,
    ) -> Result<(), crate::error::Error> {
        if params.is_empty() {
            return Ok(());
        }
        let filtered_params: HashMap<_, _> = params
            .into_iter()
            .filter(|(key, _)| self.with.as_ref().unwrap_or(&Map::new()).contains_key(key))
            .collect();
        let processed_params = self.process_params(filtered_params)?;
        let with = self.with.get_or_insert_with(Map::new);
        with.extend(processed_params);
        Ok(())
    }
}

/// Workflow-level configuration for how diagnostics of severity `Fatal`
/// are handled at run time, and per-selector overrides of the default
/// disposition. Absent (`None`) behaves byte-identically to a workflow
/// with no error-handling configuration at all — this type is additive.
///
/// Registry-aware rules (unknown diagnostic codes, codeless demotion of an
/// authored-`Fatal` override, node-id existence in the graph) are enforced
/// later, by the resolver that consumes this configuration — `validate`
/// here only checks the structural rules that don't need the registry or
/// graph.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPolicy {
    #[serde(default)]
    pub on_fatal: OnFatal,
    #[serde(default)]
    pub treat_all_as_fatal: bool,
    #[serde(default)]
    pub allow_relax_internal: bool,
    /// D7: switch that enables writing rejected features to a side file
    /// instead of only counting/logging them. Consumed by a later task.
    #[serde(default)]
    pub side_file: bool,
    #[serde(default)]
    pub overrides: Vec<PolicyOverride>,
}

impl ErrorPolicy {
    /// Structural validation only: checks that don't require the
    /// diagnostic-code registry or the workflow graph. Each violation
    /// produces one message naming the offending override's index and
    /// selectors; all violations are collected rather than short-circuiting
    /// on the first one.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        for (index, override_) in self.overrides.iter().enumerate() {
            if override_.code.is_some() && override_.category.is_some() {
                errors.push(format!(
                    "overrides[{index}]: must not set both `code` and `category` \
                     (category derives from code; pairing them adds no selectivity)"
                ));
            }
            if override_.node.is_none() && override_.code.is_none() && override_.category.is_none()
            {
                errors.push(format!(
                    "overrides[{index}]: must set at least one selector \
                     (`node`, `code`, or `category`)"
                ));
            }
        }
        for i in 0..self.overrides.len() {
            for j in (i + 1)..self.overrides.len() {
                let a = &self.overrides[i];
                let b = &self.overrides[j];
                if a.node == b.node && a.code == b.code && a.category == b.category {
                    errors.push(format!(
                        "overrides[{i}] and overrides[{j}]: identical selectors \
                         (node={:?}, code={:?}, category={:?}); duplicate overrides \
                         are rejected",
                        a.node, a.code, a.category
                    ));
                }
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// What to do when a `Fatal`-severity diagnostic is raised (after override
/// resolution) and is not otherwise handled.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum OnFatal {
    /// Stop the run. This is the safe default.
    #[default]
    Terminate,
    /// Keep running, treating the diagnostic as non-terminating.
    Continue,
}

/// Overrides the disposition of diagnostics matching the given selectors.
/// At least one of `node`, `code`, `category` must be set, and `code` and
/// `category` are mutually exclusive (see `ErrorPolicy::validate`).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PolicyOverride {
    pub node: Option<String>,
    pub code: Option<String>,
    pub category: Option<String>,
    pub disposition: PolicyDisposition,
}

/// How a matched diagnostic should be handled.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDisposition {
    /// Warn and drop the affected feature; the run continues.
    WarnDrop,
    /// Reject the affected feature (optionally to the D7 side file); the run continues.
    Reject,
    /// Treat as fatal, subject to `on_fatal`.
    Fatal,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct NodeEntity {
    pub id: Id,
    pub name: String,
    pub with: Option<NodeProperty>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type")]
pub enum Node {
    #[serde(rename = "action")]
    Action {
        #[serde(flatten)]
        entity: NodeEntity,
        action: NodeAction,
    },
    #[serde(rename = "subGraph")]
    SubGraph {
        #[serde(flatten)]
        entity: NodeEntity,
        #[serde(rename = "subGraphId")]
        sub_graph_id: Id,
    },
}

impl Node {
    pub fn id(&self) -> Id {
        match self {
            Node::Action { entity, action: _ } => entity.id,
            Node::SubGraph {
                entity,
                sub_graph_id: _,
            } => entity.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Node::Action { entity, action: _ } => &entity.name,
            Node::SubGraph {
                entity,
                sub_graph_id: _,
            } => &entity.name,
        }
    }

    pub fn action(&self) -> &str {
        match self {
            Node::Action { entity: _, action } => action.as_str(),
            Node::SubGraph {
                entity: _,
                sub_graph_id: _,
            } => "subGraph",
        }
    }

    pub fn with(&self) -> &Option<NodeProperty> {
        match self {
            Node::Action { entity, action: _ } => &entity.with,
            Node::SubGraph {
                entity,
                sub_graph_id: _,
            } => &entity.with,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub id: Id,
    pub from: Id,
    pub to: Id,
    pub from_port: String,
    pub to_port: String,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Graph {
    pub id: Id,
    pub name: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_WORKFLOW_HEADER: &str = "\
id: \"11111111-1111-1111-1111-111111111111\"
name: \"test\"
entryGraphId: \"22222222-2222-2222-2222-222222222222\"
graphs: []
";

    #[test]
    fn deserializes_full_error_policy_block() {
        let yaml = format!(
            "{MINIMAL_WORKFLOW_HEADER}\
errorPolicy:
  onFatal: continue
  treatAllAsFatal: true
  allowRelaxInternal: true
  sideFile: true
  overrides:
    - node: \"node-a\"
      disposition: warn_drop
    - code: \"GEOM_INVALID\"
      disposition: fatal
"
        );
        let workflow = Workflow::try_from(yaml.as_str()).expect("workflow should parse");
        let policy = workflow
            .error_policy
            .expect("errorPolicy block should be present");
        assert_eq!(policy.on_fatal, OnFatal::Continue);
        assert!(policy.treat_all_as_fatal);
        assert!(policy.allow_relax_internal);
        assert!(policy.side_file);
        assert_eq!(
            policy.overrides,
            vec![
                PolicyOverride {
                    node: Some("node-a".to_string()),
                    code: None,
                    category: None,
                    disposition: PolicyDisposition::WarnDrop,
                },
                PolicyOverride {
                    node: None,
                    code: Some("GEOM_INVALID".to_string()),
                    category: None,
                    disposition: PolicyDisposition::Fatal,
                },
            ]
        );
    }

    #[test]
    fn error_policy_is_none_when_block_absent() {
        let workflow = Workflow::try_from(MINIMAL_WORKFLOW_HEADER).expect("workflow should parse");
        assert!(workflow.error_policy.is_none());
    }

    #[test]
    fn error_policy_defaults_when_block_empty() {
        let yaml = format!("{MINIMAL_WORKFLOW_HEADER}errorPolicy: {{}}\n");
        let workflow = Workflow::try_from(yaml.as_str()).expect("workflow should parse");
        let policy = workflow
            .error_policy
            .expect("errorPolicy block should be present");
        assert_eq!(policy, ErrorPolicy::default());
        assert_eq!(policy.on_fatal, OnFatal::Terminate);
        assert!(!policy.treat_all_as_fatal);
        assert!(!policy.allow_relax_internal);
        assert!(!policy.side_file);
        assert!(policy.overrides.is_empty());
    }

    fn override_with(
        node: Option<&str>,
        code: Option<&str>,
        category: Option<&str>,
    ) -> PolicyOverride {
        PolicyOverride {
            node: node.map(String::from),
            code: code.map(String::from),
            category: category.map(String::from),
            disposition: PolicyDisposition::WarnDrop,
        }
    }

    #[test]
    fn validate_rejects_override_with_both_code_and_category() {
        let policy = ErrorPolicy {
            overrides: vec![override_with(None, Some("GEOM_INVALID"), Some("geometry"))],
            ..Default::default()
        };
        let errors = policy.validate().expect_err("should reject");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("overrides[0]"));
        assert!(errors[0].contains("code") && errors[0].contains("category"));
    }

    #[test]
    fn validate_allows_override_with_only_code() {
        let policy = ErrorPolicy {
            overrides: vec![override_with(None, Some("GEOM_INVALID"), None)],
            ..Default::default()
        };
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn validate_rejects_override_with_no_selectors() {
        let policy = ErrorPolicy {
            overrides: vec![override_with(None, None, None)],
            ..Default::default()
        };
        let errors = policy.validate().expect_err("should reject");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("overrides[0]"));
        assert!(errors[0].contains("selector"));
    }

    #[test]
    fn validate_allows_override_with_one_selector() {
        let policy = ErrorPolicy {
            overrides: vec![override_with(Some("node-a"), None, None)],
            ..Default::default()
        };
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn validate_rejects_duplicate_selector_tuples() {
        let policy = ErrorPolicy {
            overrides: vec![
                override_with(Some("node-a"), None, None),
                override_with(Some("node-a"), None, None),
            ],
            ..Default::default()
        };
        let errors = policy.validate().expect_err("should reject");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("overrides[0]"));
        assert!(errors[0].contains("overrides[1]"));
        assert!(errors[0].contains("duplicate"));
    }

    #[test]
    fn validate_allows_distinct_selector_tuples() {
        let policy = ErrorPolicy {
            overrides: vec![
                override_with(Some("node-a"), None, None),
                override_with(Some("node-b"), None, None),
                override_with(None, Some("GEOM_INVALID"), None),
            ],
            ..Default::default()
        };
        assert!(policy.validate().is_ok());
    }

    #[test]
    fn error_policy_round_trips_through_serde_json() {
        let policy = ErrorPolicy {
            on_fatal: OnFatal::Continue,
            treat_all_as_fatal: true,
            allow_relax_internal: true,
            side_file: true,
            overrides: vec![
                override_with(Some("node-a"), None, None),
                PolicyOverride {
                    node: None,
                    code: None,
                    category: Some("geometry".to_string()),
                    disposition: PolicyDisposition::Reject,
                },
            ],
        };
        let json = serde_json::to_string(&policy).expect("serialize");
        let round_tripped: ErrorPolicy = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(policy, round_tripped);
    }
}
