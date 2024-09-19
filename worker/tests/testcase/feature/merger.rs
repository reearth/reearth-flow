use crate::helper::execute;

#[test]
fn test_run() {
    execute("feature/merger", vec!["merger.json"]);
}
