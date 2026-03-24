// Workflow-level integration tests for NeighborFinder processor
// ==============================================================
//
// These tests verify the end-to-end behavior of the NeighborFinder processor
// using real geometry data loaded from GeoJSON files.

use reearth_flow_action_log::factory::LoggerFactory;
use reearth_flow_action_log::ActionLogger;
use reearth_flow_common::uri::Uri;
use reearth_flow_runner::runner::Runner;
use reearth_flow_state::State;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Workflow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;

use crate::helper::BUILTIN_ACTION_FACTORIES;

/// Test: NeighborFinder with GeoJSON input - explicit proximity matching verification
///
/// This test verifies that the NeighborFinder processor correctly:
/// 1. Reads geometry from GeoJSON files
/// 2. Calculates 2D Euclidean distances between points
/// 3. Identifies the nearest neighbor (closest candidate to base)
/// 4. Transfers attributes from matched candidate to base feature
/// 5. Adds distance attribute (_distance)
///
/// Test Data Setup:
/// - Points are arranged at known coordinates for predictable distances
/// - candidate_close at (0, 0) - distance 1.0 from base
/// - candidate_mid at (4, 3)   - distance 5.0 from base (3-4-5 triangle)
/// - candidate_far at (10, 0)  - distance 10.0 from base
/// - base_feature at (1, 0)    - the search point
///
/// Expected Result:
/// - base_feature enriched with candidate_close attributes
/// - _distance = 1.0 (exact Euclidean distance)
#[test]
fn test_proximity_search_with_geojson() {
    // Get the project root directory
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Paths to test fixtures (relative to project root)
    let workflow_path = manifest_dir
        .join("fixture/workflow/geometry/neighbor_finder/proximity_search_with_geojson.yaml");
    let geojson_path = manifest_dir
        .join("fixture/testdata/geometry/neighbor_finder/points_for_proximity_test.geojson");

    // Verify fixtures exist
    assert!(
        workflow_path.exists(),
        "Workflow file not found: {:?}",
        workflow_path
    );
    assert!(
        geojson_path.exists(),
        "GeoJSON file not found: {:?}",
        geojson_path
    );

    // Load workflow
    let workflow_str = std::fs::read_to_string(&workflow_path)
        .expect(&format!("Failed to read workflow: {:?}", workflow_path));
    let mut workflow = Workflow::try_from(workflow_str.as_str()).expect("Failed to parse workflow");

    // Set up temp directory and output path
    let temp_dir = tempdir().unwrap();

    // Also copy GeoJSON to ram storage path that the workflow expects
    let storage_resolver = Arc::new(StorageResolver::new());
    let storage = storage_resolver
        .resolve(&Uri::for_test("ram:///fixture/"))
        .unwrap();
    let geojson_content = std::fs::read(&geojson_path).unwrap();
    storage
        .put_sync(
            PathBuf::from(
                "/fixture/testdata/geometry/neighbor_finder/points_for_proximity_test.geojson",
            )
            .as_path(),
            bytes::Bytes::from(geojson_content),
        )
        .unwrap();

    // Merge workflow variables
    let output_path = temp_dir.path().join("result.json");
    let output_uri_str = format!("file://{}", output_path.to_str().unwrap());
    workflow
        .merge_with(HashMap::from([(
            "outputPath".to_string(),
            output_uri_str.clone(),
        )]))
        .unwrap();

    // Set up dependencies
    std::env::set_var("FLOW_RUNTIME_ACTION_LOG_DISABLE", "true");
    let logger_factory = Arc::new(LoggerFactory::new(
        ActionLogger::root(
            reearth_flow_action_log::Discard,
            reearth_flow_action_log::o!(),
        ),
        Uri::for_test("ram:///log/").path(),
    ));
    let ingress_state =
        Arc::new(State::new(&Uri::for_test("ram:///ingress/"), &storage_resolver).unwrap());
    let feature_state =
        Arc::new(State::new(&Uri::for_test("ram:///state/"), &storage_resolver).unwrap());

    // Run the workflow
    match Runner::run(
        uuid::Uuid::new_v4(),
        workflow,
        BUILTIN_ACTION_FACTORIES.clone(),
        logger_factory,
        storage_resolver.clone(),
        ingress_state,
        feature_state,
        None,
    ) {
        Ok(_) => eprintln!("Workflow completed successfully"),
        Err(e) => {
            eprintln!("Workflow failed: {:?}", e);
            panic!("Workflow execution failed: {:?}", e);
        }
    }

    // Debug: List temp directory contents
    eprintln!("Temp directory: {:?}", temp_dir.path());
    for entry in std::fs::read_dir(temp_dir.path()).unwrap() {
        eprintln!("  File: {:?}", entry.unwrap().path());
    }

    // Read the result
    let output_uri = Uri::for_test(&output_uri_str);
    let storage = storage_resolver.resolve(&output_uri).unwrap();
    let result = storage
        .get_sync(output_uri.path().as_path())
        .expect("Result file should exist after workflow execution");
    let result_str = String::from_utf8(result.to_vec()).unwrap();
    eprintln!(
        "Result content: {}",
        &result_str[..result_str.len().min(1000)]
    );

    // Parse the result
    let results: Vec<serde_json::Value> = serde_json::from_str(&result_str).unwrap();

    // Verify we got exactly 1 matched base feature (closest strategy)
    assert_eq!(
        results.len(),
        1,
        "Should have exactly one matched base feature"
    );

    let feature = &results[0];

    // Verify base feature identity is preserved
    assert_eq!(
        feature["id"], "base_feature",
        "Base feature ID should be preserved"
    );
    assert_eq!(
        feature["name"], "Base Feature",
        "Base feature name should be preserved"
    );

    // Verify closest neighbor was correctly identified
    assert_eq!(
        feature["_nearest_id"], "candidate_close",
        "Should match closest candidate (id)"
    );
    assert_eq!(
        feature["_nearest_name"], "Close Candidate",
        "Should match closest candidate (name)"
    );

    // Verify distance calculation is correct (1.0 = distance from (1,0) to (0,0))
    assert_eq!(feature["_distance"], 1.0, "Distance should be 1.0");

    eprintln!("NeighborFinder integration test passed!");
}
