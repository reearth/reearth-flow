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
