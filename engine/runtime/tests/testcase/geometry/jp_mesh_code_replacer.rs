// Workflow-level test for the JP Mesh Code Replacer action. The action only
// exists in the new-geometry world, so the whole module is gated on it.
#![cfg(feature = "new-geometry")]

use reearth_flow_action_log::factory::LoggerFactory;
use reearth_flow_action_log::ActionLogger;
use reearth_flow_common::uri::Uri;
use reearth_flow_runner::runner::Runner;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Workflow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tempfile::tempdir;

use crate::helper::BUILTIN_ACTION_FACTORIES;

fn read_output(
    storage_resolver: &Arc<StorageResolver>,
    dir: &std::path::Path,
    name: &str,
) -> Vec<serde_json::Value> {
    let uri = Uri::for_test(&format!("file://{}", dir.join(name).to_str().unwrap()));
    let storage = storage_resolver.resolve(&uri).unwrap();
    let bytes = storage
        .get_sync(uri.path().as_path())
        .unwrap_or_else(|_| panic!("{name} should exist after workflow execution"));
    serde_json::from_slice(&bytes).unwrap()
}

/// A feature whose `code` is a valid 3rd mesh code gets that cell; a feature
/// whose `code` is not a mesh code goes to the `rejected` port unchanged.
#[test]
fn test_replace_geometry_and_reject_invalid_code() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workflow_path =
        manifest_dir.join("fixture/workflow/geometry/jp_mesh_code_replacer/replace.yaml");
    let geojson_path =
        manifest_dir.join("fixture/testdata/geometry/jp_mesh_code_replacer/codes.geojson");

    let workflow_str = std::fs::read_to_string(&workflow_path)
        .unwrap_or_else(|_| panic!("Failed to read workflow: {workflow_path:?}"));
    let mut workflow = Workflow::try_from(workflow_str.as_str()).expect("Failed to parse workflow");

    let temp_dir = tempdir().unwrap();
    let storage_resolver = Arc::new(StorageResolver::new());
    let storage = storage_resolver
        .resolve(&Uri::for_test("ram:///fixture/"))
        .unwrap();
    storage
        .put_sync(
            PathBuf::from("/fixture/testdata/geometry/jp_mesh_code_replacer/codes.geojson")
                .as_path(),
            bytes::Bytes::from(std::fs::read(&geojson_path).unwrap()),
        )
        .unwrap();

    workflow
        .merge_with(HashMap::from([
            ("outputPath".to_string(), "accepted.json".to_string()),
            ("rejectedPath".to_string(), "rejected.json".to_string()),
        ]))
        .unwrap();

    crate::helper::init_test_env();
    let logger_factory = Arc::new(LoggerFactory::new(
        ActionLogger::root(
            reearth_flow_action_log::Discard,
            reearth_flow_action_log::o!(),
        ),
        Uri::for_test("ram:///log/").path(),
    ));
    let feature_state =
        Arc::new(State::new(&Uri::for_test("ram:///state/"), &storage_resolver).unwrap());
    let sandbox_root = Uri::from_str(&format!("file://{}/", temp_dir.path().to_str().unwrap()))
        .expect("sandbox_root URI must be valid");
    Runner::run_with_sandbox_root(
        uuid::Uuid::new_v4(),
        workflow,
        BUILTIN_ACTION_FACTORIES.clone(),
        logger_factory,
        storage_resolver.clone(),
        feature_state,
        None,
        sandbox_root,
    )
    .expect("Workflow execution should succeed");

    let accepted = read_output(&storage_resolver, temp_dir.path(), "accepted.json");
    assert_eq!(accepted.len(), 1, "only the valid code is replaced");
    assert_eq!(accepted[0]["name"], "valid");
    assert_eq!(accepted[0]["code"], "53394611");

    let rejected = read_output(&storage_resolver, temp_dir.path(), "rejected.json");
    assert_eq!(rejected.len(), 1, "only the invalid code is rejected");
    assert_eq!(rejected[0]["name"], "invalid");
    assert_eq!(rejected[0]["code"], "bad");
}
