use uuid::Uuid;

use crate::core::error::{eval_error, Result};
use crate::core::value::{Module, NativeFn, Value};
use crate::expect_arity;

// Lowercase hyphenated form, as returned by every function in this module.
fn uuid_to_value(u: Uuid) -> Value {
    Value::String(u.hyphenated().to_string())
}

fn uuid_v4(args: &[Value]) -> Result<Value> {
    expect_arity("uuid.v4", args, 0, 0)?;
    Ok(uuid_to_value(Uuid::new_v4()))
}

fn uuid_v5(args: &[Value]) -> Result<Value> {
    expect_arity("uuid.v5", args, 2, 2)?;
    let namespace = args[0].as_str()?;
    let namespace = Uuid::try_parse(namespace).map_err(|_| {
        eval_error(format!(
            "uuid.v5: namespace {namespace:?} is not a valid uuid"
        ))
    })?;
    let name = args[1].as_str()?;
    Ok(uuid_to_value(Uuid::new_v5(&namespace, name.as_bytes())))
}

pub fn builtin_uuid() -> Value {
    let mut m = Module::new();
    m.insert("namespace_dns".into(), uuid_to_value(Uuid::NAMESPACE_DNS));
    m.insert("namespace_url".into(), uuid_to_value(Uuid::NAMESPACE_URL));
    m.insert("namespace_oid".into(), uuid_to_value(Uuid::NAMESPACE_OID));
    m.insert("namespace_x500".into(), uuid_to_value(Uuid::NAMESPACE_X500));
    m.insert("v4".into(), Value::Fn(NativeFn::new(uuid_v4)));
    m.insert("v5".into(), Value::Fn(NativeFn::new(uuid_v5)));
    Value::module(m)
}

#[cfg(test)]
mod tests {
    use regex::Regex;

    use crate::core::test_utils::{assert_eval, run, try_run};
    use crate::core::value::Value;

    fn assert_lower_hyphenated(v: &Value, version: char) {
        let Value::String(s) = v else {
            panic!("expected string, got {v:?}");
        };
        let re =
            Regex::new(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$").unwrap();
        assert!(re.is_match(s), "{s} is not lowercase hyphenated");
        assert_eq!(
            s.chars().nth(14),
            Some(version),
            "{s} has the wrong version"
        );
    }

    #[test]
    fn test_namespace_constants() {
        // RFC 9562 section 6.6
        assert_eval(
            "uuid.namespace_dns",
            &[],
            Value::from("6ba7b810-9dad-11d1-80b4-00c04fd430c8"),
        );
        assert_eval(
            "uuid.namespace_url",
            &[],
            Value::from("6ba7b811-9dad-11d1-80b4-00c04fd430c8"),
        );
        assert_eval(
            "uuid.namespace_oid",
            &[],
            Value::from("6ba7b812-9dad-11d1-80b4-00c04fd430c8"),
        );
        assert_eval(
            "uuid.namespace_x500",
            &[],
            Value::from("6ba7b814-9dad-11d1-80b4-00c04fd430c8"),
        );
    }

    #[test]
    fn test_v5_known_vectors() {
        // RFC 9562 appendix A.4
        assert_eval(
            r#"uuid.v5(uuid.namespace_dns, "www.example.com")"#,
            &[],
            Value::from("2ed6657d-e927-568b-95e1-2665a8aea6a2"),
        );
        // Python: uuid.uuid5(uuid.NAMESPACE_DNS, "python.org")
        assert_eval(
            r#"uuid.v5(uuid.namespace_dns, "python.org")"#,
            &[],
            Value::from("886313e1-3b8a-5372-9b90-0c9aee199e5d"),
        );
        // Python: uuid.uuid5(uuid.NAMESPACE_URL, "https://example.com/")
        assert_eval(
            r#"uuid.v5(uuid.namespace_url, "https://example.com/")"#,
            &[],
            Value::from("dd2c1780-811a-5296-81c5-178a0ef488bc"),
        );
    }

    #[test]
    fn test_v5_is_deterministic() {
        let a = run(r#"uuid.v5(uuid.namespace_url, "bldg-1")"#, &[]);
        let b = run(r#"uuid.v5(uuid.namespace_url, "bldg-1")"#, &[]);
        let c = run(r#"uuid.v5(uuid.namespace_url, "bldg-2")"#, &[]);
        assert_eq!(a.to_string(), b.to_string());
        assert_ne!(a.to_string(), c.to_string());
    }

    #[test]
    fn test_v5_namespace_case_insensitive() {
        assert_eval(
            r#"uuid.v5("6BA7B810-9DAD-11D1-80B4-00C04FD430C8", "www.example.com")"#,
            &[],
            Value::from("2ed6657d-e927-568b-95e1-2665a8aea6a2"),
        );
    }

    #[test]
    fn test_format() {
        assert_lower_hyphenated(&run(r#"uuid.v5(uuid.namespace_dns, "x")"#, &[]), '5');
        assert_lower_hyphenated(&run("uuid.v4()", &[]), '4');
    }

    #[test]
    fn test_v4_differs_between_calls() {
        let v = run("[uuid.v4(), uuid.v4()]", &[]);
        let Value::List(items) = v else {
            panic!("expected list, got {v:?}");
        };
        let items = items.borrow();
        assert_ne!(items[0].to_string(), items[1].to_string());
    }

    #[test]
    fn test_v4_differs_between_evaluations() {
        // One compiled expression evaluated twice, as a host does once per feature.
        let expr = crate::compile("uuid.v4()").unwrap();
        let env = crate::default_env();
        let a = crate::eval_unsafe(&expr, &env).unwrap();
        let b = crate::eval_unsafe(&expr, &env).unwrap();
        assert_ne!(a.to_string(), b.to_string());
    }

    #[test]
    fn test_arity_errors() {
        let err = try_run("uuid.v4(1)", &[]).unwrap_err().to_string();
        assert!(
            err.contains("uuid.v4() expected 0 argument(s), got 1"),
            "{err}"
        );
        let err = try_run(r#"uuid.v5("x")"#, &[]).unwrap_err().to_string();
        assert!(
            err.contains("uuid.v5() expected 2 argument(s), got 1"),
            "{err}"
        );
    }

    #[test]
    fn test_type_errors() {
        let err = try_run("uuid.v5(uuid.namespace_dns, 1)", &[])
            .unwrap_err()
            .to_string();
        assert!(err.contains("expected string, got int"), "{err}");
        let err = try_run(r#"uuid.v5(null, "x")"#, &[])
            .unwrap_err()
            .to_string();
        assert!(err.contains("expected string, got null"), "{err}");
    }

    #[test]
    fn test_invalid_namespace() {
        let err = try_run(r#"uuid.v5("bldg", "x")"#, &[])
            .unwrap_err()
            .to_string();
        assert!(
            err.contains(r#"uuid.v5: namespace "bldg" is not a valid uuid"#),
            "{err}"
        );
    }
}
