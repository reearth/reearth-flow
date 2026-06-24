use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexMap;
use nutype::nutype;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use std::rc::Rc;

use reearth_flow_expr::{
    compile, env_bind, env_remove, eval_error, expect_arity, Env as ExprEnv, FromValue,
    Result as ExprResult, TypeValue as ExprTypeValue, Value as ExprValue,
};

use crate::error::{Error as TypesError, Result as TypesResult};

use crate::attribute::{Attribute, AttributeValue};
use crate::feature::Feature;

impl From<reearth_flow_expr::Error> for TypesError {
    fn from(e: reearth_flow_expr::Error) -> Self {
        TypesError::InternalRuntime(e.to_string())
    }
}

/// Newtype bridging `FromValue` (from `reearth_flow_expr`) and `AttributeValue`
/// (from `reearth_flow_common`); required by the orphan rule since both are foreign to this crate.
struct FlowValue(AttributeValue);

impl FromValue for FlowValue {
    type Error = TypesError;

    fn from_null() -> TypesResult<Self> {
        Ok(FlowValue(AttributeValue::Null))
    }
    fn from_bool(b: bool) -> TypesResult<Self> {
        Ok(FlowValue(AttributeValue::Bool(b)))
    }
    fn from_int(n: i64) -> TypesResult<Self> {
        Ok(FlowValue(AttributeValue::Number(n.into())))
    }
    fn from_float(f: f64) -> TypesResult<Self> {
        serde_json::Number::from_f64(f)
            .map(|n| FlowValue(AttributeValue::Number(n)))
            .ok_or_else(|| {
                TypesError::Conversion(format!(
                    "float value {f} is not representable as an attribute (nan/inf)"
                ))
            })
    }
    fn from_string(s: String) -> TypesResult<Self> {
        Ok(FlowValue(AttributeValue::String(s)))
    }
    fn from_list(items: Vec<Self>) -> TypesResult<Self> {
        Ok(FlowValue(AttributeValue::Array(
            items.into_iter().map(|FlowValue(v)| v).collect(),
        )))
    }
    fn from_dict(map: IndexMap<String, Self>) -> TypesResult<Self> {
        Ok(FlowValue(AttributeValue::Map(
            map.into_iter()
                .map(|(k, FlowValue(v))| (k, v))
                .collect::<HashMap<_, _>>(),
        )))
    }
    fn on_cycle() -> TypesResult<Self> {
        Err(TypesError::Conversion(
            "cyclic reference detected in eval return value".into(),
        ))
    }
    fn on_unconvertible(msg: String) -> TypesResult<Self> {
        Err(TypesError::Conversion(msg))
    }
}

thread_local! {
    static EVAL_ENV: ExprEnv = reearth_flow_expr::default_env();
}

fn eval_with_feature(
    expr: &reearth_flow_expr::CompiledExpr,
    feature: &Feature,
    env_vars: &Arc<serde_json::Map<String, serde_json::Value>>,
) -> TypesResult<AttributeValue> {
    EVAL_ENV.with(|env| {
        env_bind(
            env,
            "attributes",
            ExprValue::object(AttributesObject(Arc::clone(&feature.attributes))),
        );
        env_bind(
            env,
            "env",
            ExprValue::object(EnvObject(Arc::clone(env_vars))),
        );
        reearth_flow_expr::eval::<FlowValue>(expr, env).map(|FlowValue(v)| v)
    })
}

fn eval_with_vars(
    expr: &reearth_flow_expr::CompiledExpr,
    env_vars: &Arc<serde_json::Map<String, serde_json::Value>>,
) -> TypesResult<AttributeValue> {
    EVAL_ENV.with(|env| {
        env_remove(env, "attributes");
        env_bind(
            env,
            "env",
            ExprValue::object(EnvObject(Arc::clone(env_vars))),
        );
        reearth_flow_expr::eval::<FlowValue>(expr, env).map(|FlowValue(v)| v)
    })
}

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

#[repr(u32)]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum CodeType {
    FlowExpr = 1 << 0,
    String = 1 << 1,
}

impl CodeType {
    pub const ALL: u32 = Self::FlowExpr as u32 | Self::String as u32;

    pub fn all_variants() -> &'static [CodeType] {
        &[CodeType::FlowExpr, CodeType::String]
    }

    pub fn as_mask(self) -> u32 {
        self as u32
    }

    pub fn serde_name(self) -> &'static str {
        match self {
            CodeType::FlowExpr => "flowExpr",
            CodeType::String => "string",
        }
    }
}

/// Bitmask constant covering all [`CodeType`] variants; used as the default for [`Code`].
pub const ALL_CODE_TYPES: u32 = CodeType::ALL;

#[derive(Debug, Clone)]
pub struct Code<const MASK: u32 = ALL_CODE_TYPES> {
    pub ty: CodeType,
    pub value: String,
}

impl<const MASK: u32> Code<MASK> {
    pub fn compile(&self) -> reearth_flow_expr::Result<CompiledCode> {
        match self.ty {
            CodeType::FlowExpr => compile(&self.value).map(CompiledCode::Expr),
            CodeType::String => Ok(CompiledCode::Literal(self.value.clone())),
        }
    }
}

impl<const MASK: u32> Serialize for Code<MASK> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Code", 2)?;
        state.serialize_field("type", &self.ty)?;
        state.serialize_field("value", &self.value)?;
        state.end()
    }
}

impl<'de, const MASK: u32> Deserialize<'de> for Code<MASK> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Code {
            #[serde(rename = "type")]
            ty: CodeType,
            value: String,
        }
        let h = Code::deserialize(deserializer)?;
        if h.ty.as_mask() & MASK == 0 {
            let allowed: Vec<&str> = CodeType::all_variants()
                .iter()
                .copied()
                .filter(|v| v.as_mask() & MASK != 0)
                .map(|v| v.serde_name())
                .collect();
            return Err(serde::de::Error::custom(format!(
                "code type `{}` is not allowed here; allowed: [{}]",
                h.ty.serde_name(),
                allowed.join(", ")
            )));
        }
        Ok(Self {
            ty: h.ty,
            value: h.value,
        })
    }
}

impl<const MASK: u32> schemars::JsonSchema for Code<MASK> {
    fn schema_name() -> String {
        "Code".to_string()
    }

    fn is_referenceable() -> bool {
        false
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        use schemars::schema::*;

        let allowed_types: Vec<serde_json::Value> = CodeType::all_variants()
            .iter()
            .copied()
            .filter(|v| v.as_mask() & MASK != 0)
            .map(|v| serde_json::Value::String(v.serde_name().to_string()))
            .collect();

        let type_property = Schema::Object(SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::String))),
            enum_values: Some(allowed_types),
            ..Default::default()
        });

        let value_property = Schema::Object(SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::String))),
            ..Default::default()
        });

        let mut properties = schemars::Map::new();
        properties.insert("type".to_string(), type_property);
        properties.insert("value".to_string(), value_property);

        Schema::Object(SchemaObject {
            instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Object))),
            format: Some("code".to_string()),
            object: Some(Box::new(ObjectValidation {
                required: ["type".to_string(), "value".to_string()]
                    .into_iter()
                    .collect(),
                properties,
                ..Default::default()
            })),
            ..Default::default()
        })
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
    ) -> TypesResult<AttributeValue> {
        match self {
            CompiledCode::Literal(s) => Ok(AttributeValue::String(s.clone())),
            CompiledCode::Expr(e) => eval_with_feature(e, feature, &env_vars),
        }
    }

    pub fn eval_bool(
        &self,
        feature: &Feature,
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    ) -> TypesResult<bool> {
        match self {
            CompiledCode::Literal(s) => Ok(!s.is_empty()),
            CompiledCode::Expr(e) => eval_with_feature(e, feature, &env_vars)?
                .as_bool()
                .ok_or_else(|| TypesError::Conversion("eval result is not a bool".into())),
        }
    }

    pub fn eval_float(
        &self,
        feature: &Feature,
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    ) -> TypesResult<f64> {
        match self {
            CompiledCode::Literal(s) => s
                .trim()
                .parse::<f64>()
                .map_err(|_| TypesError::Conversion(format!("literal {s:?} is not a number"))),
            CompiledCode::Expr(e) => eval_with_feature(e, feature, &env_vars)?
                .as_f64()
                .ok_or_else(|| TypesError::Conversion("expected number".into())),
        }
    }

    pub fn eval_int(
        &self,
        feature: &Feature,
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    ) -> TypesResult<i64> {
        match self {
            CompiledCode::Literal(s) => s
                .trim()
                .parse::<i64>()
                .map_err(|_| TypesError::Conversion(format!("literal {s:?} is not an integer"))),
            CompiledCode::Expr(e) => eval_with_feature(e, feature, &env_vars)?
                .as_i64()
                .ok_or_else(|| TypesError::Conversion("expected integer".into())),
        }
    }

    pub fn eval_string(
        &self,
        feature: &Feature,
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    ) -> TypesResult<String> {
        match self {
            CompiledCode::Literal(s) => Ok(s.clone()),
            CompiledCode::Expr(e) => eval_with_feature(e, feature, &env_vars)?
                .as_string()
                .ok_or_else(|| TypesError::Conversion("eval result is not a string".into())),
        }
    }

    /// Evaluate with only `env` in scope (no `attributes`), returning an AttributeValue.
    pub fn eval_env_only(
        &self,
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    ) -> TypesResult<AttributeValue> {
        match self {
            CompiledCode::Literal(s) => Ok(AttributeValue::String(s.clone())),
            CompiledCode::Expr(e) => eval_with_vars(e, &env_vars),
        }
    }

    /// Evaluate as string with only `env` in scope (no `attributes`).
    /// Use this in finish-time contexts where no current feature exists.
    pub fn eval_string_env_only(
        &self,
        env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    ) -> TypesResult<String> {
        match self {
            CompiledCode::Literal(s) => Ok(s.clone()),
            CompiledCode::Expr(e) => eval_with_vars(e, &env_vars)?
                .as_string()
                .ok_or_else(|| TypesError::Conversion("eval result is not a string".into())),
        }
    }
}

/// Resolves a feature's join/group key: concatenates named attributes with "-",
/// or evaluates `attribute_ast` as a string expression. Returns "" if neither is set.
pub fn fetch_attribute_value(
    feature: &Feature,
    env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    attribute: &Option<Vec<crate::Attribute>>,
    attribute_ast: &Option<CompiledCode>,
) -> String {
    if let Some(attribute_values) = attribute {
        attribute_values
            .iter()
            .flat_map(|key| feature.get(key))
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join("-")
    } else if let Some(ast) = attribute_ast {
        ast.eval_string(feature, env_vars)
            .unwrap_or_else(|_| "".to_string())
    } else {
        "".to_string()
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
            ExprValue::list(arr.into_iter().map(json_to_value).collect())
        }
        serde_json::Value::Object(map) => ExprValue::dict(
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
    fn type_object(&self) -> Rc<ExprTypeValue> {
        thread_local! {
            static TY: Rc<ExprTypeValue> = Rc::new(ExprTypeValue::new("Attributes", None));
        }
        TY.with(Rc::clone)
    }

    fn call_method(&self, method: &str, args: &[ExprValue]) -> ExprResult<ExprValue> {
        match method {
            "__getitem__" => {
                expect_arity("Attributes.__getitem__", args, 1, 1)?;
                let ExprValue::String(name) = &args[0] else {
                    return Err(eval_error(format!(
                        "attributes index must be a string, got {}",
                        args[0].type_name()
                    )));
                };
                self.get_value(name)
                    .ok_or_else(|| eval_error(format!("attribute '{name}' not found")))
            }
            "get" => {
                expect_arity("Attributes.get", args, 1, 2)?;
                let ExprValue::String(name) = &args[0] else {
                    return Err(eval_error(format!(
                        "Attributes.get() key must be a string, got {}",
                        args[0].type_name()
                    )));
                };
                let fallback = args.get(1);
                Ok(self
                    .get_value(name)
                    .unwrap_or_else(|| fallback.cloned().unwrap_or(ExprValue::Null)))
            }
            "__iter__" => Ok(ExprValue::list(
                self.0
                    .keys()
                    .map(|k| ExprValue::String(k.as_ref().to_string()))
                    .collect(),
            )),
            "__contains__" => {
                expect_arity("Attributes.__contains__", args, 1, 1)?;
                let ExprValue::String(name) = &args[0] else {
                    return Err(eval_error(format!(
                        "'in attributes' key must be a string, got {}",
                        args[0].type_name()
                    )));
                };
                Ok(ExprValue::Bool(self.0.contains_key(&Attribute::new(name))))
            }
            m => Err(eval_error(format!("Attributes has no method '{m}'"))),
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
    fn type_object(&self) -> Rc<ExprTypeValue> {
        thread_local! {
            static TY: Rc<ExprTypeValue> = Rc::new(ExprTypeValue::new("Env", None));
        }
        TY.with(Rc::clone)
    }

    fn call_method(&self, method: &str, args: &[ExprValue]) -> ExprResult<ExprValue> {
        match method {
            "__getitem__" => {
                expect_arity("Env.__getitem__", args, 1, 1)?;
                let ExprValue::String(name) = &args[0] else {
                    return Err(eval_error(format!(
                        "env index must be a string, got {}",
                        args[0].type_name()
                    )));
                };
                self.get_value(name)
                    .ok_or_else(|| eval_error(format!("env var '{name}' not found")))
            }
            "get" => {
                expect_arity("Env.get", args, 1, 2)?;
                let ExprValue::String(name) = &args[0] else {
                    return Err(eval_error(format!(
                        "Env.get() key must be a string, got {}",
                        args[0].type_name()
                    )));
                };
                let fallback = args.get(1);
                Ok(self
                    .get_value(name)
                    .unwrap_or_else(|| fallback.cloned().unwrap_or(ExprValue::Null)))
            }
            m => Err(eval_error(format!("Env has no method '{m}'"))),
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
        let code: Code = Code {
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
    fn test_eval_string_env_only() {
        let mut env_vars = serde_json::Map::new();
        env_vars.insert(
            "key".to_string(),
            serde_json::Value::String("val".to_string()),
        );
        let env_vars = Arc::new(env_vars);

        let literal: Code = Code {
            ty: CodeType::String,
            value: "hello".to_string(),
        };
        assert_eq!(
            literal
                .compile()
                .unwrap()
                .eval_string_env_only(Arc::clone(&env_vars))
                .unwrap(),
            "hello"
        );

        let expr: Code = Code {
            ty: CodeType::FlowExpr,
            value: r#"env["key"]"#.to_string(),
        };
        assert_eq!(
            expr.compile()
                .unwrap()
                .eval_string_env_only(Arc::clone(&env_vars))
                .unwrap(),
            "val"
        );

        // attributes are not in scope
        let no_attr: Code = Code {
            ty: CodeType::FlowExpr,
            value: "attributes".to_string(),
        };
        assert!(no_attr
            .compile()
            .unwrap()
            .eval_string_env_only(Arc::clone(&env_vars))
            .is_err());
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

    #[test]
    fn code_mask_enforced_on_deserialize() {
        type FlowExprOnly = Code<{ CodeType::FlowExpr as u32 }>;
        serde_json::from_str::<FlowExprOnly>(r#"{"type":"flowExpr","value":"1+1"}"#).unwrap();
        let err =
            serde_json::from_str::<FlowExprOnly>(r#"{"type":"string","value":"x"}"#).unwrap_err();
        assert!(err.to_string().contains("not allowed"));
    }
}
