use std::str::FromStr;

use reearth_flow_common::uri::Uri;

use crate::core::error::{Error, Result};
use crate::core::value::{Value, ValueObject};

#[derive(Debug)]
pub struct UrlObject(pub Uri);

impl ValueObject for UrlObject {
    fn type_name(&self) -> &'static str {
        "Url"
    }

    fn call_method(&self, method: &str, args: &[Value]) -> Result<Value> {
        let _ = args;
        match method {
            "parent" => {
                let uri = self.0.parent().ok_or_else(|| Error::Eval {
                    msg: format!("Url has no parent: {}", self.0.as_str()),
                })?;
                Ok(Value::Object(Box::new(UrlObject(uri))))
            }
            "extension" => Ok(Value::String(
                self.0.extension().unwrap_or_default().to_string(),
            )),
            "name" => Ok(Value::String(
                self.0
                    .file_name()
                    .and_then(|p| p.to_str())
                    .unwrap_or_default()
                    .to_string(),
            )),
            "stem" => {
                let name = self
                    .0
                    .file_name()
                    .and_then(|p| p.to_str())
                    .unwrap_or_default();
                let stem = match name.rfind('.') {
                    Some(i) => &name[..i],
                    None => name,
                };
                Ok(Value::String(stem.to_string()))
            }
            "__str__" => Ok(Value::String(self.0.as_str().to_string())),
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
                    .ok_or_else(|| Error::Eval {
                        msg: "Url / requires a string".into(),
                    })?;
                let joined = self
                    .0
                    .join(rhs)
                    .map_err(|e| Error::Eval { msg: e.to_string() })?;
                Ok(Value::Object(Box::new(UrlObject(joined))))
            }
            m => Err(Error::Eval {
                msg: format!("Url has no method '{m}'"),
            }),
        }
    }

    fn clone_box(&self) -> Box<dyn ValueObject> {
        Box::new(UrlObject(self.0.clone()))
    }

    fn eq_box(&self, other: &dyn ValueObject) -> bool {
        other.type_name() == "Url" && other.display() == self.0.as_str()
    }

    fn display(&self) -> String {
        self.0.as_str().to_string()
    }
}

pub fn builtin_url(args: &[Value]) -> Result<Value> {
    let s = match args.first() {
        None => return Err(Error::Eval { msg: "Url() requires a string argument".into() }),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Object(obj)) if obj.type_name() == "Url" => obj.display(),
        Some(v) => return Err(Error::Eval {
            msg: format!("Url() expects a string, got {v:?}"),
        }),
    };
    let uri = Uri::from_str(&s).map_err(|e| Error::Eval { msg: e.to_string() })?;
    Ok(Value::Object(Box::new(UrlObject(uri))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::eval::{eval, Context};
    use crate::core::parser::parse;

    fn run(input: &str) -> Value {
        let mut ctx = Context::new();
        ctx.register("Url", Box::new(builtin_url));
        eval(&parse(input).unwrap(), &ctx).unwrap()
    }

    #[test]
    fn test_url_from_string() {
        let v = run(r#"Url("/foo/bar")"#);
        assert!(matches!(&v, Value::Object(obj) if obj.display() == "file:///foo/bar"));
    }

    #[test]
    fn test_url_rewrap() {
        let v = run(r#"Url(Url("/foo/bar"))"#);
        assert!(matches!(&v, Value::Object(obj) if obj.display() == "file:///foo/bar"));
    }

    #[test]
    fn test_url_str() {
        assert_eq!(run(r#"str(Url("/foo/bar"))"#), Value::from("file:///foo/bar"));
    }

    #[test]
    fn test_url_div() {
        assert_eq!(
            run(r#"str(Url("/foo") / "bar" / "baz")"#),
            Value::from("file:///foo/bar/baz")
        );
    }

    #[test]
    fn test_url_div_gs() {
        assert_eq!(
            run(r#"str(Url("gs://bucket/artifacts") / "output")"#),
            Value::from("gs://bucket/artifacts/output")
        );
    }

    #[test]
    fn test_url_parent() {
        let v = run(r#"Url("/foo/bar").parent()"#);
        assert!(matches!(&v, Value::Object(obj) if obj.display() == "file:///foo"));
    }

    #[test]
    fn test_url_extension() {
        assert_eq!(run(r#"Url("/foo/bar.gml").extension()"#), Value::from("gml"));
    }

    #[test]
    fn test_url_name() {
        assert_eq!(run(r#"Url("/foo/bar.gml").name()"#), Value::from("bar.gml"));
    }

    #[test]
    fn test_url_stem() {
        assert_eq!(run(r#"Url("/foo/bar.gml").stem()"#), Value::from("bar"));
    }
}
