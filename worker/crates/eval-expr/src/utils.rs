use std::collections::HashMap;

use crate::{error::Error, Value};
use rhai::{Dynamic, Map};
use serde_json::Map as JsonMap;

pub fn value_to_dymainc(v: &Value) -> Dynamic {
    match v {
        Value::Null => Dynamic::UNIT,
        Value::Bool(b) => Dynamic::from(*b),
        Value::String(s) => Dynamic::from(s.clone()),
        Value::Number(n) if n.is_i64() => Dynamic::from(n.as_i64().unwrap()),
        Value::Number(n) if n.is_f64() => Dynamic::from(n.as_f64().unwrap()),
        Value::Array(s) => Dynamic::from(array_to_dynamic(s)),
        Value::Object(m) => Dynamic::from(map_to_dynamic(m)),
        _ => Dynamic::default(),
    }
}

pub fn dynamic_to_value(value: &Dynamic) -> Value {
    if value.is::<rhai::INT>() {
        let int = value.as_int().unwrap();
        return int.into();
    } else if value.is::<rhai::FLOAT>() {
        let float = value.as_float().unwrap();
        return float.into();
    } else if value.is::<bool>() {
        let b = value.as_bool().unwrap();
        return b.into();
    } else if value.is::<rhai::ImmutableString>() {
        let s = value.clone().into_string().unwrap();
        return s.into();
    } else if value.is::<rhai::Array>() {
        let arr = value.clone().into_array().unwrap();
        let arr_values: Vec<_> = arr.iter().map(dynamic_to_value).collect();

        return Value::Array(arr_values);
    }
    Value::Null
}

pub fn array_to_dynamic(values: &Vec<Value>) -> Vec<Dynamic> {
    let mut ret = Vec::new();
    for v in values {
        ret.push(value_to_dymainc(v));
    }
    ret
}

pub fn map_to_dynamic(map: &JsonMap<String, Value>) -> Map {
    let mut ret: Map = Map::new();
    for (k, v) in map {
        let key = k.to_string();
        ret.insert(key.into(), value_to_dymainc(v));
    }
    ret
}

#[allow(dead_code)]
pub fn value_to_hash_map(value: &Value) -> anyhow::Result<HashMap<String, Vec<String>>> {
    let arr = value.as_object().ok_or(Error::Convert(format!(
        "cannot convert json '{}' to hash_map, it is not object",
        value
    )))?;

    let mut ret = HashMap::new();
    for (k, v) in arr {
        let items = v
            .as_array()
            .ok_or(Error::Convert(format!(
                "cannot convert json to vec, it is not array type in key '{}'",
                k
            )))?
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        ret.insert(k.to_string(), items);
    }
    Ok(ret)
}
