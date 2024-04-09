use reearth_flow_action::{ActionValue, Port};

use crate::helper::init_test_runner;

#[tokio::test]
async fn test_run() {
    let executor = init_test_runner("attribute/renamer", vec!["renamer.json"]).await;
    let result = executor.start().await;
    println!("{:?}", result);
    assert!(result.is_ok());
    let result = result.unwrap();
    let default_port = result.get(&Port::new("default")).unwrap();
    assert!(default_port.is_some());
    let default_port = default_port.clone().unwrap();
    match default_port {
        ActionValue::Map(kv) => {
            let kv = match kv.get("foo_foo").unwrap() {
                ActionValue::Map(kv) => kv,
                _ => panic!("unexpected value"),
            };
            kv.get("foo_bar1").unwrap();
            kv.get("foo_bar2").unwrap();
        },
        _ => panic!("unexpected value"),
    }
}