use std::collections::HashMap;
use std::sync::LazyLock;

use crate::core::error::{eval_error, Result};
use crate::core::value::{NativeFn, Value};
use crate::expect_arity;

use super::MethodFn;

static METHODS: LazyLock<HashMap<&'static str, MethodFn>> = LazyLock::new(|| {
    HashMap::from([
        ("trim", trim as MethodFn),
        ("split", split as MethodFn),
        ("rsplit", rsplit as MethodFn),
        ("starts_with", starts_with as MethodFn),
        ("ends_with", ends_with as MethodFn),
        ("replace", replace as MethodFn),
        ("remove_prefix", remove_prefix as MethodFn),
        ("remove_suffix", remove_suffix as MethodFn),
        ("join", join as MethodFn),
    ])
});

pub fn resolve_method(recv: Value, method: &str) -> Result<NativeFn> {
    let f = METHODS
        .get(method)
        .copied()
        .ok_or_else(|| eval_error(format!("String has no method '{method}'")))?;
    Ok(NativeFn::new(move |args| {
        let mut a = vec![recv.clone()];
        a.extend_from_slice(args);
        f(&a)
    }))
}

fn trim(args: &[Value]) -> Result<Value> {
    expect_arity("str.trim", &args[1..], 0, 0)?;
    Ok(Value::String(args[0].as_str()?.trim().to_string()))
}

fn split_limit(v: &Value) -> Result<usize> {
    let n = v.as_int()?;
    if n < 0 {
        return Err(eval_error("limit must be non-negative"));
    }
    Ok(n as usize)
}

fn split(args: &[Value]) -> Result<Value> {
    expect_arity("str.split", &args[1..], 1, 2)?;
    let s = args[0].as_str()?;
    let sep = args[1].as_str()?;
    let n = args.get(2).map(split_limit).transpose()?;
    let parts: Vec<Value> = match n {
        Some(n) => s
            .splitn(n + 1, sep)
            .map(|p| Value::String(p.to_string()))
            .collect(),
        None => s.split(sep).map(|p| Value::String(p.to_string())).collect(),
    };
    Ok(Value::array(parts))
}

fn rsplit(args: &[Value]) -> Result<Value> {
    expect_arity("str.rsplit", &args[1..], 1, 2)?;
    let s = args[0].as_str()?;
    let sep = args[1].as_str()?;
    let n = args.get(2).map(split_limit).transpose()?;
    let mut parts: Vec<Value> = match n {
        Some(n) => s
            .rsplitn(n + 1, sep)
            .map(|p| Value::String(p.to_string()))
            .collect(),
        None => s.split(sep).map(|p| Value::String(p.to_string())).collect(),
    };
    if n.is_some() {
        parts.reverse();
    }
    Ok(Value::array(parts))
}

fn starts_with(args: &[Value]) -> Result<Value> {
    expect_arity("str.starts_with", &args[1..], 1, 1)?;
    Ok(Value::Bool(
        args[0].as_str()?.starts_with(args[1].as_str()?),
    ))
}

fn ends_with(args: &[Value]) -> Result<Value> {
    expect_arity("str.ends_with", &args[1..], 1, 1)?;
    Ok(Value::Bool(args[0].as_str()?.ends_with(args[1].as_str()?)))
}

fn remove_prefix(args: &[Value]) -> Result<Value> {
    expect_arity("str.remove_prefix", &args[1..], 1, 1)?;
    let s = args[0].as_str()?;
    let p = args[1].as_str()?;
    Ok(Value::String(s.strip_prefix(p).unwrap_or(s).to_string()))
}

fn replace(args: &[Value]) -> Result<Value> {
    expect_arity("str.replace", &args[1..], 2, 2)?;
    Ok(Value::String(
        args[0]
            .as_str()?
            .replace(args[1].as_str()?, args[2].as_str()?),
    ))
}

fn join(args: &[Value]) -> Result<Value> {
    expect_arity("str.join", &args[1..], 1, 1)?;
    let sep = args[0].as_str()?;
    let Value::Array(list) = &args[1] else {
        return Err(eval_error(format!(
            "join() argument must be an array, got {}",
            args[1].type_name()
        )));
    };
    let parts = list
        .borrow()
        .iter()
        .map(|v| match v {
            Value::String(s) => Ok(s.clone()),
            other => Err(eval_error(format!(
                "join() array elements must be strings, got {}",
                other.type_name()
            ))),
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(Value::String(parts.join(sep)))
}

fn remove_suffix(args: &[Value]) -> Result<Value> {
    expect_arity("str.remove_suffix", &args[1..], 1, 1)?;
    let s = args[0].as_str()?;
    let suf = args[1].as_str()?;
    Ok(Value::String(s.strip_suffix(suf).unwrap_or(s).to_string()))
}

#[cfg(test)]
mod tests {
    use crate::core::test_utils::assert_eval;
    use crate::core::value::Value;

    #[test]
    fn test_starts_with() {
        assert_eval(
            r#""hello_world".starts_with("hello")"#,
            &[],
            Value::Bool(true),
        );
        assert_eval(
            r#""hello_world".starts_with("foo")"#,
            &[],
            Value::Bool(false),
        );
    }

    #[test]
    fn test_ends_with() {
        assert_eval(
            r#""hello_world".ends_with("world")"#,
            &[],
            Value::Bool(true),
        );
        assert_eval(r#""hello_world".ends_with("foo")"#, &[], Value::Bool(false));
    }

    #[test]
    fn test_remove_prefix() {
        assert_eval(
            r#""hello_world".remove_prefix("hello_")"#,
            &[],
            Value::from("world"),
        );
        assert_eval(
            r#""hello_world".remove_prefix("foo")"#,
            &[],
            Value::from("hello_world"),
        );
    }

    #[test]
    fn test_remove_suffix() {
        assert_eval(
            r#""hello_world".remove_suffix("_world")"#,
            &[],
            Value::from("hello"),
        );
        assert_eval(
            r#""hello_world".remove_suffix("foo")"#,
            &[],
            Value::from("hello_world"),
        );
    }

    #[test]
    fn test_split() {
        assert_eval(r#""foo:bar".split(":")[0]"#, &[], Value::from("foo"));
        assert_eval(r#""foo:bar".split(":")[-1]"#, &[], Value::from("bar"));
        assert_eval(r#""a/b/c".split("/", 1)[0]"#, &[], Value::from("a"));
        assert_eval(r#""a/b/c".split("/", 1)[1]"#, &[], Value::from("b/c"));
    }

    #[test]
    fn test_rsplit() {
        assert_eval(r#""a/b/c".rsplit("/")[-1]"#, &[], Value::from("c"));
        assert_eval(r#""a/b/c".rsplit("/", 1)[-1]"#, &[], Value::from("c"));
        assert_eval(r#""a/b/c".rsplit("/", 1)[0]"#, &[], Value::from("a/b"));
        assert_eval(
            r#""path/to/file.txt".rsplit("/", 1)[-1]"#,
            &[],
            Value::from("file.txt"),
        );
    }

    #[test]
    fn test_replace() {
        assert_eval(r#""a/b/c".replace("/", "_")"#, &[], Value::from("a_b_c"));
        assert_eval(
            r#""foo_op_bar_op_baz".replace("_op_", "/")"#,
            &[],
            Value::from("foo/bar/baz"),
        );
        assert_eval(r#""hello".replace("x", "y")"#, &[], Value::from("hello"));
    }

    #[test]
    fn test_join() {
        assert_eval(r#"", ".join(["a", "b", "c"])"#, &[], Value::from("a, b, c"));
        assert_eval(r#""".join(["x", "y"])"#, &[], Value::from("xy"));
        assert_eval(r#""-".join([])"#, &[], Value::from(""));
    }

    #[test]
    fn test_trim() {
        assert_eval(r#""  hello  ".trim()"#, &[], Value::from("hello"));
    }

    #[test]
    fn test_len() {
        assert_eval(r#"len("hello")"#, &[], Value::Int(5));
        assert_eval(r#"len("")"#, &[], Value::Int(0));
    }
}
