pub mod error;
pub mod utils;

use std::cmp::Ordering;
use std::fmt::Display;
use std::{collections::HashMap, sync::Arc};

use bytes::Bytes;
use reearth_flow_common::uri::Uri;
use rhai::serde::from_dynamic;
use serde::{Deserialize, Serialize};
use serde_json::Number;

use reearth_flow_action_log::ActionLogger;
use reearth_flow_common::str::base64_encode;
use reearth_flow_common::xml::XmlXpathValue;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_workflow::graph::NodeProperty;
use reearth_flow_workflow::id::Id;

pub type Port = String;
pub const DEFAULT_PORT: &str = "default";
pub const REJECTED_PORT: &str = "rejected";
pub type ActionDataframe = HashMap<Port, Option<ActionValue>>;
pub type ActionValueIndex = HashMap<String, HashMap<String, Vec<ActionValue>>>;
pub type ActionResult = std::result::Result<ActionDataframe, error::Error>;
pub type Result<T, E = error::Error> = std::result::Result<T, E>;

#[async_trait::async_trait]
#[typetag::serde(tag = "action", content = "with")]
pub trait Action: Send + Sync {
    async fn run(&self, ctx: ActionContext, input: Option<ActionDataframe>) -> ActionResult;
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum ActionValue {
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<ActionValue>),
    Bytes(Bytes),
    Map(HashMap<String, ActionValue>),
}

impl ActionValue {
    pub fn extend(self, value: Self) -> Result<Self> {
        match (self, value) {
            (ActionValue::Map(mut a), ActionValue::Map(b)) => {
                for (k, v) in b {
                    a.insert(k, v);
                }
                Ok(ActionValue::Map(a))
            }
            (ActionValue::Array(mut a), ActionValue::Array(b)) => {
                a.extend(b);
                Ok(ActionValue::Array(a))
            }
            _ => Err(error::Error::internal_runtime("Cannot extend")),
        }
    }
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
    type Error = error::Error;

    fn try_from(value: rhai::Dynamic) -> std::result::Result<Self, Self::Error> {
        let value: serde_json::Value =
            from_dynamic(&value).map_err(error::Error::internal_runtime)?;
        Ok(value.into())
    }
}

impl TryFrom<Uri> for ActionValue {
    type Error = error::Error;

    fn try_from(value: Uri) -> std::result::Result<Self, Self::Error> {
        let value: serde_json::Value =
            serde_json::to_value(value).map_err(error::Error::internal_runtime)?;
        Ok(value.into())
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

impl Default for ActionContext {
    fn default() -> Self {
        Self {
            job_id: Id::default(),
            workflow_id: Id::default(),
            node_id: Id::default(),
            node_name: "".to_owned(),
            node_property: Default::default(),
            expr_engine: Arc::new(Engine::new()),
            storage_resolver: Arc::new(StorageResolver::new()),
            logger: Arc::new(ActionLogger::root(
                reearth_flow_action_log::Discard,
                reearth_flow_action_log::o!(),
            )),
            root_span: tracing::Span::current(),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_rhai_dynamic() {
        let dynamic_value = rhai::Dynamic::from(42);
        let action_value: std::result::Result<ActionValue, _> = dynamic_value.try_into();
        assert_eq!(action_value.unwrap(), ActionValue::Number(Number::from(42)));

        let dynamic_value = rhai::Dynamic::from("Hello");
        let action_value: std::result::Result<ActionValue, _> = dynamic_value.try_into();
        assert_eq!(
            action_value.unwrap(),
            ActionValue::String("Hello".to_string())
        );
    }

    #[test]
    fn test_partial_ord() {
        let number1 = ActionValue::Number(Number::from(42));
        let number2 = ActionValue::Number(Number::from(42));
        assert_eq!(number1.partial_cmp(&number2), Some(Ordering::Equal));

        let string1 = ActionValue::String("Hello".to_string());
        let string2 = ActionValue::String("World".to_string());
        assert_eq!(string1.partial_cmp(&string2), Some(Ordering::Less));
    }

    #[test]
    fn test_eq() {
        let number1 = ActionValue::Number(Number::from(42));
        let number2 = ActionValue::Number(Number::from(42));
        assert_eq!(number1, number2);

        let string1 = ActionValue::String("Hello".to_string());
        let string2 = ActionValue::String("Hello".to_string());
        assert_eq!(string1, string2);

        let map1 = ActionValue::Map(
            vec![("key".to_string(), ActionValue::String("value".to_string()))]
                .into_iter()
                .collect(),
        );
        let map2 = ActionValue::Map(
            vec![("key".to_string(), ActionValue::String("value".to_string()))]
                .into_iter()
                .collect(),
        );
        assert_eq!(map1, map2);
    }

    #[test]
    fn test_compare_numbers() {
        let number1 = Number::from(42);
        let number2 = Number::from(42);
        assert_eq!(compare_numbers(&number1, &number2), Some(Ordering::Equal));

        let number1 = Number::from(42);
        let number2 = Number::from(43);
        assert_eq!(compare_numbers(&number1, &number2), Some(Ordering::Less));

        let number1 = Number::from(43);
        let number2 = Number::from(42);
        assert_eq!(compare_numbers(&number1, &number2), Some(Ordering::Greater));
    }
}
