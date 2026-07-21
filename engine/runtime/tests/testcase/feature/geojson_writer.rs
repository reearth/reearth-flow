use serde_json::Value;

use crate::helper::execute;

#[test]
fn test_run() {
    let tempdir = execute("feature/geojson_writer", vec!["input.json"]).unwrap();

    // The writer produced a GeoJSON FeatureCollection with one feature per input.
    let geojson = std::fs::read_to_string(tempdir.path().join("cities.geojson")).unwrap();
    let geojson: Value = serde_json::from_str(&geojson).unwrap();

    assert_eq!(geojson["type"], "FeatureCollection");
    let features = geojson["features"].as_array().unwrap();
    assert_eq!(features.len(), 3);

    // Order is not guaranteed (no group_by), so compare the set of city values.
    let mut cities = features
        .iter()
        .map(|f| f["properties"]["city"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    cities.sort();
    assert_eq!(cities, vec!["Nagoya", "Osaka", "Tokyo"]);

    // The re-emitted filePath feature flowed downstream and was written.
    let metadata = std::fs::read_to_string(tempdir.path().join("metadata.json")).unwrap();
    assert!(metadata.contains("filePath"), "metadata: {metadata}");
    assert!(metadata.contains("cities.geojson"), "metadata: {metadata}");
}
