use crate::reader::{read_u16, read_u32, GltfReaderError};
use indexmap::IndexMap;
use reearth_flow_types::AttributeValue;
use serde_json::Value;

/// Extract feature IDs from EXT_mesh_features extension
pub fn read_mesh_features(
    primitive: &gltf::Primitive,
    buffer_data: &[Vec<u8>],
) -> Result<Option<Vec<u32>>, GltfReaderError> {
    let mesh_features = match primitive.extension_value("EXT_mesh_features") {
        Some(mf) => mf,
        None => return Ok(None),
    };

    let feature_ids = match mesh_features.get("featureIds") {
        Some(Value::Array(ids)) => ids,
        _ => return Ok(None),
    };

    // Get the first feature ID set
    if feature_ids.is_empty() {
        return Ok(None);
    }

    let feature_id_obj = match &feature_ids[0] {
        Value::Object(obj) => obj,
        _ => return Ok(None),
    };

    // Check if feature IDs are stored in an attribute or constant
    if let Some(Value::Number(constant)) = feature_id_obj.get("constant") {
        // All vertices have the same feature ID
        let feature_id = constant.as_u64().unwrap_or(0) as u32;
        return Ok(Some(vec![feature_id]));
    }

    if let Some(Value::Number(attribute_index)) = feature_id_obj.get("attribute") {
        let attribute_idx = attribute_index.as_u64().unwrap_or(0) as usize;

        // Find the accessor for _FEATURE_ID_N attribute
        let attribute_name = format!("_FEATURE_ID_{}", attribute_idx);

        for (semantic, accessor) in primitive.attributes() {
            if semantic.to_string() == attribute_name {
                return read_feature_id_accessor(&accessor, buffer_data);
            }
        }
    }

    Ok(None)
}

fn read_feature_id_accessor(
    accessor: &gltf::Accessor,
    buffer_data: &[Vec<u8>],
) -> Result<Option<Vec<u32>>, GltfReaderError> {
    let view = accessor.view().ok_or_else(|| {
        GltfReaderError::Accessor("Feature ID accessor has no buffer view".to_string())
    })?;

    let buffer = &buffer_data[view.buffer().index()];
    let start = view.offset() + accessor.offset();
    let stride = view.stride().unwrap_or(accessor.size());

    let mut feature_ids = Vec::new();

    match accessor.data_type() {
        gltf::accessor::DataType::U16 => {
            for i in 0..accessor.count() {
                let offset = start + i * stride;
                let id = read_u16(buffer, offset)?;
                feature_ids.push(id as u32);
            }
        }
        gltf::accessor::DataType::U32 => {
            for i in 0..accessor.count() {
                let offset = start + i * stride;
                let id = read_u32(buffer, offset)?;
                feature_ids.push(id);
            }
        }
        gltf::accessor::DataType::U8 => {
            for i in 0..accessor.count() {
                let offset = start + i * stride;
                let id = buffer.get(offset).ok_or_else(|| {
                    GltfReaderError::Accessor("Feature ID out of bounds".to_string())
                })?;
                feature_ids.push(*id as u32);
            }
        }
        _ => {
            return Err(GltfReaderError::Accessor(format!(
                "Unsupported feature ID data type: {:?}",
                accessor.data_type()
            )))
        }
    }

    Ok(Some(feature_ids))
}

/// Extract property tables from EXT_structural_metadata extension
pub fn read_structural_metadata(
    gltf: &gltf::Gltf,
    buffer_data: &[Vec<u8>],
) -> Result<Option<PropertyTables>, GltfReaderError> {
    let structural_metadata = match gltf.extension_value("EXT_structural_metadata") {
        Some(sm) => sm,
        None => return Ok(None),
    };

    let schema = match structural_metadata.get("schema") {
        Some(s) => s,
        None => return Ok(None),
    };

    let property_tables = match structural_metadata.get("propertyTables") {
        Some(Value::Array(tables)) => tables,
        _ => return Ok(None),
    };

    let mut result = PropertyTables {
        schema: schema.clone(),
        tables: Vec::new(),
    };

    for table in property_tables {
        if let Value::Object(table_obj) = table {
            let parsed_table = parse_property_table(table_obj, buffer_data)?;
            result.tables.push(parsed_table);
        }
    }

    Ok(Some(result))
}

#[derive(Debug, Clone)]
pub struct PropertyTables {
    pub schema: Value,
    pub tables: Vec<PropertyTable>,
}

#[derive(Debug, Clone)]
pub struct PropertyTable {
    pub class: Option<String>,
    pub count: usize,
    pub properties: IndexMap<String, PropertyData>,
}

#[derive(Debug, Clone)]
pub struct PropertyData {
    pub values: Vec<AttributeValue>,
}

fn parse_property_table(
    table_obj: &serde_json::Map<String, Value>,
    buffer_data: &[Vec<u8>],
) -> Result<PropertyTable, GltfReaderError> {
    let class = table_obj
        .get("class")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let count = table_obj.get("count").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

    let properties = match table_obj.get("properties") {
        Some(Value::Object(props)) => props,
        _ => {
            return Ok(PropertyTable {
                class,
                count,
                properties: IndexMap::new(),
            })
        }
    };

    let mut parsed_properties = IndexMap::new();

    for (key, prop_def) in properties {
        if let Value::Object(prop_obj) = prop_def {
            // Extract string properties using buffer views
            if let Some(string_values) = parse_string_property(prop_obj, buffer_data, count)? {
                parsed_properties.insert(
                    key.clone(),
                    PropertyData {
                        values: string_values
                            .into_iter()
                            .map(AttributeValue::String)
                            .collect(),
                    },
                );
            } else {
                // TODO: Handle other property types (numeric, etc.)
                parsed_properties.insert(key.clone(), PropertyData { values: Vec::new() });
            }
        }
    }

    Ok(PropertyTable {
        class,
        count,
        properties: parsed_properties,
    })
}

/// Parse string property from buffer views
fn parse_string_property(
    prop_obj: &serde_json::Map<String, Value>,
    buffer_data: &[Vec<u8>],
    count: usize,
) -> Result<Option<Vec<String>>, GltfReaderError> {
    let values_idx = match prop_obj.get("values").and_then(|v| v.as_u64()) {
        Some(idx) => idx as usize,
        None => return Ok(None),
    };

    let string_offsets_idx = match prop_obj.get("stringOffsets").and_then(|v| v.as_u64()) {
        Some(idx) => idx as usize,
        None => return Ok(None), // Not a string property
    };

    // Read offsets buffer
    let offsets_buffer = buffer_data.get(string_offsets_idx).ok_or_else(|| {
        GltfReaderError::Buffer(format!(
            "String offsets buffer {} not found",
            string_offsets_idx
        ))
    })?;

    let offsets: Vec<u32> = offsets_buffer
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    if offsets.len() != count + 1 {
        return Err(GltfReaderError::Buffer(format!(
            "String offsets length mismatch: expected {}, got {}",
            count + 1,
            offsets.len()
        )));
    }

    // Read values buffer
    let values_buffer = buffer_data.get(values_idx).ok_or_else(|| {
        GltfReaderError::Buffer(format!("String values buffer {} not found", values_idx))
    })?;

    // Extract strings
    let mut strings = Vec::new();
    for i in 0..count {
        let start = offsets[i] as usize;
        let end = offsets[i + 1] as usize;
        let s = std::str::from_utf8(&values_buffer[start..end]).map_err(|e| {
            GltfReaderError::Buffer(format!("Invalid UTF-8 in string property: {}", e))
        })?;
        strings.push(s.to_string());
    }

    Ok(Some(strings))
}

/// Extract feature properties as JSON values from a GLB file
/// Returns a map of gml_id -> properties for each feature
pub fn extract_feature_properties(
    gltf: &gltf::Gltf,
) -> Result<IndexMap<String, serde_json::Map<String, Value>>, GltfReaderError> {
    let mut result = IndexMap::new();

    // Get EXT_structural_metadata extension
    let metadata_value = match gltf.extension_value("EXT_structural_metadata") {
        Some(v) => v,
        None => return Ok(result),
    };

    // Get property tables
    let property_tables = metadata_value
        .get("propertyTables")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            GltfReaderError::Parse("No propertyTables in EXT_structural_metadata".to_string())
        })?;

    if property_tables.is_empty() {
        return Ok(result);
    }

    // Process first property table
    let prop_table = &property_tables[0];
    let count = prop_table["count"]
        .as_u64()
        .ok_or_else(|| GltfReaderError::Parse("Missing count in property table".to_string()))?
        as usize;

    let properties = prop_table["properties"].as_object().ok_or_else(|| {
        GltfReaderError::Parse("Missing properties in property table".to_string())
    })?;

    // Get buffer views and binary blob
    let buffer_views: Vec<_> = gltf.views().collect();
    let binary_blob = gltf
        .blob
        .as_ref()
        .ok_or_else(|| GltfReaderError::Buffer("Missing binary blob".to_string()))?;

    // Initialize feature properties
    let mut feature_props: Vec<serde_json::Map<String, Value>> =
        (0..count).map(|_| serde_json::Map::new()).collect();

    // Extract each property
    for (prop_name, prop_def) in properties {
        let values_idx = prop_def["values"].as_u64().ok_or_else(|| {
            GltfReaderError::Parse(format!("Missing values index for property {}", prop_name))
        })? as usize;

        // Check if it's a string property
        if let Some(offsets_idx) = prop_def.get("stringOffsets").and_then(|v| v.as_u64()) {
            let strings = extract_strings_from_glb(
                binary_blob,
                &buffer_views,
                values_idx,
                offsets_idx as usize,
                count,
            )?;
            for (i, s) in strings.into_iter().enumerate() {
                feature_props[i].insert(prop_name.clone(), Value::String(s));
            }
        }
        // TODO: Handle other property types (numeric, etc.)
    }

    // Key by gml_id
    for props in feature_props {
        if let Some(Value::String(gml_id)) = props.get("gml_id") {
            result.insert(gml_id.clone(), props);
        } else {
            return Err(GltfReaderError::Parse("Feature missing gml_id".to_string()));
        }
    }

    Ok(result)
}

/// Extract string array from glTF binary buffers (for GLB files)
fn extract_strings_from_glb(
    binary_blob: &[u8],
    buffer_views: &[gltf::buffer::View],
    values_idx: usize,
    offsets_idx: usize,
    count: usize,
) -> Result<Vec<String>, GltfReaderError> {
    // Read offsets
    let offsets_view = &buffer_views[offsets_idx];
    let offsets_start = offsets_view.offset();
    let offsets_len = offsets_view.length();
    let offsets_data = binary_blob
        .get(offsets_start..offsets_start + offsets_len)
        .ok_or_else(|| GltfReaderError::Buffer("Offsets buffer out of bounds".to_string()))?;

    let offsets: Vec<u32> = offsets_data
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    if offsets.len() != count + 1 {
        return Err(GltfReaderError::Buffer(format!(
            "Offsets length mismatch: {} vs {}",
            offsets.len() - 1,
            count
        )));
    }

    // Read string data
    let values_view = &buffer_views[values_idx];
    let values_start = values_view.offset();
    let values_len = values_view.length();
    let strings_data = binary_blob
        .get(values_start..values_start + values_len)
        .ok_or_else(|| GltfReaderError::Buffer("Values buffer out of bounds".to_string()))?;

    // Extract strings
    let mut strings = Vec::new();
    for i in 0..count {
        let start = offsets[i] as usize;
        let end = offsets[i + 1] as usize;
        let s = std::str::from_utf8(&strings_data[start..end])
            .map_err(|e| GltfReaderError::Buffer(format!("Invalid UTF-8 in string: {}", e)))?;
        strings.push(s.to_string());
    }

    Ok(strings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_gltf;

    #[test]
    fn test_extract_feature_properties() {
        // Load test GLB file with EXT_structural_metadata
        let glb_data = include_bytes!("test_data_39255_tran_AuxiliaryTrafficArea.glb");
        let gltf = parse_gltf(&bytes::Bytes::from(&glb_data[..])).expect("Failed to parse GLB");

        // Extract feature properties
        let features = extract_feature_properties(&gltf).expect("Failed to extract features");

        // Verify we have the expected features
        // Based on Python reader output, this file contains 4+ features
        assert!(!features.is_empty(), "Should have extracted features");
        assert!(
            features.len() >= 4,
            "Expected at least 4 features, got {}",
            features.len()
        );

        // Verify specific features by gml_id
        let expected_gml_ids = vec![
            "tran_4d448e8a-db1d-48ef-8f04-feb24b49b701",
            "tran_3b28a7b2-a741-4569-bf09-0dadaf5996f4",
            "tran_ddf91fb3-b1db-4bdb-91d9-ae67ba146e62",
            "tran_8a8270ea-3e6a-491a-b98f-b2fd6869d3be",
        ];

        for gml_id in &expected_gml_ids {
            assert!(
                features.contains_key(*gml_id),
                "Missing feature with gml_id: {}",
                gml_id
            );
        }

        // Verify properties of the first feature
        let feature1 = features
            .get("tran_4d448e8a-db1d-48ef-8f04-feb24b49b701")
            .expect("Feature 1 should exist");

        // Check expected properties
        assert_eq!(
            feature1.get("gml_id").and_then(|v| v.as_str()),
            Some("tran_4d448e8a-db1d-48ef-8f04-feb24b49b701")
        );
        assert_eq!(
            feature1.get("meshcode").and_then(|v| v.as_str()),
            Some("54401008")
        );
        assert_eq!(
            feature1.get("tran:class").and_then(|v| v.as_str()),
            Some("road traffic")
        );
        assert_eq!(
            feature1.get("feature_type").and_then(|v| v.as_str()),
            Some("tran:AuxiliaryTrafficArea")
        );
        assert_eq!(
            feature1.get("core:creationDate").and_then(|v| v.as_str()),
            Some("2024-03-19")
        );
        assert_eq!(
            feature1.get("city_code").and_then(|v| v.as_str()),
            Some("08220")
        );
        assert_eq!(
            feature1.get("city_name").and_then(|v| v.as_str()),
            Some("茨城県つくば市")
        );
        assert_eq!(
            feature1.get("tran:function").and_then(|v| v.as_str()),
            Some("路肩")
        );

        // Verify the second feature has correct gml_id
        let feature2 = features
            .get("tran_3b28a7b2-a741-4569-bf09-0dadaf5996f4")
            .expect("Feature 2 should exist");
        assert_eq!(
            feature2.get("gml_id").and_then(|v| v.as_str()),
            Some("tran_3b28a7b2-a741-4569-bf09-0dadaf5996f4")
        );
        assert_eq!(
            feature2.get("tran:function").and_then(|v| v.as_str()),
            Some("路肩")
        );
    }

    #[test]
    fn test_extract_feature_properties_no_extension() {
        // Create a minimal GLB without EXT_structural_metadata
        // For now, just verify it returns empty without error
        // A proper test would need a real GLB without the extension
        // This is a placeholder to demonstrate the API
    }
}
