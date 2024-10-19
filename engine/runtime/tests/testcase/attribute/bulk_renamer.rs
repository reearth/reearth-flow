use crate::helper::execute_with_test_assert;

#[test]
fn test_all_add_prefix() {
    execute_with_test_assert("attribute/bulk_renamer/all_add_prefix", "expect.json");
}

#[test]
fn test_all_add_suffix() {
    execute_with_test_assert("attribute/bulk_renamer/all_add_suffix", "expect.json");
}

#[test]
fn test_all_remove_prefix_error() {
    execute_with_test_assert(
        "attribute/bulk_renamer/all_remove_prefix_error",
        "expect.json",
    );
}
