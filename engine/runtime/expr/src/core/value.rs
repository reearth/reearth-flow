use std::cell::RefCell;
use std::fmt;
use std::rc::{Rc, Weak};

use indexmap::IndexMap;

pub type Module = IndexMap<String, Value>;

use super::ast::Expr;
use super::env::Frame;
use crate::core::error::{eval_error, Result};

/// A user-defined closure: parameter names, body AST, and the lexical env captured at definition.
#[derive(Clone)]
pub struct ClosureValue {
    pub params: Vec<String>,
    pub body: Rc<Expr>,
    pub captured: Weak<RefCell<Frame>>,
}

impl fmt::Debug for ClosureValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn({})>", self.params.join(", "))
    }
}

/// A first-class type value: a comparable identity and an optional callable constructor.
/// Identity is pointer-based (`Rc::ptr_eq`), not name-based, so two independently registered
/// types with the same name remain distinct.
#[derive(Debug, Clone)]
pub struct TypeValue {
    pub name: String,
    pub constructor: Option<NativeFn>,
}

impl TypeValue {
    pub fn new(name: impl Into<String>, constructor: Option<NativeFn>) -> Self {
        Self {
            name: name.into(),
            constructor,
        }
    }

    pub fn call_ctor(&self, args: &[Value]) -> Result<Value> {
        match &self.constructor {
            Some(f) => f.call(args),
            None => Err(eval_error(format!(
                "type '{}' is not constructible",
                self.name
            ))),
        }
    }
}

/// Trait for typed objects that can respond to method calls.
///
/// All methods take `&self` — objects are immutable from the expression
/// language's perspective. Implementations that need internal state must use
/// their own `RefCell` internally.
pub trait ImmutableObject: std::fmt::Debug {
    /// Return the canonical `Rc<TypeValue>` for this object's type.
    /// Two objects of the same type must return `Rc`s that compare equal via `Rc::ptr_eq`.
    fn type_object(&self) -> Rc<TypeValue>;
    fn call_method(&self, method: &str, args: &[Value]) -> Result<Value>;
    fn get_property(&self, _name: &str) -> Option<Result<Value>> {
        None
    }
    fn display(&self) -> String {
        format!("<{}>", self.type_object().name)
    }
    fn serialize(&self) -> Option<Value> {
        None
    }
}

type NativeFnInner = Rc<dyn Fn(&[Value]) -> Result<Value>>;

/// A native (Rust) function callable from the expression language.
#[derive(Clone)]
pub struct NativeFn(NativeFnInner);

impl NativeFn {
    pub fn new(f: impl Fn(&[Value]) -> Result<Value> + 'static) -> Self {
        Self(Rc::new(f))
    }

    pub fn call(&self, args: &[Value]) -> Result<Value> {
        (self.0)(args)
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl std::fmt::Debug for NativeFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<fn>")
    }
}

/// Runtime value type for the expression evaluator.
///
/// `Array` and `Map` use `Rc<RefCell<...>>` for reference semantics: cloning a
/// value shares the same backing allocation, so mutations through one alias are
/// visible through all others (Python-style). Circular references are the
/// caller's responsibility and are not detected.
///
/// `Object` uses `Rc<dyn ImmutableObject>` without `RefCell` — objects are
/// immutable from the expression language's perspective.
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
    /// A user-defined closure capturing a lexical env frame.
    Closure(ClosureValue),
    /// A typed object that can respond to method calls via [`ImmutableObject`].
    Object(Rc<dyn ImmutableObject>),
    Module(Rc<Module>),
    /// A first-class type value: callable as a constructor, compared by pointer identity.
    Type(Rc<TypeValue>),
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
        Value::Object(Rc::new(obj))
    }

    /// The kind of value this is, as a string. Used in error messages.
    pub fn type_name(&self) -> String {
        match self {
            Value::Null => "null".into(),
            Value::Bool(_) => "bool".into(),
            Value::Int(_) => "int".into(),
            Value::Float(_) => "float".into(),
            Value::String(_) => "str".into(),
            Value::Array(_) => "list".into(),
            Value::Map(_) => "dict".into(),
            Value::Fn(_) | Value::Closure(_) => "function".into(),
            Value::Object(rc) => rc.type_object().name.clone(),
            Value::Module(_) => "module".into(),
            Value::Type(_) => "type".into(),
        }
    }

    pub fn module(m: Module) -> Self {
        Value::Module(Rc::new(m))
    }

    pub fn as_str(&self) -> Result<&str> {
        match self {
            Value::String(s) => Ok(s.as_str()),
            other => Err(eval_error(format!(
                "expected string, got {}",
                other.type_name()
            ))),
        }
    }

    pub fn as_int(&self) -> Result<i64> {
        match self {
            Value::Int(n) => Ok(*n),
            other => Err(eval_error(format!(
                "expected int, got {}",
                other.type_name()
            ))),
        }
    }

    pub fn as_f64(&self) -> Result<f64> {
        match self {
            Value::Float(x) => Ok(*x),
            Value::Int(x) => Ok(*x as f64),
            other => Err(eval_error(format!(
                "expected number, got {}",
                other.type_name()
            ))),
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(f) => *f != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.borrow().is_empty(),
            Value::Map(o) => !o.borrow().is_empty(),
            Value::Fn(_)
            | Value::Closure(_)
            | Value::Object(_)
            | Value::Module(_)
            | Value::Type(_) => true,
        }
    }
}

/// Format a float: decimal for magnitudes in [1e-4, 1e16), shortest scientific otherwise.
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
    let abs = n.abs();
    if abs == 0.0 || (1e-4..1e16).contains(&abs) {
        let s = format!("{n}");
        if s.contains('.') || s.contains('e') || s.contains('E') {
            s
        } else {
            s + ".0"
        }
    } else {
        // the formatting is deliberately not aligned with Python formatting
        // to discourage users from assuming the unguaranteed stability of the string formatting
        format!("{:e}", n)
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
            Value::Closure(c) => write!(f, "<fn({})>", c.params.join(", ")),
            Value::Object(rc) => write!(f, "{}", rc.display()),
            Value::Module(_) => write!(f, "<module>"),
            Value::Type(tv) => write!(f, "{}", tv.name),
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

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(v: Vec<T>) -> Self {
        Value::array(v.into_iter().map(Into::into).collect())
    }
}
