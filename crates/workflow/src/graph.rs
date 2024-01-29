use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::action::Action;
use crate::id::Id;

pub type PropertyValue = Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum NodePropertyKind {
    #[serde(rename = "general")]
    General,
    #[serde(rename = "output")]
    Output,
}

impl ToString for NodePropertyKind {
    fn to_string(&self) -> String {
        match self {
            NodePropertyKind::General => "general".to_string(),
            NodePropertyKind::Output => "output".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeProperty {
    pub name: String,
    pub kind: NodePropertyKind,
    pub value: PropertyValue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NodeEntity {
    pub id: Id,
    pub name: String,
    pub properties: Vec<NodeProperty>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Node {
    #[serde(rename = "action")]
    Action {
        #[serde(flatten)]
        entity: NodeEntity,
        #[serde(flatten)]
        action: Action,
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

    pub fn properties(&self) -> &Vec<NodeProperty> {
        match self {
            Node::Action { entity, action: _ } => &entity.properties,
            Node::SubGraph {
                entity,
                sub_graph_id: _,
            } => &entity.properties,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EdgeProperty {
    pub from_output: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub id: Id,
    pub from: Id,
    pub to: Id,
    pub edge_properties: EdgeProperty,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Graph {
    pub id: Id,
    pub name: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}
