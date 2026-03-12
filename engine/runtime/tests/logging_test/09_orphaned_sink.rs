mod logging_helper;

use logging_helper::{execute_logging_test, verify_user_facing_log};

const FIXTURE_DIR: &str = "logging/09_orphaned_sink";
const WORKFLOW_NAME: &str = "Orphaned Sink Test";

/// Regression test: a sink node with no incoming edges should be silently skipped,
/// not cause the entire workflow to fail.
#[test]
fn test_orphaned_sink_is_skipped() {
    let result = execute_logging_test(FIXTURE_DIR, WORKFLOW_NAME);
    verify_user_facing_log(FIXTURE_DIR, &result);
}
