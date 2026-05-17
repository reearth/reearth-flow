use std::sync::Arc;

use indexmap::IndexMap;

use crate::core::error::HResult;

/// Trait for typed objects that can respond to method calls.
///
/// Implement this to introduce new object types (e.g. `Url`, `DateTime`)
/// that expression users can construct and call methods on.
pub trait Object: std::fmt::Debug + Send + Sync {
    fn type_name(&self) -> &'static str;
    fn call_method(&self, method: &str, args: &[Value]) -> HResult<Value>;
    fn clone_box(&self) -> Box<dyn Object>;
    /// Object equality — implementations may compare by content or return false.
    fn eq_box(&self, other: &dyn Object) -> bool;
    /// Human-readable representation. Defaults to `<TypeName>`.
    fn display(&self) -> String {
        format!("<{}>", self.type_name())
    }
    /// Serialization hint. If `Some`, used by consumers instead of falling back to `"<TypeName>"`.
    fn serialize(&self) -> Option<Value> {
        None
    }
}

impl Clone for Box<dyn Object> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

impl PartialEq for Box<dyn Object> {
    fn eq(&self, other: &Self) -> bool {
        self.eq_box(other.as_ref())
    }
}

type NativeFnInner = Arc<dyn Fn(&[Value]) -> HResult<Value> + Send + Sync>;

/// A native (Rust) function callable from the expression language.
#[derive(Clone)]
pub struct NativeFn(pub NativeFnInner);

impl NativeFn {
    pub fn new(f: impl Fn(&[Value]) -> HResult<Value> + Send + Sync + 'static) -> Self {
        Self(Arc::new(f))
    }

    pub fn call(&self, args: &[Value]) -> HResult<Value> {
        (self.0)(args)
    }
}

impl std::fmt::Debug for NativeFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn>")
    }
}

impl PartialEq for NativeFn {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
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
    /// A native Rust function seeded into the environment.
    Fn(NativeFn),
    /// A typed object that can respond to method calls via [`Object`].
    Object(Box<dyn Object>),
}

impl Value {
    pub fn type_name(&self) -> &str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Array(_) => "list",
            Value::Map(_) => "map",
            Value::Fn(_) => "function",
            Value::Object(obj) => obj.type_name(),
        }
    }
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
            Value::Fn(_) => write!(f, "<fn>"),
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
