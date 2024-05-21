use crate::helper::execute;

#[test]
fn test_run() {
    execute("attribute/duplicate", vec!["duplicate.json"]);
}
