use crate::core::error::{InnerError, InnerResult};
use crate::core::value::{ImmutableObject, Value};
use crate::unpack_args;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct RegexObject {
    pattern: String,
    regex: Regex,
}

impl ImmutableObject for RegexObject {
    fn type_name(&self) -> &'static str {
        "Regex"
    }

    fn get_property(&self, _name: &str) -> Option<InnerResult<Value>> {
        None
    }

    fn call_method(&self, method: &str, args: &[Value]) -> InnerResult<Value> {
        match method {
            "find" => {
                unpack_args!(args => s);
                let Value::String(s) = s else {
                    return Err(InnerError::new("Regex.find() requires a string argument"));
                };
                Ok(regex_find(&self.regex, &s))
            }
            "findall" => {
                unpack_args!(args => s);
                let Value::String(s) = s else {
                    return Err(InnerError::new("Regex.findall() requires a string argument"));
                };
                Ok(Value::array(regex_findall(&self.regex, &s)))
            }
            m => Err(InnerError::new(format!("Regex has no method '{m}'"))),
        }
    }

    fn display(&self) -> String {
        self.pattern.clone()
    }

    fn serialize(&self) -> Option<Value> {
        Some(Value::String(self.pattern.clone()))
    }
}

fn capture_to_value(cap: &regex::Captures, num_groups: usize) -> Value {
    if num_groups == 1 {
        Value::String(cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string())
    } else {
        Value::array(
            (1..=num_groups)
                .map(|i| Value::String(cap.get(i).map(|m| m.as_str()).unwrap_or("").to_string()))
                .collect(),
        )
    }
}

fn regex_find(regex: &Regex, s: &str) -> Value {
    let num_groups = regex.captures_len() - 1;
    if num_groups == 0 {
        regex
            .find(s)
            .map(|m| Value::String(m.as_str().to_string()))
            .unwrap_or(Value::Null)
    } else {
        regex
            .captures(s)
            .map(|cap| capture_to_value(&cap, num_groups))
            .unwrap_or(Value::Null)
    }
}

fn regex_findall(regex: &Regex, s: &str) -> Vec<Value> {
    let num_groups = regex.captures_len() - 1;
    if num_groups == 0 {
        regex
            .find_iter(s)
            .map(|m| Value::String(m.as_str().to_string()))
            .collect()
    } else {
        regex
            .captures_iter(s)
            .map(|cap| capture_to_value(&cap, num_groups))
            .collect()
    }
}

pub fn builtin_regex(args: &[Value]) -> InnerResult<Value> {
    if args.len() != 1 {
        return Err(InnerError::new(format!(
            "Regex() expected 1 argument, got {}",
            args.len()
        )));
    }
    let pattern = match &args[0] {
        Value::String(s) => s.clone(),
        v => {
            return Err(InnerError::new(format!(
                "Regex() expects a string, got {}",
                v.type_name()
            )))
        }
    };
    let regex = Regex::new(&pattern).map_err(|e| InnerError::new(format!("invalid regex: {e}")))?;
    Ok(Value::object(RegexObject { pattern, regex }))
}

#[cfg(test)]
mod tests {
    use crate::core::test_utils::{assert_val, run};
    use crate::core::value::Value;

    #[test]
    fn test_regex_find() {
        assert_val(
            &run(r#"Regex(r"\d+").find("abc 123 def 456")"#, &[]),
            &Value::from("123"),
        );
        assert_val(
            &run(r#"Regex(r"(\d+)").find("abc 123")"#, &[]),
            &Value::from("123"),
        );
        assert_val(
            &run(r#"Regex(r"\d+").find("no digits")"#, &[]),
            &Value::Null,
        );
    }

    #[test]
    fn test_regex_findall() {
        assert_val(
            &run(r#"Regex(r"\d+").findall("abc 123 def 456")"#, &[]),
            &Value::array(vec![Value::from("123"), Value::from("456")]),
        );
        assert_val(
            &run(r#"Regex(r"(\d+)").findall("abc 123 def 456")"#, &[]),
            &Value::array(vec![Value::from("123"), Value::from("456")]),
        );
        assert_val(
            &run(r#"Regex(r"(\w+)@(\w+)").findall("a@b x@y")"#, &[]),
            &Value::array(vec![
                Value::array(vec![Value::from("a"), Value::from("b")]),
                Value::array(vec![Value::from("x"), Value::from("y")]),
            ]),
        );
        assert_val(
            &run(r#"Regex(r"\d+").findall("no digits here")"#, &[]),
            &Value::array(vec![]),
        );
    }
}
