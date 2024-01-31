use std::collections::HashMap;
use std::str::FromStr;

use bytes::Bytes;
use futures::Future;
use serde::{Deserialize, Serialize};
use serde_json::Number;
use strum_macros::EnumString;

use reearth_flow_workflow::error::Error::WorkflowConfigError;
use reearth_flow_workflow::graph::{NodeAction, NodeProperty};
use reearth_flow_workflow::id::Id;

pub type ActionInputPort = String;
pub type ActionOutputPort = String;
pub type ActionOutput = HashMap<ActionOutputPort, ActionValue>;
pub type ActionInput = HashMap<ActionInputPort, ActionValue>;

#[derive(Debug, Clone)]
pub enum ActionValue {
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<ActionValue>),
    Bytes(Bytes),
    Object(HashMap<String, ActionValue>),
}

#[derive(Serialize, Deserialize, EnumString, Debug, Clone)]
pub enum Action {
    #[strum(serialize = "featureReader")]
    FeatureReader,
}

impl TryFrom<NodeAction> for Action {
    type Error = anyhow::Error;

    fn try_from(action: NodeAction) -> Result<Self, Self::Error> {
        Self::from_str(&action.to_string())
            .map_err(|e| WorkflowConfigError(format!("unknown action: {}", e)).into())
    }
}

#[derive(Debug)]
pub struct ActionContext {
    pub node_id: Id,
    pub node_name: String,
    pub property: NodeProperty,
}

pub struct ActionResult {}

impl Action {
    pub fn run(
        &self,
        ctx: ActionContext,
        inputs: ActionInput,
    ) -> Box<dyn Future<Output = anyhow::Result<ActionOutput>> + Send> {
        match self {
            Action::FeatureReader => Box::new(feature_reader(ctx, inputs)),
        }
    }
}

async fn feature_reader(ctx: ActionContext, inputs: ActionInput) -> anyhow::Result<ActionOutput> {
    println!("FeatureReader {:?}", ctx);
    println!("inputs {:?}", inputs);
    Ok(ActionOutput::new())
}
