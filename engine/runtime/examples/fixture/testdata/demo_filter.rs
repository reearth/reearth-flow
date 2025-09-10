// Demo script showing the native JSON filtering functionality
// This replaces the jq dependency with pure Rust implementation

use serde_json;

fn main() {
    let test_data = r#"{
  "id": "8c52c701-1199-4bd2-aa33-9e64e543e98b",
  "attributes": {
    "_num_invalid_bldginst_geom_type": 2,
    "other_field": "should be filtered"
  },
  "geometry": {
    "epsg": null,
    "value": "none"
  },
  "metadata": {
    "featureId": null,
    "featureType": null,
    "lod": null
  }
}"#;

    println!("Original data:");
    println!("{}\n", test_data);
    
    // Parse JSON
    let json: serde_json::Value = serde_json::from_str(test_data).unwrap();
    
    // Demonstrate different filter types
    
    // 1. Extract only attributes field (equivalent to jq: {attributes})
    let mut result = serde_json::Map::new();
    if let Some(attributes) = json.get("attributes") {
        result.insert("attributes".to_string(), attributes.clone());
    }
    println!("Filter {{attributes}} result:");
    println!("{}\n", serde_json::to_string_pretty(&result).unwrap());
    
    // 2. Extract attributes value directly (equivalent to jq: .attributes)
    if let Some(attributes) = json.get("attributes") {
        println!("Filter .attributes result:");
        println!("{}\n", serde_json::to_string_pretty(attributes).unwrap());
    }
}