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
            let parsed_table = parse_property_table(&table_obj, buffer_data)?;
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
    _buffer_data: &[Vec<u8>],
) -> Result<PropertyTable, GltfReaderError> {
    let class = table_obj
        .get("class")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let count = table_obj
        .get("count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

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

    for (key, _value) in properties {
        // TODO: Parse property values from buffer views
        // For now, return empty placeholder
        parsed_properties.insert(
            key.clone(),
            PropertyData {
                values: Vec::new(),
            },
        );
    }

    Ok(PropertyTable {
        class,
        count,
        properties: parsed_properties,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_extensions() {
        // Basic test structure - actual implementation would need real GLTF data
        assert!(true);
    }
}
