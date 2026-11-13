// Workflow-level test for the JP Standard Grid Accumulator action. Its
// cell-collecting behaviour exists only in the new-geometry world, so the whole
// module is gated on it.
//
// The GeoJSON reader in that world tags coordinates as `Crs(4326)` (stored
// lat/lng), so no reprojector node is needed in front of the accumulator.
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

/// A polygon across the corner of 3rd meshes 54377085, 54377086, 54377095 and
/// 54377096 touches all four; a footprint inside 54377085 adds no new cell; a
/// point in London is out of range and goes to the `rejected` port.
#[test]
fn test_collect_touched_cells_and_reject_out_of_range() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workflow_path =
        manifest_dir.join("fixture/workflow/geometry/jp_standard_grid_accumulator/cells.yaml");
    let geojson_path = manifest_dir
        .join("fixture/testdata/geometry/jp_standard_grid_accumulator/footprints.geojson");

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
            PathBuf::from(
                "/fixture/testdata/geometry/jp_standard_grid_accumulator/footprints.geojson",
            )
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
    let mut codes: Vec<&str> = accepted
        .iter()
        .map(|cell| cell["meshCode"].as_str().expect("meshCode is a string"))
        .collect();
    codes.sort_unstable();
    assert_eq!(
        codes,
        ["54377085", "54377086", "54377095", "54377096"],
        "one feature per touched cell, each cell once"
    );

    let rejected = read_output(&storage_resolver, temp_dir.path(), "rejected.json");
    assert_eq!(rejected.len(), 1, "only the London point is rejected");
    assert_eq!(rejected[0]["name"], "london");
}
