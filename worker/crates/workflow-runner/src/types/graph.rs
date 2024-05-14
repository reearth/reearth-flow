use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use uuid::Uuid;

use reearth_flow_common::serde::from_str;

pub type Id = Uuid;
pub type NodeProperty = Map<String, Value>;
pub type NodeAction = String;
pub type Parameter = Map<String, Value>;

#[derive(Serialize, Deserialize, Debug)]
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
    pub fn try_from_str(s: &str) -> crate::Result<Self> {
        from_str(s).map_err(|e| crate::Error::Init(format!("Failed to parse workflow: {}", e)))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeEntity {
    pub id: Id,
    pub name: String,
    pub with: Option<NodeProperty>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub id: Id,
    pub from: Id,
    pub to: Id,
    pub from_port: String,
    pub to_port: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Graph {
    pub id: Id,
    pub name: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}
