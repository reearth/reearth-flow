use uuid::Uuid;

use crate::core::error::{eval_error, Result};
use crate::core::value::{format_float, Module, NativeFn, Value};
use crate::expect_arity;

/// Namespace `uuid.of` uses when none is given. Fixed forever: changing it
/// changes every id a workflow derives. It is the v5 uuid of the URL
/// `https://reearth.io/flow/uuid` in the RFC 9562 URL namespace.
const DEFAULT_NAMESPACE: Uuid = Uuid::from_u128(0x8d8d96bb_df81_560f_bbd5_794fe22e23c0);

// Lowercase hyphenated form, as returned by every function in this module.
fn uuid_to_value(u: Uuid) -> Value {
    Value::String(u.hyphenated().to_string())
}

// Numbers are hashed in the same text form `str()` gives them.
fn name_text(v: &Value) -> Result<String> {
    match v {
        Value::String(s) => Ok(s.clone()),
        Value::Int(n) => Ok(n.to_string()),
        Value::Float(f) => Ok(format_float(*f)),
        other => Err(eval_error(format!(
            "expected string or number, got {}",
            other.type_name()
        ))),
    }
}

fn uuid_of(args: &[Value]) -> Result<Value> {
    expect_arity("uuid.of", args, 1, 2)?;
    let name = name_text(&args[0])?;
    let namespace = match args.get(1) {
        None => DEFAULT_NAMESPACE,
        Some(ns) => {
            let ns = ns.as_str()?;
            Uuid::try_parse(ns)
                .map_err(|_| eval_error(format!("uuid.of: namespace {ns:?} is not a valid uuid")))?
        }
    };
    Ok(uuid_to_value(Uuid::new_v5(&namespace, name.as_bytes())))
}

fn uuid_random(args: &[Value]) -> Result<Value> {
    expect_arity("uuid.random", args, 0, 0)?;
    Ok(uuid_to_value(Uuid::new_v4()))
}

pub fn builtin_uuid() -> Value {
    let mut m = Module::new();
    m.insert("of".into(), Value::Fn(NativeFn::new(uuid_of)));
    m.insert("random".into(), Value::Fn(NativeFn::new(uuid_random)));
    Value::module(m)
}

#[cfg(test)]
mod tests {
    use regex::Regex;
    use uuid::Uuid;

    use super::DEFAULT_NAMESPACE;
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
    fn test_default_namespace_derivation() {
        assert_eq!(
            DEFAULT_NAMESPACE,
            Uuid::new_v5(&Uuid::NAMESPACE_URL, b"https://reearth.io/flow/uuid")
        );
    }

    #[test]
    fn test_of_known_vectors() {
        // Python: uuid.uuid5(uuid.UUID("8d8d96bb-df81-560f-bbd5-794fe22e23c0"), name)
        assert_eval(
            r#"uuid.of("13101-bldg-1")"#,
            &[],
            Value::from("9a37aa71-d7f5-55ce-b79a-79a40298237d"),
        );
        assert_eval(
            r#"uuid.of("www.example.com")"#,
            &[],
            Value::from("d8b44a80-9ae1-5b50-b959-8c507482f987"),
        );
        // RFC 9562 appendix A.4: "www.example.com" in the DNS namespace
        assert_eval(
            r#"uuid.of("www.example.com", "6ba7b810-9dad-11d1-80b4-00c04fd430c8")"#,
            &[],
            Value::from("2ed6657d-e927-568b-95e1-2665a8aea6a2"),
        );
    }

    #[test]
    fn test_of_is_deterministic() {
        let a = run(r#"uuid.of("bldg-1")"#, &[]);
        let b = run(r#"uuid.of("bldg-1")"#, &[]);
        let c = run(r#"uuid.of("bldg-2")"#, &[]);
        assert_eq!(a.to_string(), b.to_string());
        assert_ne!(a.to_string(), c.to_string());
    }

    #[test]
    fn test_of_numbers_hash_as_their_str_form() {
        assert_eval(
            "uuid.of(1)",
            &[],
            Value::from("144684ee-5071-5be1-a37e-c76a39d34488"),
        );
        assert_eval(r#"uuid.of(1) == uuid.of("1")"#, &[], Value::Bool(true));
        assert_eval("uuid.of(1.5) == uuid.of(str(1.5))", &[], Value::Bool(true));
    }

    #[test]
    fn test_of_namespace_case_insensitive() {
        assert_eval(
            r#"uuid.of("www.example.com", "6BA7B810-9DAD-11D1-80B4-00C04FD430C8")"#,
            &[],
            Value::from("2ed6657d-e927-568b-95e1-2665a8aea6a2"),
        );
    }

    #[test]
    fn test_format() {
        assert_lower_hyphenated(&run(r#"uuid.of("x")"#, &[]), '5');
        assert_lower_hyphenated(&run("uuid.random()", &[]), '4');
    }

    #[test]
    fn test_random_differs_between_calls() {
        let v = run("[uuid.random(), uuid.random()]", &[]);
        let Value::List(items) = v else {
            panic!("expected list, got {v:?}");
        };
        let items = items.borrow();
        assert_ne!(items[0].to_string(), items[1].to_string());
    }

    #[test]
    fn test_random_differs_between_evaluations() {
        // One compiled expression evaluated twice, as a host does once per feature.
        let expr = crate::compile("uuid.random()").unwrap();
        let env = crate::default_env();
        let a = crate::eval_unsafe(&expr, &env).unwrap();
        let b = crate::eval_unsafe(&expr, &env).unwrap();
        assert_ne!(a.to_string(), b.to_string());
    }

    #[test]
    fn test_arity_errors() {
        let err = try_run("uuid.random(1)", &[]).unwrap_err().to_string();
        assert!(
            err.contains("uuid.random() expected 0 argument(s), got 1"),
            "{err}"
        );
        let err = try_run("uuid.of()", &[]).unwrap_err().to_string();
        assert!(
            err.contains("uuid.of() expected 1 to 2 argument(s), got 0"),
            "{err}"
        );
    }

    #[test]
    fn test_type_errors() {
        // null usually means a missing attribute; hashing it would give every
        // such feature the same id.
        let err = try_run("uuid.of(null)", &[]).unwrap_err().to_string();
        assert!(err.contains("expected string or number, got nullType"), "{err}");
        let err = try_run("uuid.of(true)", &[]).unwrap_err().to_string();
        assert!(err.contains("expected string or number, got bool"), "{err}");
        let err = try_run(r#"uuid.of("x", 1)"#, &[]).unwrap_err().to_string();
        assert!(err.contains("expected string, got int"), "{err}");
    }

    #[test]
    fn test_invalid_namespace() {
        let err = try_run(r#"uuid.of("x", "bldg")"#, &[])
            .unwrap_err()
            .to_string();
        assert!(
            err.contains(r#"uuid.of: namespace "bldg" is not a valid uuid"#),
            "{err}"
        );
    }
}
