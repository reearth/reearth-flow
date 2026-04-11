use indexmap::IndexMap;

use crate::core::error::Result;

/// Trait for typed objects that can respond to method calls.
///
/// Implement this to introduce new object types (e.g. `Url`, `DateTime`)
/// that expression users can construct and call methods on.
pub trait ValueObject: std::fmt::Debug + Send + Sync {
    fn type_name(&self) -> &'static str;
    fn call_method(&self, method: &str, args: &[Value]) -> Result<Value>;
    fn clone_box(&self) -> Box<dyn ValueObject>;
    /// Object equality — implementations may compare by content or return false.
    fn eq_box(&self, other: &dyn ValueObject) -> bool;
    /// Human-readable representation. Defaults to `<TypeName>`.
    fn display(&self) -> String {
        format!("<{}>", self.type_name())
    }
}

impl Clone for Box<dyn ValueObject> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl PartialEq for Box<dyn ValueObject> {
    fn eq(&self, other: &Self) -> bool {
        self.eq_box(other.as_ref())
    }
}

/// Runtime value type for the expression evaluator.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<Value>),
    Map(IndexMap<String, Value>),
    /// A typed object that can respond to method calls via [`ValueObject`].
    Object(Box<dyn ValueObject>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(n) => write!(f, "{n}"),
            Value::String(s) => write!(f, "{s:?}"),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            Value::Map(map) => {
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{k:?}: {v}")?;
                }
                write!(f, "}}")
            }
            Value::Object(obj) => write!(f, "{}", obj.display()),
        }
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Int(n)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float(f)
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
