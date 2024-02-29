use std::cmp::Ordering;
use std::fmt::Display;
use std::pin::Pin;
use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use futures::Future;
use serde::{Deserialize, Serialize};
use serde_json::Number;
use strum_macros::{Display, EnumString};

use reearth_flow_action_log::ActionLogger;
use reearth_flow_common::str::base64_encode;
use reearth_flow_common::xml::XmlXpathValue;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_workflow::graph::NodeProperty;
use reearth_flow_workflow::id::Id;

use crate::{
    attribute_aggregator, attribute_keeper, attribute_manager, attribute_merger, color_converter,
    dataframe_transformer, entity_filter, entity_transformer, file_reader, file_writer,
    xml_xpath_extractor, zip_extractor,
};

pub type Port = String;
pub const DEFAULT_PORT: &str = "default";
pub type ActionDataframe = HashMap<Port, Option<ActionValue>>;
pub type ActionValueIndex = HashMap<String, HashMap<String, Vec<ActionValue>>>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
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

impl From<XmlXpathValue> for ActionValue {
    fn from(value: XmlXpathValue) -> Self {
        std::convert::Into::<ActionValue>::into(
            value.to_string().parse::<serde_json::Value>().unwrap(),
        )
    }
}

impl TryFrom<rhai::Dynamic> for ActionValue {
    type Error = anyhow::Error;

    fn try_from(value: rhai::Dynamic) -> Result<Self, Self::Error> {
        let json = serde_json::to_string(&value)?;
        let result: serde_json::Value = serde_json::from_str(&json)?;
        Ok(result.into())
    }
}

impl PartialOrd for ActionValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (ActionValue::Number(a), ActionValue::Number(b)) => compare_numbers(a, b),
            (ActionValue::String(a), ActionValue::String(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

fn compare_numbers(n1: &Number, n2: &Number) -> Option<Ordering> {
    if let Some(i1) = n1.as_i64() {
        if let Some(i2) = n2.as_i64() {
            return i1.partial_cmp(&i2);
        }
    }
    if let Some(f1) = n1.as_f64() {
        if let Some(f2) = n2.as_f64() {
            return f1.partial_cmp(&f2);
        }
    }
    None
}

#[derive(Serialize, Deserialize, EnumString, Display, Debug, Clone)]
pub enum Action {
    #[strum(serialize = "attributeKeeper")]
    AttributeKeeper,
    #[strum(serialize = "attributeMerger")]
    AttributeMerger,
    #[strum(serialize = "attributeManager")]
    AttributeManager,
    #[strum(serialize = "attributeAggregator")]
    AttributeAggregator,
    #[strum(serialize = "colorConverter")]
    ColorConverter,
    #[strum(serialize = "dataframeTransformer")]
    DataframeTransformer,
    #[strum(serialize = "entityFilter")]
    EntityFilter,
    #[strum(serialize = "entityTransformer")]
    EntityTransformer,
    #[strum(serialize = "fileReader")]
    FileReader,
    #[strum(serialize = "fileWriter")]
    FileWriter,
    #[strum(serialize = "xmlXPathExtractor")]
    XmlXPathExtractor,
    #[strum(serialize = "zipExtractor")]
    ZipExtractor,
}

#[derive(Debug, Clone)]
pub struct ActionContext {
    pub job_id: Id,
    pub workflow_id: Id,
    pub node_id: Id,
    pub node_name: String,
    pub node_property: NodeProperty,
    pub expr_engine: Arc<Engine>,
    pub storage_resolver: Arc<StorageResolver>,
    pub logger: Arc<ActionLogger>,
    pub root_span: tracing::Span,
}

impl ActionContext {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        job_id: Id,
        workflow_id: Id,
        node_id: Id,
        node_name: String,
        node_property: NodeProperty,
        expr_engine: Arc<Engine>,
        storage_resolver: Arc<StorageResolver>,
        logger: ActionLogger,
        root_span: tracing::Span,
    ) -> Self {
        Self {
            job_id,
            workflow_id,
            node_id,
            node_name,
            node_property,
            expr_engine,
            storage_resolver,
            logger: Arc::new(logger),
            root_span,
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
            Action::AttributeKeeper => Box::pin(attribute_keeper::run(ctx, input)),
            Action::AttributeMerger => Box::pin(attribute_merger::run(ctx, input)),
            Action::AttributeManager => Box::pin(attribute_manager::run(ctx, input)),
            Action::AttributeAggregator => Box::pin(attribute_aggregator::run(ctx, input)),
            Action::DataframeTransformer => Box::pin(dataframe_transformer::run(ctx, input)),
            Action::ColorConverter => Box::pin(color_converter::run(ctx, input)),
            Action::EntityFilter => Box::pin(entity_filter::run(ctx, input)),
            Action::EntityTransformer => Box::pin(entity_transformer::run(ctx, input)),
            Action::FileReader => Box::pin(file_reader::run(ctx, input)),
            Action::FileWriter => Box::pin(file_writer::run(ctx, input)),
            Action::XmlXPathExtractor => Box::pin(xml_xpath_extractor::run(ctx, input)),
            Action::ZipExtractor => Box::pin(zip_extractor::run(ctx, input)),
        }
    }
}
