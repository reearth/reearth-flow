use std::cmp::Ordering;

use crate::core::error::{eval_error, Error, Result};
use crate::core::eval::{call_value, coerce_numeric, value_add, Numeric};
use crate::core::value::{Module, NativeFn, Value};
use crate::expect_arity;

fn value_cmp(a: &Value, b: &Value) -> Result<Ordering> {
    match coerce_numeric(a, b) {
        Ok((Numeric::Int(x), Numeric::Int(y))) => Ok(x.cmp(&y)),
        Ok((Numeric::Float(x), Numeric::Float(y))) => {
            Ok(x.partial_cmp(&y).unwrap_or(Ordering::Equal))
        }
        Ok(_) => unreachable!(),
        Err(_) => match (a, b) {
            (Value::String(x), Value::String(y)) => Ok(x.as_str().cmp(y.as_str())),
            _ => Err(eval_error(format!(
                "cannot compare {} and {}",
                a.type_name(),
                b.type_name()
            ))),
        },
    }
}

fn sorted(args: &[Value]) -> Result<Value> {
    expect_arity("itertools.sorted", args, 1, 2)?;
    let items = match &args[0] {
        Value::Array(rc) => rc.borrow().clone(),
        other => {
            return Err(eval_error(format!(
                "itertools.sorted() first argument must be a list, got {}",
                other.type_name()
            )))
        }
    };
    match args.get(1) {
        None => {
            let mut out = items;
            let mut sort_err: Option<Error> = None;
            out.sort_by(|a, b| match value_cmp(a, b) {
                Ok(ord) => ord,
                Err(e) => {
                    if sort_err.is_none() {
                        sort_err = Some(e);
                    }
                    Ordering::Equal
                }
            });
            if let Some(e) = sort_err {
                return Err(e);
            }
            Ok(Value::array(out))
        }
        Some(key_fn) => {
            let keyed = items
                .into_iter()
                .map(|item| {
                    let key = call_value(key_fn.clone(), vec![item.clone()])?;
                    Ok((item, key))
                })
                .collect::<Result<Vec<_>>>()?;
            let mut keyed = keyed;
            let mut sort_err: Option<Error> = None;
            keyed.sort_by(|(_, ka), (_, kb)| match value_cmp(ka, kb) {
                Ok(ord) => ord,
                Err(e) => {
                    if sort_err.is_none() {
                        sort_err = Some(e);
                    }
                    Ordering::Equal
                }
            });
            if let Some(e) = sort_err {
                return Err(e);
            }
            Ok(Value::array(
                keyed.into_iter().map(|(item, _)| item).collect(),
            ))
        }
    }
}

fn map(args: &[Value]) -> Result<Value> {
    expect_arity("itertools.map", args, 2, 2)?;
    let items = match &args[0] {
        Value::Array(rc) => rc.borrow().clone(),
        other => {
            return Err(eval_error(format!(
                "itertools.map() first argument must be a list, got {}",
                other.type_name()
            )))
        }
    };
    let f = args[1].clone();
    let result = items
        .into_iter()
        .map(|item| call_value(f.clone(), vec![item]))
        .collect::<Result<Vec<_>>>()?;
    Ok(Value::array(result))
}

fn sum(args: &[Value]) -> Result<Value> {
    expect_arity("itertools.sum", args, 1, 1)?;
    let items = match &args[0] {
        Value::Array(rc) => rc.borrow().clone(),
        other => {
            return Err(eval_error(format!(
                "itertools.sum() argument must be a list, got {}",
                other.type_name()
            )))
        }
    };
    let mut iter = items.into_iter();
    let init = match iter.next() {
        Some(v) => v,
        None => return Ok(Value::Int(0)),
    };
    iter.try_fold(init, value_add)
}

pub fn builtin_itertools() -> Value {
    let mut m = Module::new();
    m.insert("sorted".into(), Value::Fn(NativeFn::new(sorted)));
    m.insert("map".into(), Value::Fn(NativeFn::new(map)));
    m.insert("sum".into(), Value::Fn(NativeFn::new(sum)));
    Value::module(m)
}

#[cfg(test)]
mod tests {
    use crate::core::test_utils::{assert_eval, try_run};
    use crate::core::value::Value;

    #[test]
    fn test_sorted() {
        assert_eval(
            "itertools.sorted([3, 1, 2])",
            &[],
            Value::from(vec![1i64, 2i64, 3i64]),
        );
        assert_eval(
            r#"itertools.sorted(["banana", "apple", "cherry"])"#,
            &[],
            Value::from(vec!["apple", "banana", "cherry"]),
        );
        assert_eval(
            "itertools.sorted([3, 1, 2], fn(x) { -x })",
            &[],
            Value::from(vec![3i64, 2i64, 1i64]),
        );
        assert_eval(
            r#"itertools.sorted([{"k": 3}, {"k": 1}, {"k": 2}], fn(x) { x["k"] })"#,
            &[],
            Value::from(vec![
                Value::map(indexmap::indexmap! { "k".into() => Value::from(1i64) }),
                Value::map(indexmap::indexmap! { "k".into() => Value::from(2i64) }),
                Value::map(indexmap::indexmap! { "k".into() => Value::from(3i64) }),
            ]),
        );
        assert_eval("itertools.sorted([])", &[], Value::from(vec![] as Vec<i64>));
        assert!(try_run("itertools.sorted([1, \"a\"])", &[]).is_err());
    }

    #[test]
    fn test_map() {
        assert_eval(
            "itertools.map([1, 2, 3], fn(x) { x * 2 })",
            &[],
            Value::from(vec![2i64, 4i64, 6i64]),
        );
        assert_eval(
            r#"itertools.map(["a", "b"], fn(x) { x + "!" })"#,
            &[],
            Value::from(vec!["a!", "b!"]),
        );
        assert_eval(
            "itertools.map([], fn(x) { x })",
            &[],
            Value::from(vec![] as Vec<i64>),
        );
    }

    #[test]
    fn test_sum() {
        assert_eval("itertools.sum([1, 2, 3])", &[], Value::from(6i64));
        assert_eval("itertools.sum([1, 2.5])", &[], Value::from(3.5f64));
        assert_eval(r#"itertools.sum(["a", "b", "c"])"#, &[], Value::from("abc"));
        assert_eval("itertools.sum([])", &[], Value::from(0i64));
        assert!(try_run("itertools.sum([1, \"a\"])", &[]).is_err());
    }
}
