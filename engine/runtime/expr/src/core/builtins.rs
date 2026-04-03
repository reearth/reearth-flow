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
                Ok(Value::String(
                    p.canonicalize()
                        .map(|c| c.to_string_lossy().into_owned())
                        .unwrap_or_else(|_| self.0.clone()),
                ))
            }
            "parent" => {
                let _ = args;
                Ok(Value::String(
                    std::path::Path::new(&self.0)
                        .parent()
                        .map(|p| p.to_string_lossy().into_owned())
                        .unwrap_or_default(),
                ))
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
