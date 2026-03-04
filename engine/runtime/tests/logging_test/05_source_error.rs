mod logging_helper;

use logging_helper::{execute_logging_error_test, verify_action_log, verify_user_facing_log};

const FIXTURE_DIR: &str = "logging/05_source_error";
const WORKFLOW_NAME: &str = "Source Runtime Error Test";

#[test]
fn test_logging_source_error() {
    let result = execute_logging_error_test(FIXTURE_DIR, WORKFLOW_NAME);

    verify_action_log(FIXTURE_DIR, &result);
    verify_user_facing_log(FIXTURE_DIR, &result);
}
