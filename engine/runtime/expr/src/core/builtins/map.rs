use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::LazyLock;

use indexmap::IndexMap;

use crate::core::error::{InnerError, InnerResult};
use crate::core::eval::eval_eq;
use crate::core::value::{NativeFn, Value};
use crate::unpack_args;

type MethodFn = fn(&[Value]) -> InnerResult<Value>;

static METHODS: LazyLock<HashMap<&'static str, MethodFn>> = LazyLock::new(|| {
    HashMap::from([
        ("len", len as MethodFn),
        ("keys", keys as MethodFn),
        ("values", values as MethodFn),
        ("items", items as MethodFn),
    ])
});

pub fn resolve_method(method: &str) -> InnerResult<NativeFn> {
    METHODS
        .get(method)
        .map(|&f| NativeFn::new(f))
        .ok_or_else(|| InnerError::new(format!("Map has no method '{method}'")))
}

fn len(args: &[Value]) -> InnerResult<Value> {
    unpack_args!(args => recv);
    let Value::Map(rc) = recv else {
        return Err(InnerError::new("expected map receiver"));
    };
    Ok(Value::Int(rc.borrow().len() as i64))
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
