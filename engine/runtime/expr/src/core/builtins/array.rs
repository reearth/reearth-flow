use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::LazyLock;

use crate::core::error::{InnerError, InnerResult};
use crate::core::eval::eval_eq;
use crate::core::value::{NativeFn, Value};

use super::{expect_arity, expect_int, MethodFn};

static METHODS: LazyLock<HashMap<&'static str, MethodFn>> =
    LazyLock::new(|| HashMap::from([("get", get as MethodFn)]));

pub fn resolve_method(recv: Value, method: &str) -> InnerResult<NativeFn> {
    let f = METHODS
        .get(method)
        .copied()
        .ok_or_else(|| InnerError::new(format!("Array has no method '{method}'")))?;
    Ok(NativeFn::new(move |args| {
        let mut a = vec![recv.clone()];
        a.extend_from_slice(args);
        f(&a)
    }))
}

/// Resolves a possibly-negative index into a concrete `usize`, or `None` if out of bounds.
pub fn resolve_index(i: i64, len: usize) -> Option<usize> {
    let pos = if i >= 0 {
        i as usize
    } else {
        len.checked_sub(i.unsigned_abs() as usize)?
    };
    (pos < len).then_some(pos)
}

fn get(args: &[Value]) -> InnerResult<Value> {
    expect_arity(args, 1, 2)?;
    let Value::Array(rc) = &args[0] else {
        return Err(InnerError::new("expected array receiver"));
    };
    let i = expect_int(&args[1])?;
    let fallback = args.get(2);
    let arr = rc.borrow();
    let elem = resolve_index(i, arr.len()).map(|pos| arr[pos].clone());
    Ok(elem.unwrap_or_else(|| fallback.cloned().unwrap_or(Value::Null)))
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
        assert_eval("len(arr)", &[("arr", arr)], Value::from(3i64));
    }

    #[test]
    fn test_get() {
        let arr = Value::from(vec![10i64, 20i64, 30i64]);
        assert_eval("arr.get(0)", &[("arr", arr.clone())], Value::from(10i64));
        assert_eval("arr.get(2)", &[("arr", arr.clone())], Value::from(30i64));
        assert_eval("arr.get(-1)", &[("arr", arr.clone())], Value::from(30i64));
        assert_eval("arr.get(-3)", &[("arr", arr.clone())], Value::from(10i64));
        assert_eval("arr.get(5)", &[("arr", arr.clone())], Value::Null);
        assert_eval("arr.get(-5)", &[("arr", arr.clone())], Value::Null);
        assert_eval(
            "arr.get(5, 99)",
            &[("arr", arr.clone())],
            Value::from(99i64),
        );
        assert_eval(
            "arr.get(1, 99)",
            &[("arr", arr.clone())],
            Value::from(20i64),
        );
    }
}
