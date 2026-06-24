use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::fmt;
use std::rc::{Rc, Weak};

use indexmap::IndexMap;

pub type Module = IndexMap<String, Value>;

use super::ast::Expr;
use super::env::Frame;
use super::eval::type_of;
use crate::core::error::{eval_error, Result};

thread_local! {
    pub(crate) static LIVE_ALLOC: Cell<usize> = const { Cell::new(0) };
}

pub struct TrackedRc<T: ?Sized>(Rc<T>);

impl<T> TrackedRc<T> {
    pub fn new(val: T) -> Self {
        LIVE_ALLOC.with(|c| c.set(c.get() + 1));
        TrackedRc(Rc::new(val))
    }
}

impl<T: ?Sized> TrackedRc<T> {
    pub fn from_rc(rc: Rc<T>) -> Self {
        LIVE_ALLOC.with(|c| c.set(c.get() + 1));
        TrackedRc(rc)
    }

    pub fn ptr_eq(a: &Self, b: &Self) -> bool {
        Rc::ptr_eq(&a.0, &b.0)
    }
}

impl<T: ?Sized> Clone for TrackedRc<T> {
    fn clone(&self) -> Self {
        TrackedRc(Rc::clone(&self.0))
    }
}

impl<T: ?Sized> Drop for TrackedRc<T> {
    fn drop(&mut self) {
        if Rc::strong_count(&self.0) == 1 {
            LIVE_ALLOC.with(|c| c.set(c.get() - 1));
        }
    }
}

impl<T: ?Sized> std::ops::Deref for TrackedRc<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for TrackedRc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

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
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    List(TrackedRc<RefCell<Vec<Value>>>),
    Dict(TrackedRc<RefCell<IndexMap<String, Value>>>),
    Fn(NativeFn),
    Closure(ClosureValue),
    Object(TrackedRc<dyn ImmutableObject>),
    Module(Rc<Module>),
    Type(Rc<TypeValue>),
}

impl Value {
    pub fn list(items: Vec<Value>) -> Self {
        Value::List(TrackedRc::new(RefCell::new(items)))
    }

    pub fn dict(entries: IndexMap<String, Value>) -> Self {
        Value::Dict(TrackedRc::new(RefCell::new(entries)))
    }

    /// Construct an object value, wrapping `obj` in a fresh shared allocation.
    pub fn object(obj: impl ImmutableObject + 'static) -> Self {
        Value::Object(TrackedRc::from_rc(Rc::new(obj)))
    }

    /// The kind of value this is, as a string. Used in error messages.
    pub fn type_name(&self) -> String {
        type_of(self).name.clone()
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
            Value::List(a) => !a.borrow().is_empty(),
            Value::Dict(o) => !o.borrow().is_empty(),
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
            Value::List(arr) => {
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
            Value::Dict(map) => {
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
        Value::list(v.into_iter().map(Into::into).collect())
    }
}

/// Convert a [`Value`] produced by the evaluator into a caller-defined type.
///
/// Implement this trait on your output type and pass it as `T` to [`crate::eval`].
/// Cycle detection and the LIVE_ALLOC guard are handled by the evaluator — the
/// trait only needs to describe how to fold each `Value` variant into `T`.
pub trait FromValue: Sized {
    type Error;
    fn from_null() -> std::result::Result<Self, Self::Error>;
    fn from_bool(b: bool) -> std::result::Result<Self, Self::Error>;
    fn from_int(n: i64) -> std::result::Result<Self, Self::Error>;
    fn from_float(f: f64) -> std::result::Result<Self, Self::Error>;
    fn from_string(s: String) -> std::result::Result<Self, Self::Error>;
    fn from_list(items: Vec<Self>) -> std::result::Result<Self, Self::Error>;
    fn from_dict(map: IndexMap<String, Self>) -> std::result::Result<Self, Self::Error>;
    /// Called when a cycle is detected in List or Dict.
    fn on_cycle() -> std::result::Result<Self, Self::Error>;
    /// Called for Value variants that cannot be represented (Fn, Closure, Module, Type,
    /// or a non-serializable Object).
    fn on_unconvertible(msg: String) -> std::result::Result<Self, Self::Error>;
}

pub(crate) fn convert_value<T: FromValue>(v: Value) -> std::result::Result<T, T::Error> {
    convert_inner(v, &mut HashSet::new())
}

fn convert_inner<T: FromValue>(
    v: Value,
    seen: &mut HashSet<usize>,
) -> std::result::Result<T, T::Error> {
    match v {
        Value::Null => T::from_null(),
        Value::Bool(b) => T::from_bool(b),
        Value::Int(n) => T::from_int(n),
        Value::Float(f) => T::from_float(f),
        Value::String(s) => T::from_string(s),
        Value::List(arr) => {
            let ptr = &*arr as *const _ as usize;
            if !seen.insert(ptr) {
                return T::on_cycle();
            }
            let items = arr
                .borrow()
                .iter()
                .map(|v| convert_inner(v.clone(), seen))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            seen.remove(&ptr);
            T::from_list(items)
        }
        Value::Dict(map) => {
            let ptr = &*map as *const _ as usize;
            if !seen.insert(ptr) {
                return T::on_cycle();
            }
            let entries = map
                .borrow()
                .iter()
                .map(|(k, v)| convert_inner(v.clone(), seen).map(|v| (k.clone(), v)))
                .collect::<std::result::Result<IndexMap<_, _>, _>>()?;
            seen.remove(&ptr);
            T::from_dict(entries)
        }
        Value::Object(rc) => match rc.serialize() {
            Some(v) => convert_inner(v, seen),
            None => T::on_unconvertible(format!("{} cannot be converted", rc.type_object().name)),
        },
        Value::Fn(_) => T::on_unconvertible("function cannot be converted".into()),
        Value::Closure(_) => T::on_unconvertible("closure cannot be converted".into()),
        Value::Module(_) => T::on_unconvertible("module cannot be converted".into()),
        Value::Type(tv) => T::on_unconvertible(format!("type '{}' cannot be converted", tv.name)),
    }
}
