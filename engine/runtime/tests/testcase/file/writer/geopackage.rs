use crate::helper::execute;

#[test]
fn test_geopackage_writer() {
    let result = execute("file/writer/geopackage", vec![]);
    assert!(result.is_ok());
}

