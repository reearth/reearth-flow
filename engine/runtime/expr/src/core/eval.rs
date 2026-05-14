use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexMap;

use super::ast::{BinOp, Expr, ExprKind, UnaryOp};
use super::builtins::builtin_url;
use super::error::{Error, EvalHelperError, HResult, Result};
use super::value::Value;

pub type NativeFn = Box<dyn Fn(&[Value]) -> HResult<Value> + Send + Sync>;

trait ToEvalError<T> {
    fn to_eval_error(self, pos: usize) -> Result<T>;
}

impl<T> ToEvalError<T> for HResult<T> {
    fn to_eval_error(self, pos: usize) -> Result<T> {
        self.map_err(|e| Error::Eval { pos, msg: e.msg })
    }
}

pub struct Context {
    funcs: HashMap<String, NativeFn>,
}

impl Context {
    pub fn new() -> Self {
        let mut ctx = Self {
            funcs: HashMap::new(),
        };
        ctx.register("map", Box::new(builtin_map));
        ctx.register("str", Box::new(builtin_str));
        ctx.register("int", Box::new(builtin_int));
        ctx.register("float", Box::new(builtin_float));
        ctx.register("bool", Box::new(builtin_bool));
        ctx.register("list", Box::new(builtin_list));
        ctx.register("Url", Box::new(builtin_url));
        ctx
    }

    pub fn register(&mut self, name: impl Into<String>, f: NativeFn) {
        self.funcs.insert(name.into(), f);
    }

    fn call(&self, name: &str, args: Vec<Value>) -> HResult<Value> {
        match self.funcs.get(name) {
            Some(f) => f(&args),
            None => Err(EvalHelperError::new(format!("unknown function '{name}'"))),
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

// Lexical environment: immutable linked list via Arc.
// Each `let` evaluation prepends one frame; lookup walks the chain.
struct EnvFrame {
    name: String,
    value: Value,
    parent: Env,
}

type Env = Option<Arc<EnvFrame>>;

fn env_lookup(env: &Env, name: &str) -> Option<Value> {
    let mut cur = env.as_deref();
    while let Some(frame) = cur {
        if frame.name == name {
            return Some(frame.value.clone());
        }
        cur = frame.parent.as_deref();
    }
    None
}

fn env_extend(parent: &Env, name: String, value: Value) -> Env {
    Some(Arc::new(EnvFrame {
        name,
        value,
        parent: parent.clone(),
    }))
}

// Public entry point — env starts empty.
pub fn eval(expr: &Expr, ctx: &Context) -> Result<Value> {
    eval_inner(expr, ctx, &None)
}

fn eval_inner(expr: &Expr, ctx: &Context, env: &Env) -> Result<Value> {
    let pos = expr.span.start;
    match &expr.kind {
        ExprKind::Null => Ok(Value::Null),
        ExprKind::Bool(b) => Ok(Value::Bool(*b)),
        ExprKind::Int(n) => Ok(Value::Int(*n)),
        ExprKind::Float(f) => Ok(Value::Float(*f)),
        ExprKind::Str(s) => Ok(Value::String(s.clone())),
        ExprKind::Array(items) => {
            let values: Result<Vec<_>> = items.iter().map(|e| eval_inner(e, ctx, env)).collect();
            Ok(Value::Array(values?))
        }
        ExprKind::Map(entries) => {
            let mut map = IndexMap::new();
            for (k, v) in entries {
                let key = eval_inner(k, ctx, env)?;
                let key_str = match key {
                    Value::String(s) => s,
                    Value::Int(n) => n.to_string(),
                    other => {
                        return Err(Error::Eval {
                            pos,
                            msg: format!("map key must evaluate to a string, got {other:?}"),
                        })
                    }
                };
                map.insert(key_str, eval_inner(v, ctx, env)?);
            }
            Ok(Value::Map(map))
        }
        ExprKind::Var(name) => env_lookup(env, name).ok_or_else(|| Error::Eval {
            pos,
            msg: format!("unknown variable '{name}'"),
        }),
        ExprKind::Index(target, key) => {
            let target = eval_inner(target, ctx, env)?;
            let key = eval_inner(key, ctx, env)?;
            eval_index(target, key).to_eval_error(pos)
        }
        ExprKind::Slice {
            target,
            start,
            stop,
            step,
        } => {
            let target = eval_inner(target, ctx, env)?;
            let start = start
                .as_deref()
                .map(|e| eval_inner(e, ctx, env))
                .transpose()?;
            let stop = stop
                .as_deref()
                .map(|e| eval_inner(e, ctx, env))
                .transpose()?;
            let step = step
                .as_deref()
                .map(|e| eval_inner(e, ctx, env))
                .transpose()?;
            eval_slice(target, start, stop, step).to_eval_error(pos)
        }
        ExprKind::FuncCall { name, args } => {
            let args: Result<Vec<_>> = args.iter().map(|e| eval_inner(e, ctx, env)).collect();
            ctx.call(name, args?).to_eval_error(pos)
        }
        ExprKind::Unary(op, e) => {
            let val = eval_inner(e, ctx, env)?;
            eval_unary(op, val).to_eval_error(pos)
        }
        ExprKind::MethodCall {
            receiver,
            method,
            args,
        } => {
            let recv = eval_inner(receiver, ctx, env)?;
            let evaled: Result<Vec<_>> = args.iter().map(|e| eval_inner(e, ctx, env)).collect();
            eval_method(recv, method, &evaled?).to_eval_error(pos)
        }
        ExprKind::Binary(left, op, right) => {
            match op {
                BinOp::And => {
                    let l = eval_inner(left, ctx, env)?;
                    if !is_truthy(&l) {
                        return Ok(Value::Bool(false));
                    }
                    let r = eval_inner(right, ctx, env)?;
                    return Ok(Value::Bool(is_truthy(&r)));
                }
                BinOp::Or => {
                    let l = eval_inner(left, ctx, env)?;
                    if is_truthy(&l) {
                        return Ok(Value::Bool(true));
                    }
                    let r = eval_inner(right, ctx, env)?;
                    return Ok(Value::Bool(is_truthy(&r)));
                }
                _ => {}
            }
            let left = eval_inner(left, ctx, env)?;
            let right = eval_inner(right, ctx, env)?;
            eval_binary(op, left, right).to_eval_error(pos)
        }
        ExprKind::Let { name, value, body } => {
            let v = eval_inner(value, ctx, env)?;
            let new_env = env_extend(env, name.clone(), v);
            eval_inner(body, ctx, &new_env)
        }
        ExprKind::Block(exprs) => {
            let mut result = Value::Null;
            for e in exprs {
                result = eval_inner(e, ctx, env)?;
            }
            Ok(result)
        }
        ExprKind::If { cond, then, else_ } => {
            let c = eval_inner(cond, ctx, env)?;
            if is_truthy(&c) {
                eval_inner(then, ctx, env)
            } else {
                eval_inner(else_, ctx, env)
            }
        }
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
            let _ = args;
            Ok(Value::Int(s.chars().count() as i64))
        }
        "trim" => {
            let _ = args;
            Ok(Value::String(s.trim().to_string()))
        }
        "split" => {
            let sep = match args.first() {
                Some(Value::String(sep)) => sep.as_str(),
                Some(v) => {
                    return Err(EvalHelperError::new(format!(
                        "split() separator must be a string, got {v:?}"
                    )))
                }
                None => {
                    return Err(EvalHelperError::new(
                        "split() requires a separator argument",
                    ))
                }
            };
            Ok(Value::Array(
                s.split(sep).map(|p| Value::String(p.to_string())).collect(),
            ))
        }
        "contains" => {
            let needle = match args.first() {
                Some(Value::String(n)) => n.as_str(),
                Some(v) => {
                    return Err(EvalHelperError::new(format!(
                        "contains() argument must be a string, got {v:?}"
                    )))
                }
                None => return Err(EvalHelperError::new("contains() requires an argument")),
            };
            Ok(Value::Bool(s.contains(needle)))
        }
        "starts_with" => {
            let prefix = match args.first() {
                Some(Value::String(p)) => p.as_str(),
                Some(v) => {
                    return Err(EvalHelperError::new(format!(
                        "starts_with() argument must be a string, got {v:?}"
                    )))
                }
                None => return Err(EvalHelperError::new("starts_with() requires an argument")),
            };
            Ok(Value::Bool(s.starts_with(prefix)))
        }
        "ends_with" => {
            let suffix = match args.first() {
                Some(Value::String(suf)) => suf.as_str(),
                Some(v) => {
                    return Err(EvalHelperError::new(format!(
                        "ends_with() argument must be a string, got {v:?}"
                    )))
                }
                None => return Err(EvalHelperError::new("ends_with() requires an argument")),
            };
            Ok(Value::Bool(s.ends_with(suffix)))
        }
        "replace" => {
            let (from, to) = match (args.first(), args.get(1)) {
                (Some(Value::String(f)), Some(Value::String(t))) => (f.as_str(), t.as_str()),
                _ => {
                    return Err(EvalHelperError::new(
                        "replace() requires two string arguments: replace(from, to)",
                    ))
                }
            };
            Ok(Value::String(s.replace(from, to)))
        }
        m => Err(EvalHelperError::new(format!("String has no method '{m}'"))),
    }
}

fn eval_array_method(a: Vec<Value>, method: &str, args: &[Value]) -> HResult<Value> {
    match method {
        "len" => {
            let _ = args;
            Ok(Value::Int(a.len() as i64))
        }
        "contains" => {
            let needle = args.first().unwrap_or(&Value::Null);
            Ok(Value::Bool(a.iter().any(|v| values_equal(v, needle))))
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
            i += step;
        }
    } else {
        while i > stop {
            indices.push(i as usize);
            i += step;
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
        BinOp::And | BinOp::Or => None,
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

fn coerce_numeric(a: Value, b: Value) -> HResult<(Numeric, Numeric)> {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => Ok((Numeric::Int(a), Numeric::Int(b))),
        (Value::Int(a), Value::Float(b)) => Ok((Numeric::Float(a as f64), Numeric::Float(b))),
        (Value::Float(a), Value::Int(b)) => Ok((Numeric::Float(a), Numeric::Float(b as f64))),
        (Value::Float(a), Value::Float(b)) => Ok((Numeric::Float(a), Numeric::Float(b))),
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
    match coerce_numeric(left, right)? {
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
            (a, b) => match coerce_numeric(a, b) {
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
        BinOp::Div => match coerce_numeric(left, right) {
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
    let ord = match coerce_numeric(left.clone(), right.clone()) {
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
    a == b
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
        Value::Object(_) => true,
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
        let ctx = Context::new();
        let env = vars.iter().fold(None, |env, (name, val)| {
            env_extend(&env, name.to_string(), val.clone())
        });
        eval_inner(&parse(input).unwrap(), &ctx, &env)
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
        assert_eq!(run("true && false", &[]), Value::from(false));
        assert_eq!(run("true || false", &[]), Value::from(true));
        assert_eq!(run("!true", &[]), Value::from(false));
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
    fn test_array_contains() {
        let pkgs = Value::Array(vec![Value::from("bldg"), Value::from("tran")]);
        assert_eq!(
            run(r#"pkgs.contains("bldg")"#, &[("pkgs", pkgs.clone())]),
            Value::from(true)
        );
        assert_eq!(
            run(r#"pkgs.contains("fld")"#, &[("pkgs", pkgs)]),
            Value::from(false)
        );
    }

    #[test]
    fn test_string_contains_starts_ends_with() {
        assert_eq!(
            run(r#""hello world".contains("world")"#, &[]),
            Value::from(true)
        );
        assert_eq!(
            run(r#""hello world".contains("xyz")"#, &[]),
            Value::from(false)
        );
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
                r#"let s = "city_bldg"; let sfx = "_bldg"; if s.ends_with(sfx) { s[:s.len() - sfx.len()] } else { s }"#,
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
    fn test_let() {
        assert_eq!(run("let x = 42; x", &[]), Value::from(42i64));
        assert_eq!(run("let x = 3; x * x", &[]), Value::from(9i64));
        assert_eq!(
            run("let x = 2; let y = x + 1; x * y", &[]),
            Value::from(6i64)
        );
        assert_eq!(run("let x = 1; let x = 99; x", &[]), Value::from(99i64));
        assert_eq!(
            run("let x = 7; x", &[("x", Value::from(999i64))]),
            Value::from(7i64)
        );
        assert_eq!(
            run("let x = 10; (let x = 99; x) + x", &[]),
            Value::from(109i64)
        );
        assert_eq!(
            run("let x = 10; { let x = 99; x } + x", &[]),
            Value::from(109i64)
        );
        assert_eq!(run("(let x = 10; x) * 2", &[]), Value::from(20i64));
    }

    #[test]
    fn test_block() {
        assert_eq!(run("{ 1; 2; 3 }", &[]), Value::from(3i64));
        assert_eq!(run("{ 42; }", &[]), Value::Null);
        assert_eq!(run("{}", &[]), Value::Null);
        assert_eq!(run("{ let x = 5; x * 2 }", &[]), Value::from(10i64));
        assert_eq!(
            run("{ let a = 3; let b = 4; a * a + b * b }", &[]),
            Value::from(25i64)
        );
        assert_eq!(run("{ let x = 1; }", &[]), Value::Null);
        assert_eq!(
            run("{ let x = 3; x } + { let y = 4; y }", &[]),
            Value::from(7i64)
        );
        assert_eq!(
            run("{ let x = 1; { let y = 2; x + y } }", &[]),
            Value::from(3i64)
        );
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
            run("if true { let x = 7; x * 2 } else { 0 }", &[]),
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

        impl super::super::value::ValueObject for Counter {
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
            fn clone_box(&self) -> Box<dyn super::super::value::ValueObject> {
                Box::new(self.clone())
            }
            fn eq_box(&self, _: &dyn super::super::value::ValueObject) -> bool {
                false
            }
            fn display(&self) -> String {
                self.0.to_string()
            }
        }

        let ctx = Context::new();
        let env = env_extend(&None, "c".into(), Value::Object(Box::new(Counter(10))));
        let result = eval_inner(&parse("c + 5").unwrap(), &ctx, &env).unwrap();
        assert!(matches!(result, Value::Object(_)));
        assert_eq!(result.to_string(), "15");
        let env2 = env_extend(&env, "d".into(), Value::Object(Box::new(Counter(10))));
        assert_eq!(
            eval_inner(&parse("c == d").unwrap(), &ctx, &env2).unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_var() {
        let ctx = Context::new();
        assert!(eval(&parse("missing").unwrap(), &ctx).is_err());
    }

    #[test]
    fn test_native_func() {
        let mut ctx = Context::new();
        ctx.register(
            "join_path",
            Box::new(|args: &[Value]| {
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
            }),
        );
        assert_eq!(
            eval(&parse(r#"join_path("base", "file.json")"#).unwrap(), &ctx).unwrap(),
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
                r#"feature["extension"] == "gml" && packages.contains(feature["package"])"#,
                &[("feature", feature), ("packages", pkgs)]
            ),
            Value::from(true)
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
}
