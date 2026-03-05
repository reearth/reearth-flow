use crate::helper::execute;

#[test]
fn test_joiner_left() {
    let result = execute("feature/joiner", vec!["joiner.json"]);
    assert!(result.is_ok());
}

#[test]
fn test_joiner_inner() {
    let result = execute("feature/joiner_inner", vec![]);
    assert!(result.is_ok());
}

#[test]
fn test_joiner_full() {
    let result = execute("feature/joiner_full", vec![]);
    assert!(result.is_ok());
}
