use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::LazyLock;

use crate::core::error::{InnerError, InnerResult};
use crate::core::eval::eval_eq;
use crate::core::value::{NativeFn, Value};
use crate::unpack_args;

type MethodFn = fn(&[Value]) -> InnerResult<Value>;

static METHODS: LazyLock<HashMap<&'static str, MethodFn>> =
    LazyLock::new(|| HashMap::from([("len", len as MethodFn)]));

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

pub fn eq_inner(a: &Rc<RefCell<Vec<Value>>>, b: &Rc<RefCell<Vec<Value>>>) -> InnerResult<bool> {
    if Rc::ptr_eq(a, b) {
        return Ok(true);
    }
    let a = a.borrow();
    let b = b.borrow();
    if a.len() != b.len() {
        return Ok(false);
    }
    for (x, y) in a.iter().zip(b.iter()) {
        if !eval_eq(x.clone(), y.clone())? {
            return Ok(false);
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
        let arr = Value::from(vec![1i64, 2i64, 3i64]);
        assert_eval("arr.len()", &[("arr", arr)], Value::from(3i64));
    }
}
