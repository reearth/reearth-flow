use crate::helper::execute;

/// Integration test for HTTPCaller action.
/// Ignored by default as it requires network access to jsonplaceholder.typicode.com.
/// Run with: cargo test test_http_caller -- --ignored
#[test]
#[ignore]
fn test_http_caller() {
    let result = execute("http/caller", vec!["test_data.json"]);
    assert!(result.is_ok());
}
