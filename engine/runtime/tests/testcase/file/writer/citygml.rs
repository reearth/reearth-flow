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

    // Verify PLATEAU extension attributes
    assert!(
        content.contains("buildingIDAttribute") || content.contains("BuildingIDAttribute"),
        "Output should contain buildingIDAttribute"
    );
    assert!(
        content.contains("TEST-BLDG-001"),
        "Output should contain building ID value"
    );
    assert!(
        content.contains("buildingDetailAttribute") || content.contains("BuildingDetailAttribute"),
        "Output should contain buildingDetailAttribute"
    );
    assert!(
        content.contains("totalFloorArea"),
        "Output should contain totalFloorArea"
    );

    // Print output for manual verification
    println!("=== CityGML Writer Output ===");
    println!("{}", content);
    println!("=== End Output ===");
}

#[test]
fn test_citygml_writer_full_plateau() {
    let tempdir = execute("file/writer/citygml", vec!["input.gml"]).unwrap();

    let storage_resolver = Arc::new(StorageResolver::new());
    let output_path = tempdir.path().join("result.json");
    let uri = Uri::for_test(output_path.to_str().unwrap());
    let storage = storage_resolver.resolve(&uri).unwrap();

    let result = storage.get_sync(uri.path().as_path());
    assert!(result.is_ok(), "Output file should exist");

    let content = String::from_utf8(result.unwrap().to_vec()).unwrap();

    // Verify two buildings present
    let building_count = content.matches("bldg:Building").count();
    assert!(
        building_count >= 2,
        "Output should contain at least 2 buildings, found {}",
        building_count
    );

    // Verify PLATEAU attributes preserved
    assert!(
        content.contains("1621-bldg-77") || content.contains("16211-bldg-78"),
        "Output should contain building IDs from input"
    );

    // Verify codelists preserved
    assert!(
        content.contains("codeSpace") || content.contains("codelists"),
        "Output should preserve codelist references"
    );

    // Print for verification
    println!("=== Full PLATEAU Output (first 5000 chars) ===");
    println!("{}", &content[..content.len().min(5000)]);
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
