use crate::core::error::{InnerError, InnerResult};
use crate::core::value::{Object, Value};
use crate::unpack_args;

fn parse_url(s: &str) -> Result<String, String> {
    if s.contains("://") {
        Ok(s.to_string())
    } else if s.starts_with('/') {
        Ok(format!("file://{s}"))
    } else {
        Err(format!("not a valid URI: {s}"))
    }
}

fn url_tail(s: &str) -> &str {
    s.rsplit('/').next().unwrap_or(s)
}

fn url_parent(s: &str) -> Option<String> {
    let s = s.trim_end_matches('/');
    let min = s.find("://").map(|i| i + 3).unwrap_or(0);
    let last = s[min..].rfind('/')? + min;
    if last <= min {
        return None;
    }
    Some(s[..last].to_string())
}

#[derive(Debug, Clone)]
pub struct UrlObject(pub String);

impl Object for UrlObject {
    fn type_name(&self) -> &'static str {
        "Url"
    }

    fn call_method(&mut self, method: &str, args: &[Value]) -> InnerResult<Value> {
        match method {
            "parent" => {
                unpack_args!(args =>);
                let s = url_parent(&self.0)
                    .ok_or_else(|| InnerError::new(format!("Url has no parent: {}", self.0)))?;
                Ok(Value::object(UrlObject(s)))
            }
            "extension" => {
                unpack_args!(args =>);
                let ext = std::path::Path::new(url_tail(&self.0))
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                Ok(Value::String(ext.to_string()))
            }
            "name" => {
                unpack_args!(args =>);
                Ok(Value::String(url_tail(&self.0).to_string()))
            }
            "stem" => {
                unpack_args!(args =>);
                let tail = url_tail(&self.0);
                let stem = std::path::Path::new(tail)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(tail);
                Ok(Value::String(stem.to_string()))
            }
            "__eq__" => {
                let rhs = args
                    .first()
                    .ok_or_else(|| InnerError::new("Url == requires an argument"))?;
                match rhs {
                    Value::Object(obj) if obj.borrow().type_name() == "Url" => {
                        Ok(Value::Bool(self.0 == obj.borrow().display()))
                    }
                    _ => Ok(Value::Bool(false)),
                }
            }
            "__str__" => {
                unpack_args!(args =>);
                Ok(Value::String(self.0.clone()))
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
                let joined = format!("{}/{rhs}", self.0.trim_end_matches('/'));
                Ok(Value::object(UrlObject(joined)))
            }
            m => Err(InnerError::new(format!("Url has no method '{m}'"))),
        }
    }

    fn display(&self) -> String {
        self.0.clone()
    }

    fn serialize(&self) -> Option<Value> {
        Some(Value::String(self.0.clone()))
    }
}

pub fn builtin_url(args: &[Value]) -> InnerResult<Value> {
    let s = match args.first() {
        None => return Err(InnerError::new("Url() requires a string argument")),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Object(obj)) if obj.borrow().type_name() == "Url" => obj.borrow().display(),
        Some(v) => {
            return Err(InnerError::new(format!(
                "Url() expects a string, got {v:?}"
            )))
        }
    };
    let url = parse_url(&s).map_err(InnerError::new)?;
    Ok(Value::object(UrlObject(url)))
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
}
