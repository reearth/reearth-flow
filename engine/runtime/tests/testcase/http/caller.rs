use crate::helper::execute;

#[test]
#[ignore]
fn test_http_caller() {
    let result = execute("http/caller", vec!["test_data.json"]);
    assert!(result.is_ok());
}
