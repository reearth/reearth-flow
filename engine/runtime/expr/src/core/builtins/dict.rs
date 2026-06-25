use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::LazyLock;

use indexmap::IndexMap;

use crate::core::error::{eval_error, Result};
use crate::core::eval::eval_eq;
use crate::core::value::{NativeFn, TrackedRc, Value};
use crate::expect_arity;

use super::MethodFn;

static METHODS: LazyLock<HashMap<&'static str, MethodFn>> = LazyLock::new(|| {
    HashMap::from([
        ("keys", keys as MethodFn),
        ("values", values as MethodFn),
        ("items", items as MethodFn),
        ("get", get as MethodFn),
        ("pop", pop as MethodFn),
        ("update", update as MethodFn),
    ])
});

pub fn resolve_method(recv: Value, method: &str) -> Result<NativeFn> {
    let f = METHODS
        .get(method)
        .copied()
        .ok_or_else(|| eval_error(format!("dict has no method '{method}'")))?;
    Ok(NativeFn::new(move |args| {
        let mut a = vec![recv.clone()];
        a.extend_from_slice(args);
        f(&a)
    }))
}

fn keys(args: &[Value]) -> Result<Value> {
    expect_arity("dict.keys", &args[1..], 0, 0)?;
    let Value::Dict(rc) = &args[0] else {
        return Err(eval_error("expected dict receiver"));
    };
    Ok(Value::list(
        rc.borrow()
            .keys()
            .map(|k| Value::String(k.clone()))
            .collect(),
    ))
}

fn values(args: &[Value]) -> Result<Value> {
    expect_arity("dict.values", &args[1..], 0, 0)?;
    let Value::Dict(rc) = &args[0] else {
        return Err(eval_error("expected dict receiver"));
    };
    Ok(Value::list(rc.borrow().values().cloned().collect()))
}

fn items(args: &[Value]) -> Result<Value> {
    expect_arity("dict.items", &args[1..], 0, 0)?;
    let Value::Dict(rc) = &args[0] else {
        return Err(eval_error("expected dict receiver"));
    };
    Ok(Value::list(
        rc.borrow()
            .iter()
            .map(|(k, v)| Value::list(vec![Value::String(k.clone()), v.clone()]))
            .collect(),
    ))
}

fn get(args: &[Value]) -> Result<Value> {
    expect_arity("dict.get", &args[1..], 1, 2)?;
    let Value::Dict(rc) = &args[0] else {
        return Err(eval_error("expected dict receiver"));
    };
    let k = args[1].as_str()?;
    let fallback = args.get(2);
    Ok(rc
        .borrow()
        .get(k)
        .cloned()
        .unwrap_or_else(|| fallback.cloned().unwrap_or(Value::Null)))
}

fn pop(args: &[Value]) -> Result<Value> {
    expect_arity("dict.pop", &args[1..], 1, 1)?;
    let Value::Dict(rc) = &args[0] else {
        return Err(eval_error("expected dict receiver"));
    };
    let k = args[1].as_str()?;
    Ok(rc.borrow_mut().shift_remove(k).unwrap_or(Value::Null))
}

fn update(args: &[Value]) -> Result<Value> {
    expect_arity("dict.update", &args[1..], 1, 1)?;
    let Value::Dict(rc) = &args[0] else {
        return Err(eval_error("expected dict receiver"));
    };
    let Value::Dict(other) = &args[1] else {
        return Err(eval_error("update() argument must be a dict"));
    };
    if TrackedRc::ptr_eq(rc, other) {
        return Ok(Value::Null);
    }
    let mut dst = rc.borrow_mut();
    for (k, v) in other.borrow().iter() {
        dst.insert(k.clone(), v.clone());
    }
    Ok(Value::Null)
}

pub fn eq_inner(
    a: &TrackedRc<RefCell<IndexMap<String, Value>>>,
    b: &TrackedRc<RefCell<IndexMap<String, Value>>>,
) -> Result<bool> {
    if TrackedRc::ptr_eq(a, b) {
        return Ok(true);
    }
    let a = a.borrow();
    let b = b.borrow();
    if a.len() != b.len() {
        return Ok(false);
    }
    for (k, va) in a.iter() {
        match b.get(k) {
            Some(vb) => {
                if !eval_eq(va.clone(), vb.clone())? {
                    return Ok(false);
                }
            }
            None => return Ok(false),
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use crate::core::test_utils::assert_eval;
    use crate::core::value::Value;

    #[test]
    fn test_len() {
        let m = Value::dict(indexmap::indexmap! {
            "x".into() => Value::from(1i64),
            "y".into() => Value::from(2i64),
        });
        assert_eval("len(m)", &[("m", m)], Value::from(2i64));
    }

    #[test]
    fn test_keys() {
        let m = Value::dict(indexmap::indexmap! {
            "x".into() => Value::from(1i64),
            "y".into() => Value::from(2i64),
        });
        assert_eval("m.keys()", &[("m", m)], Value::from(vec!["x", "y"]));
    }

    #[test]
    fn test_values() {
        let m = Value::dict(indexmap::indexmap! {
            "x".into() => Value::from(1i64),
            "y".into() => Value::from(2i64),
        });
        assert_eval("m.values()", &[("m", m)], Value::from(vec![1i64, 2i64]));
    }

    #[test]
    fn test_get() {
        let m = Value::dict(indexmap::indexmap! {
            "x".into() => Value::from(1i64),
        });
        assert_eval("m.get(\"x\")", &[("m", m.clone())], Value::from(1i64));
        assert_eval("m.get(\"z\")", &[("m", m.clone())], Value::Null);
        assert_eval("m.get(\"z\", 42)", &[("m", m.clone())], Value::from(42i64));
        assert_eval("m.get(\"x\", 99)", &[("m", m)], Value::from(1i64));
    }

    #[test]
    fn test_pop() {
        let m = Value::dict(indexmap::indexmap! {
            "x".into() => Value::from(1i64),
            "y".into() => Value::from(2i64),
        });
        assert_eval("m.pop(\"x\")", &[("m", m.clone())], Value::from(1i64));
        assert_eval("m.pop(\"z\")", &[("m", m.clone())], Value::Null);
        assert_eval(
            "m.pop(\"x\"); m.keys()",
            &[("m", m)],
            Value::from(vec!["y"]),
        );
    }

    #[test]
    fn test_update() {
        let m = Value::dict(indexmap::indexmap! {
            "x".into() => Value::from(1i64),
        });
        assert_eval(
            "m.update({\"y\": 2, \"x\": 99}); m.get(\"x\")",
            &[("m", m.clone())],
            Value::from(99i64),
        );
        assert_eval(
            "m.update({\"y\": 2}); m.keys()",
            &[("m", m.clone())],
            Value::from(vec!["x", "y"]),
        );
        assert_eval(
            "m.update(m); m.get(\"x\")",
            &[("m", m)],
            Value::from(1i64),
        );
    }

    #[test]
    fn test_items() {
        let m = Value::dict(indexmap::indexmap! {
            "x".into() => Value::from(1i64),
            "y".into() => Value::from(2i64),
        });
        assert_eval(
            "m.items()",
            &[("m", m)],
            Value::from(vec![
                Value::from(vec![Value::from("x"), Value::from(1i64)]),
                Value::from(vec![Value::from("y"), Value::from(2i64)]),
            ]),
        );
    }
}
