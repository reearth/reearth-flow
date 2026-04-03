use crate::core::error::{Error, Result};
use crate::core::value::{Value, ValueObject};

#[derive(Debug)]
pub struct PathObject(pub String);

impl ValueObject for PathObject {
    fn type_name(&self) -> &'static str {
        "Path"
    }

    fn call_method(&self, method: &str, args: &[Value]) -> Result<Value> {
        match method {
            "resolve" => {
                let _ = args;
                let p = std::path::Path::new(&self.0);
                let s = p.canonicalize()
                    .map(|c| c.to_string_lossy().into_owned())
                    .unwrap_or_else(|_| self.0.clone());
                Ok(Value::Object(Box::new(PathObject(s))))
            }
            "parent" => {
                let _ = args;
                let s = std::path::Path::new(&self.0)
                    .parent()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_default();
                Ok(Value::Object(Box::new(PathObject(s))))
            }
            "extension" => {
                let _ = args;
                Ok(Value::String(
                    std::path::Path::new(&self.0)
                        .extension()
                        .map(|e| e.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                ))
            }
            "filename" => {
                let _ = args;
                Ok(Value::String(
                    std::path::Path::new(&self.0)
                        .file_name()
                        .map(|f| f.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                ))
            }
            "__str__" => {
                let _ = args;
                Ok(Value::String(self.0.clone()))
            }
            "__div__" => {
                let rhs = args.first().and_then(|v| {
                    if let Value::String(s) = v { Some(s.as_str()) } else { None }
                }).ok_or_else(|| Error::Eval {
                    msg: "Path / requires a string".into(),
                })?;
                Ok(Value::Object(Box::new(PathObject(
                    std::path::Path::new(&self.0).join(rhs).to_string_lossy().into_owned(),
                ))))
            }
            m => Err(Error::Eval {
                msg: format!("Path has no method '{m}'"),
            }),
        }
    }

    fn clone_box(&self) -> Box<dyn ValueObject> {
        Box::new(PathObject(self.0.clone()))
    }

    fn eq_box(&self, other: &dyn ValueObject) -> bool {
        other.type_name() == "Path" && format!("{other:?}") == format!("{self:?}")
    }

    fn display(&self) -> String {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::eval::Context;
    use crate::core::eval::eval;
    use crate::core::parser::parse;

    fn run(input: &str) -> Value {
        eval(&parse(input).unwrap(), &Context::new()).unwrap()
    }

    #[test]
    fn test_path_default() {
        let v = run("Path()");
        assert!(matches!(&v, Value::Object(obj) if obj.type_name() == "Path" && obj.display() == "."));
    }

    #[test]
    fn test_path_from_string() {
        let v = run(r#"Path("/foo/bar")"#);
        assert!(matches!(&v, Value::Object(obj) if obj.display() == "/foo/bar"));
    }

    #[test]
    fn test_path_rewrap() {
        let v = run(r#"Path(Path("/foo/bar"))"#);
        assert!(matches!(&v, Value::Object(obj) if obj.display() == "/foo/bar"));
    }

    #[test]
    fn test_path_str() {
        assert_eq!(run(r#"str(Path("/foo/bar"))"#), Value::from("/foo/bar"));
    }

    #[test]
    fn test_path_div() {
        assert_eq!(run(r#"str(Path("/foo") / "bar" / "baz")"#), Value::from("/foo/bar/baz"));
    }

    #[test]
    fn test_path_parent() {
        let v = run(r#"Path("/foo/bar").parent()"#);
        assert!(matches!(&v, Value::Object(obj) if obj.type_name() == "Path" && obj.display() == "/foo"));
    }

    #[test]
    fn test_path_extension() {
        assert_eq!(run(r#"Path("/foo/bar.gml").extension()"#), Value::from("gml"));
    }

    #[test]
    fn test_path_filename() {
        assert_eq!(run(r#"Path("/foo/bar.gml").filename()"#), Value::from("bar.gml"));
    }
}

pub fn builtin_path(args: &[Value]) -> Result<Value> {
    let s = match args.first() {
        None => ".".to_string(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Object(obj)) if obj.type_name() == "Path" => obj.display(),
        Some(v) => return Err(Error::Eval {
            msg: format!("Path() expects a string, got {v:?}"),
        }),
    };
    Ok(Value::Object(Box::new(PathObject(s))))
}
