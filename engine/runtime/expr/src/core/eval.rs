use std::collections::HashMap;

use indexmap::IndexMap;

use super::ast::{BinOp, Expr, UnaryOp};
use super::builtins::PathObject;
use super::error::{Error, Result};
use super::value::Value;

pub type NativeFn = Box<dyn Fn(&[Value]) -> Result<Value> + Send + Sync>;

pub struct Context {
    funcs: HashMap<String, NativeFn>,
}

impl Context {
    pub fn new() -> Self {
        let mut ctx = Self { funcs: HashMap::new() };
        ctx.register("map", Box::new(builtin_map));
        ctx.register("Path", Box::new(|args| {
            let s = args.first().and_then(|v| {
                if let Value::String(s) = v { Some(s.clone()) } else { None }
            }).unwrap_or_default();
            Ok(Value::Object(Box::new(PathObject(s))))
        }));
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

pub fn eval(expr: &Expr, ctx: &Context) -> Result<Value> {
    match expr {
        Expr::Null => Ok(Value::Null),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Int(n) => Ok(Value::Number((*n).into())),
        Expr::Float(f) => Ok(serde_json::Number::from_f64(*f)
            .map(Value::Number)
            .unwrap_or(Value::Null)),
        Expr::Str(s) => Ok(Value::String(s.clone())),
        Expr::Array(items) => {
            let values: Result<Vec<_>> = items.iter().map(|e| eval(e, ctx)).collect();
            Ok(Value::Array(values?))
        }
        Expr::Var(name) => ctx.call("__resolve", vec![Value::String(name.clone())]),
        Expr::Index(target, key) => {
            let target = eval(target, ctx)?;
            let key = eval(key, ctx)?;
            eval_index(target, key)
        }
        Expr::Slice { target, start, stop, step } => {
            let target = eval(target, ctx)?;
            let start = start.as_deref().map(|e| eval(e, ctx)).transpose()?;
            let stop = stop.as_deref().map(|e| eval(e, ctx)).transpose()?;
            let step = step.as_deref().map(|e| eval(e, ctx)).transpose()?;
            eval_slice(target, start, stop, step)
        }
        Expr::FuncCall { name, args } => {
            let args: Result<Vec<_>> = args.iter().map(|e| eval(e, ctx)).collect();
            ctx.call(name, args?)
        }
        Expr::Unary(op, expr) => {
            let val = eval(expr, ctx)?;
            eval_unary(op, val)
        }
        Expr::MethodCall { receiver, method, args } => {
            let recv = eval(receiver, ctx)?;
            let evaled: Result<Vec<_>> = args.iter().map(|e| eval(e, ctx)).collect();
            eval_method(recv, method, &evaled?)
        }
        Expr::Binary(left, op, right) => {
            match op {
                BinOp::And => {
                    let l = eval(left, ctx)?;
                    if !is_truthy(&l) {
                        return Ok(Value::Bool(false));
                    }
                    let r = eval(right, ctx)?;
                    return Ok(Value::Bool(is_truthy(&r)));
                }
                BinOp::Or => {
                    let l = eval(left, ctx)?;
                    if is_truthy(&l) {
                        return Ok(Value::Bool(true));
                    }
                    let r = eval(right, ctx)?;
                    return Ok(Value::Bool(is_truthy(&r)));
                }
                _ => {}
            }
            let left = eval(left, ctx)?;
            let right = eval(right, ctx)?;
            eval_binary(op, left, right)
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
            Ok(Value::Number((s.chars().count() as i64).into()))
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
            Ok(Value::Number((a.len() as i64).into()))
        }
        m => Err(Error::Eval {
            msg: format!("Array has no method '{m}'"),
        }),
    }
}

fn eval_index(target: Value, key: Value) -> Result<Value> {
    match (target, key) {
        (Value::Map(map), Value::String(k)) => Ok(map.get(&k).cloned().unwrap_or(Value::Null)),
        (Value::Array(arr), Value::Number(n)) => {
            let i = n.as_i64().ok_or_else(|| Error::Eval {
                msg: "array index must be an integer".into(),
            })?;
            let len = arr.len() as i64;
            let i = if i < 0 { len + i } else { i };
            if i < 0 || i >= len {
                return Ok(Value::Null);
            }
            Ok(arr.into_iter().nth(i as usize).unwrap_or(Value::Null))
        }
        (Value::String(s), Value::Number(n)) => {
            let i = n.as_i64().ok_or_else(|| Error::Eval {
                msg: "string index must be an integer".into(),
            })?;
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
        Value::Number(n) => n.as_i64().ok_or_else(|| Error::Eval {
            msg: format!("slice {what} must be an integer"),
        }),
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

fn eval_slice(target: Value, start: Option<Value>, stop: Option<Value>, step: Option<Value>) -> Result<Value> {
    let step = match step {
        None => 1i64,
        Some(v) => as_slice_index(v, "step")?,
    };
    if step == 0 {
        return Err(Error::Eval { msg: "slice step cannot be zero".into() });
    }
    let start = start.map(|v| as_slice_index(v, "start")).transpose()?;
    let stop = stop.map(|v| as_slice_index(v, "stop")).transpose()?;

    match target {
        Value::Array(arr) => {
            let indices = slice_indices(arr.len(), start, stop, step);
            Ok(Value::Array(indices.into_iter().map(|i| arr[i].clone()).collect()))
        }
        Value::String(s) => {
            let chars: Vec<char> = s.chars().collect();
            let indices = slice_indices(chars.len(), start, stop, step);
            Ok(Value::String(indices.into_iter().map(|i| chars[i]).collect()))
        }
        v => Err(Error::Eval { msg: format!("cannot slice {v:?}") }),
    }
}

fn eval_unary(op: &UnaryOp, val: Value) -> Result<Value> {
    match op {
        UnaryOp::Not => Ok(Value::Bool(!is_truthy(&val))),
        UnaryOp::Neg => match val {
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(Value::Number((-i).into()))
                } else if let Some(f) = n.as_f64() {
                    Ok(serde_json::Number::from_f64(-f)
                        .map(Value::Number)
                        .unwrap_or(Value::Null))
                } else {
                    Err(Error::Eval {
                        msg: "cannot negate value".into(),
                    })
                }
            }
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
        BinOp::Eq  => Some("__eq__"),
        BinOp::Ne  => Some("__ne__"),
        BinOp::Lt  => Some("__lt__"),
        BinOp::Le  => Some("__le__"),
        BinOp::Gt  => Some("__gt__"),
        BinOp::Ge  => Some("__ge__"),
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

fn eval_binary(op: &BinOp, left: Value, right: Value) -> Result<Value> {
    match op {
        BinOp::Add => match (left, right) {
            (Value::Number(a), Value::Number(b)) => add_numbers(a, b),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + b.as_str())),
            (Value::String(a), b) => Ok(Value::String(a + value_to_string(&b).as_str())),
            (Value::Array(mut a), Value::Array(b)) => {
                a.extend(b);
                Ok(Value::Array(a))
            }
            (a, b) => try_object_op(op, a, b),
        },
        BinOp::Sub => numeric_op(left, right, |a, b| a - b, |a, b| a - b),
        BinOp::Mul => numeric_op(left, right, |a, b| a * b, |a, b| a * b),
        BinOp::Div => match (left, right) {
            (Value::Number(a), Value::Number(b)) => {
                let b_f = b.as_f64().unwrap_or(0.0);
                if b_f == 0.0 {
                    return Err(Error::Eval {
                        msg: "division by zero".into(),
                    });
                }
                let a_f = a.as_f64().unwrap_or(0.0);
                Ok(serde_json::Number::from_f64(a_f / b_f)
                    .map(Value::Number)
                    .unwrap_or(Value::Null))
            }
            (a, b) => try_object_op(op, a, b),
        },
        BinOp::Eq => Ok(Value::Bool(values_equal(&left, &right))),
        BinOp::Ne => Ok(Value::Bool(!values_equal(&left, &right))),
        BinOp::Lt => compare_values(left, right, |o| o == std::cmp::Ordering::Less),
        BinOp::Le => compare_values(left, right, |o| o != std::cmp::Ordering::Greater),
        BinOp::Gt => compare_values(left, right, |o| o == std::cmp::Ordering::Greater),
        BinOp::Ge => compare_values(left, right, |o| o != std::cmp::Ordering::Less),
        BinOp::In => match (left, right) {
            (left, Value::Array(arr)) => Ok(Value::Bool(arr.iter().any(|v| values_equal(v, &left)))),
            (Value::String(needle), Value::String(haystack)) => Ok(Value::Bool(haystack.contains(needle.as_str()))),
            (Value::String(key), Value::Map(map)) => Ok(Value::Bool(map.contains_key(&key))),
            (_, Value::Null) => Ok(Value::Bool(false)),
            (l, r) => Err(Error::Eval {
                msg: format!("'in' not supported between {l:?} and {r:?}"),
            }),
        },
        BinOp::And | BinOp::Or => unreachable!("handled with short-circuit above"),
    }
}

fn add_numbers(a: serde_json::Number, b: serde_json::Number) -> Result<Value> {
    if let (Some(ai), Some(bi)) = (a.as_i64(), b.as_i64()) {
        return Ok(Value::Number((ai + bi).into()));
    }
    let af = a.as_f64().unwrap_or(0.0);
    let bf = b.as_f64().unwrap_or(0.0);
    Ok(serde_json::Number::from_f64(af + bf)
        .map(Value::Number)
        .unwrap_or(Value::Null))
}

fn numeric_op(
    left: Value,
    right: Value,
    int_op: impl Fn(i64, i64) -> i64,
    float_op: impl Fn(f64, f64) -> f64,
) -> Result<Value> {
    match (left, right) {
        (Value::Number(a), Value::Number(b)) => {
            if let (Some(ai), Some(bi)) = (a.as_i64(), b.as_i64()) {
                return Ok(Value::Number(int_op(ai, bi).into()));
            }
            let af = a.as_f64().unwrap_or(0.0);
            let bf = b.as_f64().unwrap_or(0.0);
            Ok(serde_json::Number::from_f64(float_op(af, bf))
                .map(Value::Number)
                .unwrap_or(Value::Null))
        }
        (a, b) => Err(Error::Eval {
            msg: format!("cannot apply numeric op to {a:?} and {b:?}"),
        }),
    }
}

fn compare_values(
    left: Value,
    right: Value,
    pred: impl Fn(std::cmp::Ordering) -> bool,
) -> Result<Value> {
    let ord = match (&left, &right) {
        (Value::Number(a), Value::Number(b)) => a
            .as_f64()
            .partial_cmp(&b.as_f64())
            .unwrap_or(std::cmp::Ordering::Equal),
        (Value::String(a), Value::String(b)) => a.cmp(b),
        _ => {
            return Err(Error::Eval {
                msg: format!("cannot compare {left:?} and {right:?}"),
            })
        }
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
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
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
        Value::Number(n) => n.to_string(),
        Value::Array(_) | Value::Map(_) => format!("{v:?}"),
        Value::Object(obj) => obj.display(),
    }
}

fn builtin_map(args: &[Value]) -> Result<Value> {
    let pairs = match args.first() {
        Some(Value::Array(a)) => a,
        _ => return Err(Error::Eval { msg: "map() expects an array of [key, value] pairs".into() }),
    };
    let mut out = IndexMap::new();
    for (i, pair) in pairs.iter().enumerate() {
        match pair {
            Value::Array(kv) if kv.len() == 2 => {
                let key = match &kv[0] {
                    Value::String(s) => s.clone(),
                    v => return Err(Error::Eval { msg: format!("map() key at index {i} must be a string, got {v:?}") }),
                };
                out.insert(key, kv[1].clone());
            }
            _ => return Err(Error::Eval { msg: format!("map() entry at index {i} must be a 2-element array") }),
        }
    }
    Ok(Value::Map(out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::parser::parse;

    fn ctx_with_vars(vars: &[(&str, Value)]) -> Context {
        use std::sync::Arc;
        let map: HashMap<String, Value> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();
        let map = Arc::new(map);
        let mut ctx = Context::new();
        ctx.register(
            "__resolve",
            Box::new(move |args| {
                let name = args.first().and_then(|v| {
                    if let Value::String(s) = v { Some(s.as_str()) } else { None }
                }).unwrap_or("");
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
    }

    #[test]
    fn test_string_concat() {
        assert_eq!(
            run(r#""hello" + "_" + "world""#, &[]),
            Value::from("hello_world")
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
    fn test_var_and_index() {
        let feature = Value::Map(indexmap::indexmap! {
            "package".into() => Value::from("bldg"),
            "extension".into() => Value::from("gml"),
        });
        assert_eq!(
            run(r#"feature["package"]"#, &[("feature", feature.clone())]),
            Value::from("bldg")
        );
        assert_eq!(
            run(r#"feature["extension"] == "gml""#, &[("feature", feature)]),
            Value::from(true)
        );
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
    fn test_nested_index() {
        let data = Value::Map(indexmap::indexmap! {
            "cityGmlPath".into() => Value::from("/data/city.gml"),
        });
        assert_eq!(
            run(r#"value["cityGmlPath"]"#, &[("value", data)]),
            Value::from("/data/city.gml")
        );
    }

    #[test]
    fn test_native_func() {
        let mut ctx = Context::new();
        ctx.register(
            "join_path",
            Box::new(|args| {
                let parts: Vec<&str> = args.iter().map(|v| {
                    if let Value::String(s) = v { s.as_str() } else { "" }
                }).collect();
                Ok(Value::String(parts.join("/")))
            }),
        );
        assert_eq!(
            eval(&parse(r#"join_path("base", "file.json")"#).unwrap(), &ctx).unwrap(),
            Value::from("base/file.json")
        );
    }

    #[test]
    fn test_unknown_var_returns_null() {
        assert_eq!(run("missing", &[]), Value::Null);
    }

    #[test]
    fn test_no_resolver_errors() {
        let ctx = Context::new();
        let result = eval(&parse("missing").unwrap(), &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_map_builtin() {
        let result = run(r#"map([["a", 1], ["b", 2]])"#, &[]);
        assert_eq!(result, Value::Map(indexmap::indexmap! {
            "a".into() => Value::from(1i64),
            "b".into() => Value::from(2i64),
        }));
    }

    #[test]
    fn test_string_trim() {
        assert_eq!(run(r#""  hello  ".trim()"#, &[]), Value::from("hello"));
    }

    #[test]
    fn test_negative_index() {
        assert_eq!(run(r#""abcde"[-1]"#, &[]), Value::from("e"));
        let arr = Value::Array(vec![Value::from(1i64), Value::from(2i64), Value::from(3i64)]);
        assert_eq!(run("arr[-1]", &[("arr", arr)]), Value::from(3i64));
    }

    #[test]
    fn test_string_index() {
        assert_eq!(run(r#""hello"[0]"#, &[]), Value::from("h"));
        assert_eq!(run(r#""hello"[4]"#, &[]), Value::from("o"));
        assert_eq!(run(r#""hello"[-1]"#, &[]), Value::from("o"));
    }

    #[test]
    fn test_slice_str() {
        assert_eq!(run(r#""abcde"[1:3]"#, &[]), Value::from("bc"));
        assert_eq!(run(r#""abcde"[:3]"#, &[]), Value::from("abc"));
        assert_eq!(run(r#""abcde"[2:]"#, &[]), Value::from("cde"));
        assert_eq!(run(r#""abcde"[:]"#, &[]), Value::from("abcde"));
        assert_eq!(run(r#""abcde"[::-1]"#, &[]), Value::from("edcba"));
        assert_eq!(run(r#""abcde"[-1::-2]"#, &[]), Value::from("eca"));
        assert_eq!(run(r#""abcde"[::2]"#, &[]), Value::from("ace"));
    }

    #[test]
    fn test_slice_array() {
        let arr = Value::Array(vec![
            Value::from(0i64), Value::from(1i64), Value::from(2i64),
            Value::from(3i64), Value::from(4i64),
        ]);
        assert_eq!(
            run("arr[1:3]", &[("arr", arr.clone())]),
            Value::Array(vec![Value::from(1i64), Value::from(2i64)])
        );
        assert_eq!(
            run("arr[::-1]", &[("arr", arr.clone())]),
            Value::Array(vec![Value::from(4i64), Value::from(3i64), Value::from(2i64), Value::from(1i64), Value::from(0i64)])
        );
        assert_eq!(
            run("arr[-2:]", &[("arr", arr)]),
            Value::Array(vec![Value::from(3i64), Value::from(4i64)])
        );
    }

    #[test]
    fn test_len_method() {
        assert_eq!(run(r#""hello".len()"#, &[]), Value::from(5i64));
        assert_eq!(run(r#""".len()"#, &[]), Value::from(0i64));
        let arr = Value::Array(vec![Value::from(1i64), Value::from(2i64), Value::from(3i64)]);
        assert_eq!(run("arr.len()", &[("arr", arr)]), Value::from(3i64));
    }

    #[test]
    fn test_array_concat() {
        let a = Value::Array(vec![Value::from(1i64), Value::from(2i64)]);
        let b = Value::Array(vec![Value::from(3i64), Value::from(4i64)]);
        assert_eq!(
            run("a + b", &[("a", a), ("b", b)]),
            Value::Array(vec![Value::from(1i64), Value::from(2i64), Value::from(3i64), Value::from(4i64)])
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
