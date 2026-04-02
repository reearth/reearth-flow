use reearth_flow_types::{Attribute, AttributeValue, Feature};
use serde_json::Value;

use crate::core::eval::Context;

pub fn context_from_feature(feature: &Feature) -> Context {
    let attrs = std::sync::Arc::clone(&feature.attributes);
    let attrs2 = std::sync::Arc::clone(&feature.attributes);
    let mut ctx = Context::new();
    ctx.register(
        "__resolve",
        Box::new(move |args| {
            let name = args.first().and_then(|v| v.as_str()).unwrap_or("");
            Ok(attrs
                .get(&Attribute::new(name))
                .map(|v| v.clone().into())
                .unwrap_or(Value::Null))
        }),
    );
    ctx.register(
        "value",
        Box::new(move |args| {
            let name = args.first().and_then(|v| v.as_str()).unwrap_or("");
            Ok(attrs2
                .get(&Attribute::new(name))
                .map(|v| v.clone().into())
                .unwrap_or(Value::Null))
        }),
    );
    ctx
}

pub fn attribute_value_from_json(v: Value) -> AttributeValue {
    match v {
        Value::Null => AttributeValue::Null,
        Value::Bool(b) => AttributeValue::Bool(b),
        Value::Number(n) => AttributeValue::Number(n),
        Value::String(s) => AttributeValue::String(s),
        other => AttributeValue::String(other.to_string()),
    }
}
