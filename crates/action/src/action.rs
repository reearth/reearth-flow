use std::fmt::Display;
use std::pin::Pin;
use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use futures::Future;
use serde::{Deserialize, Serialize};
use serde_json::Number;
use strum_macros::EnumString;

use reearth_flow_common::str::base64_encode;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_workflow::graph::NodeProperty;
use reearth_flow_workflow::id::Id;

use crate::{attribute_filter, attribute_keeper, file_reader, file_writer};

pub type Port = String;
pub const DEFAULT_PORT: &str = "default";
pub type ActionDataframe = HashMap<Port, Option<ActionValue>>;

#[derive(Debug, Clone)]
pub enum ActionValue {
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<ActionValue>),
    Bytes(Bytes),
    Map(HashMap<String, ActionValue>),
}

impl Default for ActionValue {
    fn default() -> Self {
        Self::String("".to_owned())
    }
}

impl Display for ActionValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionValue::Bool(v) => write!(f, "{}", v),
            ActionValue::Number(v) => write!(f, "{}", v),
            ActionValue::String(v) => write!(f, "{}", v),
            ActionValue::Array(v) => write!(f, "{:?}", v),
            ActionValue::Bytes(v) => write!(f, "{:?}", v),
            ActionValue::Map(v) => write!(f, "{:?}", v),
        }
    }
}

impl From<serde_json::Value> for ActionValue {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Bool(v) => ActionValue::Bool(v),
            serde_json::Value::Number(v) => ActionValue::Number(v),
            serde_json::Value::String(v) => ActionValue::String(v),
            serde_json::Value::Array(v) => {
                ActionValue::Array(v.into_iter().map(ActionValue::from).collect::<Vec<_>>())
            }
            serde_json::Value::Object(v) => ActionValue::Map(
                v.into_iter()
                    .map(|(k, v)| (k, ActionValue::from(v)))
                    .collect::<HashMap<_, _>>(),
            ),
            _ => ActionValue::String("".to_owned()),
        }
    }
}

impl From<ActionValue> for serde_json::Value {
    fn from(value: ActionValue) -> Self {
        match value {
            ActionValue::Bool(v) => serde_json::Value::Bool(v),
            ActionValue::Number(v) => serde_json::Value::Number(v),
            ActionValue::String(v) => serde_json::Value::String(v),
            ActionValue::Array(v) => serde_json::Value::Array(
                v.into_iter()
                    .map(serde_json::Value::from)
                    .collect::<Vec<_>>(),
            ),
            ActionValue::Bytes(v) => serde_json::Value::String(base64_encode(v.as_ref())),
            ActionValue::Map(v) => serde_json::Value::Object(
                v.into_iter()
                    .map(|(k, v)| (k, serde_json::Value::from(v)))
                    .collect::<serde_json::Map<_, _>>(),
            ),
        }
    }
}

#[derive(Serialize, Deserialize, EnumString, Debug, Clone)]
pub enum Action {
    #[strum(serialize = "fileReader")]
    FileReader,
    #[strum(serialize = "attributeKeeper")]
    AttributeKeeper,
    #[strum(serialize = "fileWriter")]
    FileWriter,
    #[strum(serialize = "attributeFilter")]
    AttributeFilter,
}

#[derive(Debug, Default, Clone)]
pub struct ActionContext {
    pub node_id: Id,
    pub node_name: String,
    pub node_property: NodeProperty,
    pub expr_engine: Arc<Engine>,
}

impl ActionContext {
    pub fn new(
        node_id: Id,
        node_name: String,
        node_property: NodeProperty,
        expr_engine: Arc<Engine>,
    ) -> Self {
        Self {
            node_id,
            node_name,
            node_property,
            expr_engine,
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
            Action::FileReader => Box::pin(file_reader::run(ctx, input)),
            Action::AttributeKeeper => Box::pin(attribute_keeper::run(ctx, input)),
            Action::FileWriter => Box::pin(file_writer::run(ctx, input)),
            Action::AttributeFilter => Box::pin(attribute_filter::run(ctx, input)),
        }
    }
}
