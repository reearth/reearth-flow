use crate::core::error::{InnerError, InnerResult};
use crate::core::value::{ImmutableObject, Value};
use crate::unpack_args;
use url::Url;

fn parse_url(s: &str) -> Result<UrlObject, String> {
    let url = if s.contains("://") {
        Url::parse(s).map_err(|e| format!("not a valid URI: {e}"))?
    } else if s.starts_with('/') {
        Url::parse(&format!("file://{s}")).map_err(|e| format!("not a valid URI: {e}"))?
    } else {
        return Err(format!("not a valid URI: {s}"));
    };
    Ok(UrlObject { url })
}

#[derive(Debug, Clone)]
pub struct UrlObject {
    url: Url,
}

impl UrlObject {
    fn name(&self) -> &str {
        self.url
            .path()
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .unwrap_or("")
    }

    fn parent(&self) -> Self {
        let path = self.url.path();
        let new_path = if let Some(stripped) = path.strip_suffix('/') {
            if stripped.is_empty() || !stripped.contains('/') {
                return self.clone();
            }
            stripped.to_string()
        } else {
            match path.rfind('/') {
                None => return self.clone(),
                Some(0) => "/".to_string(),
                Some(i) => path[..i].to_string(),
            }
        };
        let mut url = self.url.clone();
        url.set_path(&new_path);
        Self { url }
    }
}

impl ImmutableObject for UrlObject {
    fn type_name(&self) -> &'static str {
        "Url"
    }

    fn get_property(&self, name: &str) -> Option<crate::core::error::InnerResult<Value>> {
        match name {
            "parent" => Some(Ok(Value::object(self.parent()))),
            "name" => Some(Ok(Value::String(self.name().to_string()))),
            "suffix" => {
                let ext = std::path::Path::new(self.name())
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                Some(Ok(Value::String(ext.to_string())))
            }
            _ => None,
        }
    }

    fn call_method(&self, method: &str, args: &[Value]) -> InnerResult<Value> {
        match method {
            "__eq__" => {
                unpack_args!(args => rhs);
                match rhs {
                    Value::Object(obj) if obj.type_name() == "Url" => {
                        Ok(Value::Bool(self.url.as_str() == obj.display()))
                    }
                    _ => Ok(Value::Bool(false)),
                }
            }
            "__str__" => {
                unpack_args!(args =>);
                Ok(Value::String(self.url.to_string()))
            }
            "__div__" => {
                unpack_args!(args => rhs);
                let Value::String(rhs) = rhs else {
                    return Err(InnerError::new("Url / requires a string"));
                };
                let new_path = format!("{}/{rhs}", self.url.path().trim_end_matches('/'));
                let mut url = self.url.clone();
                url.set_path(&new_path);
                Ok(Value::object(Self { url }))
            }
            m => Err(InnerError::new(format!("Url has no method '{m}'"))),
        }
    }

    fn display(&self) -> String {
        self.url.to_string()
    }

    fn serialize(&self) -> Option<Value> {
        Some(Value::String(self.url.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use crate::core::test_utils::{assert_val, run};
    use crate::core::value::Value;

    fn url_display(expr: &str) -> String {
        match run(expr, &[]) {
            Value::Object(obj) => obj.display(),
            v => panic!("expected Object, got {v:?}"),
        }
    }

    #[test]
    fn test_url_construct() {
        assert_eq!(url_display(r#"Url("/foo/bar")"#), "file:///foo/bar");
        assert_eq!(url_display(r#"Url(Url("/foo/bar"))"#), "file:///foo/bar");
    }

    #[test]
    fn test_url_str() {
        assert_val(
            &run(r#"str(Url("/foo/bar"))"#, &[]),
            &Value::from("file:///foo/bar"),
        );
    }

    #[test]
    fn test_url_div() {
        assert_val(
            &run(r#"str(Url("/foo") / "bar" / "baz")"#, &[]),
            &Value::from("file:///foo/bar/baz"),
        );
        assert_val(
            &run(r#"str(Url("gs://bucket/artifacts") / "output")"#, &[]),
            &Value::from("gs://bucket/artifacts/output"),
        );
    }

    #[test]
    fn test_url_parent() {
        assert_eq!(url_display(r#"Url("/foo/bar").parent"#), "file:///foo");
        assert_eq!(url_display(r#"Url("/foo").parent"#), "file:///");
        assert_eq!(url_display(r#"Url("/foo/bar/").parent"#), "file:///foo/bar");
        assert_val(
            &run(r#"str(Url("file:///").parent)"#, &[]),
            &Value::from("file:///"),
        );
        assert_val(
            &run(r#"str(Url("gs://bucket").parent)"#, &[]),
            &Value::from("gs://bucket"),
        );
    }

    #[test]
    fn test_url_name() {
        assert_val(&run(r#"Url("gs://bucket").name"#, &[]), &Value::from(""));
        assert_val(
            &run(r#"Url("/foo/bar.gml").name"#, &[]),
            &Value::from("bar.gml"),
        );
        assert_val(&run(r#"Url("/foo/").name"#, &[]), &Value::from("foo"));
        assert_val(&run(r#"Url("/foo/bar/").name"#, &[]), &Value::from("bar"));
    }

    #[test]
    fn test_url_suffix() {
        assert_val(
            &run(r#"Url("/foo/bar.gml").suffix"#, &[]),
            &Value::from("gml"),
        );
        assert_val(
            &run(r#"Url("/foo/bar.gml/").suffix"#, &[]),
            &Value::from("gml"),
        );
    }

    #[test]
    fn test_url_eq() {
        assert_val(
            &run(r#"Url("/foo/bar") == Url("/foo/bar")"#, &[]),
            &Value::Bool(true),
        );
        assert_val(
            &run(r#"Url("/foo/bar") == Url("/foo/baz")"#, &[]),
            &Value::Bool(false),
        );
    }

    #[test]
    fn test_url_in_array() {
        assert_val(
            &run(r#"Url("/foo/bar") in [Url("/foo/bar")]"#, &[]),
            &Value::Bool(true),
        );
        assert_val(
            &run(r#"Url("/foo/bar") in [Url("/foo/baz")]"#, &[]),
            &Value::Bool(false),
        );
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
        Some(Value::Object(obj)) if obj.type_name() == "Url" => obj.display(),
        Some(v) => {
            return Err(InnerError::new(format!(
                "Url() expects a string, got {}",
                v.type_name()
            )))
        }
    };
    parse_url(&s).map(Value::object).map_err(InnerError::new)
}
