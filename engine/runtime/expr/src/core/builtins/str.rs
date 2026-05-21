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
            "remove_suffix() argument must be a string, got {}",
            suffix.type_name()
        )));
    };
    Ok(Value::String(
        s.strip_suffix(suffix.as_str()).unwrap_or(s).to_string(),
    ))
}
