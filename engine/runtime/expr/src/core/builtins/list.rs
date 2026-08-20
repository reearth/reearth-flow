use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::core::error::{eval_error, Result};
use crate::core::eval::eval_eq;
use crate::core::value::{NativeFn, TrackedRc, Value};

use crate::expect_arity;

use super::MethodFn;

static METHODS: LazyLock<HashMap<&'static str, MethodFn>> = LazyLock::new(|| {
    HashMap::from([
        ("get", get as MethodFn),
        ("append", append as MethodFn),
        ("pop", pop as MethodFn),
        ("extend", extend as MethodFn),
        ("index", index as MethodFn),
        ("rindex", rindex as MethodFn),
        ("truncate", truncate as MethodFn),
    ])
});

pub fn resolve_method(recv: Value, method: &str) -> Result<NativeFn> {
    let f = METHODS
        .get(method)
        .copied()
        .ok_or_else(|| eval_error(format!("list has no method '{method}'")))?;
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

fn get(args: &[Value]) -> Result<Value> {
    expect_arity("list.get", &args[1..], 1, 2)?;
    let Value::List(rc) = &args[0] else {
        return Err(eval_error("expected list receiver"));
    };
    let i = args[1].as_int()?;
    let fallback = args.get(2);
    let arr = rc.borrow();
    let elem = resolve_index(i, arr.len()).map(|pos| arr[pos].clone());
    Ok(elem.unwrap_or_else(|| fallback.cloned().unwrap_or(Value::Null)))
}

fn append(args: &[Value]) -> Result<Value> {
    expect_arity("list.append", &args[1..], 1, 1)?;
    let Value::List(rc) = &args[0] else {
        return Err(eval_error("expected list receiver"));
    };
    rc.borrow_mut().push(args[1].clone());
    Ok(Value::Null)
}

fn pop(args: &[Value]) -> Result<Value> {
    expect_arity("list.pop", &args[1..], 0, 1)?;
    let Value::List(rc) = &args[0] else {
        return Err(eval_error("expected list receiver"));
    };
    let mut arr = rc.borrow_mut();
    if arr.is_empty() {
        return Err(eval_error("pop from empty list"));
    }
    let pos = match args.get(1) {
        Some(v) => resolve_index(v.as_int()?, arr.len())
            .ok_or_else(|| eval_error("pop index out of range"))?,
        None => arr.len() - 1,
    };
    Ok(arr.remove(pos))
}

fn extend(args: &[Value]) -> Result<Value> {
    expect_arity("list.extend", &args[1..], 1, 1)?;
    let Value::List(rc) = &args[0] else {
        return Err(eval_error("expected list receiver"));
    };
    let Value::List(other) = &args[1] else {
        return Err(eval_error("extend() argument must be a list"));
    };
    // If receiver and argument alias the same RefCell, snapshot first
    if TrackedRc::ptr_eq(rc, other) {
        let snapshot: Vec<Value> = rc.borrow().clone();
        rc.borrow_mut().extend(snapshot);
    } else {
        rc.borrow_mut().extend(other.borrow().iter().cloned());
    }
    Ok(Value::Null)
}

fn index(args: &[Value]) -> Result<Value> {
    expect_arity("list.index", &args[1..], 1, 1)?;
    let Value::List(rc) = &args[0] else {
        return Err(eval_error("expected list receiver"));
    };
    let needle = &args[1];
    for (i, v) in rc.borrow().iter().enumerate() {
        if eval_eq(v.clone(), needle.clone())? {
            return Ok(Value::Int(i as i64));
        }
    }
    Ok(Value::Null)
}

fn rindex(args: &[Value]) -> Result<Value> {
    expect_arity("list.rindex", &args[1..], 1, 1)?;
    let Value::List(rc) = &args[0] else {
        return Err(eval_error("expected list receiver"));
    };
    let needle = &args[1];
    let arr = rc.borrow();
    for (i, v) in arr.iter().enumerate().rev() {
        if eval_eq(v.clone(), needle.clone())? {
            return Ok(Value::Int(i as i64));
        }
    }
    Ok(Value::Null)
}

fn truncate(args: &[Value]) -> Result<Value> {
    expect_arity("list.truncate", &args[1..], 1, 1)?;
    let Value::List(rc) = &args[0] else {
        return Err(eval_error("expected list receiver"));
    };
    let n = args[1].as_int()?;
    if n < 0 {
        return Err(eval_error("truncate() argument must be non-negative"));
    }
    let mut arr = rc.borrow_mut();
    arr.truncate(n as usize);
    Ok(Value::Null)
}

pub fn eq_inner(
    a: &TrackedRc<RefCell<Vec<Value>>>,
    b: &TrackedRc<RefCell<Vec<Value>>>,
) -> Result<bool> {
    if TrackedRc::ptr_eq(a, b) {
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
    fn test_append() {
        let arr = Value::from(vec![1i64, 2i64]);
        assert_eval(
            "arr.append(3); arr",
            &[("arr", arr)],
            Value::from(vec![1i64, 2i64, 3i64]),
        );
    }

    #[test]
    fn test_append_alias() {
        // Both names share the same backing Rc<RefCell<Vec>>; mutating one is visible via the other.
        assert_eval(
            "let b = a; a.append(3); b",
            &[("a", Value::from(vec![1i64, 2i64]))],
            Value::from(vec![1i64, 2i64, 3i64]),
        );
    }

    #[test]
    fn test_len() {
        let arr = Value::from(vec![1i64, 2i64, 3i64]);
        assert_eval("len(arr)", &[("arr", arr)], Value::from(3i64));
    }

    #[test]
    fn test_pop() {
        assert_eval(
            "arr.pop()",
            &[("arr", Value::from(vec![10i64, 20i64, 30i64]))],
            Value::from(30i64),
        );
        assert_eval(
            "arr.pop(0)",
            &[("arr", Value::from(vec![10i64, 20i64, 30i64]))],
            Value::from(10i64),
        );
        assert_eval(
            "arr.pop(-1)",
            &[("arr", Value::from(vec![10i64, 20i64, 30i64]))],
            Value::from(30i64),
        );
        assert_eval(
            "arr.pop(); arr",
            &[("arr", Value::from(vec![10i64, 20i64, 30i64]))],
            Value::from(vec![10i64, 20i64]),
        );
    }

    #[test]
    fn test_extend() {
        assert_eval(
            "arr.extend([3, 4]); arr",
            &[("arr", Value::from(vec![1i64, 2i64]))],
            Value::from(vec![1i64, 2i64, 3i64, 4i64]),
        );
        assert_eval(
            "arr.extend(arr); arr",
            &[("arr", Value::from(vec![1i64, 2i64]))],
            Value::from(vec![1i64, 2i64, 1i64, 2i64]),
        );
    }

    #[test]
    fn test_index() {
        let arr = || Value::from(vec![10i64, 20i64, 30i64, 20i64]);
        assert_eval("arr.index(20)", &[("arr", arr())], Value::from(1i64));
        assert_eval("arr.index(99)", &[("arr", arr())], Value::Null);
        assert_eval("arr.rindex(20)", &[("arr", arr())], Value::from(3i64));
        assert_eval("arr.rindex(99)", &[("arr", arr())], Value::Null);
    }

    #[test]
    fn test_truncate() {
        assert_eval(
            "arr.truncate(2); arr",
            &[("arr", Value::from(vec![1i64, 2i64, 3i64, 4i64]))],
            Value::from(vec![1i64, 2i64]),
        );
        assert_eval(
            "arr.truncate(10); arr",
            &[("arr", Value::from(vec![1i64, 2i64, 3i64, 4i64]))],
            Value::from(vec![1i64, 2i64, 3i64, 4i64]),
        );
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
