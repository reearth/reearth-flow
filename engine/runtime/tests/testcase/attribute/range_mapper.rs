use crate::helper::execute;

#[test]
fn test_attribute_range_mapper() {
    let result = execute("attribute/range_mapper", vec![]);
    assert!(
        result.is_ok(),
        "Range mapper test failed: {:?}",
        result.err()
    );

    // Verify the output file was created
    let tempdir = result.unwrap();
    let output = tempdir.path().join("result.json");

    // Debug: List files in temp directory
    if let Ok(entries) = std::fs::read_dir(tempdir.path()) {
        println!("Files in temp directory:");
        for entry in entries.flatten() {
            println!("  - {:?}", entry.path());
        }
    }

    assert!(
        output.exists(),
        "Output file was not created at {:?}",
        output
    );

    // Read and verify content
    let content = std::fs::read_to_string(&output).expect("Failed to read output");

    // Verify that features have the color attribute assigned
    assert!(
        content.contains("color"),
        "Output should contain color field"
    );

    // Verify specific color assignments for different depth ranges
    assert!(
        content.contains("#f7f5a9") || content.contains("#ffd8c0"),
        "Output should contain colors from lower ranges"
    );
    assert!(
        content.contains("#ffb7b7") || content.contains("#ff9191"),
        "Output should contain colors from middle ranges"
    );
    assert!(
        content.contains("#f285c9") || content.contains("#dc7adc"),
        "Output should contain colors from upper ranges"
    );
    assert!(
        content.contains("#cccccc"),
        "Output should contain default color for values outside all ranges"
    );
}
