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

#[test]
fn test_all_remove_prefix() {
    execute_with_test_assert("attribute/bulk_renamer/all_remove_prefix", "expect.json");
}

#[test]
fn test_all_remove_suffix_error() {
    execute_with_test_assert(
        "attribute/bulk_renamer/all_remove_suffix_error",
        "expect.json",
    );
}

#[test]
fn test_all_remove_suffix() {
    execute_with_test_assert("attribute/bulk_renamer/all_remove_suffix", "expect.json");
}

#[test]
fn test_all_string_replace_error() {
    execute_with_test_assert(
        "attribute/bulk_renamer/all_string_replace_error",
        "expect.json",
    );
}

#[test]
fn test_all_string_replace() {
    execute_with_test_assert("attribute/bulk_renamer/all_string_replace", "expect.json");
}

#[test]
fn test_delete_attribute() {
    execute_with_test_assert("attribute/bulk_renamer/delete_attribute", "expect.json");
}

#[test]
fn test_selected_add_prefix() {
    execute_with_test_assert("attribute/bulk_renamer/selected_add_prefix", "expect.json");
}

#[test]
fn test_selected_add_suffix() {
    execute_with_test_assert("attribute/bulk_renamer/selected_add_suffix", "expect.json");
}

#[test]
fn test_selected_remove_prefix_error() {
    execute_with_test_assert(
        "attribute/bulk_renamer/selected_remove_prefix_error",
        "expect.json",
    );
}

#[test]
fn test_selected_remove_prefix() {
    execute_with_test_assert(
        "attribute/bulk_renamer/selected_remove_prefix",
        "expect.json",
    );
}

#[test]
fn test_selected_remove_suffix_error() {
    execute_with_test_assert(
        "attribute/bulk_renamer/selected_remove_suffix_error",
        "expect.json",
    );
}

#[test]
fn test_selected_remove_suffix() {
    execute_with_test_assert(
        "attribute/bulk_renamer/selected_remove_suffix",
        "expect.json",
    );
}

#[test]
fn test_selected_string_replace_error() {
    execute_with_test_assert(
        "attribute/bulk_renamer/selected_string_replace_error",
        "expect.json",
    );
}

#[test]
fn test_selected_string_replace() {
    execute_with_test_assert(
        "attribute/bulk_renamer/selected_string_replace",
        "expect.json",
    );
}
