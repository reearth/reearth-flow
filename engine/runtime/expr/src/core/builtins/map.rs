use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::LazyLock;

use indexmap::IndexMap;

use crate::core::error::{InnerError, InnerResult};
use crate::core::eval::eval_eq;
use crate::core::value::{NativeFn, Value};
use crate::unpack_args;

use super::MethodFn;

static METHODS: LazyLock<HashMap<&'static str, MethodFn>> = LazyLock::new(|| {
    HashMap::from([
        ("keys", keys as MethodFn),
        ("values", values as MethodFn),
        ("items", items as MethodFn),
        ("get", get as MethodFn),
    ])
});

pub fn resolve_method(recv: Value, method: &str) -> InnerResult<NativeFn> {
    let f = METHODS
        .get(method)
        .copied()
        .ok_or_else(|| InnerError::new(format!("Map has no method '{method}'")))?;
    Ok(NativeFn::new(move |args| {
        let mut a = vec![recv.clone()];
        a.extend_from_slice(args);
        f(&a)
    }))
}

fn keys(args: &[Value]) -> InnerResult<Value> {
    unpack_args!(args => recv);
    let Value::Map(rc) = recv else {
        return Err(InnerError::new("expected map receiver"));
    };
    Ok(Value::array(
        rc.borrow()
            .keys()
            .map(|k| Value::String(k.clone()))
            .collect(),
    ))
}

fn values(args: &[Value]) -> InnerResult<Value> {
    unpack_args!(args => recv);
    let Value::Map(rc) = recv else {
        return Err(InnerError::new("expected map receiver"));
    };
    Ok(Value::array(rc.borrow().values().cloned().collect()))
}

fn items(args: &[Value]) -> InnerResult<Value> {
    unpack_args!(args => recv);
    let Value::Map(rc) = recv else {
        return Err(InnerError::new("expected map receiver"));
    };
    Ok(Value::array(
        rc.borrow()
            .iter()
            .map(|(k, v)| Value::array(vec![Value::String(k.clone()), v.clone()]))
            .collect(),
    ))
}

fn get(args: &[Value]) -> InnerResult<Value> {
    let (recv, key, fallback) = match args {
        [recv, key] => (recv, key, None),
        [recv, key, fallback] => (recv, key, Some(fallback)),
        _ => return Err(InnerError::new("map.get() requires 1 or 2 arguments")),
    };
    let Value::Map(rc) = recv else {
        return Err(InnerError::new("expected map receiver"));
    };
    let Value::String(k) = key else {
        return Err(InnerError::new("map.get() key must be a string"));
    };
    Ok(rc
        .borrow()
        .get(k.as_str())
        .cloned()
        .unwrap_or_else(|| fallback.cloned().unwrap_or(Value::Null)))
}

pub fn eq_inner(
    a: &Rc<RefCell<IndexMap<String, Value>>>,
    b: &Rc<RefCell<IndexMap<String, Value>>>,
) -> InnerResult<bool> {
    if Rc::ptr_eq(a, b) {
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
        let m = Value::map(indexmap::indexmap! {
            "x".into() => Value::from(1i64),
            "y".into() => Value::from(2i64),
        });
        assert_eval("len(m)", &[("m", m)], Value::from(2i64));
    }

    #[test]
    fn test_keys() {
        let m = Value::map(indexmap::indexmap! {
            "x".into() => Value::from(1i64),
            "y".into() => Value::from(2i64),
        });
        assert_eval("m.keys()", &[("m", m)], Value::from(vec!["x", "y"]));
    }

    #[test]
    fn test_values() {
        let m = Value::map(indexmap::indexmap! {
            "x".into() => Value::from(1i64),
            "y".into() => Value::from(2i64),
        });
        assert_eval("m.values()", &[("m", m)], Value::from(vec![1i64, 2i64]));
    }

    #[test]
    fn test_get() {
        let m = Value::map(indexmap::indexmap! {
            "x".into() => Value::from(1i64),
        });
        assert_eval("m.get(\"x\")", &[("m", m.clone())], Value::from(1i64));
        assert_eval("m.get(\"z\")", &[("m", m.clone())], Value::Null);
        assert_eval("m.get(\"z\", 42)", &[("m", m.clone())], Value::from(42i64));
        assert_eval("m.get(\"x\", 99)", &[("m", m)], Value::from(1i64));
    }

    #[test]
    fn test_items() {
        let m = Value::map(indexmap::indexmap! {
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
