use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::action::Action;
use crate::id::Id;

pub type PropertyValue = Value;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum PropertyKind {
    General,
    Output,
}

impl ToString for PropertyKind {
    fn to_string(&self) -> String {
        match self {
            PropertyKind::General => "general".to_string(),
            PropertyKind::Output => "output".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Property {
    pub name: String,
    pub kind: PropertyKind,
    pub value: PropertyValue,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct NodeEntity {
    pub id: Id,
    pub name: String,
    pub properties: Vec<Property>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum Node {
    ActionNode(NodeEntity, Action),
    GraphNode(NodeEntity, Graph),
}

impl Node {
    pub fn id(&self) -> Id {
        match self {
            Node::ActionNode(entity, _) => entity.id,
            Node::GraphNode(entity, _) => entity.id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Node::ActionNode(entity, _) => &entity.name,
            Node::GraphNode(entity, _) => &entity.name,
        }
    }

    pub fn properties(&self) -> &Vec<Property> {
        match self {
            Node::ActionNode(entity, _) => &entity.properties,
            Node::GraphNode(entity, _) => &entity.properties,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub id: Id,
    pub from: Node,
    pub to: Node,
    pub from_output: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Graph {
    pub id: Id,
    pub name: String,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}
