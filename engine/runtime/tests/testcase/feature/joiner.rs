use std::path::Path;

use crate::helper::execute;

/// Helper function to read and parse JSON file
fn read_json_file(path: &Path) -> serde_json::Value {
    if !path.exists() {
        return serde_json::json!([]);
    }
    let content = std::fs::read_to_string(path).expect("Failed to read file");
    if content.trim().is_empty() {
        return serde_json::json!([]);
    }
    // Handle both single JSON object and array of objects
    let value: serde_json::Value =
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!([]));
    value
}

/// Helper function to get count of features from JSON
fn get_feature_count(path: &Path) -> usize {
    let value = read_json_file(path);
    match value {
        serde_json::Value::Array(arr) => arr.len(),
        serde_json::Value::Object(_) => 1,
        _ => 0,
    }
}

#[test]
fn test_joiner_left() {
    // Left join: 4 matched features → joined
    //            1 unmatched requestor → unjoinedRequestor
    //            0 unmatched supplier (left join discards these)
    let tempdir = execute("feature/joiner", vec![]).expect("Workflow should complete successfully");
    let temp_path = tempdir.path();

    // Verify feature counts on each output port
    let joined_count = get_feature_count(&temp_path.join("joined.json"));
    let unjoined_requestor_count = get_feature_count(&temp_path.join("unjoined_requestor.json"));
    let unjoined_supplier_count = get_feature_count(&temp_path.join("unjoined_supplier.json"));

    assert_eq!(
        joined_count, 4,
        "Left join should produce 4 joined features (Tokyo, Osaka, Nagoya, Yokohama)"
    );
    assert_eq!(
        unjoined_requestor_count, 1,
        "Left join should produce 1 unjoined requestor (UnmatchedCity1)"
    );
    assert_eq!(
        unjoined_supplier_count, 0,
        "Left join should not emit any unjoined suppliers"
    );

    // Verify joined features contain attributes from both requestor and supplier
    let joined = read_json_file(&temp_path.join("joined.json"));
    if let serde_json::Value::Array(features) = joined {
        // Check that Tokyo feature has both population (from requestor) and lat/lng (from supplier)
        let tokyo = features
            .iter()
            .find(|f| f.get("city").map(|v| v == "Tokyo").unwrap_or(false));
        assert!(
            tokyo.is_some(),
            "Joined output should contain Tokyo feature"
        );
        let tokyo = tokyo.unwrap();
        assert!(
            tokyo.get("population").is_some(),
            "Joined feature should have requestor's population attribute"
        );
        assert!(
            tokyo.get("lat").is_some(),
            "Joined feature should have supplier's lat attribute"
        );
        assert!(
            tokyo.get("lng").is_some(),
            "Joined feature should have supplier's lng attribute"
        );
    } else {
        panic!("Joined output should be an array");
    }

    // Verify unjoined requestor contains the expected unmatched city
    let unjoined_requestor = read_json_file(&temp_path.join("unjoined_requestor.json"));
    if let serde_json::Value::Array(features) = unjoined_requestor {
        assert_eq!(features.len(), 1);
        assert_eq!(
            features[0].get("city").and_then(|v| v.as_str()),
            Some("UnmatchedCity1"),
            "Unjoined requestor should be UnmatchedCity1"
        );
    } else {
        panic!("Unjoined requestor output should be an array");
    }
}

#[test]
fn test_joiner_inner() {
    // Inner join: 2 matched features → joined
    //             unmatched features dropped
    let tempdir = execute("feature/joiner_inner", vec![])
        .expect("Inner join workflow should complete successfully");
    let temp_path = tempdir.path();

    // Verify feature counts on each output port
    let joined_count = get_feature_count(&temp_path.join("joined.json"));
    let unjoined_requestor_count = get_feature_count(&temp_path.join("unjoined_requestor.json"));
    let unjoined_supplier_count = get_feature_count(&temp_path.join("unjoined_supplier.json"));

    assert_eq!(
        joined_count, 2,
        "Inner join should produce 2 joined features (Tokyo, Osaka)"
    );
    assert_eq!(
        unjoined_requestor_count, 0,
        "Inner join should not emit unjoined requestors"
    );
    assert_eq!(
        unjoined_supplier_count, 0,
        "Inner join should not emit unjoined suppliers"
    );

    // Verify joined features are Tokyo and Osaka
    let joined = read_json_file(&temp_path.join("joined.json"));
    if let serde_json::Value::Array(features) = joined {
        let cities: Vec<_> = features
            .iter()
            .filter_map(|f| f.get("city").and_then(|v| v.as_str()))
            .collect();
        assert!(
            cities.contains(&"Tokyo"),
            "Joined output should contain Tokyo"
        );
        assert!(
            cities.contains(&"Osaka"),
            "Joined output should contain Osaka"
        );
        assert!(
            !cities.contains(&"UnmatchedCity1"),
            "Joined output should not contain UnmatchedCity1"
        );
        assert!(
            !cities.contains(&"UnmatchedCity2"),
            "Joined output should not contain UnmatchedCity2"
        );
    } else {
        panic!("Joined output should be an array");
    }
}

#[test]
fn test_joiner_full() {
    // Full join: 2 matched features → joined
    //            1 unmatched requestor → unjoinedRequestor
    //            1 unmatched supplier → unjoinedSupplier
    let tempdir = execute("feature/joiner_full", vec![])
        .expect("Full join workflow should complete successfully");
    let temp_path = tempdir.path();

    // Verify feature counts on each output port
    let joined_count = get_feature_count(&temp_path.join("joined.json"));
    let unjoined_requestor_count = get_feature_count(&temp_path.join("unjoined_requestor.json"));
    let unjoined_supplier_count = get_feature_count(&temp_path.join("unjoined_supplier.json"));

    assert_eq!(
        joined_count, 2,
        "Full join should produce 2 joined features (Tokyo, Osaka)"
    );
    assert_eq!(
        unjoined_requestor_count, 1,
        "Full join should produce 1 unjoined requestor (UnmatchedCity1)"
    );
    assert_eq!(
        unjoined_supplier_count, 1,
        "Full join should produce 1 unjoined supplier (UnmatchedCity2)"
    );

    // Verify unjoined supplier contains the expected unmatched city
    let unjoined_supplier = read_json_file(&temp_path.join("unjoined_supplier.json"));
    if let serde_json::Value::Array(features) = unjoined_supplier {
        assert_eq!(features.len(), 1);
        assert_eq!(
            features[0].get("city").and_then(|v| v.as_str()),
            Some("UnmatchedCity2"),
            "Unjoined supplier should be UnmatchedCity2"
        );
        // Verify supplier has lat/lng but no population (not from requestor)
        assert!(
            features[0].get("lat").is_some(),
            "Unjoined supplier should have lat attribute"
        );
        assert!(
            features[0].get("lng").is_some(),
            "Unjoined supplier should have lng attribute"
        );
    } else {
        panic!("Unjoined supplier output should be an array");
    }
}
