use std::collections::HashMap;
use std::rc::Rc;
use std::sync::LazyLock;

use crate::core::error::{InnerError, InnerResult};
use crate::core::eval::eval_eq;
use crate::core::value::{NativeFn, Value};
use crate::unpack_args;

type MethodFn = fn(&[Value]) -> InnerResult<Value>;

static METHODS: LazyLock<HashMap<&'static str, MethodFn>> =
    LazyLock::new(|| HashMap::from([("len", len as MethodFn), ("__eq__", eq as MethodFn)]));

pub fn resolve_method(method: &str) -> InnerResult<NativeFn> {
    METHODS
        .get(method)
        .map(|&f| NativeFn::new(f))
        .ok_or_else(|| InnerError::new(format!("Array has no method '{method}'")))
}

fn len(args: &[Value]) -> InnerResult<Value> {
    unpack_args!(args => recv);
    let Value::Array(rc) = recv else {
        return Err(InnerError::new("expected array receiver"));
    };
    Ok(Value::Int(rc.borrow().len() as i64))
}

pub fn eq(args: &[Value]) -> InnerResult<Value> {
    match (args.get(0), args.get(1)) {
        (Some(Value::Array(a)), Some(Value::Array(b))) => {
            if Rc::ptr_eq(a, b) {
                return Ok(Value::Bool(true));
            }
            let a = a.borrow();
            let b = b.borrow();
            if a.len() != b.len() {
                return Ok(Value::Bool(false));
            }
            for (x, y) in a.iter().zip(b.iter()) {
                if !eval_eq(x.clone(), y.clone())? {
                    return Ok(Value::Bool(false));
                }
            }
            Ok(Value::Bool(true))
        }
        _ => Ok(Value::Bool(false)),
    }
}
