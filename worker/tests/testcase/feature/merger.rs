use crate::helper::execute;

#[test]
fn test_run() {
    let result = execute("feature/merger", vec!["merger.json"]);
    assert!(result.is_ok());
}
