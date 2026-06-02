use std::sync::Arc;

use nutype::nutype;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use reearth_flow_expr::{
    bool_cast, compile, eval, str_cast, Error as ExprError, InnerError, InnerResult,
    Value as ExprValue,
};

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
    FlowExpr,
    String,
}

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
        feature: &Feature,
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    ) -> reearth_flow_expr::Result<AttributeValue> {
        let v = match self {
            CompiledCode::Expr(e) => eval(e, &mut env_from_feature(feature, env_vars))?,
            CompiledCode::Literal(s) => ExprValue::String(s.clone()),
        };
        attribute_value_from_eval(v)
    }

    pub fn eval_bool(
        &self,
        feature: &Feature,
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    ) -> reearth_flow_expr::Result<bool> {
        match self {
            CompiledCode::Expr(e) => {
                eval(e, &mut env_from_feature(feature, env_vars)).map(bool_cast)
            }
            CompiledCode::Literal(s) => Ok(!s.is_empty()),
        }
    }

    pub fn eval_string(
        &self,
        feature: &Feature,
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    ) -> reearth_flow_expr::Result<String> {
        match self {
            CompiledCode::Expr(e) => eval(e, &mut env_from_feature(feature, env_vars))
                .and_then(|v| str_cast(v).map_err(|e| ExprError::EvalString { msg: e.msg })),
            CompiledCode::Literal(s) => Ok(s.clone()),
        }
    }
}

pub fn json_to_value(v: serde_json::Value) -> ExprValue {
    match v {
        serde_json::Value::Null => ExprValue::Null,
        serde_json::Value::Bool(b) => ExprValue::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                ExprValue::Int(i)
            } else if let Some(f) = n.as_f64() {
                ExprValue::Float(f)
            } else {
                tracing::warn!(value = %n, "flow expr unrepresentable number converted to null");
                ExprValue::Null
            }
        }
        serde_json::Value::String(s) => ExprValue::String(s),
        serde_json::Value::Array(arr) => {
            ExprValue::array(arr.into_iter().map(json_to_value).collect())
        }
        serde_json::Value::Object(map) => ExprValue::map(
            map.into_iter()
                .map(|(k, v)| (k, json_to_value(v)))
                .collect(),
        ),
    }
}

#[derive(Debug)]
struct AttributesObject(Arc<crate::feature::Attributes>);

impl AttributesObject {
    fn get_value(&self, name: &str) -> Option<ExprValue> {
        self.0
            .get(&Attribute::new(name))
            .map(|v| json_to_value(serde_json::Value::from(v.clone())))
    }
}

impl reearth_flow_expr::ImmutableObject for AttributesObject {
    fn type_name(&self) -> &'static str {
        "Attributes"
    }

    fn call_method(&self, method: &str, args: &[ExprValue]) -> InnerResult<ExprValue> {
        match method {
            "__getitem__" => {
                reearth_flow_expr::unpack_args!(args => key);
                let ExprValue::String(name) = key else {
                    return Err(InnerError::new(format!(
                        "attributes index must be a string, got {}",
                        key.type_name()
                    )));
                };
                self.get_value(name)
                    .ok_or_else(|| InnerError::new(format!("attribute '{name}' not found")))
            }
            "get" => {
                let (key, fallback) = match args {
                    [key] => (key, None),
                    [key, fallback] => (key, Some(fallback)),
                    _ => {
                        return Err(InnerError::new(
                            "attributes.get() requires 1 or 2 arguments",
                        ))
                    }
                };
                let ExprValue::String(name) = key else {
                    return Err(InnerError::new(format!(
                        "attributes.get() key must be a string, got {}",
                        key.type_name()
                    )));
                };
                Ok(self
                    .get_value(name)
                    .unwrap_or_else(|| fallback.cloned().unwrap_or(ExprValue::Null)))
            }
            "__iter__" => Ok(ExprValue::array(
                self.0
                    .keys()
                    .map(|k| ExprValue::String(k.as_ref().to_string()))
                    .collect(),
            )),
            "__contains__" => {
                reearth_flow_expr::unpack_args!(args => key);
                let ExprValue::String(name) = key else {
                    return Err(InnerError::new(format!(
                        "'in attributes' key must be a string, got {}",
                        key.type_name()
                    )));
                };
                Ok(ExprValue::Bool(self.get_value(name).is_some()))
            }
            m => Err(InnerError::new(format!("Attributes has no method '{m}'"))),
        }
    }
}

#[derive(Debug)]
struct EnvObject(Arc<serde_json::Map<String, serde_json::Value>>);

impl EnvObject {
    fn get_value(&self, name: &str) -> Option<ExprValue> {
        self.0.get(name).cloned().map(json_to_value)
    }
}

impl reearth_flow_expr::ImmutableObject for EnvObject {
    fn type_name(&self) -> &'static str {
        "Env"
    }

    fn call_method(&self, method: &str, args: &[ExprValue]) -> InnerResult<ExprValue> {
        match method {
            "__getitem__" => {
                reearth_flow_expr::unpack_args!(args => key);
                let ExprValue::String(name) = key else {
                    return Err(InnerError::new(format!(
                        "env index must be a string, got {}",
                        key.type_name()
                    )));
                };
                self.get_value(name)
                    .ok_or_else(|| InnerError::new(format!("env var '{name}' not found")))
            }
            "get" => {
                let (key, fallback) = match args {
                    [key] => (key, None),
                    [key, fallback] => (key, Some(fallback)),
                    _ => return Err(InnerError::new("env.get() requires 1 or 2 arguments")),
                };
                let ExprValue::String(name) = key else {
                    return Err(InnerError::new(format!(
                        "env.get() key must be a string, got {}",
                        key.type_name()
                    )));
                };
                Ok(self
                    .get_value(name)
                    .unwrap_or_else(|| fallback.cloned().unwrap_or(ExprValue::Null)))
            }
            m => Err(InnerError::new(format!("Env has no method '{m}'"))),
        }
    }
}

fn env_from_feature(
    feature: &Feature,
    env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
) -> reearth_flow_expr::Env {
    let mut env = reearth_flow_expr::default_env();
    env.insert(
        "attributes".into(),
        ExprValue::object(AttributesObject(Arc::clone(&feature.attributes))),
    );
    env.insert("env".into(), ExprValue::object(EnvObject(env_vars)));
    env
}

/// Cyclic values are unsupported — see expr/docs/design.md#no-cycle-detection
fn attribute_value_from_eval(v: ExprValue) -> reearth_flow_expr::Result<AttributeValue> {
    let eval_err = |msg: String| ExprError::Eval { pos: 0, msg };
    match v {
        ExprValue::Null => Ok(AttributeValue::Null),
        ExprValue::Bool(b) => Ok(AttributeValue::Bool(b)),
        ExprValue::Int(n) => Ok(AttributeValue::Number(n.into())),
        ExprValue::Float(f) => serde_json::Number::from_f64(f)
            .map(AttributeValue::Number)
            .ok_or_else(|| {
                eval_err(format!(
                    "float value {f} is not representable as an attribute (nan/inf)"
                ))
            }),
        ExprValue::String(s) => Ok(AttributeValue::String(s)),
        ExprValue::Array(arr) => Ok(AttributeValue::Array(
            arr.borrow()
                .iter()
                .map(|v| attribute_value_from_eval(v.clone()))
                .collect::<reearth_flow_expr::Result<Vec<_>>>()?,
        )),
        ExprValue::Map(map) => Ok(AttributeValue::Map(
            map.borrow()
                .iter()
                .map(|(k, v)| attribute_value_from_eval(v.clone()).map(|v| (k.clone(), v)))
                .collect::<reearth_flow_expr::Result<_>>()?,
        )),
        ExprValue::Fn(_) => Err(eval_err(
            "function value cannot be stored as an attribute".into(),
        )),
        ExprValue::Module(_) => Err(eval_err(
            "module value cannot be stored as an attribute".into(),
        )),
        ExprValue::Object(rc) => {
            if let Some(v) = rc.serialize() {
                attribute_value_from_eval(v)
            } else {
                Err(eval_err(format!(
                    "{} object cannot be stored as an attribute",
                    rc.type_name()
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use indexmap::indexmap;

    use super::*;
    use crate::attribute::AttributeValue;
    use crate::feature::Feature;

    fn eval_bool(expr: &str, feature: &Feature) -> bool {
        let code = Code {
            ty: CodeType::FlowExpr,
            value: expr.to_string(),
        };
        let env_vars = Arc::new(serde_json::Map::new());
        code.compile()
            .unwrap()
            .eval_bool(feature, env_vars)
            .unwrap()
    }

    #[test]
    fn test_attributes_in_operator() {
        let feature = Feature::from(indexmap! {
            "foo".to_string() => AttributeValue::String("bar".to_string()),
        });
        assert!(eval_bool(r#""foo" in attributes"#, &feature));
        assert!(!eval_bool(r#""missing" in attributes"#, &feature));
        assert!(!eval_bool(r#""foo" not in attributes"#, &feature));
        assert!(eval_bool(r#""missing" not in attributes"#, &feature));
    }
}
