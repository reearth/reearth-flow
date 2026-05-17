use std::cell::Cell;
use std::collections::HashMap;

use indexmap::IndexMap;

use super::ast::{BinOp, Expr, ExprKind, UnaryOp};
use super::builtins::builtin_url;
use super::error::{Error, EvalHelperError, HResult, Result};
use super::value::{NativeFn, Value};
use crate::unpack_args;

#[cfg(debug_assertions)]
const MAX_EVAL_DEPTH: usize = 64;
#[cfg(not(debug_assertions))]
const MAX_EVAL_DEPTH: usize = 1024;

thread_local! {
    static EVAL_DEPTH: Cell<usize> = Cell::new(0);
}

struct DepthGuard;

impl DepthGuard {
    fn enter(pos: usize) -> Result<Self> {
        let depth = EVAL_DEPTH.with(|d| {
            let v = d.get() + 1;
            d.set(v);
            v
        });
        if depth > MAX_EVAL_DEPTH {
            EVAL_DEPTH.with(|d| d.set(d.get() - 1));
            Err(Error::Eval {
                pos,
                msg: format!("expression exceeds maximum evaluation depth ({MAX_EVAL_DEPTH})"),
            })
        } else {
            Ok(DepthGuard)
        }
    }
}

impl Drop for DepthGuard {
    fn drop(&mut self) {
        EVAL_DEPTH.with(|d| d.set(d.get() - 1));
    }
}

trait ToEvalError<T> {
    fn to_eval_error(self, pos: usize) -> Result<T>;
}

impl<T> ToEvalError<T> for HResult<T> {
    fn to_eval_error(self, pos: usize) -> Result<T> {
        self.map_err(|e| Error::Eval { pos, msg: e.msg })
    }
}

pub type Env = HashMap<String, Value>;

pub fn default_env() -> Env {
    let mut env = Env::new();
    env.insert("str".into(), Value::Fn(NativeFn::new(builtin_str)));
    env.insert("int".into(), Value::Fn(NativeFn::new(builtin_int)));
    env.insert("float".into(), Value::Fn(NativeFn::new(builtin_float)));
    env.insert("bool".into(), Value::Fn(NativeFn::new(builtin_bool)));
    env.insert("list".into(), Value::Fn(NativeFn::new(builtin_list)));
    env.insert("map".into(), Value::Fn(NativeFn::new(builtin_map)));
    env.insert("Url".into(), Value::Fn(NativeFn::new(builtin_url)));
    env
}

pub fn eval(expr: &Expr, env: &mut Env) -> Result<Value> {
    eval_inner(expr, env)
}

fn eval_inner(expr: &Expr, env: &mut Env) -> Result<Value> {
    let pos = expr.span.start;
    let _depth = DepthGuard::enter(pos)?;
    match &expr.kind {
        ExprKind::Null => Ok(Value::Null),
        ExprKind::Bool(b) => Ok(Value::Bool(*b)),
        ExprKind::Int(n) => Ok(Value::Int(*n)),
        ExprKind::Float(f) => Ok(Value::Float(*f)),
        ExprKind::Str(s) => Ok(Value::String(s.clone())),
        ExprKind::Array(items) => {
            let mut values = Vec::with_capacity(items.len());
            for e in items {
                values.push(eval_inner(e, env)?);
            }
            Ok(Value::Array(values))
        }
        ExprKind::Map(entries) => {
            let mut map = IndexMap::new();
            for (k, v) in entries {
                let key = eval_inner(k, env)?;
                let key_str = match key {
                    Value::String(s) => s,
                    other => {
                        return Err(Error::Eval {
                            pos,
                            msg: format!("map key must be a string, got {other:?}"),
                        })
                    }
                };
                map.insert(key_str, eval_inner(v, env)?);
            }
            Ok(Value::Map(map))
        }
        ExprKind::Var(name) => env.get(name.as_str()).cloned().ok_or_else(|| Error::Eval {
            pos,
            msg: format!("unknown variable '{name}'"),
        }),
        ExprKind::Index(target, key) => {
            let target = eval_inner(target, env)?;
            let key = eval_inner(key, env)?;
            eval_index(target, key).to_eval_error(pos)
        }
        ExprKind::Slice {
            target,
            start,
            stop,
            step,
        } => {
            let target = eval_inner(target, env)?;
            let start = start.as_deref().map(|e| eval_inner(e, env)).transpose()?;
            let stop = stop.as_deref().map(|e| eval_inner(e, env)).transpose()?;
            let step = step.as_deref().map(|e| eval_inner(e, env)).transpose()?;
            eval_slice(target, start, stop, step).to_eval_error(pos)
        }
        ExprKind::FuncCall { name, args } => {
            let f = env.get(name.as_str()).cloned().ok_or_else(|| Error::Eval {
                pos,
                msg: format!("unknown function '{name}'"),
            })?;
            match f {
                Value::Fn(native_fn) => {
                    let mut evaled = Vec::with_capacity(args.len());
                    for a in args {
                        evaled.push(eval_inner(a, env)?);
                    }
                    native_fn.call(&evaled).to_eval_error(pos)
                }
                _ => Err(Error::Eval {
                    pos,
                    msg: format!("'{name}' is not a function"),
                }),
            }
        }
        ExprKind::Unary(op, e) => {
            let val = eval_inner(e, env)?;
            eval_unary(op, val).to_eval_error(pos)
        }
        ExprKind::MethodCall {
            receiver,
            method,
            args,
        } => {
            let recv = eval_inner(receiver, env)?;
            let mut evaled = Vec::with_capacity(args.len());
            for a in args {
                evaled.push(eval_inner(a, env)?);
            }
            eval_method(recv, method, &evaled).to_eval_error(pos)
        }
        ExprKind::Binary(left, op, right) => {
            match op {
                BinOp::And => {
                    let l = eval_inner(left, env)?;
                    if !is_truthy(&l) {
                        return Ok(Value::Bool(false));
                    }
                    let r = eval_inner(right, env)?;
                    return Ok(Value::Bool(is_truthy(&r)));
                }
                BinOp::Or => {
                    let l = eval_inner(left, env)?;
                    if is_truthy(&l) {
                        return Ok(Value::Bool(true));
                    }
                    let r = eval_inner(right, env)?;
                    return Ok(Value::Bool(is_truthy(&r)));
                }
                _ => {}
            }
            let left = eval_inner(left, env)?;
            let right = eval_inner(right, env)?;
            eval_binary(op, left, right).to_eval_error(pos)
        }
        ExprKind::Assign { lvalue, value } => {
            let v = eval_inner(value, env)?;
            eval_assign_lvalue(lvalue, v.clone(), env)?;
            Ok(v)
        }
        ExprKind::Block(exprs) => {
            let mut result = Value::Null;
            for e in exprs {
                result = eval_inner(e, env)?;
            }
            Ok(result)
        }
        ExprKind::If { cond, then, else_ } => {
            let c = eval_inner(cond, env)?;
            if is_truthy(&c) {
                eval_inner(then, env)
            } else {
                eval_inner(else_, env)
            }
        }
    }
}

/// Assign `value` to the lvalue expression, recursing for nested index chains.
///
/// For `x[i][j] = v` the algorithm walks the chain outward: it reads the current
/// container, mutates the element, then recurses to write the modified container
/// one level up until it reaches a bare `Var` and does `env.insert`.
/// Returns `()` — the caller is responsible for returning the assigned value.
fn eval_assign_lvalue(lvalue: &Expr, value: Value, env: &mut Env) -> Result<()> {
    let pos = lvalue.span.start;
    match &lvalue.kind {
        ExprKind::Var(name) => {
            env.insert(name.clone(), value);
            Ok(())
        }
        ExprKind::Index(target, key) => {
            let key_val = eval_inner(key, env)?;
            let mut container = eval_inner(target, env)?;
            match (&mut container, &key_val) {
                (Value::Array(arr), Value::Int(i)) => {
                    let len = arr.len() as i64;
                    let idx = if *i < 0 { len + i } else { *i };
                    if idx < 0 || idx as usize >= arr.len() {
                        return Err(Error::Eval {
                            pos,
                            msg: format!("array index {} out of range (len {})", i, arr.len()),
                        });
                    }
                    arr[idx as usize] = value;
                }
                (Value::Map(map), Value::String(k)) => {
                    map.insert(k.clone(), value);
                }
                (c, k) => {
                    return Err(Error::Eval {
                        pos,
                        msg: format!("cannot index-assign {c:?} with {k:?}"),
                    })
                }
            }
            eval_assign_lvalue(target, container, env)
        }
        _ => Err(Error::Eval {
            pos,
            msg: "invalid assignment target".into(),
        }),
    }
}

fn eval_method(recv: Value, method: &str, args: &[Value]) -> HResult<Value> {
    match recv {
        Value::String(s) => eval_string_method(s, method, args),
        Value::Array(a) => eval_array_method(a, method, args),
        Value::Object(obj) => obj.call_method(method, args),
        v => Err(EvalHelperError::new(format!(
            "{v:?} has no method '{method}'"
        ))),
    }
}

fn eval_string_method(s: String, method: &str, args: &[Value]) -> HResult<Value> {
    match method {
        "len" => {
            unpack_args!(args =>);
            Ok(Value::Int(s.chars().count() as i64))
        }
        "trim" => {
            unpack_args!(args =>);
            Ok(Value::String(s.trim().to_string()))
        }
        "split" => {
            unpack_args!(args => sep);
            let Value::String(sep) = sep else {
                return Err(EvalHelperError::new(format!(
                    "split() separator must be a string, got {sep:?}"
                )));
            };
            Ok(Value::Array(
                s.split(sep.as_str())
                    .map(|p| Value::String(p.to_string()))
                    .collect(),
            ))
        }
        "starts_with" => {
            unpack_args!(args => prefix);
            let Value::String(prefix) = prefix else {
                return Err(EvalHelperError::new(format!(
                    "starts_with() argument must be a string, got {prefix:?}"
                )));
            };
            Ok(Value::Bool(s.starts_with(prefix.as_str())))
        }
        "ends_with" => {
            unpack_args!(args => suffix);
            let Value::String(suffix) = suffix else {
                return Err(EvalHelperError::new(format!(
                    "ends_with() argument must be a string, got {suffix:?}"
                )));
            };
            Ok(Value::Bool(s.ends_with(suffix.as_str())))
        }
        "replace" => {
            unpack_args!(args => from, to);
            let (Value::String(from), Value::String(to)) = (from, to) else {
                return Err(EvalHelperError::new(
                    "replace() requires two string arguments: replace(from, to)",
                ));
            };
            Ok(Value::String(s.replace(from.as_str(), to.as_str())))
        }
        m => Err(EvalHelperError::new(format!("String has no method '{m}'"))),
    }
}

fn eval_array_method(a: Vec<Value>, method: &str, args: &[Value]) -> HResult<Value> {
    match method {
        "len" => {
            unpack_args!(args =>);
            Ok(Value::Int(a.len() as i64))
        }
        m => Err(EvalHelperError::new(format!("Array has no method '{m}'"))),
    }
}

fn eval_index(target: Value, key: Value) -> HResult<Value> {
    match (target, key) {
        (Value::Map(map), Value::String(k)) => Ok(map.get(&k).cloned().unwrap_or(Value::Null)),
        (Value::Array(arr), Value::Int(i)) => {
            let i = if i < 0 { arr.len() as i64 + i } else { i };
            Ok(arr.get(i as usize).cloned().unwrap_or(Value::Null))
        }
        (Value::String(s), Value::Int(i)) => {
            let chars: Vec<char> = s.chars().collect();
            let len = chars.len() as i64;
            let i = if i < 0 { len + i } else { i };
            if i < 0 || i >= len {
                return Ok(Value::Null);
            }
            Ok(Value::String(chars[i as usize].to_string()))
        }
        (Value::Null, _) => Ok(Value::Null),
        (target, key) => Err(EvalHelperError::new(format!(
            "cannot index {target:?} with {key:?}"
        ))),
    }
}

fn as_slice_index(v: Value, what: &str) -> HResult<i64> {
    match v {
        Value::Int(n) => Ok(n),
        v => Err(EvalHelperError::new(format!(
            "slice {what} must be an integer, got {v:?}"
        ))),
    }
}

/// Returns the concrete element indices for `target[start:stop:step]`.
/// Follows standard slice semantics: negative indices count from the end,
/// defaults and clamping depend on the sign of `step`.
fn slice_indices(len: usize, start: Option<i64>, stop: Option<i64>, step: i64) -> Vec<usize> {
    let n = len as i64;
    let normalize = |i: i64, clamp_lo: i64, clamp_hi: i64| -> i64 {
        let i = if i < 0 { i + n } else { i };
        i.clamp(clamp_lo, clamp_hi)
    };
    let (start, stop) = if step > 0 {
        let s = start.map(|i| normalize(i, 0, n)).unwrap_or(0);
        let e = stop.map(|i| normalize(i, 0, n)).unwrap_or(n);
        (s, e)
    } else {
        let s = start.map(|i| normalize(i, -1, n - 1)).unwrap_or(n - 1);
        let e = stop.map(|i| normalize(i, -1, n - 1)).unwrap_or(-1);
        (s, e)
    };
    let mut indices = Vec::new();
    let mut i = start;
    if step > 0 {
        while i < stop {
            indices.push(i as usize);
            i = i.saturating_add(step);
        }
    } else {
        while i > stop {
            indices.push(i as usize);
            i = i.saturating_add(step);
        }
    }
    indices
}

fn eval_slice(
    target: Value,
    start: Option<Value>,
    stop: Option<Value>,
    step: Option<Value>,
) -> HResult<Value> {
    let step = match step {
        None => 1i64,
        Some(v) => as_slice_index(v, "step")?,
    };
    if step == 0 {
        return Err(EvalHelperError::new("slice step cannot be zero"));
    }
    let start = start.map(|v| as_slice_index(v, "start")).transpose()?;
    let stop = stop.map(|v| as_slice_index(v, "stop")).transpose()?;

    match target {
        Value::Array(arr) => {
            let indices = slice_indices(arr.len(), start, stop, step);
            Ok(Value::Array(
                indices.into_iter().map(|i| arr[i].clone()).collect(),
            ))
        }
        Value::String(s) => {
            let chars: Vec<char> = s.chars().collect();
            let indices = slice_indices(chars.len(), start, stop, step);
            Ok(Value::String(
                indices.into_iter().map(|i| chars[i]).collect(),
            ))
        }
        v => Err(EvalHelperError::new(format!("cannot slice {v:?}"))),
    }
}

fn eval_unary(op: &UnaryOp, val: Value) -> HResult<Value> {
    match op {
        UnaryOp::Not => Ok(Value::Bool(!is_truthy(&val))),
        UnaryOp::Neg => match val {
            Value::Int(n) => n
                .checked_neg()
                .map(Value::Int)
                .ok_or_else(|| EvalHelperError::new("integer overflow")),
            Value::Float(f) => Ok(Value::Float(-f)),
            v => Err(EvalHelperError::new(format!("cannot negate {v:?}"))),
        },
    }
}

fn binop_dunder(op: &BinOp) -> Option<&'static str> {
    match op {
        BinOp::Add => Some("__add__"),
        BinOp::Sub => Some("__sub__"),
        BinOp::Mul => Some("__mul__"),
        BinOp::Div => Some("__div__"),
        BinOp::Eq => Some("__eq__"),
        BinOp::Ne => None, // derived as !__eq__ in eval_binary
        BinOp::Lt => Some("__lt__"),
        BinOp::Le => Some("__le__"),
        BinOp::Gt => Some("__gt__"),
        BinOp::Ge => Some("__ge__"),
        BinOp::And | BinOp::Or | BinOp::In | BinOp::NotIn => None,
    }
}

fn try_object_op(op: &BinOp, left: Value, right: Value) -> HResult<Value> {
    let dunder = binop_dunder(op)
        .ok_or_else(|| EvalHelperError::new(format!("operator not overloadable for {left:?}")))?;
    if let Value::Object(ref obj) = left {
        return obj.call_method(dunder, &[right]);
    }
    Err(EvalHelperError::new(format!(
        "operator '{dunder}' not supported between {left:?} and {right:?}"
    )))
}

enum Numeric {
    Int(i64),
    Float(f64),
}

fn coerce_numeric(a: &Value, b: &Value) -> HResult<(Numeric, Numeric)> {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => Ok((Numeric::Int(*a), Numeric::Int(*b))),
        (Value::Int(a), Value::Float(b)) => Ok((Numeric::Float(*a as f64), Numeric::Float(*b))),
        (Value::Float(a), Value::Int(b)) => Ok((Numeric::Float(*a), Numeric::Float(*b as f64))),
        (Value::Float(a), Value::Float(b)) => Ok((Numeric::Float(*a), Numeric::Float(*b))),
        (a, b) => Err(EvalHelperError::new(format!(
            "cannot apply numeric op to {a:?} and {b:?}"
        ))),
    }
}

fn int_overflow() -> EvalHelperError {
    EvalHelperError::new("integer overflow")
}

fn numeric_op(
    left: Value,
    right: Value,
    int_op: impl Fn(i64, i64) -> HResult<i64>,
    float_op: impl Fn(f64, f64) -> f64,
) -> HResult<Value> {
    match coerce_numeric(&left, &right)? {
        (Numeric::Int(a), Numeric::Int(b)) => Ok(Value::Int(int_op(a, b)?)),
        (Numeric::Float(a), Numeric::Float(b)) => Ok(Value::Float(float_op(a, b))),
        _ => unreachable!(),
    }
}

fn eval_binary(op: &BinOp, left: Value, right: Value) -> HResult<Value> {
    if matches!(left, Value::Object(_)) {
        if op == &BinOp::Ne {
            let eq = try_object_op(&BinOp::Eq, left, right)?;
            return match eq {
                Value::Bool(b) => Ok(Value::Bool(!b)),
                _ => Err(EvalHelperError::new("__eq__ must return a bool")),
            };
        }
        if binop_dunder(op).is_some() {
            return try_object_op(op, left, right);
        }
    }
    match op {
        BinOp::Add => match (left, right) {
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + b.as_str())),
            (Value::String(a), b) => Ok(Value::String(a + value_to_string(&b).as_str())),
            (Value::Array(mut a), Value::Array(b)) => {
                a.extend(b);
                Ok(Value::Array(a))
            }
            (a, b) => match coerce_numeric(&a, &b) {
                Ok((Numeric::Int(a), Numeric::Int(b))) => {
                    a.checked_add(b).map(Value::Int).ok_or_else(int_overflow)
                }
                Ok((Numeric::Float(a), Numeric::Float(b))) => Ok(Value::Float(a + b)),
                Ok(_) => unreachable!(),
                Err(_) => Err(EvalHelperError::new("'+' not supported for these types")),
            },
        },
        BinOp::Sub => numeric_op(
            left,
            right,
            |a, b| a.checked_sub(b).ok_or_else(int_overflow),
            |a, b| a - b,
        ),
        BinOp::Mul => numeric_op(
            left,
            right,
            |a, b| a.checked_mul(b).ok_or_else(int_overflow),
            |a, b| a * b,
        ),
        BinOp::Div => match coerce_numeric(&left, &right) {
            Ok((Numeric::Int(a), Numeric::Int(b))) => {
                if b == 0 {
                    return Err(EvalHelperError::new("division by zero"));
                }
                if b == -1 {
                    return a.checked_neg().map(Value::Int).ok_or_else(int_overflow);
                }
                if a % b == 0 {
                    Ok(Value::Int(a / b))
                } else {
                    Ok(Value::Float(a as f64 / b as f64))
                }
            }
            Ok((Numeric::Float(a), Numeric::Float(b))) => {
                if b == 0.0 {
                    return Err(EvalHelperError::new("division by zero"));
                }
                Ok(Value::Float(a / b))
            }
            Ok(_) => unreachable!(),
            Err(_) => Err(EvalHelperError::new("'/' not supported for these types")),
        },
        BinOp::Eq => Ok(Value::Bool(values_equal(&left, &right))),
        BinOp::Ne => Ok(Value::Bool(!values_equal(&left, &right))),
        BinOp::Lt => compare_values(left, right, |o| o == std::cmp::Ordering::Less),
        BinOp::Le => compare_values(left, right, |o| o != std::cmp::Ordering::Greater),
        BinOp::Gt => compare_values(left, right, |o| o == std::cmp::Ordering::Greater),
        BinOp::Ge => compare_values(left, right, |o| o != std::cmp::Ordering::Less),
        BinOp::In => match (left, right) {
            (left, Value::Array(arr)) => {
                Ok(Value::Bool(arr.iter().any(|v| values_equal(v, &left))))
            }
            (Value::String(s), Value::String(haystack)) => {
                Ok(Value::Bool(haystack.contains(s.as_str())))
            }
            (Value::String(key), Value::Map(map)) => Ok(Value::Bool(map.contains_key(&key))),
            (_, Value::Null) => Ok(Value::Bool(false)),
            (l, r) => Err(EvalHelperError::new(format!(
                "'in' not supported between {l:?} and {r:?}"
            ))),
        },
        BinOp::NotIn => match (left, right) {
            (left, Value::Array(arr)) => {
                Ok(Value::Bool(!arr.iter().any(|v| values_equal(v, &left))))
            }
            (Value::String(s), Value::String(haystack)) => {
                Ok(Value::Bool(!haystack.contains(s.as_str())))
            }
            (Value::String(key), Value::Map(map)) => Ok(Value::Bool(!map.contains_key(&key))),
            (_, Value::Null) => Ok(Value::Bool(true)),
            (l, r) => Err(EvalHelperError::new(format!(
                "'not in' not supported between {l:?} and {r:?}"
            ))),
        },
        BinOp::And | BinOp::Or => {
            unreachable!("short-circuited in eval_inner before eval_binary is called")
        }
    }
}

fn compare_values(
    left: Value,
    right: Value,
    pred: impl Fn(std::cmp::Ordering) -> bool,
) -> HResult<Value> {
    let ord = match coerce_numeric(&left, &right) {
        Ok((Numeric::Int(a), Numeric::Int(b))) => a.cmp(&b),
        Ok((Numeric::Float(a), Numeric::Float(b))) => match a.partial_cmp(&b) {
            Some(ord) => ord,
            None => return Ok(Value::Bool(false)),
        },
        Ok(_) => unreachable!(),
        Err(_) => match (&left, &right) {
            (Value::String(a), Value::String(b)) => a.as_str().cmp(b.as_str()),
            _ => {
                return Err(EvalHelperError::new(format!(
                    "cannot compare {left:?} and {right:?}"
                )))
            }
        },
    };
    Ok(Value::Bool(pred(ord)))
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match coerce_numeric(a, b) {
        Ok((Numeric::Int(x), Numeric::Int(y))) => x == y,
        Ok((Numeric::Float(x), Numeric::Float(y))) => x == y,
        _ => a == b,
    }
}

fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Int(n) => *n != 0,
        Value::Float(f) => *f != 0.0,
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Map(o) => !o.is_empty(),
        Value::Fn(_) | Value::Object(_) => true,
    }
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Int(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Array(_) | Value::Map(_) => format!("{v:?}"),
        Value::Fn(_) => "<fn>".into(),
        Value::Object(obj) => obj.display(),
    }
}

fn builtin_str(args: &[Value]) -> HResult<Value> {
    match args.first() {
        None | Some(Value::Null) => Ok(Value::String("null".into())),
        Some(Value::String(s)) => Ok(Value::String(s.clone())),
        Some(Value::Bool(b)) => Ok(Value::String(b.to_string())),
        Some(Value::Int(n)) => Ok(Value::String(n.to_string())),
        Some(Value::Float(f)) => Ok(Value::String(f.to_string())),
        Some(Value::Object(obj)) => obj.call_method("__str__", &[]),
        Some(v) => Err(EvalHelperError::new(format!(
            "str() not supported for {v:?}"
        ))),
    }
}

fn builtin_int(args: &[Value]) -> HResult<Value> {
    match args.first() {
        Some(Value::Int(n)) => Ok(Value::Int(*n)),
        Some(Value::Float(f)) => {
            let t = f.trunc();
            if !t.is_finite() || t < i64::MIN as f64 || t >= -(i64::MIN as f64) {
                Err(EvalHelperError::new(format!(
                    "int() value out of range: {f}"
                )))
            } else {
                Ok(Value::Int(t as i64))
            }
        }
        Some(Value::Bool(b)) => Ok(Value::Int(*b as i64)),
        Some(Value::String(s)) => s
            .trim()
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|_| EvalHelperError::new(format!("int() cannot parse {s:?}"))),
        Some(v) => Err(EvalHelperError::new(format!(
            "int() not supported for {v:?}"
        ))),
        None => Err(EvalHelperError::new("int() requires an argument")),
    }
}

fn builtin_float(args: &[Value]) -> HResult<Value> {
    match args.first() {
        Some(Value::Float(f)) => Ok(Value::Float(*f)),
        Some(Value::Int(n)) => Ok(Value::Float(*n as f64)),
        Some(Value::Bool(b)) => Ok(Value::Float(if *b { 1.0 } else { 0.0 })),
        Some(Value::String(s)) => s
            .trim()
            .parse::<f64>()
            .map(Value::Float)
            .map_err(|_| EvalHelperError::new(format!("float() cannot parse {s:?}"))),
        Some(v) => Err(EvalHelperError::new(format!(
            "float() not supported for {v:?}"
        ))),
        None => Err(EvalHelperError::new("float() requires an argument")),
    }
}

fn builtin_bool(args: &[Value]) -> HResult<Value> {
    Ok(Value::Bool(args.first().map(is_truthy).unwrap_or(false)))
}

fn builtin_list(args: &[Value]) -> HResult<Value> {
    match args.first() {
        Some(Value::Array(a)) => Ok(Value::Array(a.clone())),
        Some(Value::String(s)) => Ok(Value::Array(
            s.chars().map(|c| Value::String(c.to_string())).collect(),
        )),
        Some(Value::Map(m)) => Ok(Value::Array(
            m.keys().map(|k| Value::String(k.clone())).collect(),
        )),
        Some(v) => Err(EvalHelperError::new(format!(
            "list() not supported for {v:?}"
        ))),
        None => Ok(Value::Array(vec![])),
    }
}

fn builtin_map(args: &[Value]) -> HResult<Value> {
    let pairs = match args.first() {
        Some(Value::Array(a)) => a,
        _ => {
            return Err(EvalHelperError::new(
                "map() expects an array of [key, value] pairs",
            ))
        }
    };
    let mut out = IndexMap::new();
    for (i, pair) in pairs.iter().enumerate() {
        match pair {
            Value::Array(kv) if kv.len() == 2 => {
                let key = match &kv[0] {
                    Value::String(s) => s.clone(),
                    v => {
                        return Err(EvalHelperError::new(format!(
                            "map() key at index {i} must be a string, got {v:?}"
                        )))
                    }
                };
                out.insert(key, kv[1].clone());
            }
            _ => {
                return Err(EvalHelperError::new(format!(
                    "map() entry at index {i} must be a 2-element array"
                )))
            }
        }
    }
    Ok(Value::Map(out))
}

#[cfg(test)]
mod tests {
    use super::super::parser::parse;
    use super::*;

    fn run(input: &str, vars: &[(&str, Value)]) -> Value {
        try_run(input, vars).unwrap()
    }

    fn try_run(input: &str, vars: &[(&str, Value)]) -> Result<Value> {
        let mut env = default_env();
        for (k, v) in vars {
            env.insert(k.to_string(), v.clone());
        }
        eval_inner(&parse(input).unwrap(), &mut env)
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(run("1 + 2", &[]), Value::from(3i64));
        assert_eq!(run("10 - 3", &[]), Value::from(7i64));
        assert_eq!(run("2 * 5", &[]), Value::from(10i64));
        assert_eq!(run("10 / 4", &[]), Value::from(2.5f64));
        // string concatenation via +
        assert_eq!(
            run(r#""hello" + "_" + "world""#, &[]),
            Value::from("hello_world")
        );
        // array concatenation via +
        let a = Value::Array(vec![Value::from(1i64), Value::from(2i64)]);
        let b = Value::Array(vec![Value::from(3i64), Value::from(4i64)]);
        assert_eq!(
            run("a + b", &[("a", a), ("b", b)]),
            Value::Array(vec![
                Value::from(1i64),
                Value::from(2i64),
                Value::from(3i64),
                Value::from(4i64)
            ])
        );
    }

    #[test]
    fn test_integer_overflow() {
        let max = Value::Int(i64::MAX);
        let min = Value::Int(i64::MIN);

        // overflow → error
        assert!(try_run("a + b", &[("a", max.clone()), ("b", Value::Int(1))]).is_err());
        assert!(try_run("a - b", &[("a", min.clone()), ("b", Value::Int(1))]).is_err());
        assert!(try_run("a * b", &[("a", max.clone()), ("b", Value::Int(2))]).is_err());
        assert!(try_run("-a", &[("a", min.clone())]).is_err());
        // i64::MIN / -1 overflows
        assert!(try_run("a / b", &[("a", min.clone()), ("b", Value::Int(-1))]).is_err());

        // non-overflowing ops still produce Int
        assert_eq!(
            try_run("a + b", &[("a", max.clone()), ("b", Value::Int(0))]).unwrap(),
            max
        );
        assert_eq!(
            try_run("a / b", &[("a", Value::Int(6)), ("b", Value::Int(-1))]).unwrap(),
            Value::Int(-6)
        );
        assert_eq!(
            try_run("-a", &[("a", Value::Int(5))]).unwrap(),
            Value::Int(-5)
        );

        // int ops that would overflow promote to float via mixed arithmetic
        assert_eq!(
            try_run("a + b", &[("a", max.clone()), ("b", Value::Float(1.0))]).unwrap(),
            Value::Float(i64::MAX as f64 + 1.0)
        );
    }

    #[test]
    fn test_comparison() {
        assert_eq!(run("1 == 1", &[]), Value::from(true));
        assert_eq!(run("1 != 2", &[]), Value::from(true));
        assert_eq!(run("2 > 1", &[]), Value::from(true));
        assert_eq!(run("1 >= 1", &[]), Value::from(true));
    }

    #[test]
    fn test_logical() {
        assert_eq!(run("true and false", &[]), Value::from(false));
        assert_eq!(run("true or false", &[]), Value::from(true));
        assert_eq!(run("not true", &[]), Value::from(false));
    }

    #[test]
    fn test_index() {
        let feature = Value::Map(indexmap::indexmap! {
            "package".into() => Value::from("bldg"),
            "cityGmlPath".into() => Value::from("/data/city.gml"),
        });
        assert_eq!(
            run(r#"feature["package"]"#, &[("feature", feature.clone())]),
            Value::from("bldg")
        );
        assert_eq!(
            run(r#"feature["cityGmlPath"]"#, &[("feature", feature.clone())]),
            Value::from("/data/city.gml")
        );
        assert_eq!(
            run(r#"feature["missing"]"#, &[("feature", feature)]),
            Value::Null
        );
        let arr = Value::Array(vec![
            Value::from(1i64),
            Value::from(2i64),
            Value::from(3i64),
        ]);
        assert_eq!(run("arr[0]", &[("arr", arr.clone())]), Value::from(1i64));
        assert_eq!(run("arr[-1]", &[("arr", arr)]), Value::from(3i64));
        assert_eq!(run(r#""hello"[0]"#, &[]), Value::from("h"));
        assert_eq!(run(r#""hello"[4]"#, &[]), Value::from("o"));
        assert_eq!(run(r#""hello"[-1]"#, &[]), Value::from("o"));
    }

    #[test]
    fn test_slice() {
        assert_eq!(run(r#""abcde"[1:3]"#, &[]), Value::from("bc"));
        assert_eq!(run(r#""abcde"[:3]"#, &[]), Value::from("abc"));
        assert_eq!(run(r#""abcde"[2:]"#, &[]), Value::from("cde"));
        assert_eq!(run(r#""abcde"[:]"#, &[]), Value::from("abcde"));
        assert_eq!(run(r#""abcde"[::-1]"#, &[]), Value::from("edcba"));
        assert_eq!(run(r#""abcde"[-1::-2]"#, &[]), Value::from("eca"));
        assert_eq!(run(r#""abcde"[::2]"#, &[]), Value::from("ace"));
        let arr = Value::Array((0i64..5).map(Value::from).collect());
        assert_eq!(
            run("arr[1:3]", &[("arr", arr.clone())]),
            Value::Array(vec![Value::from(1i64), Value::from(2i64)])
        );
        assert_eq!(
            run("arr[-2:]", &[("arr", arr.clone())]),
            Value::Array(vec![Value::from(3i64), Value::from(4i64)])
        );
        assert_eq!(
            run("arr[::-1]", &[("arr", arr.clone())]),
            Value::Array((0i64..5).rev().map(Value::from).collect())
        );
        assert!(try_run("arr[s:]", &[("arr", arr.clone()), ("s", Value::Float(1.0))]).is_err());
        assert!(try_run("arr[:s]", &[("arr", arr.clone()), ("s", Value::Float(3.0))]).is_err());
        assert!(try_run("arr[::s]", &[("arr", arr), ("s", Value::Float(2.0))]).is_err());
    }

    #[test]
    fn test_method_call() {
        assert_eq!(run(r#""  hello  ".trim()"#, &[]), Value::from("hello"));
        assert_eq!(run(r#""hello".len()"#, &[]), Value::from(5i64));
        assert_eq!(run(r#""".len()"#, &[]), Value::from(0i64));
        let arr = Value::Array(vec![
            Value::from(1i64),
            Value::from(2i64),
            Value::from(3i64),
        ]);
        assert_eq!(run("arr.len()", &[("arr", arr)]), Value::from(3i64));
    }

    #[test]
    fn test_string_split() {
        assert_eq!(
            run(r#""bldg:Building".split(":")[0]"#, &[]),
            Value::from("bldg")
        );
        assert_eq!(
            run(r#""bldg:Building".split(":")[-1]"#, &[]),
            Value::from("Building")
        );
        assert_eq!(
            run(r#""hello".split(":")"#, &[]),
            Value::Array(vec![Value::from("hello")])
        );
        assert_eq!(
            run(r#""a::b".split(":")"#, &[]),
            Value::Array(vec![Value::from("a"), Value::from(""), Value::from("b")])
        );
    }

    #[test]
    fn test_in_operator() {
        let pkgs = Value::Array(vec![Value::from("bldg"), Value::from("tran")]);
        assert_eq!(
            run(r#""bldg" in pkgs"#, &[("pkgs", pkgs.clone())]),
            Value::from(true)
        );
        assert_eq!(
            run(r#""fld" in pkgs"#, &[("pkgs", pkgs.clone())]),
            Value::from(false)
        );
        assert_eq!(
            run(r#""bldg" not in pkgs"#, &[("pkgs", pkgs.clone())]),
            Value::from(false)
        );
        assert_eq!(
            run(r#""fld" not in pkgs"#, &[("pkgs", pkgs)]),
            Value::from(true)
        );
        assert_eq!(run(r#""world" in "hello world""#, &[]), Value::from(true));
        assert_eq!(run(r#""xyz" in "hello world""#, &[]), Value::from(false));
        assert_eq!(run(r#""xyz" not in "hello world""#, &[]), Value::from(true));
        assert_eq!(run(r#""" in "hello""#, &[]), Value::from(true));
        let m = Value::Map(indexmap::indexmap! {
            "a".into() => Value::from(1i64),
            "b".into() => Value::from(2i64),
        });
        assert_eq!(run(r#""a" in m"#, &[("m", m.clone())]), Value::from(true));
        assert_eq!(run(r#""c" in m"#, &[("m", m.clone())]), Value::from(false));
        assert_eq!(run(r#""a" not in m"#, &[("m", m)]), Value::from(false));
        assert_eq!(run(r#""x" in null"#, &[]), Value::from(false));
        assert_eq!(run(r#""x" not in null"#, &[]), Value::from(true));
        // not applies to the whole comparison: `not ("a" in pkgs)`
        let pkgs2 = Value::Array(vec![Value::from("a")]);
        assert_eq!(
            run(r#"not "a" in pkgs"#, &[("pkgs", pkgs2)]),
            Value::from(false)
        );
    }

    #[test]
    fn test_string_starts_ends_with() {
        assert_eq!(
            run(r#""bldg_lod1".starts_with("tran")"#, &[]),
            Value::from(false)
        );
        assert_eq!(
            run(r#""bldg_lod1".ends_with("lod1")"#, &[]),
            Value::from(true)
        );
        assert_eq!(
            run(
                r#"s = "city_bldg"; sfx = "_bldg"; if s.ends_with(sfx) { s[:s.len() - sfx.len()] } else { s }"#,
                &[]
            ),
            Value::from("city")
        );
    }

    #[test]
    fn test_string_replace() {
        assert_eq!(
            run(r#""a/b/c".replace("/", "_")"#, &[]),
            Value::from("a_b_c")
        );
        assert_eq!(
            run(r#""foo_op_bar_op_baz".replace("_op_", "/")"#, &[]),
            Value::from("foo/bar/baz")
        );
        assert_eq!(
            run(r#""hello".replace("x", "y")"#, &[]),
            Value::from("hello")
        );
    }

    #[test]
    fn test_assign() {
        assert_eq!(run("x = 42; x", &[]), Value::from(42i64));
        assert_eq!(run("x = 3; x * x", &[]), Value::from(9i64));
        assert_eq!(run("x = 2; y = x + 1; x * y", &[]), Value::from(6i64));
        // reassignment overwrites
        assert_eq!(run("x = 1; x = 99; x", &[]), Value::from(99i64));
        // assign shadows outer env binding
        assert_eq!(
            run("x = 7; x", &[("x", Value::from(999i64))]),
            Value::from(7i64)
        );
        // assign returns the value
        assert_eq!(run("(x = 10) * 2", &[]), Value::from(20i64));
        // function-scoped: inner block mutates outer x
        assert_eq!(run("x = 10; { x = 99 }; x", &[]), Value::from(99i64));
    }

    #[test]
    fn test_block() {
        assert_eq!(run("{ 1; 2; 3 }", &[]), Value::from(3i64));
        assert_eq!(run("{ 42; }", &[]), Value::Null);
        assert_eq!(run("{}", &[]), Value::Null);
        assert_eq!(run("{ x = 5; x * 2 }", &[]), Value::from(10i64));
        assert_eq!(
            run("{ a = 3; b = 4; a * a + b * b }", &[]),
            Value::from(25i64)
        );
        assert_eq!(run("{ x = 1; }", &[]), Value::Null);
        // function-scoped: x leaks out of inner block
        assert_eq!(run("{ x = 3 } + { y = 4 }", &[]), Value::from(7i64));
        assert_eq!(run("{ x = 1; { y = 2; x + y } }", &[]), Value::from(3i64));
    }

    #[test]
    fn test_if() {
        assert_eq!(run("if true { 1 } else { 2 }", &[]), Value::from(1i64));
        assert_eq!(run("if false { 1 } else { 2 }", &[]), Value::from(2i64));
        assert_eq!(run("if 1 == 1 { 42 } else { 0 }", &[]), Value::from(42i64));
        assert_eq!(run("if 1 == 2 { 42 } else { 0 }", &[]), Value::from(0i64));
        assert_eq!(run("if null { 1 } else { 2 }", &[]), Value::from(2i64));
        assert_eq!(
            run("if false { 1 } else if false { 2 } else { 3 }", &[]),
            Value::from(3i64)
        );
        assert_eq!(
            run("if false { 1 } else if true { 2 } else { 3 }", &[]),
            Value::from(2i64)
        );
        assert_eq!(
            run(
                "(if true { 10 } else { 0 }) + (if false { 0 } else { 5 })",
                &[]
            ),
            Value::from(15i64)
        );
        assert_eq!(
            run("if true { x = 7; x * 2 } else { 0 }", &[]),
            Value::from(14i64)
        );
        assert_eq!(run("if true { 42 }", &[]), Value::from(42i64));
        assert_eq!(run("if false { 42 }", &[]), Value::Null);
    }

    #[test]
    fn test_map() {
        assert_eq!(
            run(r#"map([["a", 1], ["b", 2]])"#, &[]),
            Value::Map(indexmap::indexmap! {
                "a".into() => Value::from(1i64),
                "b".into() => Value::from(2i64),
            })
        );
        assert_eq!(
            run(r#"{"a": 1, "b": 2}"#, &[]),
            Value::Map(indexmap::indexmap! {
                "a".into() => Value::from(1i64),
                "b".into() => Value::from(2i64),
            })
        );
        assert_eq!(
            run(r#"{"x": true,}"#, &[]),
            Value::Map(indexmap::indexmap! { "x".into() => Value::Bool(true) })
        );
        assert_eq!(
            run(r#"{"pre" + "fix": 9}["prefix"]"#, &[]),
            Value::from(9i64)
        );
        assert_eq!(
            run(r#"{"a": {"b": 2}}"#, &[]),
            Value::Map(indexmap::indexmap! {
                "a".into() => Value::Map(indexmap::indexmap! { "b".into() => Value::from(2i64) }),
            })
        );
        assert_eq!(run("{}", &[]), Value::Null);
    }

    #[test]
    fn test_cast() {
        assert_eq!(run(r#"str("hello")"#, &[]), Value::from("hello"));
        assert_eq!(run(r#"str(42)"#, &[]), Value::from("42"));
        assert_eq!(run(r#"str(3.14)"#, &[]), Value::from("3.14"));
        assert_eq!(run(r#"str(true)"#, &[]), Value::from("true"));
        assert_eq!(run(r#"str(false)"#, &[]), Value::from("false"));
        assert_eq!(run(r#"str(null)"#, &[]), Value::from("null"));
        assert_eq!(run(r#"int(42)"#, &[]), Value::from(42i64));
        assert_eq!(run(r#"int(3.9)"#, &[]), Value::from(3i64));
        assert_eq!(run(r#"int(-3.9)"#, &[]), Value::from(-3i64));
        assert_eq!(run(r#"int("42")"#, &[]), Value::from(42i64));
        assert_eq!(run(r#"int(true)"#, &[]), Value::from(1i64));
        assert_eq!(run(r#"int(false)"#, &[]), Value::from(0i64));
        assert!(try_run("int(f)", &[("f", Value::Float(f64::NAN))]).is_err());
        assert!(try_run("int(f)", &[("f", Value::Float(f64::INFINITY))]).is_err());
        assert!(try_run("int(f)", &[("f", Value::Float(f64::NEG_INFINITY))]).is_err());
        assert!(try_run("int(f)", &[("f", Value::Float(1e100))]).is_err());
        assert_eq!(
            try_run("int(f)", &[("f", Value::Float(i64::MIN as f64))]).unwrap(),
            Value::from(i64::MIN)
        );
        assert_eq!(run(r#"float(42)"#, &[]), Value::from(42.0f64));
        assert_eq!(run(r#"float(1.5)"#, &[]), Value::from(1.5f64));
        assert_eq!(run(r#"float("1.5")"#, &[]), Value::from(1.5f64));
        assert_eq!(run(r#"float(true)"#, &[]), Value::from(1.0f64));
        assert_eq!(run(r#"float(false)"#, &[]), Value::from(0.0f64));
        assert_eq!(run(r#"bool(1)"#, &[]), Value::from(true));
        assert_eq!(run(r#"bool(0)"#, &[]), Value::from(false));
        assert_eq!(run(r#"bool("")"#, &[]), Value::from(false));
        assert_eq!(run(r#"bool("x")"#, &[]), Value::from(true));
        assert_eq!(run(r#"bool(null)"#, &[]), Value::from(false));
        assert_eq!(
            run(r#"list("abc")"#, &[]),
            Value::Array(vec![Value::from("a"), Value::from("b"), Value::from("c")])
        );
        let arr = Value::Array(vec![Value::from(1i64), Value::from(2i64)]);
        assert_eq!(run("list(arr)", &[("arr", arr.clone())]), arr);
        let m = Value::Map(
            indexmap::indexmap! { "x".into() => Value::from(1i64), "y".into() => Value::from(2i64) },
        );
        assert_eq!(
            run("list(m)", &[("m", m)]),
            Value::Array(vec![Value::from("x"), Value::from("y")])
        );
    }

    #[test]
    fn test_nan_comparisons() {
        let nan = Value::Float(f64::NAN);
        for expr in &[
            "nan < 1.0",
            "nan > 1.0",
            "nan <= 1.0",
            "nan >= 1.0",
            "1.0 < nan",
        ] {
            assert_eq!(
                run(expr, &[("nan", nan.clone())]),
                Value::Bool(false),
                "expected false for: {expr}"
            );
        }
        assert_eq!(
            run("nan == nan", &[("nan", nan.clone())]),
            Value::Bool(false)
        );
        assert_eq!(run("nan != nan", &[("nan", nan)]), Value::Bool(true));
    }

    #[test]
    fn test_object_operator_overload() {
        #[derive(Debug, Clone)]
        struct Counter(i64);

        impl super::super::value::Object for Counter {
            fn type_name(&self) -> &'static str {
                "Counter"
            }
            fn call_method(&self, method: &str, args: &[Value]) -> HResult<Value> {
                match method {
                    "__add__" => match args.first() {
                        Some(Value::Int(n)) => Ok(Value::Object(Box::new(Counter(self.0 + n)))),
                        _ => Err(EvalHelperError::new("expected int")),
                    },
                    "__eq__" => match args.first() {
                        Some(Value::Object(other)) => {
                            let other = other.display().parse::<i64>().unwrap_or(i64::MIN);
                            Ok(Value::Bool(self.0 == other))
                        }
                        _ => Ok(Value::Bool(false)),
                    },
                    m => Err(EvalHelperError::new(format!("no method {m}"))),
                }
            }
            fn clone_box(&self) -> Box<dyn super::super::value::Object> {
                Box::new(self.clone())
            }
            fn eq_box(&self, _: &dyn super::super::value::Object) -> bool {
                false
            }
            fn display(&self) -> String {
                self.0.to_string()
            }
        }

        let mut env = default_env();
        env.insert("c".to_string(), Value::Object(Box::new(Counter(10))));
        let result = eval_inner(&parse("c + 5").unwrap(), &mut env).unwrap();
        assert!(matches!(result, Value::Object(_)));
        assert_eq!(result.to_string(), "15");
        env.insert("d".to_string(), Value::Object(Box::new(Counter(10))));
        assert_eq!(
            eval_inner(&parse("c == d").unwrap(), &mut env).unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_var() {
        let mut env = default_env();
        assert!(eval(&parse("missing").unwrap(), &mut env).is_err());
    }

    #[test]
    fn test_native_func() {
        let mut env = default_env();
        env.insert(
            "join_path".into(),
            Value::Fn(NativeFn::new(|args: &[Value]| {
                let parts: Vec<&str> = args
                    .iter()
                    .map(|v| {
                        if let Value::String(s) = v {
                            s.as_str()
                        } else {
                            ""
                        }
                    })
                    .collect();
                Ok(Value::String(parts.join("/")))
            })),
        );
        assert_eq!(
            eval(
                &parse(r#"join_path("base", "file.json")"#).unwrap(),
                &mut env
            )
            .unwrap(),
            Value::from("base/file.json")
        );
    }

    #[test]
    fn test_complex_expr() {
        let feature = Value::Map(indexmap::indexmap! {
            "extension".into() => Value::from("gml"),
            "package".into() => Value::from("bldg"),
        });
        let pkgs = Value::Array(vec![Value::from("bldg"), Value::from("tran")]);
        assert_eq!(
            run(
                r#"feature["extension"] == "gml" and feature["package"] in packages"#,
                &[("feature", feature), ("packages", pkgs)]
            ),
            Value::from(true)
        );
    }

    #[test]
    fn test_depth_limit_ok() {
        let n = MAX_EVAL_DEPTH - 1;
        let expr = format!("1{}", "+1".repeat(n));
        assert_eq!(
            eval(&parse(&expr).unwrap(), &mut default_env()).unwrap(),
            Value::Int(n as i64 + 1)
        );
    }

    #[test]
    fn test_eval_error_pos() {
        // division by zero: the Binary node for "1 / 0" spans the whole expr, starting at byte 0
        let err = try_run("1 / 0", &[]).unwrap_err();
        match err {
            Error::Eval { pos, msg } => {
                assert_eq!(msg, "division by zero");
                assert_eq!(pos, 0);
            }
            other => panic!("expected Eval error, got {other:?}"),
        }
    }

    #[test]
    fn test_list_index_assign() {
        // basic element replacement
        assert_eq!(
            run("a = [1, 2, 3]; a[1] = 99; a", &[]),
            Value::Array(vec![
                Value::from(1i64),
                Value::from(99i64),
                Value::from(3i64)
            ])
        );
        // negative index
        assert_eq!(
            run("a = [10, 20, 30]; a[-1] = 99; a", &[]),
            Value::Array(vec![
                Value::from(10i64),
                Value::from(20i64),
                Value::from(99i64)
            ])
        );
        // assign expression evaluates to the assigned value
        assert_eq!(run("a = [0]; a[0] = 7", &[]), Value::from(7i64));
        // nested: a[0][1] = v
        assert_eq!(
            run("a = [[1, 2], [3, 4]]; a[0][1] = 99; a[0]", &[]),
            Value::Array(vec![Value::from(1i64), Value::from(99i64)])
        );
    }

    #[test]
    fn test_list_index_assign_out_of_range() {
        let err = try_run("a = [1, 2]; a[5] = 9", &[]).unwrap_err();
        assert!(
            matches!(&err, Error::Eval { msg, .. } if msg.contains("out of range")),
            "expected out-of-range error, got {err:?}"
        );
    }

    #[test]
    fn test_map_index_assign() {
        // insert new key
        assert_eq!(
            run(r#"m = {"a": 1}; m["b"] = 2; m["b"]"#, &[]),
            Value::from(2i64)
        );
        // overwrite existing key
        assert_eq!(
            run(r#"m = {"x": 10}; m["x"] = 99; m["x"]"#, &[]),
            Value::from(99i64)
        );
        // nested: m["k"][0] = v
        assert_eq!(
            run(r#"m = {"k": [1, 2, 3]}; m["k"][0] = 99; m["k"][0]"#, &[]),
            Value::from(99i64)
        );
        // assign returns the assigned value
        assert_eq!(run(r#"m = {"a": 0}; m["x"] = 42"#, &[]), Value::from(42i64));
    }

    #[test]
    fn test_invalid_lvalue() {
        let err = try_run("1 = 2", &[]).unwrap_err();
        assert!(
            matches!(&err, Error::Eval { msg, .. } if msg.contains("invalid assignment target")),
            "expected invalid-lvalue error, got {err:?}"
        );
    }
}
