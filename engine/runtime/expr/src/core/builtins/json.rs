use indexmap::IndexMap;
use serde_json::Value as JsonValue;

use crate::core::error::{eval_error, Result};
use crate::core::value::{Module, NativeFn, Value};
use crate::expect_arity;

fn value_to_json(v: &Value) -> Result<JsonValue> {
    match v {
        Value::Null => Ok(JsonValue::Null),
        Value::Bool(b) => Ok(JsonValue::Bool(*b)),
        Value::Int(n) => Ok(JsonValue::Number((*n).into())),
        Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(JsonValue::Number)
            .ok_or_else(|| {
                eval_error(format!(
                    "json.dumps: float value {f} is not JSON-representable (NaN/Inf)"
                ))
            }),
        Value::String(s) => Ok(JsonValue::String(s.clone())),
        Value::List(rc) => {
            let items = rc
                .borrow()
                .iter()
                .map(value_to_json)
                .collect::<Result<Vec<_>>>()?;
            Ok(JsonValue::Array(items))
        }
        Value::Dict(rc) => {
            let obj = rc
                .borrow()
                .iter()
                .map(|(k, v)| value_to_json(v).map(|jv| (k.clone(), jv)))
                .collect::<Result<serde_json::Map<_, _>>>()?;
            Ok(JsonValue::Object(obj))
        }
        Value::Object(rc) => match rc.serialize() {
            Some(serializable) => value_to_json(&serializable),
            None => Err(eval_error(format!(
                "json.dumps: {} does not support serialization",
                rc.type_object().name
            ))),
        },
        other => Err(eval_error(format!(
            "json.dumps: {} is not JSON-serializable",
            other.type_name()
        ))),
    }
}

fn json_to_value(j: JsonValue) -> Result<Value> {
    match j {
        JsonValue::Null => Ok(Value::Null),
        JsonValue::Bool(b) => Ok(Value::Bool(b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Value::Int(i))
            } else {
                n.as_f64()
                    .map(Value::Float)
                    .ok_or_else(|| eval_error(format!("json.loads: number {n} overflows f64")))
            }
        }
        JsonValue::String(s) => Ok(Value::String(s)),
        JsonValue::Array(items) => {
            let values = items
                .into_iter()
                .map(json_to_value)
                .collect::<Result<Vec<_>>>()?;
            Ok(Value::list(values))
        }
        JsonValue::Object(obj) => {
            let map = obj
                .into_iter()
                .map(|(k, v)| json_to_value(v).map(|val| (k, val)))
                .collect::<Result<IndexMap<_, _>>>()?;
            Ok(Value::dict(map))
        }
    }
}

fn json_dumps(args: &[Value]) -> Result<Value> {
    expect_arity("json.dumps", args, 1, 1)?;
    let jv = value_to_json(&args[0])?;
    serde_json::to_string(&jv)
        .map(Value::String)
        .map_err(|e| eval_error(format!("json.dumps: serialization failed: {e}")))
}

fn json_loads(args: &[Value]) -> Result<Value> {
    expect_arity("json.loads", args, 1, 1)?;
    let s = args[0].as_str()?;
    let jv: JsonValue =
        serde_json::from_str(s).map_err(|e| eval_error(format!("json.loads: parse error: {e}")))?;
    json_to_value(jv)
}

pub fn builtin_json() -> Value {
    let mut m = Module::new();
    m.insert("dumps".into(), Value::Fn(NativeFn::new(json_dumps)));
    m.insert("loads".into(), Value::Fn(NativeFn::new(json_loads)));
    Value::module(m)
}

#[cfg(test)]
mod tests {
    use crate::core::test_utils::assert_eval;
    use crate::core::value::Value;

    #[test]
    fn test_json_smoke() {
        assert_eval(r#"json.dumps({"a": 1})"#, &[], Value::from(r#"{"a":1}"#));
        assert_eval(r#"json.loads("{\"a\":1}")["a"]"#, &[], Value::from(1i64));
    }

    #[test]
    fn test_json_loads_overflow() {
        use crate::core::test_utils::try_run;
        assert!(try_run(r#"json.loads("1e10000")"#, &[]).is_err());
    }
}
