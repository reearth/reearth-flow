use std::sync::Arc;

use nutype::nutype;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use reearth_flow_expr::{compile, eval, eval_string};

use crate::attribute::{Attribute, AttributeValue};
use crate::feature::Feature;

#[nutype(
    sanitize(trim),
    derive(
        Debug,
        Display,
        Clone,
        Eq,
        PartialEq,
        PartialOrd,
        Ord,
        AsRef,
        Serialize,
        Deserialize,
        Hash,
        JsonSchema
    )
)]
pub struct Expr(String);

impl Expr {
    pub fn compile(&self) -> reearth_flow_expr::Result<CompiledCode> {
        compile(self.as_ref()).map(CompiledCode::Expr)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum CodeType {
    /// Evaluated as a Flow expression at runtime
    FlowExpr,
    /// Used as a plain string literal
    String,
}

/// A typed code value: a string paired with a [`CodeType`] that controls how it is interpreted at evaluation time.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Code {
    #[serde(rename = "type")]
    pub ty: CodeType,
    pub value: String,
}

impl Code {
    pub fn compile(&self) -> reearth_flow_expr::Result<CompiledCode> {
        match self.ty {
            CodeType::FlowExpr => compile(&self.value).map(CompiledCode::Expr),
            CodeType::String => Ok(CompiledCode::Literal(self.value.clone())),
        }
    }
}

/// A compiled form of [`Code`] or [`Expr`], ready for evaluation.
#[derive(Debug, Clone)]
pub enum CompiledCode {
    Expr(reearth_flow_expr::CompiledExpr),
    Literal(String),
}

impl CompiledCode {
    pub fn eval(
        &self,
        env: &mut reearth_flow_expr::Env,
    ) -> reearth_flow_expr::Result<reearth_flow_expr::Value> {
        match self {
            CompiledCode::Expr(e) => eval(e, env),
            CompiledCode::Literal(s) => Ok(reearth_flow_expr::Value::String(s.clone())),
        }
    }

    pub fn eval_string(
        &self,
        env: &mut reearth_flow_expr::Env,
    ) -> reearth_flow_expr::Result<String> {
        match self {
            CompiledCode::Expr(e) => eval_string(e, env),
            CompiledCode::Literal(s) => Ok(s.clone()),
        }
    }
}

pub fn json_to_value(v: serde_json::Value) -> reearth_flow_expr::Value {
    use reearth_flow_expr::Value;
    match v {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                tracing::warn!(value = %n, "flow expr unrepresentable number converted to null");
                Value::Null
            }
        }
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => Value::array(arr.into_iter().map(json_to_value).collect()),
        serde_json::Value::Object(map) => Value::map(
            map.into_iter()
                .map(|(k, v)| (k, json_to_value(v)))
                .collect(),
        ),
    }
}

pub fn env_from_feature(
    feature: &Feature,
    env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
) -> reearth_flow_expr::Env {
    use reearth_flow_expr::{NativeFn, Value};
    let mut env = reearth_flow_expr::default_env();
    let attrs = Arc::clone(&feature.attributes);
    env.insert(
        "value".into(),
        Value::Fn(NativeFn::new(move |args| {
            let Some(Value::String(name)) = args.first() else {
                return Ok(Value::Null);
            };
            Ok(attrs
                .get(&Attribute::new(name))
                .map(|v| json_to_value(serde_json::Value::from(v.clone())))
                .unwrap_or(Value::Null))
        })),
    );
    env.insert(
        "env".into(),
        Value::Fn(NativeFn::new(move |args| {
            let Some(Value::String(name)) = args.first() else {
                return Ok(Value::Null);
            };
            Ok(env_vars
                .get(name.as_str())
                .cloned()
                .map(json_to_value)
                .unwrap_or(Value::Null))
        })),
    );
    env
}

pub fn attribute_value_from_eval(v: reearth_flow_expr::Value) -> AttributeValue {
    use reearth_flow_expr::Value;
    match v {
        Value::Null => AttributeValue::Null,
        Value::Bool(b) => AttributeValue::Bool(b),
        Value::Int(n) => AttributeValue::Number(n.into()),
        Value::Float(f) => serde_json::Number::from_f64(f)
            .map(AttributeValue::Number)
            .unwrap_or_else(|| {
                tracing::warn!(
                    value = f,
                    "flow expr nan/inf float converted to null attribute"
                );
                AttributeValue::Null
            }),
        Value::String(s) => AttributeValue::String(s),
        Value::Array(arr) => AttributeValue::Array(
            arr.borrow()
                .iter()
                .map(|v| attribute_value_from_eval(v.clone()))
                .collect(),
        ),
        Value::Map(map) => AttributeValue::Map(
            map.borrow()
                .iter()
                .map(|(k, v)| (k.clone(), attribute_value_from_eval(v.clone())))
                .collect(),
        ),
        Value::Fn(_) => AttributeValue::Null,
        Value::Object(rc) => {
            let borrowed = rc.borrow();
            if let Some(v) = borrowed.serialize() {
                drop(borrowed);
                attribute_value_from_eval(v)
            } else {
                tracing::warn!(
                    type_name = borrowed.type_name(),
                    "flow expr object converted to type-name string"
                );
                AttributeValue::String(format!("<{}>", borrowed.type_name()))
            }
        }
    }
}
