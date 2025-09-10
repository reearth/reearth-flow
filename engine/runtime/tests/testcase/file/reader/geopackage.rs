use crate::helper::execute;

#[test]
fn test_run() {
    let result = execute(
        "file/reader/geopackage",
        vec!["test_geopackage.gpkg"],
    );
    assert!(result.is_ok());
}
