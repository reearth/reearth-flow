use std::collections::HashMap;
use std::sync::Arc;

use indexmap::IndexMap;

use super::ast::{BinOp, Expr, UnaryOp};
use super::builtins::builtin_url;
use super::error::{Error, Result};
use super::value::Value;

pub type NativeFn = Box<dyn Fn(&[Value]) -> Result<Value> + Send + Sync>;

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

    fn call(&self, name: &str, args: Vec<Value>) -> Result<Value> {
        match self.funcs.get(name) {
            Some(f) => f(&args),
            None => Err(Error::Eval {
                msg: format!("unknown function '{name}'"),
            }),
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
    match expr {
        Expr::Null => Ok(Value::Null),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Int(n) => Ok(Value::Int(*n)),
        Expr::Float(f) => Ok(Value::Float(*f)),
        Expr::Str(s) => Ok(Value::String(s.clone())),
        Expr::Array(items) => {
            let values: Result<Vec<_>> = items.iter().map(|e| eval_inner(e, ctx, env)).collect();
            Ok(Value::Array(values?))
        }
        Expr::Map(entries) => {
            let mut map = IndexMap::new();
            for (k, v) in entries {
                let key = eval_inner(k, ctx, env)?;
                let key_str = match key {
                    Value::String(s) => s,
                    Value::Int(n) => n.to_string(),
                    other => {
                        return Err(Error::Eval {
                            msg: format!("map key must evaluate to a string, got {other:?}"),
                        })
                    }
                };
                map.insert(key_str, eval_inner(v, ctx, env)?);
            }
            Ok(Value::Map(map))
        }
        Expr::Var(name) => {
            // Lexical env takes priority; fall through to __resolve for external vars.
            if let Some(v) = env_lookup(env, name) {
                Ok(v)
            } else {
                ctx.call("__resolve", vec![Value::String(name.clone())])
            }
        }
        Expr::Index(target, key) => {
            let target = eval_inner(target, ctx, env)?;
            let key = eval_inner(key, ctx, env)?;
            eval_index(target, key)
        }
        Expr::Slice {
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
            eval_slice(target, start, stop, step)
        }
        Expr::FuncCall { name, args } => {
            let args: Result<Vec<_>> = args.iter().map(|e| eval_inner(e, ctx, env)).collect();
            ctx.call(name, args?)
        }
        Expr::Unary(op, expr) => {
            let val = eval_inner(expr, ctx, env)?;
            eval_unary(op, val)
        }
        Expr::MethodCall {
            receiver,
            method,
            args,
        } => {
            let recv = eval_inner(receiver, ctx, env)?;
            let evaled: Result<Vec<_>> = args.iter().map(|e| eval_inner(e, ctx, env)).collect();
            eval_method(recv, method, &evaled?)
        }
        Expr::Binary(left, op, right) => {
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
            eval_binary(op, left, right)
        }
        Expr::Let { name, value, body } => {
            let v = eval_inner(value, ctx, env)?;
            let new_env = env_extend(env, name.clone(), v);
            eval_inner(body, ctx, &new_env)
        }
        Expr::Block(exprs) => {
            let mut result = Value::Null;
            for e in exprs {
                result = eval_inner(e, ctx, env)?;
            }
            Ok(result)
        }
        Expr::If { cond, then, else_ } => {
            let c = eval_inner(cond, ctx, env)?;
            if is_truthy(&c) {
                eval_inner(then, ctx, env)
            } else {
                eval_inner(else_, ctx, env)
            }
        }
    }
}

fn eval_method(recv: Value, method: &str, args: &[Value]) -> Result<Value> {
    match recv {
        Value::String(s) => eval_string_method(s, method, args),
        Value::Array(a) => eval_array_method(a, method, args),
        Value::Object(obj) => obj.call_method(method, args),
        v => Err(Error::Eval {
            msg: format!("{v:?} has no method '{method}'"),
        }),
    }
}

fn eval_string_method(s: String, method: &str, args: &[Value]) -> Result<Value> {
    match method {
        "len" => {
            let _ = args;
            Ok(Value::Int(s.chars().count() as i64))
        }
        "trim" => {
            let _ = args;
            Ok(Value::String(s.trim().to_string()))
        }
        m => Err(Error::Eval {
            msg: format!("String has no method '{m}'"),
        }),
    }
}

fn eval_array_method(a: Vec<Value>, method: &str, args: &[Value]) -> Result<Value> {
    match method {
        "len" => {
            let _ = args;
            Ok(Value::Int(a.len() as i64))
        }
        m => Err(Error::Eval {
            msg: format!("Array has no method '{m}'"),
        }),
    }
}

fn eval_index(target: Value, key: Value) -> Result<Value> {
    match (target, key) {
        (Value::Map(map), Value::String(k)) => Ok(map.get(&k).cloned().unwrap_or(Value::Null)),
        (Value::Array(arr), Value::Int(i)) => {
            let len = arr.len() as i64;
            let i = if i < 0 { len + i } else { i };
            if i < 0 || i >= len {
                return Ok(Value::Null);
            }
            Ok(arr.into_iter().nth(i as usize).unwrap_or(Value::Null))
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
        (target, key) => Err(Error::Eval {
            msg: format!("cannot index {target:?} with {key:?}"),
        }),
    }
}

fn as_slice_index(v: Value, what: &str) -> Result<i64> {
    match v {
        Value::Int(n) => Ok(n),
        Value::Float(f) if f.fract() == 0.0 => Ok(f as i64),
        v => Err(Error::Eval {
            msg: format!("slice {what} must be an integer, got {v:?}"),
        }),
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
) -> Result<Value> {
    let step = match step {
        None => 1i64,
        Some(v) => as_slice_index(v, "step")?,
    };
    if step == 0 {
        return Err(Error::Eval {
            msg: "slice step cannot be zero".into(),
        });
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
        v => Err(Error::Eval {
            msg: format!("cannot slice {v:?}"),
        }),
    }
}

fn eval_unary(op: &UnaryOp, val: Value) -> Result<Value> {
    match op {
        UnaryOp::Not => Ok(Value::Bool(!is_truthy(&val))),
        UnaryOp::Neg => match val {
            Value::Int(n) => Ok(Value::Int(-n)),
            Value::Float(f) => Ok(Value::Float(-f)),
            v => Err(Error::Eval {
                msg: format!("cannot negate {v:?}"),
            }),
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
        BinOp::Ne => Some("__ne__"),
        BinOp::Lt => Some("__lt__"),
        BinOp::Le => Some("__le__"),
        BinOp::Gt => Some("__gt__"),
        BinOp::Ge => Some("__ge__"),
        BinOp::In | BinOp::And | BinOp::Or => None,
    }
}

fn try_object_op(op: &BinOp, left: Value, right: Value) -> Result<Value> {
    let dunder = binop_dunder(op).ok_or_else(|| Error::Eval {
        msg: format!("operator not overloadable for {left:?}"),
    })?;
    if let Value::Object(ref obj) = left {
        return obj.call_method(dunder, &[right]);
    }
    Err(Error::Eval {
        msg: format!("operator '{dunder}' not supported between {left:?} and {right:?}"),
    })
}

enum Numeric {
    Int(i64),
    Float(f64),
}

fn coerce_numeric(a: Value, b: Value) -> Result<(Numeric, Numeric)> {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => Ok((Numeric::Int(a), Numeric::Int(b))),
        (Value::Int(a), Value::Float(b)) => Ok((Numeric::Float(a as f64), Numeric::Float(b))),
        (Value::Float(a), Value::Int(b)) => Ok((Numeric::Float(a), Numeric::Float(b as f64))),
        (Value::Float(a), Value::Float(b)) => Ok((Numeric::Float(a), Numeric::Float(b))),
        (a, b) => Err(Error::Eval {
            msg: format!("cannot apply numeric op to {a:?} and {b:?}"),
        }),
    }
}

fn numeric_op(
    left: Value,
    right: Value,
    int_op: impl Fn(i64, i64) -> i64,
    float_op: impl Fn(f64, f64) -> f64,
) -> Result<Value> {
    match coerce_numeric(left, right)? {
        (Numeric::Int(a), Numeric::Int(b)) => Ok(Value::Int(int_op(a, b))),
        (Numeric::Float(a), Numeric::Float(b)) => Ok(Value::Float(float_op(a, b))),
        _ => unreachable!(),
    }
}

fn eval_binary(op: &BinOp, left: Value, right: Value) -> Result<Value> {
    match op {
        BinOp::Add => match (left, right) {
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + b.as_str())),
            (Value::String(a), b) => Ok(Value::String(a + value_to_string(&b).as_str())),
            (Value::Array(mut a), Value::Array(b)) => {
                a.extend(b);
                Ok(Value::Array(a))
            }
            (a, b) => match coerce_numeric(a, b) {
                Ok((Numeric::Int(a), Numeric::Int(b))) => Ok(Value::Int(a + b)),
                Ok((Numeric::Float(a), Numeric::Float(b))) => Ok(Value::Float(a + b)),
                Ok(_) => unreachable!(),
                Err(_) => {
                    // re-construct originals is not possible after move; surface as object op error
                    Err(Error::Eval {
                        msg: "'+' not supported for these types".into(),
                    })
                }
            },
        },
        BinOp::Sub => numeric_op(left, right, |a, b| a - b, |a, b| a - b),
        BinOp::Mul => numeric_op(left, right, |a, b| a * b, |a, b| a * b),
        BinOp::Div => match coerce_numeric(left.clone(), right.clone()) {
            Ok((Numeric::Int(a), Numeric::Int(b))) => {
                if b == 0 {
                    return Err(Error::Eval {
                        msg: "division by zero".into(),
                    });
                }
                if a % b == 0 {
                    Ok(Value::Int(a / b))
                } else {
                    Ok(Value::Float(a as f64 / b as f64))
                }
            }
            Ok((Numeric::Float(a), Numeric::Float(b))) => {
                if b == 0.0 {
                    return Err(Error::Eval {
                        msg: "division by zero".into(),
                    });
                }
                Ok(Value::Float(a / b))
            }
            Ok(_) => unreachable!(),
            Err(_) => try_object_op(op, left, right),
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
            (Value::String(needle), Value::String(haystack)) => {
                Ok(Value::Bool(haystack.contains(needle.as_str())))
            }
            (Value::String(key), Value::Map(map)) => Ok(Value::Bool(map.contains_key(&key))),
            (_, Value::Null) => Ok(Value::Bool(false)),
            (l, r) => Err(Error::Eval {
                msg: format!("'in' not supported between {l:?} and {r:?}"),
            }),
        },
        BinOp::And | BinOp::Or => unreachable!("handled with short-circuit above"),
    }
}

fn compare_values(
    left: Value,
    right: Value,
    pred: impl Fn(std::cmp::Ordering) -> bool,
) -> Result<Value> {
    let ord = match coerce_numeric(left.clone(), right.clone()) {
        Ok((Numeric::Int(a), Numeric::Int(b))) => a.cmp(&b),
        Ok((Numeric::Float(a), Numeric::Float(b))) => {
            a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
        }
        Ok(_) => unreachable!(),
        Err(_) => match (&left, &right) {
            (Value::String(a), Value::String(b)) => a.as_str().cmp(b.as_str()),
            _ => {
                return Err(Error::Eval {
                    msg: format!("cannot compare {left:?} and {right:?}"),
                })
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

fn builtin_str(args: &[Value]) -> Result<Value> {
    match args.first() {
        None | Some(Value::Null) => Ok(Value::String("null".into())),
        Some(Value::String(s)) => Ok(Value::String(s.clone())),
        Some(Value::Bool(b)) => Ok(Value::String(b.to_string())),
        Some(Value::Int(n)) => Ok(Value::String(n.to_string())),
        Some(Value::Float(f)) => Ok(Value::String(f.to_string())),
        Some(Value::Object(obj)) => obj.call_method("__str__", &[]),
        Some(v) => Err(Error::Eval {
            msg: format!("str() not supported for {v:?}"),
        }),
    }
}

fn builtin_int(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::Int(n)) => Ok(Value::Int(*n)),
        Some(Value::Float(f)) => Ok(Value::Int(f.trunc() as i64)),
        Some(Value::Bool(b)) => Ok(Value::Int(*b as i64)),
        Some(Value::String(s)) => {
            s.trim()
                .parse::<i64>()
                .map(Value::Int)
                .map_err(|_| Error::Eval {
                    msg: format!("int() cannot parse {s:?}"),
                })
        }
        Some(v) => Err(Error::Eval {
            msg: format!("int() not supported for {v:?}"),
        }),
        None => Err(Error::Eval {
            msg: "int() requires an argument".into(),
        }),
    }
}

fn builtin_float(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::Float(f)) => Ok(Value::Float(*f)),
        Some(Value::Int(n)) => Ok(Value::Float(*n as f64)),
        Some(Value::Bool(b)) => Ok(Value::Float(if *b { 1.0 } else { 0.0 })),
        Some(Value::String(s)) => {
            s.trim()
                .parse::<f64>()
                .map(Value::Float)
                .map_err(|_| Error::Eval {
                    msg: format!("float() cannot parse {s:?}"),
                })
        }
        Some(v) => Err(Error::Eval {
            msg: format!("float() not supported for {v:?}"),
        }),
        None => Err(Error::Eval {
            msg: "float() requires an argument".into(),
        }),
    }
}

fn builtin_bool(args: &[Value]) -> Result<Value> {
    Ok(Value::Bool(args.first().map(is_truthy).unwrap_or(false)))
}

fn builtin_list(args: &[Value]) -> Result<Value> {
    match args.first() {
        Some(Value::Array(a)) => Ok(Value::Array(a.clone())),
        Some(Value::String(s)) => Ok(Value::Array(
            s.chars().map(|c| Value::String(c.to_string())).collect(),
        )),
        Some(Value::Map(m)) => Ok(Value::Array(
            m.keys().map(|k| Value::String(k.clone())).collect(),
        )),
        Some(v) => Err(Error::Eval {
            msg: format!("list() not supported for {v:?}"),
        }),
        None => Ok(Value::Array(vec![])),
    }
}

fn builtin_map(args: &[Value]) -> Result<Value> {
    let pairs = match args.first() {
        Some(Value::Array(a)) => a,
        _ => {
            return Err(Error::Eval {
                msg: "map() expects an array of [key, value] pairs".into(),
            })
        }
    };
    let mut out = IndexMap::new();
    for (i, pair) in pairs.iter().enumerate() {
        match pair {
            Value::Array(kv) if kv.len() == 2 => {
                let key = match &kv[0] {
                    Value::String(s) => s.clone(),
                    v => {
                        return Err(Error::Eval {
                            msg: format!("map() key at index {i} must be a string, got {v:?}"),
                        })
                    }
                };
                out.insert(key, kv[1].clone());
            }
            _ => {
                return Err(Error::Eval {
                    msg: format!("map() entry at index {i} must be a 2-element array"),
                })
            }
        }
    }
    Ok(Value::Map(out))
}

#[cfg(test)]
mod tests {
    use super::super::parser::parse;
    use super::*;

    fn ctx_with_vars(vars: &[(&str, Value)]) -> Context {
        let map: HashMap<String, Value> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();
        let map = Arc::new(map);
        let mut ctx = Context::new();
        ctx.register(
            "__resolve",
            Box::new(move |args| {
                let name = args
                    .first()
                    .and_then(|v| {
                        if let Value::String(s) = v {
                            Some(s.as_str())
                        } else {
                            None
                        }
                    })
                    .unwrap_or("");
                Ok(map.get(name).cloned().unwrap_or(Value::Null))
            }),
        );
        ctx
    }

    fn run(input: &str, vars: &[(&str, Value)]) -> Value {
        eval(&parse(input).unwrap(), &ctx_with_vars(vars)).unwrap()
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
    fn test_in_operator() {
        let pkgs = Value::Array(vec![Value::from("bldg"), Value::from("tran")]);
        assert_eq!(
            run(r#""bldg" in packages"#, &[("packages", pkgs.clone())]),
            Value::from(true)
        );
        assert_eq!(
            run(r#""fld" in packages"#, &[("packages", pkgs)]),
            Value::from(false)
        );
    }

    #[test]
    fn test_index() {
        let feature = Value::Map(indexmap::indexmap! {
            "package".into() => Value::from("bldg"),
            "cityGmlPath".into() => Value::from("/data/city.gml"),
        });
        // map index — hit and miss
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
        // array index — positive and negative
        let arr = Value::Array(vec![
            Value::from(1i64),
            Value::from(2i64),
            Value::from(3i64),
        ]);
        assert_eq!(run("arr[0]", &[("arr", arr.clone())]), Value::from(1i64));
        assert_eq!(run("arr[-1]", &[("arr", arr)]), Value::from(3i64));
        // string index — positive and negative
        assert_eq!(run(r#""hello"[0]"#, &[]), Value::from("h"));
        assert_eq!(run(r#""hello"[4]"#, &[]), Value::from("o"));
        assert_eq!(run(r#""hello"[-1]"#, &[]), Value::from("o"));
    }

    #[test]
    fn test_slice() {
        // string
        assert_eq!(run(r#""abcde"[1:3]"#, &[]), Value::from("bc"));
        assert_eq!(run(r#""abcde"[:3]"#, &[]), Value::from("abc"));
        assert_eq!(run(r#""abcde"[2:]"#, &[]), Value::from("cde"));
        assert_eq!(run(r#""abcde"[:]"#, &[]), Value::from("abcde"));
        assert_eq!(run(r#""abcde"[::-1]"#, &[]), Value::from("edcba"));
        assert_eq!(run(r#""abcde"[-1::-2]"#, &[]), Value::from("eca"));
        assert_eq!(run(r#""abcde"[::2]"#, &[]), Value::from("ace"));
        // array
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
            run("arr[::-1]", &[("arr", arr)]),
            Value::Array((0i64..5).rev().map(Value::from).collect())
        );
    }

    #[test]
    fn test_method_call() {
        // string methods
        assert_eq!(run(r#""  hello  ".trim()"#, &[]), Value::from("hello"));
        assert_eq!(run(r#""hello".len()"#, &[]), Value::from(5i64));
        assert_eq!(run(r#""".len()"#, &[]), Value::from(0i64));
        // array methods
        let arr = Value::Array(vec![
            Value::from(1i64),
            Value::from(2i64),
            Value::from(3i64),
        ]);
        assert_eq!(run("arr.len()", &[("arr", arr)]), Value::from(3i64));
    }

    #[test]
    fn test_let() {
        assert_eq!(run("let x = 42; x", &[]), Value::from(42i64));
        assert_eq!(run("let x = 3; x * x", &[]), Value::from(9i64));
        // chain
        assert_eq!(
            run("let x = 2; let y = x + 1; x * y", &[]),
            Value::from(6i64)
        );
        // shadow
        assert_eq!(run("let x = 1; let x = 99; x", &[]), Value::from(99i64));
        // shadows external var — lexical binding wins over __resolve
        assert_eq!(
            run("let x = 7; x", &[("x", Value::from(999i64))]),
            Value::from(7i64)
        );
        // scope does not leak out of parens or blocks
        assert_eq!(
            run("let x = 10; (let x = 99; x) + x", &[]),
            Value::from(109i64)
        );
        assert_eq!(
            run("let x = 10; { let x = 99; x } + x", &[]),
            Value::from(109i64)
        );
        // let in parens
        assert_eq!(run("(let x = 10; x) * 2", &[]), Value::from(20i64));
    }

    #[test]
    fn test_block() {
        // last expression is the block value
        assert_eq!(run("{ 1; 2; 3 }", &[]), Value::from(3i64));
        // trailing semicolon → Null
        assert_eq!(run("{ 42; }", &[]), Value::Null);
        // empty block → Null
        assert_eq!(run("{}", &[]), Value::Null);
        // let inside block
        assert_eq!(run("{ let x = 5; x * 2 }", &[]), Value::from(10i64));
        assert_eq!(
            run("{ let a = 3; let b = 4; a * a + b * b }", &[]),
            Value::from(25i64)
        );
        assert_eq!(run("{ let x = 1; }", &[]), Value::Null);
        // block as expression in larger expr
        assert_eq!(
            run("{ let x = 3; x } + { let y = 4; y }", &[]),
            Value::from(7i64)
        );
        // nested blocks
        assert_eq!(
            run("{ let x = 1; { let y = 2; x + y } }", &[]),
            Value::from(3i64)
        );
    }

    #[test]
    fn test_if() {
        assert_eq!(run("if true { 1 } else { 2 }", &[]), Value::from(1i64));
        assert_eq!(run("if false { 1 } else { 2 }", &[]), Value::from(2i64));
        // condition expression
        assert_eq!(run("if 1 == 1 { 42 } else { 0 }", &[]), Value::from(42i64));
        assert_eq!(run("if 1 == 2 { 42 } else { 0 }", &[]), Value::from(0i64));
        // null is falsy
        assert_eq!(run("if null { 1 } else { 2 }", &[]), Value::from(2i64));
        // else-if chain
        assert_eq!(
            run("if false { 1 } else if false { 2 } else { 3 }", &[]),
            Value::from(3i64)
        );
        assert_eq!(
            run("if false { 1 } else if true { 2 } else { 3 }", &[]),
            Value::from(2i64)
        );
        // as expression in binop
        assert_eq!(
            run(
                "(if true { 10 } else { 0 }) + (if false { 0 } else { 5 })",
                &[]
            ),
            Value::from(15i64)
        );
        // let in branch body
        assert_eq!(
            run("if true { let x = 7; x * 2 } else { 0 }", &[]),
            Value::from(14i64)
        );
        // no-else: true branch returns value, false branch → Null
        assert_eq!(run("if true { 42 }", &[]), Value::from(42i64));
        assert_eq!(run("if false { 42 }", &[]), Value::Null);
    }

    #[test]
    fn test_map() {
        // map() builtin from array of pairs
        assert_eq!(
            run(r#"map([["a", 1], ["b", 2]])"#, &[]),
            Value::Map(indexmap::indexmap! {
                "a".into() => Value::from(1i64),
                "b".into() => Value::from(2i64),
            })
        );
        // map literal — basic string keys
        assert_eq!(
            run(r#"{"a": 1, "b": 2}"#, &[]),
            Value::Map(indexmap::indexmap! {
                "a".into() => Value::from(1i64),
                "b".into() => Value::from(2i64),
            })
        );
        // trailing comma
        assert_eq!(
            run(r#"{"x": true,}"#, &[]),
            Value::Map(indexmap::indexmap! { "x".into() => Value::Bool(true) })
        );
        // expr key
        assert_eq!(
            run(r#"{"pre" + "fix": 9}["prefix"]"#, &[]),
            Value::from(9i64)
        );
        // nested
        assert_eq!(
            run(r#"{"a": {"b": 2}}"#, &[]),
            Value::Map(indexmap::indexmap! {
                "a".into() => Value::Map(indexmap::indexmap! { "b".into() => Value::from(2i64) }),
            })
        );
        // {} is Null, not an empty map
        assert_eq!(run("{}", &[]), Value::Null);
    }

    #[test]
    fn test_cast() {
        // str()
        assert_eq!(run(r#"str("hello")"#, &[]), Value::from("hello"));
        assert_eq!(run(r#"str(42)"#, &[]), Value::from("42"));
        assert_eq!(run(r#"str(3.14)"#, &[]), Value::from("3.14"));
        assert_eq!(run(r#"str(true)"#, &[]), Value::from("true"));
        assert_eq!(run(r#"str(false)"#, &[]), Value::from("false"));
        assert_eq!(run(r#"str(null)"#, &[]), Value::from("null"));
        // int()
        assert_eq!(run(r#"int(42)"#, &[]), Value::from(42i64));
        assert_eq!(run(r#"int(3.9)"#, &[]), Value::from(3i64));
        assert_eq!(run(r#"int(-3.9)"#, &[]), Value::from(-3i64));
        assert_eq!(run(r#"int("42")"#, &[]), Value::from(42i64));
        assert_eq!(run(r#"int(true)"#, &[]), Value::from(1i64));
        assert_eq!(run(r#"int(false)"#, &[]), Value::from(0i64));
        // float()
        assert_eq!(run(r#"float(42)"#, &[]), Value::from(42.0f64));
        assert_eq!(run(r#"float(3.14)"#, &[]), Value::from(3.14f64));
        assert_eq!(run(r#"float("3.14")"#, &[]), Value::from(3.14f64));
        assert_eq!(run(r#"float(true)"#, &[]), Value::from(1.0f64));
        assert_eq!(run(r#"float(false)"#, &[]), Value::from(0.0f64));
        // bool()
        assert_eq!(run(r#"bool(1)"#, &[]), Value::from(true));
        assert_eq!(run(r#"bool(0)"#, &[]), Value::from(false));
        assert_eq!(run(r#"bool("")"#, &[]), Value::from(false));
        assert_eq!(run(r#"bool("x")"#, &[]), Value::from(true));
        assert_eq!(run(r#"bool(null)"#, &[]), Value::from(false));
        // list()
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
    fn test_var() {
        // unknown var with resolver → Null
        assert_eq!(run("missing", &[]), Value::Null);
        // no resolver registered → error
        let ctx = Context::new();
        assert!(eval(&parse("missing").unwrap(), &ctx).is_err());
    }

    #[test]
    fn test_native_func() {
        let mut ctx = Context::new();
        ctx.register(
            "join_path",
            Box::new(|args| {
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
                r#"feature["extension"] == "gml" && feature["package"] in packages"#,
                &[("feature", feature), ("packages", pkgs)]
            ),
            Value::from(true)
        );
    }
}
