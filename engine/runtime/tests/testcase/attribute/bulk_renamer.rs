use crate::helper::{execute_expect_err, execute_with_test_assert};

#[test]
fn test_all_add_prefix() {
    execute_with_test_assert("attribute/bulk_renamer/all_add_prefix", "expect.json");
}

#[test]
fn test_all_add_suffix() {
    execute_with_test_assert("attribute/bulk_renamer/all_add_suffix", "expect.json");
}

// C12 / Task 6 convergence: a per-feature `process()` error (here, an
// attribute that doesn't carry the configured prefix/suffix/pattern) now
// records a synthesized fatal into the node's fatal slot, so it reaches
// `RunSummary.failed_nodes`. `helper::execute` runs through
// `Runner::run_with_sandbox_root`, whose unit-returning wrapper
// (`summary_into_unit_result`, pre-existing since Phase 2a Task 6) already
// turns ANY non-empty `failed_nodes` into `Err` regardless of `errorPolicy`
// — that all-or-nothing mapping was already in place; before Task 6 it
// simply never fired here because the divergence this task fixes kept
// `failed_nodes` empty for a per-feature error like this one. So this is
// not a behavior regression introduced by Task 6, it's the fix removing a
// blind spot: `NodeFailureHandler` (production) already reported these
// runs as failed via the event stream; now the engine's own `Result` type
// agrees, everywhere, including here.
#[test]
fn test_all_remove_prefix_error() {
    let rendered = execute_expect_err("attribute/bulk_renamer/all_remove_prefix_error", vec![]);
    assert!(
        rendered.contains("does not start with prefix"),
        "unexpected error text: {rendered}"
    );
}

#[test]
fn test_all_remove_prefix() {
    execute_with_test_assert("attribute/bulk_renamer/all_remove_prefix", "expect.json");
}

#[test]
fn test_all_remove_suffix_error() {
    let rendered = execute_expect_err("attribute/bulk_renamer/all_remove_suffix_error", vec![]);
    assert!(
        rendered.contains("does not end with suffix"),
        "unexpected error text: {rendered}"
    );
}

#[test]
fn test_all_remove_suffix() {
    execute_with_test_assert("attribute/bulk_renamer/all_remove_suffix", "expect.json");
}

#[test]
fn test_all_string_replace_error() {
    let rendered = execute_expect_err("attribute/bulk_renamer/all_string_replace_error", vec![]);
    assert!(
        rendered.contains("does not match the regex pattern"),
        "unexpected error text: {rendered}"
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

// See test_all_remove_prefix_error's comment: same C12 / Task 6 convergence,
// selected-attributes variant.
#[test]
fn test_selected_remove_prefix_error() {
    let rendered = execute_expect_err(
        "attribute/bulk_renamer/selected_remove_prefix_error",
        vec![],
    );
    assert!(
        rendered.contains("does not start with prefix"),
        "unexpected error text: {rendered}"
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
    let rendered = execute_expect_err(
        "attribute/bulk_renamer/selected_remove_suffix_error",
        vec![],
    );
    assert!(
        rendered.contains("does not end with suffix"),
        "unexpected error text: {rendered}"
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
    let rendered = execute_expect_err(
        "attribute/bulk_renamer/selected_string_replace_error",
        vec![],
    );
    assert!(
        rendered.contains("does not match the regex pattern"),
        "unexpected error text: {rendered}"
    );
}

#[test]
fn test_selected_string_replace() {
    execute_with_test_assert(
        "attribute/bulk_renamer/selected_string_replace",
        "expect.json",
    );
}
