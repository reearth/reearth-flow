use crate::helper::execute;

#[test]
fn test_run() {
    let result = execute(
        "file/reader/shapefile",
        vec!["ne_110m_admin_0_countries.zip"],
    );
    assert!(result.is_ok());
}
