use crate::helper::execute;

#[test]
fn test_run() {
    let result = execute(
        "file/reader/citygml",
        vec![
            "codelists/Common_districtsAndZonesType.xml",
            "codelists/Common_localPublicAuthorities.xml",
            "codelists/Common_validType.xml",
            "udx/urf/533834_urf_6668_sigaidev_op.gml",
        ],
    );
    assert!(result.is_ok());
}
