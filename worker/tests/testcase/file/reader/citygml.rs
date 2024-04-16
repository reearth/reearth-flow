use reearth_flow_action::Port;

use crate::helper::init_test_runner;

#[tokio::test]
async fn test_run() {
    let executor = init_test_runner(
        "file/reader/citygml",
        vec![
            "codelists/Common_districtsAndZonesType.xml",
            "codelists/Common_localPublicAuthorities.xml",
            "codelists/Common_validType.xml",
            "udx/urf/533834_urf_6668_sigaidev_op.gml",
        ],
    )
    .await;
    let result = executor.start().await;
    assert!(result.is_ok());
    let result = result.unwrap();
    let default_port = result.get(&Port::new("default")).unwrap();
    assert!(default_port.is_some());
}
