//! Integration tests for [`reearth_flow_runtime::schema_sample`].
//!
//! These live in `tests/` rather than as an inline `#[cfg(test)]` module
//! because they depend on `reearth-flow-action-source` for a real source
//! factory, and `action-source` depends back on `reearth-flow-runtime`. An
//! inline unit test would compile the runtime crate twice (once under
//! `cfg(test)`, once as the rlib `action-source` links), producing two
//! incompatible `NodeKind` types ("multiple different versions of crate
//! reearth_flow_runtime"). An integration test links the plain rlib, so the
//! types unify.

use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;

use reearth_flow_action_source::mapping::ACTION_FACTORY_MAPPINGS;
use reearth_flow_runtime::node::NodeKind;
use reearth_flow_runtime::schema_sample::sample_source;
use reearth_flow_types::attr_schema::AttrType;
use reearth_flow_types::attribute::Attribute;
use serde_json::json;

fn empty_env() -> Arc<serde_json::Map<String, serde_json::Value>> {
    Arc::new(serde_json::Map::new())
}

const FIXTURE_GEOJSON: &str = r#"{
  "type": "FeatureCollection",
  "features": [
    {
      "type": "Feature",
      "properties": {
        "id": "candidate_close",
        "name": "Close Candidate",
        "type": "candidate",
        "expected_distance_from_base": 1.0
      },
      "geometry": { "type": "Point", "coordinates": [0, 0] }
    },
    {
      "type": "Feature",
      "properties": {
        "id": "candidate_mid",
        "name": "Mid Distance Candidate",
        "type": "candidate",
        "expected_distance_from_base": 5.0
      },
      "geometry": { "type": "Point", "coordinates": [4, 3] }
    }
  ]
}"#;

fn geojson_factory() -> &'static NodeKind {
    ACTION_FACTORY_MAPPINGS
        .get("GeoJsonReader")
        .expect("GeoJsonReader factory must be registered")
}

/// Build `with` whose `dataset` is a `string`-typed [`Code`] literal that
/// evaluates to the given URI verbatim (no expression evaluation).
fn with_dataset(uri: &str) -> HashMap<String, serde_json::Value> {
    let mut with = HashMap::new();
    with.insert(
        "dataset".to_string(),
        json!({ "type": "string", "value": uri }),
    );
    with
}

#[test]
fn samples_geojson_real_attributes() {
    let mut tmp = tempfile::Builder::new()
        .suffix(".geojson")
        .tempfile()
        .expect("create temp geojson");
    tmp.write_all(FIXTURE_GEOJSON.as_bytes())
        .expect("write fixture");
    let path = tmp.path().to_str().expect("utf8 temp path").to_string();
    let uri = format!("file://{path}");

    let outcome = sample_source(
        geojson_factory(),
        &Some(with_dataset(&uri)),
        10,
        empty_env(),
    );

    assert!(
        outcome.note.is_none(),
        "expected no note, got: {:?}",
        outcome.note
    );
    assert!(!outcome.schema.open, "schema should be closed");

    let id = outcome
        .schema
        .fields
        .get(&Attribute::new("id".to_string()))
        .expect("id field present");
    assert_eq!(id.ty, AttrType::String, "id should be String");

    assert!(
        outcome
            .schema
            .fields
            .contains_key(&Attribute::new("expected_distance_from_base".to_string())),
        "expected_distance_from_base field present"
    );
}

#[test]
fn unresolved_source_falls_back_to_open_with_note() {
    let with = with_dataset("file:///nonexistent/path/does_not_exist_xyz.geojson");
    let outcome = sample_source(geojson_factory(), &Some(with), 10, empty_engine());

    assert!(outcome.schema.open, "schema should be open on failure");
    assert!(outcome.note.is_some(), "a note should explain the failure");
}

/// A `dataset` of `env.get("...")` must resolve from the engine's variables
/// while sampling — i.e. datasets provided via workflow vars (as `run` does)
/// are honored, not just hardcoded literals. This is the regression guard for
/// `probe-schema` threading `workflow.with` into the sampling engine.
#[test]
fn samples_geojson_dataset_resolved_from_engine_var() {
    let mut tmp = tempfile::Builder::new()
        .suffix(".geojson")
        .tempfile()
        .expect("create temp geojson");
    tmp.write_all(FIXTURE_GEOJSON.as_bytes())
        .expect("write fixture");
    let path = tmp.path().to_str().expect("utf8 temp path").to_string();
    let uri = format!("file://{path}");

    // dataset := env.get("datasetPath"); the value lives only in the engine vars.
    let mut with = HashMap::new();
    with.insert(
        "dataset".to_string(),
        json!({ "type": "flowExpr", "value": "env.get(\"datasetPath\")" }),
    );

    let mut vars = serde_json::Map::new();
    vars.insert("datasetPath".to_string(), json!(uri));
    let env_vars = Arc::new(vars);

    let outcome = sample_source(geojson_factory(), &Some(with), 10, env_vars);

    assert!(
        outcome.note.is_none(),
        "env.get() dataset should resolve from engine vars, got note: {:?}",
        outcome.note
    );
    assert!(!outcome.schema.open, "schema should be closed");
    assert!(
        outcome
            .schema
            .fields
            .contains_key(&Attribute::new("id".to_string())),
        "sampled schema should include the real `id` attribute"
    );
}
