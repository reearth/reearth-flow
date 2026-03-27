use std::sync::Arc;

use reearth_flow_types::{Attribute, Attributes};
use rquickjs::{Function, Object};

/// Convert a serde_json::Value into a QuickJS value.
pub(super) fn json_to_js<'js>(
    ctx: &rquickjs::Ctx<'js>,
    value: &serde_json::Value,
) -> rquickjs::Result<rquickjs::Value<'js>> {
    match value {
        serde_json::Value::Null => Ok(rquickjs::Value::new_null(ctx.clone())),
        serde_json::Value::Bool(b) => Ok(rquickjs::Value::new_bool(ctx.clone(), *b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(rquickjs::Value::new_int(ctx.clone(), i as i32))
            } else if let Some(f) = n.as_f64() {
                Ok(rquickjs::Value::new_float(ctx.clone(), f))
            } else {
                Ok(rquickjs::Value::new_null(ctx.clone()))
            }
        }
        serde_json::Value::String(s) => {
            let js_str = rquickjs::String::from_str(ctx.clone(), s)?;
            Ok(js_str.into_value())
        }
        serde_json::Value::Array(arr) => {
            let js_arr = rquickjs::Array::new(ctx.clone())?;
            for (i, v) in arr.iter().enumerate() {
                let js_val = json_to_js(ctx, v)?;
                js_arr.set(i, js_val)?;
            }
            Ok(js_arr.into_value())
        }
        serde_json::Value::Object(map) => {
            let js_obj = Object::new(ctx.clone())?;
            for (k, v) in map {
                let js_val = json_to_js(ctx, v)?;
                js_obj.set(k.as_str(), js_val)?;
            }
            Ok(js_obj.into_value())
        }
    }
}

/// Convert a QuickJS value back into serde_json::Value.
pub(super) fn js_to_json(value: &rquickjs::Value<'_>) -> serde_json::Value {
    if value.is_undefined() || value.is_null() {
        serde_json::Value::Null
    } else if let Some(b) = value.as_bool() {
        serde_json::Value::Bool(b)
    } else if let Some(i) = value.as_int() {
        serde_json::Value::Number(i.into())
    } else if let Some(f) = value.as_float() {
        serde_json::Number::from_f64(f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null)
    } else if let Some(s) = value.as_string() {
        s.to_string()
            .map(serde_json::Value::String)
            .unwrap_or(serde_json::Value::Null)
    } else if let Some(arr) = value.as_array() {
        let items: Vec<serde_json::Value> = arr
            .iter::<rquickjs::Value>()
            .filter_map(|v| v.ok().map(|v| js_to_json(&v)))
            .collect();
        serde_json::Value::Array(items)
    } else if let Some(obj) = value.as_object() {
        let mut map = serde_json::Map::new();
        if let Some(props) = obj.keys::<String>().collect::<Result<Vec<_>, _>>().ok() {
            for key in props {
                if let Ok(val) = obj.get::<_, rquickjs::Value>(&key) {
                    map.insert(key, js_to_json(&val));
                }
            }
        }
        serde_json::Value::Object(map)
    } else {
        serde_json::Value::Null
    }
}

/// Create a `value(key)` JS function that lazily fetches a single attribute from Rust.
pub(super) fn make_value_fn<'js>(
    js: &rquickjs::Ctx<'js>,
    attrs: Arc<Attributes>,
) -> rquickjs::Result<Function<'js>> {
    let js2 = js.clone();
    Function::new(js.clone(), move |key: String| {
        match attrs.get(&Attribute::new(key)) {
            Some(v) => {
                let json_val: serde_json::Value = v.clone().into();
                json_to_js(&js2, &json_val)
            }
            None => Ok(rquickjs::Value::new_undefined(js2.clone())),
        }
    })
}

/// Convert global_params into a JS env object.
pub(super) fn make_env_js<'js>(
    js: &rquickjs::Ctx<'js>,
    global_params: &Option<std::collections::HashMap<String, serde_json::Value>>,
) -> Result<rquickjs::Value<'js>, rquickjs::Error> {
    let env_value: serde_json::Value = if let Some(params) = global_params {
        serde_json::Value::Object(
            params
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        )
    } else {
        serde_json::Value::Object(serde_json::Map::new())
    };
    json_to_js(js, &env_value)
}
