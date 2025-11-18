use crate::helper::execute;

#[test]
#[ignore] // Ignore by default as it requires network access
fn test_http_caller() {
    let result = execute("http/caller", vec!["test_data.json"]);
    assert!(result.is_ok());
}

