use std::cell::RefCell;
use std::rc::Rc;

use indexmap::IndexMap;

use crate::core::error::InnerResult;

/// Trait for typed objects that can respond to method calls.
///
/// Methods receive `&self` (immutable) so that a value can be passed as both
/// receiver and argument without triggering a `RefCell` double-borrow panic
/// (e.g. `u == u`).
pub trait ImmutableObject: std::fmt::Debug {
    fn type_name(&self) -> &'static str;
    fn call_method(&self, method: &str, args: &[Value]) -> InnerResult<Value>;
    fn display(&self) -> String {
        format!("<{}>", self.type_name())
    }
    fn serialize(&self) -> Option<Value> {
        None
    }
}

type NativeFnInner = Rc<dyn Fn(&[Value]) -> InnerResult<Value>>;

/// A native (Rust) function callable from the expression language.
#[derive(Clone)]
pub struct NativeFn(pub NativeFnInner);

impl NativeFn {
    pub fn new(f: impl Fn(&[Value]) -> InnerResult<Value> + 'static) -> Self {
        Self(Rc::new(f))
    }

    pub fn call(&self, args: &[Value]) -> InnerResult<Value> {
        (self.0)(args)
    }
}

impl std::fmt::Debug for NativeFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn>")
    }
}

/// Runtime value type for the expression evaluator.
///
/// `Array`, `Map`, and `Object` use `Rc<RefCell<...>>` for reference semantics:
/// cloning a value shares the same backing allocation, so mutations through one
/// alias are visible through all others (Python-style). Circular references are
/// the caller's responsibility and are not detected.
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Rc<RefCell<Vec<Value>>>),
    Map(Rc<RefCell<IndexMap<String, Value>>>),
    /// A native Rust function seeded into the environment.
    Fn(NativeFn),
    /// A typed object that can respond to method calls via [`ImmutableObject`].
    Object(Rc<RefCell<dyn ImmutableObject>>),
}

impl Value {
    /// Construct an array value, wrapping `items` in a fresh shared allocation.
    pub fn array(items: Vec<Value>) -> Self {
        Value::Array(Rc::new(RefCell::new(items)))
    }

    /// Construct a map value, wrapping `entries` in a fresh shared allocation.
    pub fn map(entries: IndexMap<String, Value>) -> Self {
        Value::Map(Rc::new(RefCell::new(entries)))
    }

    /// Construct an object value, wrapping `obj` in a fresh shared allocation.
    pub fn object(obj: impl ImmutableObject + 'static) -> Self {
        Value::Object(Rc::new(RefCell::new(obj)))
    }

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
            Value::Object(rc) => rc.borrow().type_name(),
        }
    }
}

/// Format a float the way Python does: always include a decimal point for finite whole numbers.
pub(crate) fn format_float(n: f64) -> String {
    if n.is_nan() {
        return "nan".to_string();
    }
    if n.is_infinite() {
        return if n > 0.0 {
            "inf".to_string()
        } else {
            "-inf".to_string()
        };
    }
    let s = format!("{n}");
    if s.contains('.') || s.contains('e') || s.contains('E') {
        s
    } else {
        s + ".0"
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(n) => write!(f, "{}", format_float(*n)),
            Value::String(s) => write!(f, "{s:?}"),
            Value::Array(arr) => {
                let arr = arr.borrow();
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
                let map = map.borrow();
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
            Value::Object(rc) => write!(f, "{}", rc.borrow().display()),
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
