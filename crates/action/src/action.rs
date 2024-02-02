use std::collections::HashMap;
use std::pin::Pin;

use bytes::Bytes;
use futures::Future;
use serde::{Deserialize, Serialize};
use serde_json::Number;
use strum_macros::EnumString;

use reearth_flow_workflow::graph::NodeProperty;
use reearth_flow_workflow::id::Id;
use reearth_flow_workflow::workflow::Parameter;

use crate::{attribute_keeper, feature_reader};

pub type Port = String;
pub const DEFAULT_PORT: &str = "default";
pub type ActionDataframe = HashMap<Port, Option<ActionValue>>;

#[derive(Debug, Clone)]
pub enum ActionValue {
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<ActionValue>),
    ArrayMap(Vec<HashMap<String, ActionValue>>),
    Bytes(Bytes),
    Object(HashMap<String, ActionValue>),
}

#[derive(Serialize, Deserialize, EnumString, Debug, Clone)]
pub enum Action {
    #[strum(serialize = "featureReader")]
    FeatureReader,
    #[strum(serialize = "attributeKeeper")]
    AttributeKeeper,
}

#[derive(Debug, Clone)]
pub struct ActionContext {
    pub node_id: Id,
    pub node_name: String,
    pub node_property: NodeProperty,
    pub parameter: Parameter,
}

impl ActionContext {
    pub fn new(
        node_id: Id,
        node_name: String,
        node_property: NodeProperty,
        parameter: Parameter,
    ) -> Self {
        Self {
            node_id,
            node_name,
            node_property,
            parameter,
        }
    }
}

impl Action {
    pub fn run(
        &self,
        ctx: ActionContext,
        input: Option<ActionDataframe>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ActionDataframe>> + Send + 'static>> {
        match self {
            Action::FeatureReader => Box::pin(feature_reader::run(ctx, input)),
            Action::AttributeKeeper => Box::pin(attribute_keeper::run(ctx, input)),
        }
    }
}
