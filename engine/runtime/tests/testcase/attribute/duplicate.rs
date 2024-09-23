use crate::helper::execute;

#[test]
fn test_run() {
    let result = execute("attribute/duplicate", vec!["duplicate.json"]);
    assert!(result.is_ok());
}
