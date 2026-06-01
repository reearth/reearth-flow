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
                Ok(regex_find(&self.regex, s))
            }
            "find_all" => {
                unpack_args!(args => s);
                let Value::String(s) = s else {
                    return Err(InnerError::new(
                        "Regex.find_all() requires a string argument",
                    ));
                };
                Ok(Value::array(regex_find_all(&self.regex, s)))
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

// follow Python re.findall element type convention
fn capture_to_value(cap: &regex::Captures, num_groups: usize) -> Value {
    if num_groups == 1 {
        cap.get(1)
            .map(|m| Value::String(m.as_str().to_string()))
            .unwrap_or(Value::Null)
    } else {
        Value::array(
            (1..=num_groups)
                .map(|i| {
                    cap.get(i)
                        .map(|m| Value::String(m.as_str().to_string()))
                        .unwrap_or(Value::Null)
                })
                .collect(),
        )
    }
}

// note that `if Regex(".*").find("1")` is falsy even though constructing patterns that match empty string is considered bad practice
// Still, for dynamic/user input pattern, it is safer to test with `== null`
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

fn regex_find_all(regex: &Regex, s: &str) -> Vec<Value> {
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
    // FlowExpr is general-purpose. Do not do implicit caching for Regex patterns here.
    // Compile-time constant folding might be a proper future solution, but currently it is overkill.
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
    fn test_regex_optional_group_null() {
        // single optional group that did not participate → Null, not ""
        assert_val(&run(r#"Regex(r"a(b)?c").find("ac")"#, &[]), &Value::Null);
        // single optional group that did participate → String
        assert_val(
            &run(r#"Regex(r"a(b)?c").find("abc")"#, &[]),
            &Value::from("b"),
        );
        // multi-group: second optional group absent → Null in that slot
        assert_val(
            &run(r#"Regex(r"(\d+)(x)?").find("123")"#, &[]),
            &Value::array(vec![Value::from("123"), Value::Null]),
        );
        // find_all: optional group absent → Null per match
        assert_val(
            &run(r#"Regex(r"(\d+)(x)?").find_all("123 456x")"#, &[]),
            &Value::array(vec![
                Value::array(vec![Value::from("123"), Value::Null]),
                Value::array(vec![Value::from("456"), Value::from("x")]),
            ]),
        );
    }

    #[test]
    fn test_regex_find_all() {
        assert_val(
            &run(r#"Regex(r"\d+").find_all("abc 123 def 456")"#, &[]),
            &Value::array(vec![Value::from("123"), Value::from("456")]),
        );
        assert_val(
            &run(r#"Regex(r"(\d+)").find_all("abc 123 def 456")"#, &[]),
            &Value::array(vec![Value::from("123"), Value::from("456")]),
        );
        assert_val(
            &run(r#"Regex(r"(\w+)@(\w+)").find_all("a@b x@y")"#, &[]),
            &Value::array(vec![
                Value::array(vec![Value::from("a"), Value::from("b")]),
                Value::array(vec![Value::from("x"), Value::from("y")]),
            ]),
        );
        assert_val(
            &run(r#"Regex(r"\d+").find_all("no digits here")"#, &[]),
            &Value::array(vec![]),
        );
    }
}
