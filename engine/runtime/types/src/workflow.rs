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

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Workflow {
    pub id: Id,
    pub name: String,
    pub entry_graph_id: Id,
    pub with: Option<Parameter>,
    pub graphs: Vec<Graph>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowParameter {
    pub global: Option<Parameter>,
    pub node: Option<NodeProperty>,
}

impl Workflow {
    pub fn try_from_str(s: &str) -> Result<Self, reearth_flow_common::Error> {
        let mut workflow: Self = match from_str(s) {
            Ok(x) => x,
            Err(e) => return Err(e),
        };
        workflow.load_variables_from_environment();
        Ok(workflow)
    }

    fn load_variables_from_environment(&mut self) {
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
            return;
        }
        let mut with = if let Some(with) = self.with.clone() {
            with
        } else {
            serde_json::Map::<String, Value>::new()
        };
        with.extend(environment_vars.into_iter().map(|(key, value)| {
            let value = match determine_format(value.as_str()) {
                SerdeFormat::Json | SerdeFormat::Yaml => from_str(value.as_str()).unwrap(),
                SerdeFormat::Unknown => serde_json::to_value(value).unwrap(),
            };
            (key, value)
        }));
        self.with = Some(with);
    }

    pub fn merge_with(&mut self, params: HashMap<String, String>) {
        if params.is_empty() {
            return;
        }
        let mut with = if let Some(with) = self.with.clone() {
            with
        } else {
            serde_json::Map::<String, Value>::new()
        };
        let params = params
            .into_iter()
            .filter(|(key, _)| {
                self.with
                    .as_ref()
                    .unwrap_or(&serde_json::Map::new())
                    .contains_key(key)
            })
            .map(|(key, value)| {
                let value = match determine_format(value.as_str()) {
                    SerdeFormat::Json | SerdeFormat::Yaml => from_str(value.as_str()).unwrap(),
                    SerdeFormat::Unknown => serde_json::to_value(value).unwrap(),
                };
                (key, value)
            })
            .collect::<serde_json::Map<String, Value>>();
        with.extend(params);
        self.with = Some(with);
    }
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
