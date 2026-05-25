pub mod array;
pub mod map;
pub mod math;
pub mod str;
mod url;

pub use math::builtin_math;
pub use url::builtin_url;

#[cfg(test)]
mod tests {
    use crate::core::test_utils::{assert_val, run};
    use crate::core::value::Value;

    #[test]
    fn test_url_from_string() {
        let v = run(r#"Url("/foo/bar")"#, &[]);
        assert!(matches!(&v, Value::Object(obj) if obj.display() == "file:///foo/bar"));
    }

    #[test]
    fn test_url_rewrap() {
        let v = run(r#"Url(Url("/foo/bar"))"#, &[]);
        assert!(matches!(&v, Value::Object(obj) if obj.display() == "file:///foo/bar"));
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
    }

    #[test]
    fn test_url_div_gs() {
        assert_val(
            &run(r#"str(Url("gs://bucket/artifacts") / "output")"#, &[]),
            &Value::from("gs://bucket/artifacts/output"),
        );
    }

    #[test]
    fn test_url_parent() {
        let v = run(r#"Url("/foo/bar").parent()"#, &[]);
        assert!(matches!(&v, Value::Object(obj) if obj.display() == "file:///foo"));
    }

    #[test]
    fn test_url_parent_single_level() {
        let v = run(r#"Url("/foo").parent()"#, &[]);
        assert!(matches!(&v, Value::Object(obj) if obj.display() == "file:///"));
    }

    #[test]
    fn test_url_parent_trailing_slash() {
        let v = run(r#"Url("/foo/bar/").parent()"#, &[]);
        assert!(matches!(&v, Value::Object(obj) if obj.display() == "file:///foo/bar"));
    }

    #[test]
    fn test_url_parent_at_root() {
        let v = run(r#"str(Url("file:///").parent())"#, &[]);
        assert_val(&v, &Value::from("file:///"));
    }

    #[test]
    fn test_url_parent_authority_only() {
        let v = run(r#"str(Url("gs://bucket").parent())"#, &[]);
        assert_val(&v, &Value::from("gs://bucket"));
    }

    #[test]
    fn test_url_name_no_path() {
        assert_val(&run(r#"Url("gs://bucket").name()"#, &[]), &Value::from(""));
    }

    #[test]
    fn test_url_extension() {
        assert_val(
            &run(r#"Url("/foo/bar.gml").extension()"#, &[]),
            &Value::from("gml"),
        );
    }

    #[test]
    fn test_url_name() {
        assert_val(
            &run(r#"Url("/foo/bar.gml").name()"#, &[]),
            &Value::from("bar.gml"),
        );
    }

    #[test]
    fn test_url_stem() {
        assert_val(
            &run(r#"Url("/foo/bar.gml").stem()"#, &[]),
            &Value::from("bar"),
        );
    }

    #[test]
    fn test_url_name_trailing_slash() {
        assert_val(&run(r#"Url("/foo/").name()"#, &[]), &Value::from("foo"));
        assert_val(&run(r#"Url("/foo/bar/").name()"#, &[]), &Value::from("bar"));
    }

    #[test]
    fn test_url_stem_trailing_slash() {
        assert_val(
            &run(r#"Url("/foo/bar.gml/").stem()"#, &[]),
            &Value::from("bar"),
        );
    }

    #[test]
    fn test_url_extension_trailing_slash() {
        assert_val(
            &run(r#"Url("/foo/bar.gml/").extension()"#, &[]),
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
