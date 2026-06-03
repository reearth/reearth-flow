use std::collections::HashMap;
use std::sync::LazyLock;

use crate::core::error::{InnerError, InnerResult};
use crate::core::value::{NativeFn, Value};
use crate::unpack_args;

use super::MethodFn;

static METHODS: LazyLock<HashMap<&'static str, MethodFn>> = LazyLock::new(|| {
    HashMap::from([
        ("trim", trim as MethodFn),
        ("split", split as MethodFn),
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
    unpack_args!(args => s);
    let Value::String(s) = s else {
        return Err(InnerError::new("expected string receiver"));
    };
    Ok(Value::String(s.trim().to_string()))
}

fn split(args: &[Value]) -> InnerResult<Value> {
    unpack_args!(args => s, sep);
    let Value::String(s) = s else {
        return Err(InnerError::new("expected string receiver"));
    };
    let Value::String(sep) = sep else {
        return Err(InnerError::new(format!(
            "split() separator must be a string, got {}",
            sep.type_name()
        )));
    };
    Ok(Value::array(
        s.split(sep.as_str())
            .map(|p| Value::String(p.to_string()))
            .collect(),
    ))
}

fn starts_with(args: &[Value]) -> InnerResult<Value> {
    unpack_args!(args => s, prefix);
    let Value::String(s) = s else {
        return Err(InnerError::new("expected string receiver"));
    };
    let Value::String(prefix) = prefix else {
        return Err(InnerError::new(format!(
            "starts_with() argument must be a string, got {}",
            prefix.type_name()
        )));
    };
    Ok(Value::Bool(s.starts_with(prefix.as_str())))
}

fn ends_with(args: &[Value]) -> InnerResult<Value> {
    unpack_args!(args => s, suffix);
    let Value::String(s) = s else {
        return Err(InnerError::new("expected string receiver"));
    };
    let Value::String(suffix) = suffix else {
        return Err(InnerError::new(format!(
            "ends_with() argument must be a string, got {}",
            suffix.type_name()
        )));
    };
    Ok(Value::Bool(s.ends_with(suffix.as_str())))
}

fn remove_prefix(args: &[Value]) -> InnerResult<Value> {
    unpack_args!(args => s, prefix);
    let Value::String(s) = s else {
        return Err(InnerError::new("expected string receiver"));
    };
    let Value::String(prefix) = prefix else {
        return Err(InnerError::new(format!(
            "remove_prefix() argument must be a string, got {}",
            prefix.type_name()
        )));
    };
    Ok(Value::String(
        s.strip_prefix(prefix.as_str()).unwrap_or(s).to_string(),
    ))
}

fn replace(args: &[Value]) -> InnerResult<Value> {
    unpack_args!(args => s, from, to);
    let Value::String(s) = s else {
        return Err(InnerError::new("expected string receiver"));
    };
    let (Value::String(from), Value::String(to)) = (from, to) else {
        return Err(InnerError::new(
            "replace() requires two string arguments: replace(from, to)",
        ));
    };
    Ok(Value::String(s.replace(from.as_str(), to.as_str())))
}

fn join(args: &[Value]) -> InnerResult<Value> {
    unpack_args!(args => sep, list);
    let Value::String(sep) = sep else {
        return Err(InnerError::new("expected string receiver"));
    };
    let Value::Array(list) = list else {
        return Err(InnerError::new(format!(
            "join() argument must be an array, got {}",
            list.type_name()
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
    Ok(Value::String(parts.join(sep.as_str())))
}

fn remove_suffix(args: &[Value]) -> InnerResult<Value> {
    unpack_args!(args => s, suffix);
    let Value::String(s) = s else {
        return Err(InnerError::new("expected string receiver"));
    };
    let Value::String(suffix) = suffix else {
        return Err(InnerError::new(format!(
            "remove_suffix() argument must be a string, got {}",
            suffix.type_name()
        )));
    };
    Ok(Value::String(
        s.strip_suffix(suffix.as_str()).unwrap_or(s).to_string(),
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
        assert_eval(
            r#"", ".join(["a", "b", "c"])"#,
            &[],
            Value::from("a, b, c"),
        );
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
