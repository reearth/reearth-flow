use std::sync::Arc;

use reearth_flow_types::{Attribute, AttributeValue, Feature};

use crate::core::eval::Context;
use crate::core::value::Value;

// serde_json <-> Value conversions live here, not in core, to keep core
// independent of serde_json's container types.

pub fn json_to_value(v: serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => Value::Number(n),
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => Value::Array(arr.into_iter().map(json_to_value).collect()),
        serde_json::Value::Object(map) => {
            Value::Map(map.into_iter().map(|(k, v)| (k, json_to_value(v))).collect())
        }
    }
}

pub fn value_to_json(v: Value) -> serde_json::Value {
    match v {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(b),
        Value::Number(n) => serde_json::Value::Number(n),
        Value::String(s) => serde_json::Value::String(s),
        Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(value_to_json).collect())
        }
        Value::Map(map) => serde_json::Value::Object(
            map.into_iter().map(|(k, v)| (k, value_to_json(v))).collect(),
        ),
    }
}

pub fn context_from_feature(
    feature: &Feature,
    env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
) -> Context {
    let attrs = Arc::clone(&feature.attributes);
    let attrs2 = Arc::clone(&feature.attributes);
    let mut ctx = Context::new();
    ctx.register(
        "__resolve",
        Box::new(move |args| {
            let name = args.first().and_then(|v| {
                if let Value::String(s) = v { Some(s.as_str()) } else { None }
            }).unwrap_or("");
            Ok(attrs
                .get(&Attribute::new(name))
                .map(|v| json_to_value(serde_json::Value::from(v.clone())))
                .unwrap_or(Value::Null))
        }),
    );
    ctx.register(
        "value",
        Box::new(move |args| {
            let name = args.first().and_then(|v| {
                if let Value::String(s) = v { Some(s.as_str()) } else { None }
            }).unwrap_or("");
            Ok(attrs2
                .get(&Attribute::new(name))
                .map(|v| json_to_value(serde_json::Value::from(v.clone())))
                .unwrap_or(Value::Null))
        }),
    );
    ctx.register(
        "env",
        Box::new(move |args| {
            let name = args.first().and_then(|v| {
                if let Value::String(s) = v { Some(s.as_str()) } else { None }
            }).unwrap_or("");
            Ok(env_vars.get(name).cloned().map(json_to_value).unwrap_or(Value::Null))
        }),
    );
    ctx
}

pub fn attribute_value_from_eval(v: Value) -> AttributeValue {
    match v {
        Value::Null => AttributeValue::Null,
        Value::Bool(b) => AttributeValue::Bool(b),
        Value::Number(n) => AttributeValue::Number(n),
        Value::String(s) => AttributeValue::String(s),
        other => AttributeValue::String(value_to_json(other).to_string()),
    }
}
