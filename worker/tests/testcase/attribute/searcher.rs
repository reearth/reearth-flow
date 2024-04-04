use reearth_flow_action::{ActionValue, Port};

use crate::helper::init_test_runner;

#[tokio::test]
async fn test_run() {
    let executor = init_test_runner("attribute/searcher", vec!["searcher.json"]).await;
    let result = executor.start().await;
    assert!(result.is_ok());
    let result = result.unwrap();
    let default_port = result.get(&Port::new("default")).unwrap();
    assert!(default_port.is_some());
    let default_port = default_port.clone().unwrap();
    match default_port {
        ActionValue::Map(kv) => {
            let kv = match kv.get("sample_data").unwrap() {
                ActionValue::Map(kv) => kv,
                _ => panic!("unexpected value"),
            };
            match kv.get("string").unwrap() {
                ActionValue::Array(xs) => assert_eq!(xs.len(), 1),
                _ => panic!("unexpected value")
            };
            match kv.get("map").unwrap() {
                ActionValue::Map(kv) => {
                    match kv.get("item2").unwrap() {
                        ActionValue::Array(xs) => assert_eq!(xs.len(), 1),
                        _ => panic!("unexpected value"),
                    };
                    match kv.get("item3").unwrap() {
                        ActionValue::Array(xs) => assert_eq!(xs.len(), 3),
                        _ => panic!("unexpexted value"),
                    }
                },
                _ => panic!("unexpected")
            }
        }
        _ => panic!("unexpected value"),
    }
}