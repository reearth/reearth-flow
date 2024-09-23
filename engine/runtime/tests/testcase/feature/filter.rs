use crate::helper::execute;

#[test]
fn test_run() {
    let result = execute("feature/filter", vec!["filter.json"]);
    assert!(result.is_ok());
}
