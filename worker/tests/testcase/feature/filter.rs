use reearth_flow_action::{Attribute, AttributeValue, Port};

use crate::helper::init_test_runner;

#[tokio::test]
async fn test_run() {
    let executor = init_test_runner("feature/filter", vec!["filter.json"]).await;
    let result = executor.start().await;
    assert!(result.is_ok());
    let result = result.unwrap();
    let default_port = result.get(&Port::new("default")).unwrap();
    let default_port = default_port.clone();
    let array = default_port.features.clone();
    assert_eq!(array.len(), 2);
    let first = array.first().unwrap();
    let second = array.get(1).unwrap();
    let first = first.clone();
    let city = first.attributes.get(&Attribute::new("city")).unwrap();
    assert_eq!(city, &AttributeValue::String("Tokyo".to_string()));
    let second = second.clone();
    let city = second.attributes.get(&Attribute::new("city")).unwrap();
    assert_eq!(city, &AttributeValue::String("Osaka".to_string()));
}
