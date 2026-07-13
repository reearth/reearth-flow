mod logging_helper;

use std::fs;

use logging_helper::{execute_logging_test, verify_action_log, verify_user_facing_log};

const FIXTURE_DIR: &str = "logging/10_warn_drop_aggregation";
const WORKFLOW_NAME: &str = "Warn Drop Aggregation Test";

#[test]
fn test_logging_warn_drop_aggregation() {
    let result = execute_logging_test(FIXTURE_DIR, WORKFLOW_NAME);

    // Read the raw action log before the golden compare below, so a
    // regex/normalizer bug in the golden path can't mask a real regression
    // in the aggregation behavior itself.
    let actual_log_path = result.action_log_dir.join("all.log");
    let raw_action_log = fs::read_to_string(&actual_log_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", actual_log_path.display()));

    // exactly one aggregated summary, no per-feature warn spam
    let warn_lines: Vec<&str> = raw_action_log
        .lines()
        .filter(|l| l.contains("\"level\":\"WARNING\""))
        .collect();
    assert_eq!(
        warn_lines.len(),
        1,
        "expected exactly one WARN summary line, got: {warn_lines:?}"
    );
    assert!(warn_lines[0].contains("dropped 3 feature(s)"));
    assert!(warn_lines[0].contains("cesium3dtiles.empty_geometry"));

    verify_action_log(FIXTURE_DIR, &result);
    verify_user_facing_log(FIXTURE_DIR, &result);
}
