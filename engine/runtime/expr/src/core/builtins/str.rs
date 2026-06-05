use std::collections::HashMap;
use std::sync::LazyLock;

use crate::core::error::{InnerError, InnerResult};
use crate::core::value::{NativeFn, Value};

use super::{expect_arity, MethodFn};

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

pub fn resolve_method(recv: Value, method: &str) -> InnerResult<NativeFn> {
    let f = METHODS
        .get(method)
        .copied()
        .ok_or_else(|| InnerError::new(format!("String has no method '{method}'")))?;
    Ok(NativeFn::new(move |args| {
        let mut a = vec![recv.clone()];
        a.extend_from_slice(args);
        f(&a)
    }))
}

fn trim(args: &[Value]) -> InnerResult<Value> {
    expect_arity(args, 0, 0)?;
    Ok(Value::String(args[0].as_str()?.trim().to_string()))
}

fn split_limit(v: &Value) -> InnerResult<usize> {
    let n = v.as_int()?;
    if n < 0 {
        return Err(InnerError::new("limit must be non-negative"));
    }
    Ok(n as usize)
}

fn split(args: &[Value]) -> InnerResult<Value> {
    expect_arity(args, 1, 2)?;
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

fn rsplit(args: &[Value]) -> InnerResult<Value> {
    expect_arity(args, 1, 2)?;
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

fn starts_with(args: &[Value]) -> InnerResult<Value> {
    expect_arity(args, 1, 1)?;
    Ok(Value::Bool(
        args[0].as_str()?.starts_with(args[1].as_str()?),
    ))
}

fn ends_with(args: &[Value]) -> InnerResult<Value> {
    expect_arity(args, 1, 1)?;
    Ok(Value::Bool(args[0].as_str()?.ends_with(args[1].as_str()?)))
}

fn remove_prefix(args: &[Value]) -> InnerResult<Value> {
    expect_arity(args, 1, 1)?;
    let s = args[0].as_str()?;
    let prefix = args[1].as_str()?;
    Ok(Value::String(
        s.strip_prefix(prefix).unwrap_or(s).to_string(),
    ))
}

fn replace(args: &[Value]) -> InnerResult<Value> {
    expect_arity(args, 2, 2)?;
    let s = args[0].as_str()?;
    let from = args[1].as_str()?;
    let to = args[2].as_str()?;
    Ok(Value::String(s.replace(from, to)))
}

fn join(args: &[Value]) -> InnerResult<Value> {
    expect_arity(args, 1, 1)?;
    let sep = args[0].as_str()?;
    let Value::Array(list) = &args[1] else {
        return Err(InnerError::new(format!(
            "join() argument must be an array, got {}",
            args[1].type_name()
        )));
    };
    let parts = list
        .borrow()
        .iter()
        .map(|v| match v {
            Value::String(s) => Ok(s.clone()),
            other => Err(InnerError::new(format!(
                "join() array elements must be strings, got {}",
                other.type_name()
            ))),
        })
        .collect::<InnerResult<Vec<_>>>()?;
    Ok(Value::String(parts.join(sep)))
}

fn remove_suffix(args: &[Value]) -> InnerResult<Value> {
    expect_arity(args, 1, 1)?;
    let s = args[0].as_str()?;
    let suffix = args[1].as_str()?;
    Ok(Value::String(
        s.strip_suffix(suffix).unwrap_or(s).to_string(),
    ))
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
