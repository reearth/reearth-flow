use reearth_flow_action::{ActionValue, Port};

use crate::helper::init_test_runner;

#[tokio::test]
async fn test_add_prefix() {
    let executor = init_test_runner("attribute/renamer/add_prefix", vec!["renamer.json"]).await;
    let default = executor
        .start()
        .await
        .unwrap()
        .get(&Port::new("default"))
        .unwrap()
        .clone()
        .unwrap();
    match default {
        ActionValue::Array(xs) => {
            match &xs[0] {
                ActionValue::Map(kv) => match kv.get("foo_foo").unwrap() {
                    ActionValue::Map(kv) => {
                        kv.get("foo_bar1");
                        kv.get("foo_bar2");
                    }
                    _ => panic!("unexpected value"),
                },
                _ => panic!("unexpected value"),
            };
            match &xs[1] {
                ActionValue::Map(kv) => kv.get("foo_baz").unwrap(),
                _ => panic!("unexpected value"),
            };
        }
        _ => panic!("unexpected value"),
    };
}

#[tokio::test]
async fn test_add_suffix() {
    let executor = init_test_runner("attribute/renamer/add_suffix", vec!["renamer.json"]).await;
    let default = executor
        .start()
        .await
        .unwrap()
        .get(&Port::new("default"))
        .unwrap()
        .clone()
        .unwrap();
    match default {
        ActionValue::Array(xs) => {
            match &xs[0] {
                ActionValue::Map(kv) => match kv.get("foo_foo").unwrap() {
                    ActionValue::Map(kv) => {
                        kv.get("bar1_foo");
                        kv.get("bar2_foo");
                    }
                    _ => panic!("unexpected value"),
                },
                _ => panic!("unexpected value"),
            };
            match &xs[1] {
                ActionValue::Map(kv) => kv.get("baz_foo").unwrap(),
                _ => panic!("unexpected value"),
            };
        }
        _ => panic!("unexpected value"),
    };
}

#[tokio::test]
async fn test_remove_prefix() {
    let executor = init_test_runner("attribute/renamer/remove_prefix", vec!["renamer.json"]).await;
    let default = executor
        .start()
        .await
        .unwrap()
        .get(&Port::new("default"))
        .unwrap()
        .clone()
        .unwrap();
    match default {
        ActionValue::Array(xs) => {
            match &xs[0] {
                ActionValue::Map(kv) => match kv.get("foo").unwrap() {
                    ActionValue::Map(kv) => {
                        kv.get("foobar1");
                        kv.get("bar2");
                    }
                    _ => panic!("unexpected value"),
                },
                _ => panic!("unexpected value"),
            };
            match &xs[1] {
                ActionValue::Map(kv) => kv.get("baz").unwrap(),
                _ => panic!("unexpected value"),
            };
        }
        _ => panic!("unexpected value"),
    };
}

#[tokio::test]
async fn test_remove_suffix() {
    let executor = init_test_runner("attribute/renamer/remove_suffix", vec!["renamer.json"]).await;
    let default = executor
        .start()
        .await
        .unwrap()
        .get(&Port::new("default"))
        .unwrap()
        .clone()
        .unwrap();
    match default {
        ActionValue::Array(xs) => {
            match &xs[0] {
                ActionValue::Map(kv) => match kv.get("foo").unwrap() {
                    ActionValue::Map(kv) => {
                        kv.get("bar1foo");
                        kv.get("bar2");
                    }
                    _ => panic!("unexpected value"),
                },
                _ => panic!("unexpected value"),
            };
            match &xs[1] {
                ActionValue::Map(kv) => kv.get("baz").unwrap(),
                _ => panic!("unexpected value"),
            };
        }
        _ => panic!("unexpected value"),
    };
}

#[tokio::test]
async fn test_string_replace() {
    let executor = init_test_runner("attribute/renamer/string_replace", vec!["renamer.json"]).await;
    let default = executor
        .start()
        .await
        .unwrap()
        .get(&Port::new("default"))
        .unwrap()
        .clone()
        .unwrap();
    match default {
        ActionValue::Array(xs) => {
            match &xs[0] {
                ActionValue::Map(kv) => match kv.get("baz").unwrap() {
                    ActionValue::Map(kv) => {
                        kv.get("bazbar1");
                        kv.get("baz_bar2");
                    }
                    _ => panic!("unexpected value"),
                },
                _ => panic!("unexpected value"),
            };
            match &xs[1] {
                ActionValue::Map(kv) => kv.get("bazbazbaz").unwrap(),
                _ => panic!("unexpected value"),
            };
        }
        _ => panic!("unexpected value"),
    };
}

#[tokio::test]
async fn test_regular_expression_replace() {
    let executor = init_test_runner(
        "attribute/renamer/regular_expression_replace",
        vec!["renamer.json"],
    )
    .await;
    let default = executor
        .start()
        .await
        .unwrap()
        .get(&Port::new("default"))
        .unwrap()
        .clone()
        .unwrap();
    match default {
        ActionValue::Array(xs) => {
            match &xs[0] {
                ActionValue::Map(kv) => match kv.get("foo").unwrap() {
                    ActionValue::Map(kv) => {
                        kv.get("bar1");
                        kv.get("bar2");
                    }
                    _ => panic!("unexpected value"),
                },
                _ => panic!("unexpected value"),
            };
            match &xs[1] {
                ActionValue::Map(kv) => kv.get("xx_baz").unwrap(),
                _ => panic!("unexpected value"),
            };
        }
        _ => panic!("unexpected value"),
    };
}
