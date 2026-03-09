mod logging_helper;

use logging_helper::{execute_logging_error_test, verify_no_action_log, verify_user_facing_log};

const FIXTURE_DIR: &str = "logging/08_workflow_definition_error";
const WORKFLOW_NAME: &str = "Workflow Definition Error Test";

#[test]
fn test_logging_workflow_definition_error() {
    let result = execute_logging_error_test(FIXTURE_DIR, WORKFLOW_NAME);

    verify_no_action_log(&result);
    verify_user_facing_log(FIXTURE_DIR, &result);
}
