use std::collections::HashMap;

use serde_json::Value;

use super::ast::{BinOp, Expr, UnaryOp};
use super::error::{Error, Result};

pub type NativeFn = Box<dyn Fn(&[Value]) -> Result<Value> + Send + Sync>;

pub struct Context {
    funcs: HashMap<String, NativeFn>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            funcs: HashMap::new(),
        }
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
        Expr::FuncCall { name, args } => {
            let args: Result<Vec<_>> = args.iter().map(|e| eval(e, ctx)).collect();
            ctx.call(name, args?)
        }
        Expr::Unary(op, expr) => {
            let val = eval(expr, ctx)?;
            eval_unary(op, val)
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

fn eval_index(target: Value, key: Value) -> Result<Value> {
    match (target, key) {
        (Value::Object(map), Value::String(k)) => Ok(map.get(&k).cloned().unwrap_or(Value::Null)),
        (Value::Array(arr), Value::Number(n)) => {
            let i = n.as_i64().ok_or_else(|| Error::Eval {
                msg: "array index must be an integer".into(),
            })?;
            let i = if i < 0 { arr.len() as i64 + i } else { i } as usize;
            Ok(arr.into_iter().nth(i).unwrap_or(Value::Null))
        }
        (Value::Null, _) => Ok(Value::Null),
        (target, key) => Err(Error::Eval {
            msg: format!("cannot index {target:?} with {key:?}"),
        }),
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

fn eval_binary(op: &BinOp, left: Value, right: Value) -> Result<Value> {
    match op {
        BinOp::Add => match (left, right) {
            (Value::Number(a), Value::Number(b)) => add_numbers(a, b),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + b.as_str())),
            (Value::String(a), b) => Ok(Value::String(a + json_to_string(&b).as_str())),
            (a, b) => Err(Error::Eval {
                msg: format!("cannot add {a:?} and {b:?}"),
            }),
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
            (a, b) => Err(Error::Eval {
                msg: format!("cannot divide {a:?} by {b:?}"),
            }),
        },
        BinOp::Eq => Ok(Value::Bool(values_equal(&left, &right))),
        BinOp::Ne => Ok(Value::Bool(!values_equal(&left, &right))),
        BinOp::Lt => compare_values(left, right, |o| o == std::cmp::Ordering::Less),
        BinOp::Le => compare_values(left, right, |o| o != std::cmp::Ordering::Greater),
        BinOp::Gt => compare_values(left, right, |o| o == std::cmp::Ordering::Greater),
        BinOp::Ge => compare_values(left, right, |o| o != std::cmp::Ordering::Less),
        BinOp::In => match right {
            Value::Array(arr) => Ok(Value::Bool(arr.iter().any(|v| values_equal(v, &left)))),
            Value::Null => Ok(Value::Bool(false)),
            r => Err(Error::Eval {
                msg: format!("'in' requires an array, got {r:?}"),
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
        Value::Object(o) => !o.is_empty(),
    }
}

fn json_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        _ => v.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::parser::parse;
    use serde_json::json;

    fn ctx_with_vars(vars: &[(&str, Value)]) -> Context {
        use std::sync::Arc;
        let map: std::collections::HashMap<String, Value> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();
        let map = Arc::new(map);
        let mut ctx = Context::new();
        ctx.register(
            "__resolve",
            Box::new(move |args| {
                let name = args.first().and_then(|v| v.as_str()).unwrap_or("");
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
        assert_eq!(run("1 + 2", &[]), json!(3));
        assert_eq!(run("10 - 3", &[]), json!(7));
        assert_eq!(run("2 * 5", &[]), json!(10));
        assert_eq!(run("10 / 4", &[]), json!(2.5));
    }

    #[test]
    fn test_string_concat() {
        assert_eq!(run(r#""hello" + "_" + "world""#, &[]), json!("hello_world"));
    }

    #[test]
    fn test_comparison() {
        assert_eq!(run("1 == 1", &[]), json!(true));
        assert_eq!(run("1 != 2", &[]), json!(true));
        assert_eq!(run("2 > 1", &[]), json!(true));
        assert_eq!(run("1 >= 1", &[]), json!(true));
    }

    #[test]
    fn test_logical() {
        assert_eq!(run("true && false", &[]), json!(false));
        assert_eq!(run("true || false", &[]), json!(true));
        assert_eq!(run("!true", &[]), json!(false));
    }

    #[test]
    fn test_var_and_index() {
        let feature = json!({"package": "bldg", "extension": "gml"});
        assert_eq!(
            run(r#"feature["package"]"#, &[("feature", feature.clone())]),
            json!("bldg")
        );
        assert_eq!(
            run(r#"feature["extension"] == "gml""#, &[("feature", feature)]),
            json!(true)
        );
    }

    #[test]
    fn test_in_operator() {
        let pkgs = json!(["bldg", "tran"]);
        assert_eq!(
            run(r#""bldg" in packages"#, &[("packages", pkgs.clone())]),
            json!(true)
        );
        assert_eq!(
            run(r#""fld" in packages"#, &[("packages", pkgs)]),
            json!(false)
        );
    }

    #[test]
    fn test_nested_index() {
        let data = json!({"cityGmlPath": "/data/city.gml"});
        assert_eq!(
            run(r#"value["cityGmlPath"]"#, &[("value", data)]),
            json!("/data/city.gml")
        );
    }

    #[test]
    fn test_native_func() {
        let mut ctx = Context::new();
        ctx.register(
            "join_path",
            Box::new(|args| {
                let parts: Vec<&str> = args.iter().map(|v| v.as_str().unwrap_or("")).collect();
                Ok(Value::String(parts.join("/")))
            }),
        );
        assert_eq!(
            eval(&parse(r#"join_path("base", "file.json")"#).unwrap(), &ctx).unwrap(),
            json!("base/file.json")
        );
    }

    #[test]
    fn test_unknown_var_returns_null() {
        // resolver returns Null for unknown names — no silent string fallback,
        // but missing attributes resolve to Null (consistent with JSON semantics)
        assert_eq!(run("missing", &[]), Value::Null);
    }

    #[test]
    fn test_no_resolver_errors() {
        // if no var resolver is registered at all, Var access is an error
        let ctx = Context::new();
        let result = eval(&parse("missing").unwrap(), &ctx);
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_expr() {
        let feature = json!({"extension": "gml", "package": "bldg"});
        let pkgs = json!(["bldg", "tran"]);
        assert_eq!(
            run(
                r#"feature["extension"] == "gml" && feature["package"] in packages"#,
                &[("feature", feature), ("packages", pkgs)]
            ),
            json!(true)
        );
    }
}
