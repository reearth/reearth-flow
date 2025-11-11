use crate::helper::execute;

#[test]
fn test_run() {
    let result = execute(
        "file/reader/shapefile",
        vec!["ne_110m_admin_0_countries.zip"],
    );
    assert!(result.is_ok());
}

#[test]
fn test_run_windows1252_cpg() {
    let result = execute(
        "file/reader/shapefile_windows1252",
        vec!["ne_110m_admin_0_countries_windows1252.zip"],
    );
    assert!(result.is_ok());
}
