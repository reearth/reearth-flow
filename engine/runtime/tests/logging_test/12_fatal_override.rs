mod logging_helper;

use std::fs;

use logging_helper::{execute_logging_error_test, verify_action_log, verify_user_facing_log};

const FIXTURE_DIR: &str = "logging/12_fatal_override";
const WORKFLOW_NAME: &str = "Fatal Override Test";

const WRITER_NODE_ID: &str = "cbd5f624-b7cd-4a11-b6dd-181063c314d4";

#[test]
fn test_logging_fatal_override() {
    let result = execute_logging_error_test(FIXTURE_DIR, WORKFLOW_NAME);

    let actual_log_path = result.action_log_dir.join("all.log");
    let raw_action_log = fs::read_to_string(&actual_log_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", actual_log_path.display()));

    assert!(
        !raw_action_log.contains("\"level\":\"CRITICAL\""),
        "expected no CRITICAL line (the fatal-slot Diagnostic is never re-emitted as an \
         Event::Diagnostic -- emit_summaries only drains the non-fatal buckets), got: {raw_action_log}"
    );

    let error_lines: Vec<&str> = raw_action_log
        .lines()
        .filter(|l| l.contains("\"level\":\"ERROR\""))
        .collect();
    assert_eq!(
        error_lines.len(),
        3,
        "expected one ERROR sink-error line per dropped feature (3), got: {error_lines:?}"
    );
    assert!(
        error_lines
            .iter()
            .all(|l| l.contains("cesium3dtiles.empty_geometry") && l.contains(WRITER_NODE_ID)),
        "expected every ERROR line to name the fatal-overridden code and the writer node id, \
         got: {error_lines:?}"
    );

    let warn_lines: Vec<&str> = raw_action_log
        .lines()
        .filter(|l| l.contains("\"level\":\"WARNING\""))
        .collect();
    assert_eq!(
        warn_lines.len(),
        1,
        "expected exactly one WARN backstop line (the superseded swallowed-fatal notice), \
         got: {warn_lines:?}"
    );
    assert!(warn_lines[0].contains("swallowed fatal diagnostic"));

    verify_action_log(FIXTURE_DIR, &result);
    verify_user_facing_log(FIXTURE_DIR, &result);
}
