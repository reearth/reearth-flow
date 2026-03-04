mod logging_helper;

use serde::Deserialize;

use logging_helper::{
    execute_logging_test, verify_action_log, verify_result_json, verify_user_facing_log,
};

const FIXTURE_DIR: &str = "logging/03_duplicate_node_names";
const WORKFLOW_NAME: &str = "Duplicate Node Names Test";

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
struct ResultEntry {
    dup_node: String,
    dup_value: i64,
}

#[test]
fn test_logging_duplicate_node_names() {
    let result = execute_logging_test(FIXTURE_DIR, WORKFLOW_NAME);

    verify_action_log(FIXTURE_DIR, &result);
    verify_user_facing_log(FIXTURE_DIR, &result);
    verify_result_json::<ResultEntry>(FIXTURE_DIR, &result);
}
