use std::rc::Rc;

use crate::core::error::{eval_error, Result};
use crate::core::value::{ImmutableObject, NativeFn, TypeValue, Value};

use crate::expect_arity;
use regex::Regex;

thread_local! {
    static REGEX_TYPE: Rc<TypeValue> = Rc::new(TypeValue::new(
        "Regex",
        Some(NativeFn::new(construct)),
    ));
}

pub fn regex_type_value() -> Rc<TypeValue> {
    REGEX_TYPE.with(Rc::clone)
}

#[derive(Debug, Clone)]
pub struct RegexObject {
    pattern: String,
    regex: Regex,
}

impl ImmutableObject for RegexObject {
    fn type_object(&self) -> Rc<TypeValue> {
        regex_type_value()
    }

    fn call_method(&self, method: &str, args: &[Value]) -> Result<Value> {
        match method {
            "find" => {
                expect_arity("Regex.find", args, 1, 1)?;
                Ok(regex_find(&self.regex, args[0].as_str()?))
            }
            "find_all" => {
                expect_arity("Regex.find_all", args, 1, 1)?;
                Ok(Value::list(regex_find_all(&self.regex, args[0].as_str()?)))
            }
            "split" => {
                expect_arity("Regex.split", args, 1, 2)?;
                let s = args[0].as_str()?;
                let parts = if args.len() == 2 {
                    let limit = args[1].as_int()?;
                    self.regex
                        .splitn(s, (limit + 1) as usize)
                        .map(|p| Value::String(p.to_string()))
                        .collect()
                } else {
                    self.regex
                        .split(s)
                        .map(|p| Value::String(p.to_string()))
                        .collect()
                };
                Ok(Value::list(parts))
            }
            m => Err(eval_error(format!("Regex has no method '{m}'"))),
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
        Value::list(
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

pub fn construct(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(eval_error(format!(
            "Regex() expected 1 argument, got {}",
            args.len()
        )));
    }
    let pattern = args[0].as_str()?.to_string();
    // FlowExpr is general-purpose. Do not do implicit caching for Regex patterns here.
    // Compile-time constant folding might be a proper future solution, but currently it is overkill.
    let regex = Regex::new(&pattern).map_err(|e| eval_error(format!("invalid regex: {e}")))?;
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
            &Value::list(vec![Value::from("123"), Value::Null]),
        );
        // find_all: optional group absent → Null per match
        assert_val(
            &run(r#"Regex(r"(\d+)(x)?").find_all("123 456x")"#, &[]),
            &Value::list(vec![
                Value::list(vec![Value::from("123"), Value::Null]),
                Value::list(vec![Value::from("456"), Value::from("x")]),
            ]),
        );
    }

    #[test]
    fn test_regex_split() {
        assert_val(
            &run(r#"Regex(r"\d+").split("a1b2c")"#, &[]),
            &Value::list(vec![Value::from("a"), Value::from("b"), Value::from("c")]),
        );
        assert_val(
            &run(r#"Regex(r",").split("a,,b")"#, &[]),
            &Value::list(vec![Value::from("a"), Value::from(""), Value::from("b")]),
        );
        assert_val(
            &run(r#"Regex(r"\d+").split("no digits")"#, &[]),
            &Value::list(vec![Value::from("no digits")]),
        );
        assert_val(
            &run(r#"Regex(r"\d+").split("a1b2c3d", 2)"#, &[]),
            &Value::list(vec![Value::from("a"), Value::from("b"), Value::from("c3d")]),
        );
    }

    #[test]
    fn test_regex_find_all() {
        assert_val(
            &run(r#"Regex(r"\d+").find_all("abc 123 def 456")"#, &[]),
            &Value::list(vec![Value::from("123"), Value::from("456")]),
        );
        assert_val(
            &run(r#"Regex(r"(\d+)").find_all("abc 123 def 456")"#, &[]),
            &Value::list(vec![Value::from("123"), Value::from("456")]),
        );
        assert_val(
            &run(r#"Regex(r"(\w+)@(\w+)").find_all("a@b x@y")"#, &[]),
            &Value::list(vec![
                Value::list(vec![Value::from("a"), Value::from("b")]),
                Value::list(vec![Value::from("x"), Value::from("y")]),
            ]),
        );
        assert_val(
            &run(r#"Regex(r"\d+").find_all("no digits here")"#, &[]),
            &Value::list(vec![]),
        );
    }
}
