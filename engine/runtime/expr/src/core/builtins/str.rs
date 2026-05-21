use std::collections::HashMap;
use std::sync::LazyLock;

use crate::core::error::{InnerError, InnerResult};
use crate::core::value::{NativeFn, Value};
use crate::unpack_args;

type MethodFn = fn(&[Value]) -> InnerResult<Value>;

static METHODS: LazyLock<HashMap<&'static str, MethodFn>> = LazyLock::new(|| {
    HashMap::from([
        ("len", len as MethodFn),
        ("trim", trim as MethodFn),
        ("split", split as MethodFn),
        ("startswith", starts_with as MethodFn),
        ("endswith", ends_with as MethodFn),
        ("replace", replace as MethodFn),
        ("removeprefix", remove_prefix as MethodFn),
        ("removesuffix", remove_suffix as MethodFn),
    ])
});

pub fn resolve_method(method: &str) -> InnerResult<NativeFn> {
    METHODS
        .get(method)
        .map(|&f| NativeFn::new(f))
        .ok_or_else(|| InnerError::new(format!("String has no method '{method}'")))
}

fn len(args: &[Value]) -> InnerResult<Value> {
    unpack_args!(args => s);
    let Value::String(s) = s else {
        return Err(InnerError::new("expected string receiver"));
    };
    Ok(Value::Int(s.chars().count() as i64))
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
            "startswith() argument must be a string, got {}",
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
            "endswith() argument must be a string, got {}",
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
            "removeprefix() argument must be a string, got {}",
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

fn remove_suffix(args: &[Value]) -> InnerResult<Value> {
    unpack_args!(args => s, suffix);
    let Value::String(s) = s else {
        return Err(InnerError::new("expected string receiver"));
    };
    let Value::String(suffix) = suffix else {
        return Err(InnerError::new(format!(
            "removesuffix() argument must be a string, got {}",
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
    fn test_startswith() {
        assert_eval(
            r#""hello_world".startswith("hello")"#,
            &[],
            Value::Bool(true),
        );
        assert_eval(
            r#""hello_world".startswith("foo")"#,
            &[],
            Value::Bool(false),
        );
    }

    #[test]
    fn test_endswith() {
        assert_eval(r#""hello_world".endswith("world")"#, &[], Value::Bool(true));
        assert_eval(r#""hello_world".endswith("foo")"#, &[], Value::Bool(false));
    }

    #[test]
    fn test_removeprefix() {
        assert_eval(
            r#""hello_world".removeprefix("hello_")"#,
            &[],
            Value::from("world"),
        );
        assert_eval(
            r#""hello_world".removeprefix("foo")"#,
            &[],
            Value::from("hello_world"),
        );
    }

    #[test]
    fn test_removesuffix() {
        assert_eval(
            r#""hello_world".removesuffix("_world")"#,
            &[],
            Value::from("hello"),
        );
        assert_eval(
            r#""hello_world".removesuffix("foo")"#,
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
    fn test_trim() {
        assert_eval(r#""  hello  ".trim()"#, &[], Value::from("hello"));
    }

    #[test]
    fn test_len() {
        assert_eval(r#""hello".len()"#, &[], Value::Int(5));
        assert_eval(r#""".len()"#, &[], Value::Int(0));
    }
}
