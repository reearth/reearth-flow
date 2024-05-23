use crate::helper::execute;

#[test]
fn test_run() {
    execute("feature/filter", vec!["filter.json"]);
}
