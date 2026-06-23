use std::collections::HashMap;
use std::sync::LazyLock;

use crate::core::error::{eval_error, Result};
use crate::core::value::{format_float, NativeFn, Value};
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
        ("format", format as MethodFn),
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

struct FormatSpec {
    zero_pad: bool,
    width: Option<usize>,
    precision: Option<usize>,
    kind: Option<FormatKind>,
}

enum FormatKind {
    Float,
    Int,
}

fn parse_spec(s: &str) -> Result<FormatSpec> {
    let b = s.as_bytes();
    let mut i = 0;
    let mut zero_pad = false;

    if b.first() == Some(&b'0') {
        zero_pad = true;
        i += 1;
    }

    let start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let width = if i > start {
        Some(
            s[start..i]
                .parse::<usize>()
                .map_err(|_| eval_error(format!("invalid width in format spec '{s}'")))?,
        )
    } else {
        None
    };

    let precision = if b.get(i) == Some(&b'.') {
        i += 1;
        let start = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        Some(
            s[start..i]
                .parse::<usize>()
                .map_err(|_| eval_error(format!("invalid precision in format spec '{s}'")))?,
        )
    } else {
        None
    };

    let kind = if i < b.len() {
        let k = match b[i] {
            b'f' => FormatKind::Float,
            b'd' => FormatKind::Int,
            c => return Err(eval_error(format!("unknown format type '{}'", c as char))),
        };
        i += 1;
        Some(k)
    } else {
        None
    };

    if i != b.len() {
        return Err(eval_error(format!("invalid format spec '{s}'")));
    }

    Ok(FormatSpec {
        zero_pad,
        width,
        precision,
        kind,
    })
}

fn zero_pad(s: &str, width: usize) -> String {
    if s.len() >= width {
        return s.to_string();
    }
    let n = width - s.len();
    let padding: String = std::iter::repeat_n('0', n).collect();
    if let Some(rest) = s.strip_prefix('-') {
        format!("-{padding}{rest}")
    } else {
        format!("{padding}{s}")
    }
}

fn right_pad(s: &str, width: usize) -> String {
    if s.len() >= width {
        return s.to_string();
    }
    let n = width - s.len();
    let padding: String = std::iter::repeat_n(' ', n).collect();
    format!("{padding}{s}")
}

fn left_pad(s: &str, width: usize) -> String {
    if s.len() >= width {
        return s.to_string();
    }
    let n = width - s.len();
    let padding: String = std::iter::repeat_n(' ', n).collect();
    format!("{s}{padding}")
}

fn apply_spec(val: &Value, spec: &FormatSpec) -> Result<String> {
    match &spec.kind {
        Some(FormatKind::Float) => {
            let x = val.as_f64()?;
            let prec = spec.precision.unwrap_or(6);
            let s = format!("{x:.prec$}");
            Ok(match spec.width {
                Some(w) if spec.zero_pad => zero_pad(&s, w),
                Some(w) => right_pad(&s, w),
                None => s,
            })
        }
        Some(FormatKind::Int) => {
            let n = val.as_int()?;
            let s = format!("{n}");
            Ok(match spec.width {
                Some(w) if spec.zero_pad => zero_pad(&s, w),
                Some(w) => right_pad(&s, w),
                None => s,
            })
        }
        None => {
            let (s, is_str) = match val {
                Value::String(s) => (s.clone(), true),
                Value::Null => ("null".to_string(), false),
                Value::Bool(b) => (b.to_string(), false),
                Value::Int(n) => (n.to_string(), false),
                Value::Float(n) => (format_float(*n), false),
                other => return Err(eval_error(format!("cannot format {}", other.type_name()))),
            };
            Ok(match spec.width {
                Some(w) if is_str => left_pad(&s, w),
                Some(w) if spec.zero_pad => zero_pad(&s, w),
                Some(w) => right_pad(&s, w),
                None => s,
            })
        }
    }
}

fn format(args: &[Value]) -> Result<Value> {
    let template = args[0].as_str()?;
    let fmt_args = &args[1..];
    let mut out = String::new();
    let b = template.as_bytes();
    let mut i = 0;
    let mut auto_idx = 0usize;

    while i < b.len() {
        match b[i] {
            b'{' if b.get(i + 1) == Some(&b'{') => {
                out.push('{');
                i += 2;
            }
            b'}' if b.get(i + 1) == Some(&b'}') => {
                out.push('}');
                i += 2;
            }
            b'{' => {
                i += 1;
                let start = i;
                while i < b.len() && b[i] != b'}' {
                    i += 1;
                }
                if i >= b.len() {
                    return Err(eval_error("unclosed '{' in format string"));
                }
                let inner = &template[start..i];
                i += 1;

                let (field, spec_str) = match inner.find(':') {
                    Some(j) => (&inner[..j], &inner[j + 1..]),
                    None => (inner, ""),
                };
                let idx = if field.is_empty() {
                    let j = auto_idx;
                    auto_idx += 1;
                    j
                } else {
                    field
                        .parse::<usize>()
                        .map_err(|_| eval_error(format!("invalid field '{field}'")))?
                };
                let val = fmt_args
                    .get(idx)
                    .ok_or_else(|| eval_error(format!("argument index {idx} out of range")))?;

                let s = if let Value::Object(obj) = val {
                    match obj.call_method("__format__", &[Value::String(spec_str.to_string())]) {
                        Ok(Value::String(s)) => s,
                        Ok(_) => return Err(eval_error("__format__ must return a string")),
                        Err(_) => obj.display(),
                    }
                } else {
                    let spec = parse_spec(spec_str)?;
                    apply_spec(val, &spec)?
                };
                out.push_str(&s);
            }
            b'}' => return Err(eval_error("single '}' in format string")),
            _ => {
                let start = i;
                while i < b.len() && b[i] != b'{' && b[i] != b'}' {
                    i += 1;
                }
                out.push_str(&template[start..i]);
            }
        }
    }
    Ok(Value::String(out))
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

    use crate::core::error::Result as EvalResult;
    use crate::core::value::ImmutableObject;
    use std::rc::Rc;

    #[derive(Debug)]
    struct Point {
        x: f64,
        y: f64,
    }

    impl ImmutableObject for Point {
        fn type_object(&self) -> Rc<crate::core::value::TypeValue> {
            thread_local! {
                static TY: Rc<crate::core::value::TypeValue> =
                    Rc::new(crate::core::value::TypeValue::new("Point", None));
            }
            TY.with(Rc::clone)
        }

        fn call_method(&self, method: &str, args: &[Value]) -> EvalResult<Value> {
            match method {
                "__format__" => {
                    let spec = args[0].as_str()?;
                    let s = match spec {
                        "compact" => format!("{},{}", self.x, self.y),
                        _ => format!("({}, {})", self.x, self.y),
                    };
                    Ok(Value::String(s))
                }
                _ => Err(crate::core::error::eval_error(format!(
                    "Point has no method '{method}'"
                ))),
            }
        }

        fn display(&self) -> String {
            format!("({}, {})", self.x, self.y)
        }
    }

    #[derive(Debug)]
    struct Opaque;

    impl ImmutableObject for Opaque {
        fn type_object(&self) -> Rc<crate::core::value::TypeValue> {
            thread_local! {
                static TY: Rc<crate::core::value::TypeValue> =
                    Rc::new(crate::core::value::TypeValue::new("Opaque", None));
            }
            TY.with(Rc::clone)
        }

        fn call_method(&self, method: &str, _args: &[Value]) -> EvalResult<Value> {
            Err(crate::core::error::eval_error(format!(
                "Opaque has no method '{method}'"
            )))
        }

        fn display(&self) -> String {
            "opaque".to_string()
        }
    }

    #[test]
    fn test_format() {
        assert_eval(
            r#""Hello, {}!".format("world")"#,
            &[],
            Value::from("Hello, world!"),
        );
        assert_eval(
            r#""{0} and {1}".format("a", "b")"#,
            &[],
            Value::from("a and b"),
        );
        assert_eval(
            r#""{1} then {0}".format("b", "a")"#,
            &[],
            Value::from("a then b"),
        );
        assert_eval(r#""{} {}".format(1, 2)"#, &[], Value::from("1 2"));
        assert_eval(r#""{{}}".format()"#, &[], Value::from("{}"));

        assert_eval(r#""{:04d}".format(7)"#, &[], Value::from("0007"));
        assert_eval(r#""{:04d}".format(-7)"#, &[], Value::from("-007"));
        assert_eval(r#""{:5d}".format(7)"#, &[], Value::from("    7"));
        assert_eval(r#""{:5d}".format(-7)"#, &[], Value::from("   -7"));
        assert_eval(r#""{:0d}".format(7)"#, &[], Value::from("7")); // no width → flag is no-op

        assert_eval(r#""{:.2f}".format(3.14159)"#, &[], Value::from("3.14"));
        assert_eval(r#""{:08.2f}".format(-3.14)"#, &[], Value::from("-0003.14"));
        assert_eval(r#""{:0f}".format(3.14)"#, &[], Value::from("3.140000")); // no width → flag is no-op
        assert_eval(r#""{:0.2f}".format(3.14)"#, &[], Value::from("3.14")); // no width → flag is no-op

        assert_eval(r#""{:5}".format("hi")"#, &[], Value::from("hi   "));
        assert_eval(r#""{:5}".format(7)"#, &[], Value::from("    7"));

        let p = Value::Object(Rc::new(Point { x: 1.0, y: 2.0 }));
        assert_eval(
            r#""{:compact}".format(p)"#,
            &[("p", p.clone())],
            Value::from("1,2"),
        );
        assert_eval(r#""{}".format(p)"#, &[("p", p)], Value::from("(1, 2)"));
        let o = Value::Object(Rc::new(Opaque));
        assert_eval(r#""{}".format(o)"#, &[("o", o)], Value::from("opaque"));
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
