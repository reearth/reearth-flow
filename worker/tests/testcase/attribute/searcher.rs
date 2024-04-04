use reearth_flow_action::{ActionValue, Port};

use crate::helper::init_test_runner;

#[tokio::test]
async fn test_run() {
    let executor = init_test_runner("attribute/searcher", vec!["searcher.json"]).await;
    let result = executor.start().await;
    println!("{:?}", result);
    assert!(result.is_ok());
    let result = result.unwrap();
    let default_port = result.get(&Port::new("default")).unwrap();
    assert!(default_port.is_some());
    let default_port = default_port.clone().unwrap();
    match default_port {
        ActionValue::Array(array) => {
            assert_eq!(array.len(), 2);
            let first = array.first().unwrap();
            let second = array.get(1).unwrap();
            match first {
                ActionValue::Map(first) => {
                    let first = first.clone();
                    let city = first.get("city").unwrap();
                    assert_eq!(city, &ActionValue::String("Tokyo".to_string()));
                }
                _ => panic!("unexpected value"),
            }
            match second {
                ActionValue::Map(second) => {
                    let second = second.clone();
                    let city = second.get("city").unwrap();
                    assert_eq!(city, &ActionValue::String("Osaka".to_string()));
                }
                _ => panic!("unexpected value"),
            }
        }
        _ => panic!("unexpected value"),
    }
}