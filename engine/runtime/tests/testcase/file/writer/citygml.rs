use std::sync::Arc;

use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;

use crate::helper::execute;

#[test]
fn test_citygml_writer_simple() {
    let tempdir = execute("file/writer/citygml_simple", vec!["simple.gml"]).unwrap();

    let storage_resolver = Arc::new(StorageResolver::new());
    let output_path = tempdir.path().join("result.json");
    let uri = Uri::for_test(output_path.to_str().unwrap());
    let storage = storage_resolver.resolve(&uri).unwrap();

    let result = storage.get_sync(uri.path().as_path());
    assert!(result.is_ok(), "Output file should exist");

    let content = String::from_utf8(result.unwrap().to_vec()).unwrap();

    // Basic structure checks
    assert!(
        content.contains("CityModel"),
        "Output should contain CityModel root element"
    );
    assert!(
        content.contains("bldg:Building"),
        "Output should contain Building element"
    );

    // Verify geometry preservation
    assert!(
        content.contains("gml:Solid") || content.contains("lod1Solid"),
        "Output should contain LOD1 Solid geometry"
    );
    assert!(
        content.contains("gml:MultiSurface") || content.contains("lod0RoofEdge"),
        "Output should contain LOD0 RoofEdge geometry"
    );

    // Print output for manual verification
    println!("=== CityGML Writer Output ===");
    println!("{}", content);
    println!("=== End Output ===");
}

#[test]
fn test_citygml_writer_geometry_coordinates() {
    let tempdir = execute("file/writer/citygml_simple", vec!["simple.gml"]).unwrap();

    let storage_resolver = Arc::new(StorageResolver::new());
    let output_path = tempdir.path().join("result.json");
    let uri = Uri::for_test(output_path.to_str().unwrap());
    let storage = storage_resolver.resolve(&uri).unwrap();

    let result = storage.get_sync(uri.path().as_path());
    assert!(result.is_ok(), "Output file should exist");

    let content = String::from_utf8(result.unwrap().to_vec()).unwrap();

    // Check coordinate values are present (in lat/lon order: 35.0 135.0)
    assert!(
        content.contains("35") && content.contains("135"),
        "Output should contain coordinate values around lat 35, lon 135"
    );

    // Check posList element
    assert!(
        content.contains("posList"),
        "Output should contain posList elements for coordinates"
    );
}
