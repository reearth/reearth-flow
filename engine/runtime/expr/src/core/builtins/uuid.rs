use uuid::Uuid;

use crate::core::error::{eval_error, Result};
use crate::core::value::{Module, NativeFn, Value};
use crate::expect_arity;

/// `uuid.NAMESPACE_FLOW`. Fixed forever: changing it changes every id a
/// workflow derives in it. It is the v5 uuid of the URL
/// `https://reearth.io/flow/uuid` in the RFC 9562 URL namespace.
const NAMESPACE_FLOW: Uuid = Uuid::from_u128(0x8d8d96bb_df81_560f_bbd5_794fe22e23c0);

// Lowercase hyphenated form, as returned by every function in this module.
fn uuid_to_value(u: Uuid) -> Value {
    Value::String(u.hyphenated().to_string())
}

fn uuid_uuid5(args: &[Value]) -> Result<Value> {
    expect_arity("uuid.uuid5", args, 2, 2)?;
    let ns = args[0].as_str()?;
    let namespace = Uuid::try_parse(ns)
        .map_err(|_| eval_error(format!("uuid.uuid5: namespace {ns:?} is not a valid uuid")))?;
    // Only strings, as in Python: a number's text form is easy to get wrong
    // (`1` vs `1.0`), so the caller converts it with `str()`.
    let name = args[1].as_str()?;
    Ok(uuid_to_value(Uuid::new_v5(&namespace, name.as_bytes())))
}

fn uuid_uuid4(args: &[Value]) -> Result<Value> {
    expect_arity("uuid.uuid4", args, 0, 0)?;
    Ok(uuid_to_value(Uuid::new_v4()))
}

pub fn builtin_uuid() -> Value {
    let mut m = Module::new();
    m.insert("uuid4".into(), Value::Fn(NativeFn::new(uuid_uuid4)));
    m.insert("uuid5".into(), Value::Fn(NativeFn::new(uuid_uuid5)));
    m.insert("NAMESPACE_FLOW".into(), uuid_to_value(NAMESPACE_FLOW));
    Value::module(m)
}

#[cfg(test)]
mod tests {
    use regex::Regex;
    use uuid::Uuid;

    use super::NAMESPACE_FLOW;
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
    fn test_namespace_flow() {
        assert_eq!(
            NAMESPACE_FLOW,
            Uuid::new_v5(&Uuid::NAMESPACE_URL, b"https://reearth.io/flow/uuid")
        );
        assert_eval(
            "uuid.NAMESPACE_FLOW",
            &[],
            Value::from("8d8d96bb-df81-560f-bbd5-794fe22e23c0"),
        );
    }

    #[test]
    fn test_uuid5_known_vectors() {
        // Python: uuid.uuid5(uuid.UUID("8d8d96bb-df81-560f-bbd5-794fe22e23c0"), name)
        assert_eval(
            r#"uuid.uuid5(uuid.NAMESPACE_FLOW, "13101-bldg-1")"#,
            &[],
            Value::from("9a37aa71-d7f5-55ce-b79a-79a40298237d"),
        );
        assert_eval(
            r#"uuid.uuid5(uuid.NAMESPACE_FLOW, "www.example.com")"#,
            &[],
            Value::from("d8b44a80-9ae1-5b50-b959-8c507482f987"),
        );
        // RFC 9562 appendix A.4: "www.example.com" in the DNS namespace
        assert_eval(
            r#"uuid.uuid5("6ba7b810-9dad-11d1-80b4-00c04fd430c8", "www.example.com")"#,
            &[],
            Value::from("2ed6657d-e927-568b-95e1-2665a8aea6a2"),
        );
    }

    #[test]
    fn test_uuid5_is_deterministic() {
        let a = run(r#"uuid.uuid5(uuid.NAMESPACE_FLOW, "bldg-1")"#, &[]);
        let b = run(r#"uuid.uuid5(uuid.NAMESPACE_FLOW, "bldg-1")"#, &[]);
        let c = run(r#"uuid.uuid5(uuid.NAMESPACE_FLOW, "bldg-2")"#, &[]);
        assert_eq!(a.to_string(), b.to_string());
        assert_ne!(a.to_string(), c.to_string());
    }

    #[test]
    fn test_uuid5_numbers_need_str() {
        assert_eval(
            "uuid.uuid5(uuid.NAMESPACE_FLOW, str(1))",
            &[],
            Value::from("144684ee-5071-5be1-a37e-c76a39d34488"),
        );
    }

    #[test]
    fn test_uuid5_namespace_case_insensitive() {
        assert_eval(
            r#"uuid.uuid5("6BA7B810-9DAD-11D1-80B4-00C04FD430C8", "www.example.com")"#,
            &[],
            Value::from("2ed6657d-e927-568b-95e1-2665a8aea6a2"),
        );
    }

    #[test]
    fn test_format() {
        assert_lower_hyphenated(&run(r#"uuid.uuid5(uuid.NAMESPACE_FLOW, "x")"#, &[]), '5');
        assert_lower_hyphenated(&run("uuid.uuid4()", &[]), '4');
    }

    #[test]
    fn test_uuid4_differs_between_calls() {
        let v = run("[uuid.uuid4(), uuid.uuid4()]", &[]);
        let Value::List(items) = v else {
            panic!("expected list, got {v:?}");
        };
        let items = items.borrow();
        assert_ne!(items[0].to_string(), items[1].to_string());
    }

    #[test]
    fn test_uuid4_differs_between_evaluations() {
        // One compiled expression evaluated twice, as a host does once per feature.
        let expr = crate::compile("uuid.uuid4()").unwrap();
        let env = crate::default_env();
        let a = crate::eval_unsafe(&expr, &env).unwrap();
        let b = crate::eval_unsafe(&expr, &env).unwrap();
        assert_ne!(a.to_string(), b.to_string());
    }

    #[test]
    fn test_arity_errors() {
        let err = try_run("uuid.uuid4(1)", &[]).unwrap_err().to_string();
        assert!(
            err.contains("uuid.uuid4() expected 0 argument(s), got 1"),
            "{err}"
        );
        // The namespace is required, as in Python.
        let err = try_run(r#"uuid.uuid5("x")"#, &[]).unwrap_err().to_string();
        assert!(
            err.contains("uuid.uuid5() expected 2 argument(s), got 1"),
            "{err}"
        );
    }

    #[test]
    fn test_type_errors() {
        for name in ["1", "1.5", "true", "null"] {
            let expr = format!("uuid.uuid5(uuid.NAMESPACE_FLOW, {name})");
            let err = try_run(&expr, &[]).unwrap_err().to_string();
            assert!(err.contains("expected string, got"), "{expr}: {err}");
        }
        let err = try_run(r#"uuid.uuid5(1, "x")"#, &[])
            .unwrap_err()
            .to_string();
        assert!(err.contains("expected string, got int"), "{err}");
    }

    #[test]
    fn test_invalid_namespace() {
        // The Python argument order: a name passed first is not a namespace.
        let err = try_run(r#"uuid.uuid5("bldg", uuid.NAMESPACE_FLOW)"#, &[])
            .unwrap_err()
            .to_string();
        assert!(
            err.contains(r#"uuid.uuid5: namespace "bldg" is not a valid uuid"#),
            "{err}"
        );
    }
}
