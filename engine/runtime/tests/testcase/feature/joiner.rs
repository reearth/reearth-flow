use crate::helper::execute;

#[test]
fn test_joiner_left() {
    // Left join: 4 matched features → joined
    //            1 unmatched requestor → unjoinedRequestor
    //            0 unmatched supplier (left join discards these)
    let result = execute("feature/joiner", vec![]);
    assert!(
        result.is_ok(),
        "Left join workflow should complete successfully"
    );
}

#[test]
fn test_joiner_inner() {
    // Inner join: 2 matched features → joined
    //             unmatched features dropped
    let result = execute("feature/joiner_inner", vec![]);
    assert!(
        result.is_ok(),
        "Inner join workflow should complete successfully"
    );
}

#[test]
fn test_joiner_full() {
    // Full join: 2 matched features → joined
    //            1 unmatched requestor → unjoinedRequestor
    //            1 unmatched supplier → unjoinedSupplier
    let result = execute("feature/joiner_full", vec![]);
    assert!(
        result.is_ok(),
        "Full join workflow should complete successfully"
    );
}
