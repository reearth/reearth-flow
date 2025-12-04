// HTTPCaller integration tests
// Note: Network-dependent tests are commented out to avoid flaky tests in CI.
// To run network tests, enable them and run with `cargo test -- --ignored`

// use crate::helper::execute;

#[test]
fn test_workflow_parsing() {
    // Test that the workflow YAML can be parsed correctly
    let workflow_content = include_str!("../../fixture/workflow/http/caller.yaml");
    let workflow: Result<reearth_flow_types::Workflow, _> =
        reearth_flow_types::Workflow::try_from(workflow_content);
    assert!(
        workflow.is_ok(),
        "Failed to parse HTTPCaller workflow: {:?}",
        workflow.err()
    );
}

// Network-dependent test - run with: cargo test -- --ignored
// #[test]
// #[ignore]
// fn test_http_caller_integration() {
//     let result = execute("http/caller", vec![]);
//     assert!(result.is_ok());
// }
