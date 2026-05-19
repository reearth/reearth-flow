use crate::core::error::{InnerError, InnerResult};
use crate::core::value::{ImmutableObject, Value};
use crate::unpack_args;

fn parse_url(s: &str) -> Result<UrlObject, String> {
    let full = if s.contains("://") {
        s.to_string()
    } else if s.starts_with('/') {
        format!("file://{s}")
    } else {
        return Err(format!("not a valid URI: {s}"));
    };
    let (scheme, rest) = full
        .split_once("://")
        .ok_or_else(|| format!("not a valid URI: {s}"))?;
    let (netloc, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, ""),
    };
    Ok(UrlObject {
        scheme: scheme.to_string(),
        netloc: netloc.to_string(),
        path: path.to_string(),
    })
}

#[derive(Debug, Clone)]
pub struct UrlObject {
    scheme: String,
    netloc: String,
    path: String,
}

impl UrlObject {
    fn as_string(&self) -> String {
        format!("{}://{}{}", self.scheme, self.netloc, self.path)
    }

    fn name(&self) -> &str {
        self.path
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("")
    }

    fn parent(&self) -> Self {
        let new_path = if let Some(stripped) = self.path.strip_suffix('/') {
            if stripped.is_empty() || !stripped.contains('/') {
                return self.clone();
            }
            stripped.to_string()
        } else if !self.path.contains('/') {
            return self.clone();
        } else {
            let last = self.path.rfind('/').unwrap();
            self.path[..last].to_string()
        };
        Self {
            scheme: self.scheme.clone(),
            netloc: self.netloc.clone(),
            path: new_path,
        }
    }
}

impl ImmutableObject for UrlObject {
    fn type_name(&self) -> &'static str {
        "Url"
    }

    fn call_method(&self, method: &str, args: &[Value]) -> InnerResult<Value> {
        match method {
            "parent" => {
                unpack_args!(args =>);
                Ok(Value::object(self.parent()))
            }
            "extension" => {
                unpack_args!(args =>);
                let ext = std::path::Path::new(self.name())
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                Ok(Value::String(ext.to_string()))
            }
            "name" => {
                unpack_args!(args =>);
                Ok(Value::String(self.name().to_string()))
            }
            "stem" => {
                unpack_args!(args =>);
                let name = self.name();
                let stem = std::path::Path::new(name)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(name);
                Ok(Value::String(stem.to_string()))
            }
            "__eq__" => {
                let rhs = args
                    .first()
                    .ok_or_else(|| InnerError::new("Url == requires an argument"))?;
                match rhs {
                    Value::Object(obj) if obj.borrow().type_name() == "Url" => {
                        Ok(Value::Bool(self.as_string() == obj.borrow().display()))
                    }
                    _ => Ok(Value::Bool(false)),
                }
            }
            "__str__" => {
                unpack_args!(args =>);
                Ok(Value::String(self.as_string()))
            }
            "__div__" => {
                let rhs = args
                    .first()
                    .and_then(|v| {
                        if let Value::String(s) = v {
                            Some(s.as_str())
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| InnerError::new("Url / requires a string"))?;
                let path = format!("{}/{rhs}", self.path.trim_end_matches('/'));
                Ok(Value::object(Self {
                    scheme: self.scheme.clone(),
                    netloc: self.netloc.clone(),
                    path,
                }))
            }
            m => Err(InnerError::new(format!("Url has no method '{m}'"))),
        }
    }

    fn display(&self) -> String {
        self.as_string()
    }

    fn serialize(&self) -> Option<Value> {
        Some(Value::String(self.as_string()))
    }
}

pub fn builtin_url(args: &[Value]) -> InnerResult<Value> {
    if args.len() > 1 {
        return Err(InnerError::new(format!(
            "Url() expected at most 1 argument, got {}",
            args.len()
        )));
    }
    let s = match args.first() {
        None => return Err(InnerError::new("Url() requires a string argument")),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Object(obj)) if obj.borrow().type_name() == "Url" => obj.borrow().display(),
        Some(v) => {
            return Err(InnerError::new(format!(
                "Url() expects a string, got {}",
                v.type_name()
            )))
        }
    };
    parse_url(&s).map(Value::object).map_err(InnerError::new)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::eval::{default_env, eval, values_equal};
    use crate::core::parser::parse;

    fn run(input: &str) -> Value {
        let mut env = default_env();
        eval(&parse(input).unwrap(), &mut env).unwrap()
    }

    #[track_caller]
    fn assert_val(a: &Value, b: &Value) {
        assert!(values_equal(a, b), "\nleft:  {a:?}\nright: {b:?}");
    }

    #[test]
    fn test_url_from_string() {
        let v = run(r#"Url("/foo/bar")"#);
        assert!(matches!(&v, Value::Object(obj) if obj.borrow().display() == "file:///foo/bar"));
    }

    #[test]
    fn test_url_rewrap() {
        let v = run(r#"Url(Url("/foo/bar"))"#);
        assert!(matches!(&v, Value::Object(obj) if obj.borrow().display() == "file:///foo/bar"));
    }

    #[test]
    fn test_url_str() {
        assert_val(
            &run(r#"str(Url("/foo/bar"))"#),
            &Value::from("file:///foo/bar"),
        );
    }

    #[test]
    fn test_url_div() {
        assert_val(
            &run(r#"str(Url("/foo") / "bar" / "baz")"#),
            &Value::from("file:///foo/bar/baz"),
        );
    }

    #[test]
    fn test_url_div_gs() {
        assert_val(
            &run(r#"str(Url("gs://bucket/artifacts") / "output")"#),
            &Value::from("gs://bucket/artifacts/output"),
        );
    }

    #[test]
    fn test_url_parent() {
        let v = run(r#"Url("/foo/bar").parent()"#);
        assert!(matches!(&v, Value::Object(obj) if obj.borrow().display() == "file:///foo"));
    }

    #[test]
    fn test_url_parent_single_level() {
        let v = run(r#"Url("/foo").parent()"#);
        assert!(matches!(&v, Value::Object(obj) if obj.borrow().display() == "file://"));
    }

    #[test]
    fn test_url_parent_trailing_slash() {
        let v = run(r#"Url("/foo/bar/").parent()"#);
        assert!(matches!(&v, Value::Object(obj) if obj.borrow().display() == "file:///foo/bar"));
    }

    #[test]
    fn test_url_parent_at_root() {
        let v = run(r#"str(Url("file:///").parent())"#);
        assert_val(&v, &Value::from("file:///"));
    }

    #[test]
    fn test_url_parent_authority_only() {
        let v = run(r#"str(Url("gs://bucket").parent())"#);
        assert_val(&v, &Value::from("gs://bucket"));
    }

    #[test]
    fn test_url_name_no_path() {
        assert_val(&run(r#"Url("gs://bucket").name()"#), &Value::from(""));
    }

    #[test]
    fn test_url_extension() {
        assert_val(
            &run(r#"Url("/foo/bar.gml").extension()"#),
            &Value::from("gml"),
        );
    }

    #[test]
    fn test_url_name() {
        assert_val(
            &run(r#"Url("/foo/bar.gml").name()"#),
            &Value::from("bar.gml"),
        );
    }

    #[test]
    fn test_url_stem() {
        assert_val(&run(r#"Url("/foo/bar.gml").stem()"#), &Value::from("bar"));
    }

    #[test]
    fn test_url_name_trailing_slash() {
        assert_val(&run(r#"Url("/foo/").name()"#), &Value::from("foo"));
        assert_val(&run(r#"Url("/foo/bar/").name()"#), &Value::from("bar"));
    }

    #[test]
    fn test_url_stem_trailing_slash() {
        assert_val(&run(r#"Url("/foo/bar.gml/").stem()"#), &Value::from("bar"));
    }

    #[test]
    fn test_url_extension_trailing_slash() {
        assert_val(
            &run(r#"Url("/foo/bar.gml/").extension()"#),
            &Value::from("gml"),
        );
    }

    #[test]
    fn test_url_eq() {
        assert_val(
            &run(r#"Url("/foo/bar") == Url("/foo/bar")"#),
            &Value::Bool(true),
        );
        assert_val(
            &run(r#"Url("/foo/bar") == Url("/foo/baz")"#),
            &Value::Bool(false),
        );
    }

    #[test]
    fn test_url_in_array() {
        assert_val(
            &run(r#"Url("/foo/bar") in [Url("/foo/bar")]"#),
            &Value::Bool(true),
        );
        assert_val(
            &run(r#"Url("/foo/bar") in [Url("/foo/baz")]"#),
            &Value::Bool(false),
        );
    }
}
