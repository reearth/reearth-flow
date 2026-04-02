use indexmap::IndexMap;

/// Runtime value type for the expression evaluator.
///
/// [`Number`] borrows `serde_json::Number` for precision parity, but the type
/// does not depend on serde_json's container types. A typed `Object` variant
/// for user-defined types will be added when method dispatch is implemented.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<Value>),
    Map(IndexMap<String, Value>),
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Number(n.into())
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        serde_json::Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or(Value::Null)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}
