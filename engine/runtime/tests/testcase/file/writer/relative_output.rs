use crate::helper::{execute, execute_expect_err};

#[test]
fn relative_output_writes_under_sandbox_root() {
    // Fixture uses `output: test_output.gpkg` (relative).
    // After Runner::run_with_sandbox_root joins it against sandbox_root,
    // the file should land at <sandbox_root>/test_output.gpkg.
    let tempdir = execute("file/writer/relative_output", vec!["test_geopackage.gpkg"])
        .expect("workflow should succeed");
    let expected_path = tempdir.path().join("test_output.gpkg");
    assert!(
        expected_path.exists(),
        "expected file {:?} to exist under sandbox_root after relative-path sink write",
        expected_path
    );
}

#[test]
fn absolute_output_fails_with_migration_hint() {
    // Fixture uses OLD Url(env["workerArtifactPath"]) / "x" pattern.
    // The new chokepoint rejects absolute URIs; the error must name
    // workerArtifactPath so customers can locate the migration from logs.
    let err = execute_expect_err(
        "file/writer/absolute_output_fails",
        vec!["test_geopackage.gpkg"],
    );
    assert!(
        err.contains("workerArtifactPath"),
        "error must name workerArtifactPath; got: {err}"
    );
}
