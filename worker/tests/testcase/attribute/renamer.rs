use reearth_flow_action::{ActionValue, Port};

use crate::helper::init_test_runner;

#[tokio::test]
async fn test_add_prefix() {
    let executor = init_test_runner("attribute/renamer/add_prefix", vec!["renamer.json"]).await;
    let result = executor.start().await;
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

#[tokio::test]
async fn test_add_suffix() {
    let executor = init_test_runner("attribute/renamer/add_suffix", vec!["renamer.json"]).await;
    let result = executor.start().await;
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
            kv.get("bar1_foo").unwrap();
            kv.get("bar2_foo").unwrap();
        },
        _ => panic!("unexpected value"),
    }
}

#[tokio::test]
async fn test_remove_prefix() {
    let executor = init_test_runner("attribute/renamer/remove_prefix", vec!["renamer.json"]).await;
    let result = executor.start().await;
    assert!(result.is_ok());
    let result = result.unwrap();
    let default_port = result.get(&Port::new("default")).unwrap();
    assert!(default_port.is_some());
    let default_port = default_port.clone().unwrap();
    match default_port {
        ActionValue::Map(kv) => {
            let kv = match kv.get("foo").unwrap() {
                ActionValue::Map(kv) => kv,
                _ => panic!("unexpected value"),
            };
            kv.get("foobar1").unwrap();
            kv.get("bar2").unwrap();
        },
        _ => panic!("unexpected value"),
    }
}

#[tokio::test]
async fn test_remove_suffix() {
    let executor = init_test_runner("attribute/renamer/remove_suffix", vec!["renamer.json"]).await;
    let result = executor.start().await;
    assert!(result.is_ok());
    let result = result.unwrap();
    let default_port = result.get(&Port::new("default")).unwrap();
    assert!(default_port.is_some());
    let default_port = default_port.clone().unwrap();
    match default_port {
        ActionValue::Map(kv) => {
            let kv = match kv.get("foo").unwrap() {
                ActionValue::Map(kv) => kv,
                _ => panic!("unexpected value"),
            };
            kv.get("bar1foo").unwrap();
            kv.get("bar2").unwrap();
        },
        _ => panic!("unexpected value"),
    }
}