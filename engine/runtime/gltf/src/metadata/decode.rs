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

        // `Semantic::Extras`'s inner name excludes the glTF-spec-mandated
        // leading underscore; the crate adds it on (de)serialization.
        let expected = gltf::Semantic::Extras(format!("FEATURE_ID_{attribute_idx}"));

        for (semantic, accessor) in primitive.attributes() {
            if semantic == expected {
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
        gltf::accessor::DataType::F32 => {
            // Some tools store feature IDs as floats - convert to u32
            for i in 0..accessor.count() {
                let offset = start + i * stride;
                let bytes = buffer.get(offset..offset + 4).ok_or_else(|| {
                    GltfReaderError::Accessor("Feature ID out of bounds".to_string())
                })?;
                let float_id = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                feature_ids.push(float_id as u32);
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
            let parsed_table = parse_property_table(gltf, table_obj, schema, buffer_data)?;
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
    gltf: &gltf::Gltf,
    table_obj: &serde_json::Map<String, Value>,
    schema: &Value,
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
        let Value::Object(prop_obj) = prop_def else {
            continue;
        };

        let no_data = schema_property_field(schema, class.as_deref(), key, "noData");

        // Extract string properties using buffer views. A row matching the
        // schema's `noData` (compared as a string) becomes `Null` so
        // `feature_properties` can elide it, the same as numeric `noData`.
        if let Some(string_values) = parse_string_property(gltf, prop_obj, buffer_data, count)? {
            let no_data_str = no_data.and_then(|nd| nd.as_str());
            parsed_properties.insert(
                key.clone(),
                PropertyData {
                    values: string_values
                        .into_iter()
                        .map(|s| {
                            if no_data_str == Some(s.as_str()) {
                                AttributeValue::Null
                            } else {
                                AttributeValue::String(s)
                            }
                        })
                        .collect(),
                },
            );
            continue;
        }

        // BOOLEAN properties have no `componentType` (their `type` IS
        // "BOOLEAN").
        let property_type =
            schema_property_field(schema, class.as_deref(), key, "type").and_then(|v| v.as_str());

        if property_type == Some("BOOLEAN") {
            if let Some(values) = parse_boolean_property(gltf, prop_obj, buffer_data, count)? {
                parsed_properties.insert(key.clone(), PropertyData { values });
                continue;
            }
        }

        // VEC2/3/4, MAT2/3/4, and array-flagged SCALAR properties also carry
        // a `componentType`, but decoding them with the flat-scalar reader
        // below would read the wrong stride and produce corrupt values,
        // so only dispatch to it for genuine scalar (non-array) properties.
        let is_array = schema_property_field(schema, class.as_deref(), key, "array")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if property_type == Some("SCALAR") && !is_array {
            let component_type =
                schema_property_field(schema, class.as_deref(), key, "componentType")
                    .and_then(|v| v.as_str());

            if let Some(component_type) = component_type {
                if let Some(values) = parse_numeric_property(
                    gltf,
                    prop_obj,
                    buffer_data,
                    count,
                    component_type,
                    no_data,
                )? {
                    parsed_properties.insert(key.clone(), PropertyData { values });
                    continue;
                }
            }
        }

        // TODO: Handle other property types (ENUM, VECN/MATN, arrays, ...)
        parsed_properties.insert(key.clone(), PropertyData { values: Vec::new() });
    }

    Ok(PropertyTable {
        class,
        count,
        properties: parsed_properties,
    })
}

/// Look up `/classes/<class>/properties/<prop_name>/<field>` in a
/// `EXT_structural_metadata` schema (e.g. `componentType`, `type`, `noData`).
fn schema_property_field<'a>(
    schema: &'a Value,
    class: Option<&str>,
    prop_name: &str,
    field: &str,
) -> Option<&'a Value> {
    schema.pointer(&format!(
        "/classes/{}/properties/{}/{}",
        class?, prop_name, field
    ))
}

/// Resolve an `EXT_structural_metadata` property's `values`/`stringOffsets`/
/// `arrayOffsets` field to its raw bytes. These fields name a **bufferView**
/// index, not a buffer index (the same convention [`extract_feature_properties`]
/// resolves via `gltf.views()` off the blob); `buffer_data` is still indexed
/// by *buffer*, so the view's own `buffer`/`byteOffset`/`byteLength` pick the
/// right slice out of it.
fn resolve_metadata_buffer_view<'a>(
    gltf: &gltf::Gltf,
    view_index: usize,
    buffer_data: &'a [Vec<u8>],
) -> Result<&'a [u8], GltfReaderError> {
    let view = gltf
        .views()
        .nth(view_index)
        .ok_or_else(|| GltfReaderError::Buffer(format!("bufferView {view_index} not found")))?;
    let buffer = buffer_data.get(view.buffer().index()).ok_or_else(|| {
        GltfReaderError::Buffer(format!(
            "buffer {} (referenced by bufferView {view_index}) not found",
            view.buffer().index()
        ))
    })?;
    buffer
        .get(view.offset()..view.offset() + view.length())
        .ok_or_else(|| GltfReaderError::Buffer(format!("bufferView {view_index} out of bounds")))
}

/// Parse a bit-packed BOOLEAN property (one bit per row, LSB-first within
/// each byte, per the `EXT_structural_metadata` spec) from the bufferView
/// identified by the property's `values` index.
fn parse_boolean_property(
    gltf: &gltf::Gltf,
    prop_obj: &serde_json::Map<String, Value>,
    buffer_data: &[Vec<u8>],
    count: usize,
) -> Result<Option<Vec<AttributeValue>>, GltfReaderError> {
    let values_idx = match prop_obj.get("values").and_then(|v| v.as_u64()) {
        Some(idx) => idx as usize,
        None => return Ok(None),
    };

    let values_buffer = resolve_metadata_buffer_view(gltf, values_idx, buffer_data)?;

    let mut values = Vec::with_capacity(count);
    for i in 0..count {
        let byte = values_buffer.get(i / 8).ok_or_else(|| {
            GltfReaderError::Buffer("Boolean property value out of bounds".to_string())
        })?;
        values.push(AttributeValue::Bool((byte >> (i % 8)) & 1 != 0));
    }
    Ok(Some(values))
}

/// Parse a numeric property (any EXT_structural_metadata numeric
/// `componentType`) from the bufferView identified by the property's `values`
/// index. Mirrors the component-type handling in [`extract_feature_properties`],
/// resolving bytes the same way it does (via `gltf`'s bufferViews), just
/// against the caller-supplied `buffer_data` instead of the glTF binary blob.
/// Only ever called for genuine SCALAR, non-array properties (see the caller
/// in [`parse_property_table`]): VEC2/3/4, MAT2/3/4, and fixed/variable-length
/// arrays also carry a `componentType` but need a different, wider stride and
/// are intentionally left unhandled.
///
/// A row whose raw decoded value equals the schema's `noData` becomes
/// `AttributeValue::Null`; [`feature_properties`] skips `Null` entries.
fn parse_numeric_property(
    gltf: &gltf::Gltf,
    prop_obj: &serde_json::Map<String, Value>,
    buffer_data: &[Vec<u8>],
    count: usize,
    component_type: &str,
    no_data: Option<&Value>,
) -> Result<Option<Vec<AttributeValue>>, GltfReaderError> {
    let values_idx = match prop_obj.get("values").and_then(|v| v.as_u64()) {
        Some(idx) => idx as usize,
        None => return Ok(None),
    };

    let values_buffer = resolve_metadata_buffer_view(gltf, values_idx, buffer_data)?;

    let mut values = Vec::with_capacity(count);
    for i in 0..count {
        values.push(decode_numeric_element(
            values_buffer,
            i,
            component_type,
            no_data,
        )?);
    }
    Ok(Some(values))
}

fn is_no_data_i64(no_data: Option<&Value>, v: i64) -> bool {
    no_data.and_then(|nd| nd.as_i64()).is_some_and(|nd| nd == v)
}

fn is_no_data_u64(no_data: Option<&Value>, v: u64) -> bool {
    no_data.and_then(|nd| nd.as_u64()).is_some_and(|nd| nd == v)
}

fn is_no_data_f64(no_data: Option<&Value>, v: f64) -> bool {
    no_data
        .and_then(|nd| nd.as_f64())
        .is_some_and(|nd| (nd - v).abs() < f64::EPSILON)
}

/// Decode the `index`-th element of `component_type` out of `buffer`,
/// little-endian, mapping signed types to `AttributeValue::Number` backed by
/// `i64`, unsigned types by `u64`, and floats via `serde_json::Number::from_f64`.
/// Returns `AttributeValue::Null` when the decoded value matches `no_data`
/// (comparison mirrors [`extract_feature_properties`]'s `is_no_data` checks).
fn decode_numeric_element(
    buffer: &[u8],
    index: usize,
    component_type: &str,
    no_data: Option<&Value>,
) -> Result<AttributeValue, GltfReaderError> {
    fn bytes_at(buffer: &[u8], start: usize, len: usize) -> Result<&[u8], GltfReaderError> {
        buffer.get(start..start + len).ok_or_else(|| {
            GltfReaderError::Buffer("Property value buffer read out of bounds".to_string())
        })
    }

    Ok(match component_type {
        "INT8" => {
            let v = bytes_at(buffer, index, 1)?[0] as i8 as i64;
            if is_no_data_i64(no_data, v) {
                AttributeValue::Null
            } else {
                AttributeValue::Number(v.into())
            }
        }
        "UINT8" => {
            let v = bytes_at(buffer, index, 1)?[0] as u64;
            if is_no_data_u64(no_data, v) {
                AttributeValue::Null
            } else {
                AttributeValue::Number(v.into())
            }
        }
        "INT16" => {
            let b = bytes_at(buffer, index * 2, 2)?;
            let v = i16::from_le_bytes([b[0], b[1]]) as i64;
            if is_no_data_i64(no_data, v) {
                AttributeValue::Null
            } else {
                AttributeValue::Number(v.into())
            }
        }
        "UINT16" => {
            let b = bytes_at(buffer, index * 2, 2)?;
            let v = u16::from_le_bytes([b[0], b[1]]) as u64;
            if is_no_data_u64(no_data, v) {
                AttributeValue::Null
            } else {
                AttributeValue::Number(v.into())
            }
        }
        "INT32" => {
            let b = bytes_at(buffer, index * 4, 4)?;
            let v = i32::from_le_bytes([b[0], b[1], b[2], b[3]]) as i64;
            if is_no_data_i64(no_data, v) {
                AttributeValue::Null
            } else {
                AttributeValue::Number(v.into())
            }
        }
        "UINT32" => {
            let b = bytes_at(buffer, index * 4, 4)?;
            let v = u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as u64;
            if is_no_data_u64(no_data, v) {
                AttributeValue::Null
            } else {
                AttributeValue::Number(v.into())
            }
        }
        "INT64" => {
            let b = bytes_at(buffer, index * 8, 8)?;
            let v = i64::from_le_bytes(b.try_into().unwrap());
            if is_no_data_i64(no_data, v) {
                AttributeValue::Null
            } else {
                AttributeValue::Number(v.into())
            }
        }
        "UINT64" => {
            let b = bytes_at(buffer, index * 8, 8)?;
            let v = u64::from_le_bytes(b.try_into().unwrap());
            if is_no_data_u64(no_data, v) {
                AttributeValue::Null
            } else {
                AttributeValue::Number(v.into())
            }
        }
        "FLOAT32" => {
            let b = bytes_at(buffer, index * 4, 4)?;
            let v = f32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f64;
            if is_no_data_f64(no_data, v) {
                AttributeValue::Null
            } else {
                AttributeValue::Number(serde_json::Number::from_f64(v).unwrap_or_else(|| 0.into()))
            }
        }
        "FLOAT64" => {
            let b = bytes_at(buffer, index * 8, 8)?;
            let v = f64::from_le_bytes(b.try_into().unwrap());
            if is_no_data_f64(no_data, v) {
                AttributeValue::Null
            } else {
                AttributeValue::Number(serde_json::Number::from_f64(v).unwrap_or_else(|| 0.into()))
            }
        }
        other => {
            return Err(GltfReaderError::Parse(format!(
                "Unsupported componentType '{}' in structural metadata property",
                other
            )))
        }
    })
}

/// Look up `feature_id`'s attributes in property table `table_index`.
/// Returns an empty map if the table or the feature id is out of range.
/// Rows decoded as `AttributeValue::Null` (i.e. matched the schema's
/// `noData`) are skipped, so `noData` sentinels never surface as attributes.
pub fn feature_properties(
    tables: &PropertyTables,
    table_index: usize,
    feature_id: u32,
) -> IndexMap<String, AttributeValue> {
    let mut out = IndexMap::new();
    let Some(table) = tables.tables.get(table_index) else {
        return out;
    };
    let row = feature_id as usize;
    if row >= table.count {
        return out;
    }
    for (name, data) in &table.properties {
        if let Some(v) = data.values.get(row) {
            if !matches!(v, AttributeValue::Null) {
                out.insert(name.clone(), v.clone());
            }
        }
    }
    out
}

/// Parse string property from buffer views
fn parse_string_property(
    gltf: &gltf::Gltf,
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
    let offsets_buffer = resolve_metadata_buffer_view(gltf, string_offsets_idx, buffer_data)?;

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
    let values_buffer = resolve_metadata_buffer_view(gltf, values_idx, buffer_data)?;

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
/// Returns a list of properties for each feature
pub fn extract_feature_properties(
    gltf: &gltf::Gltf,
) -> Result<Vec<serde_json::Map<String, Value>>, GltfReaderError> {
    // Get EXT_structural_metadata extension
    let metadata_value = match gltf.extension_value("EXT_structural_metadata") {
        Some(v) => v,
        None => return Ok(Vec::new()),
    };

    // Get property tables
    let property_tables = metadata_value
        .get("propertyTables")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            GltfReaderError::Parse("No propertyTables in EXT_structural_metadata".to_string())
        })?;

    if property_tables.is_empty() {
        return Ok(Vec::new());
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

    // Get component type from schema for each property
    let get_component_type = |prop_name: &str| -> Option<&str> {
        metadata_value
            .pointer(&format!(
                "/schema/classes/{}/properties/{}/componentType",
                prop_table.get("class")?.as_str()?,
                prop_name
            ))
            .and_then(|v| v.as_str())
    };

    // Get noData value from schema for each property
    let get_no_data = |prop_name: &str| -> Option<&Value> {
        metadata_value.pointer(&format!(
            "/schema/classes/{}/properties/{}/noData",
            prop_table.get("class")?.as_str()?,
            prop_name
        ))
    };

    // Extract each property
    for (prop_name, prop_def) in properties {
        let values_idx = prop_def["values"].as_u64().ok_or_else(|| {
            GltfReaderError::Parse(format!("Missing values index for property {}", prop_name))
        })? as usize;

        let values_view = &buffer_views[values_idx];
        let data = &binary_blob[values_view.offset()..values_view.offset() + values_view.length()];

        // Get noData value from schema if it exists (used by both string and numeric properties)
        let no_data = get_no_data(prop_name);

        // String property
        if let Some(offsets_idx) = prop_def.get("stringOffsets").and_then(|v| v.as_u64()) {
            let offsets_view = &buffer_views[offsets_idx as usize];
            let offsets_data =
                &binary_blob[offsets_view.offset()..offsets_view.offset() + offsets_view.length()];
            let offsets: Vec<u32> = offsets_data
                .chunks_exact(4)
                .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect();

            for i in 0..count {
                let s = std::str::from_utf8(&data[offsets[i] as usize..offsets[i + 1] as usize])
                    .map_err(|e| GltfReaderError::Buffer(format!("Invalid UTF-8: {}", e)))?;

                // Check if this string matches noData
                let is_no_data = no_data
                    .and_then(|nd| nd.as_str())
                    .map(|nd| s == nd)
                    .unwrap_or(false);

                // Only insert if not noData
                if !is_no_data {
                    feature_props[i].insert(prop_name.clone(), Value::String(s.to_string()));
                }
            }
            continue;
        }

        // Numeric/enum properties - must have component type
        let component_type = get_component_type(prop_name).ok_or_else(|| {
            GltfReaderError::Parse(format!("Missing componentType for property {}", prop_name))
        })?;

        for i in 0..count {
            let value = match component_type {
                "INT8" => {
                    let v = data[i] as i8;
                    let is_no_data = no_data
                        .and_then(|nd| nd.as_i64())
                        .map(|nd| v as i64 == nd)
                        .unwrap_or(false);
                    (!is_no_data).then(|| Value::Number((v as i64).into()))
                }
                "UINT8" => {
                    let v = data[i];
                    let is_no_data = no_data
                        .and_then(|nd| nd.as_u64())
                        .map(|nd| v as u64 == nd)
                        .unwrap_or(false);
                    (!is_no_data).then(|| Value::Number((v as u64).into()))
                }
                "INT16" => {
                    let v = i16::from_le_bytes(data[i * 2..i * 2 + 2].try_into().unwrap());
                    let is_no_data = no_data
                        .and_then(|nd| nd.as_i64())
                        .map(|nd| v as i64 == nd)
                        .unwrap_or(false);
                    (!is_no_data).then(|| Value::Number((v as i64).into()))
                }
                "UINT16" => {
                    let v = u16::from_le_bytes(data[i * 2..i * 2 + 2].try_into().unwrap());
                    let is_no_data = no_data
                        .and_then(|nd| nd.as_u64())
                        .map(|nd| v as u64 == nd)
                        .unwrap_or(false);
                    (!is_no_data).then(|| Value::Number((v as u64).into()))
                }
                "INT32" => {
                    let v = i32::from_le_bytes(data[i * 4..i * 4 + 4].try_into().unwrap());
                    let is_no_data = no_data
                        .and_then(|nd| nd.as_i64())
                        .map(|nd| v as i64 == nd)
                        .unwrap_or(false);
                    (!is_no_data).then(|| Value::Number((v as i64).into()))
                }
                "UINT32" => {
                    let v = u32::from_le_bytes(data[i * 4..i * 4 + 4].try_into().unwrap());
                    let is_no_data = no_data
                        .and_then(|nd| nd.as_u64())
                        .map(|nd| v as u64 == nd)
                        .unwrap_or(false);
                    (!is_no_data).then(|| Value::Number(v.into()))
                }
                "INT64" => {
                    let v = i64::from_le_bytes(data[i * 8..i * 8 + 8].try_into().unwrap());
                    let is_no_data = no_data
                        .and_then(|nd| nd.as_i64())
                        .map(|nd| v == nd)
                        .unwrap_or(false);
                    (!is_no_data).then(|| Value::Number(v.into()))
                }
                "UINT64" => {
                    let v = u64::from_le_bytes(data[i * 8..i * 8 + 8].try_into().unwrap());
                    let is_no_data = no_data
                        .and_then(|nd| nd.as_u64())
                        .map(|nd| v == nd)
                        .unwrap_or(false);
                    (!is_no_data).then(|| Value::Number(v.into()))
                }
                "FLOAT32" => {
                    let v = f32::from_le_bytes(data[i * 4..i * 4 + 4].try_into().unwrap());
                    let is_no_data = no_data
                        .and_then(|nd| nd.as_f64())
                        .map(|nd| (v as f64 - nd).abs() < f64::EPSILON)
                        .unwrap_or(false);
                    (!is_no_data)
                        .then(|| serde_json::Number::from_f64(v as f64))
                        .flatten()
                        .map(Value::Number)
                }
                "FLOAT64" => {
                    let v = f64::from_le_bytes(data[i * 8..i * 8 + 8].try_into().unwrap());
                    let is_no_data = no_data
                        .and_then(|nd| nd.as_f64())
                        .map(|nd| (v - nd).abs() < f64::EPSILON)
                        .unwrap_or(false);
                    (!is_no_data)
                        .then(|| serde_json::Number::from_f64(v))
                        .flatten()
                        .map(Value::Number)
                }
                _ => {
                    return Err(GltfReaderError::Parse(format!(
                        "Unsupported componentType '{}' for property {}",
                        component_type, prop_name
                    )))
                }
            };
            if let Some(v) = value {
                feature_props[i].insert(prop_name.clone(), v);
            }
        }
    }

    Ok(feature_props)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_gltf;

    /// A bufferless-data `gltf::Gltf` whose `buffers`/`bufferViews` are an
    /// identity mapping: bufferView `i` covers the whole of buffer `i`. Lets a
    /// unit test's `"values"`/`"stringOffsets"` property indices keep
    /// addressing its `buffer_data` vec directly by position (as before
    /// `resolve_metadata_buffer_view` was introduced), while still exercising
    /// the real bufferView-resolution path.
    fn gltf_with_identity_buffer_views(buffer_lens: &[usize]) -> gltf::Gltf {
        let buffers: Vec<Value> = buffer_lens
            .iter()
            .map(|len| serde_json::json!({"byteLength": len}))
            .collect();
        let buffer_views: Vec<Value> = buffer_lens
            .iter()
            .enumerate()
            .map(|(i, len)| serde_json::json!({"buffer": i, "byteOffset": 0, "byteLength": len}))
            .collect();
        let doc = serde_json::json!({
            "asset": {"version": "2.0"},
            "buffers": buffers,
            "bufferViews": buffer_views,
        });
        gltf::Gltf::from_slice_without_validation(&serde_json::to_vec(&doc).unwrap())
            .expect("identity buffer-view fixture should parse")
    }

    #[test]
    fn test_extract_feature_properties() {
        // Load test GLB file with EXT_structural_metadata
        let glb_data =
            crate::test_utils::load_testdata("test_data_39255_tran_AuxiliaryTrafficArea.glb");
        let gltf = parse_gltf(&bytes::Bytes::from(glb_data)).expect("Failed to parse GLB");

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
                features
                    .iter()
                    .any(|f| f.get("gml_id").and_then(|v| v.as_str()) == Some(*gml_id)),
                "Missing feature with gml_id: {}",
                gml_id
            );
        }

        // Verify properties of the first feature
        let feature1 = features
            .iter()
            .find(|f| {
                f.get("gml_id").and_then(|v| v.as_str())
                    == Some("tran_4d448e8a-db1d-48ef-8f04-feb24b49b701")
            })
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
            .iter()
            .find(|f| {
                f.get("gml_id").and_then(|v| v.as_str())
                    == Some("tran_3b28a7b2-a741-4569-bf09-0dadaf5996f4")
            })
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
        // Test with minimal GLB that has NO EXT_structural_metadata extension
        // Should return empty IndexMap without error
        let glb_bytes = crate::test_utils::load_testdata("minimal_rectangle.glb");

        let gltf = gltf::Gltf::from_slice(&glb_bytes).expect("Failed to parse GLB");

        let result =
            extract_feature_properties(&gltf).expect("Should not error on missing extension");

        // Should return empty IndexMap when extension is not present
        assert!(
            result.is_empty(),
            "Expected empty result when EXT_structural_metadata is not present"
        );
    }

    #[test]
    fn decodes_numeric_and_string_properties_by_feature_id() {
        // Two features; "height" UINT32 = [10, 20], "name" string = ["a","b"].
        let (gltf, buffers) = fixtures::metadata_glb_two_features();
        let tables = read_structural_metadata(&gltf, &buffers).unwrap().unwrap();

        let f0 = feature_properties(&tables, 0, 0);
        assert_eq!(
            f0.get("height"),
            Some(&AttributeValue::Number(10u64.into()))
        );
        assert_eq!(f0.get("name"), Some(&AttributeValue::String("a".into())));

        let f1 = feature_properties(&tables, 0, 1);
        assert_eq!(
            f1.get("height"),
            Some(&AttributeValue::Number(20u64.into()))
        );
        assert_eq!(f1.get("name"), Some(&AttributeValue::String("b".into())));

        // Out-of-range feature id -> empty, no panic.
        assert!(feature_properties(&tables, 0, 99).is_empty());

        // Out-of-range table index -> empty, no panic.
        assert!(feature_properties(&tables, 1, 0).is_empty());
    }

    #[test]
    fn decodes_all_numeric_component_types() {
        // One property per supported componentType, one feature (row 0).
        // Buffer bytes are hand-verified little-endian encodings of the
        // values noted alongside each property below.
        let schema = serde_json::json!({
            "classes": {
                "Feature": {
                    "properties": {
                        "i8": {"type": "SCALAR", "componentType": "INT8"},
                        "u8": {"type": "SCALAR", "componentType": "UINT8"},
                        "i16": {"type": "SCALAR", "componentType": "INT16"},
                        "u16": {"type": "SCALAR", "componentType": "UINT16"},
                        "i32": {"type": "SCALAR", "componentType": "INT32"},
                        "u32": {"type": "SCALAR", "componentType": "UINT32"},
                        "i64": {"type": "SCALAR", "componentType": "INT64"},
                        "u64": {"type": "SCALAR", "componentType": "UINT64"},
                        "f32": {"type": "SCALAR", "componentType": "FLOAT32"},
                        "f64": {"type": "SCALAR", "componentType": "FLOAT64"},
                        "flag": {"type": "BOOLEAN"},
                    }
                }
            }
        });

        let table_obj_value = serde_json::json!({
            "class": "Feature",
            "count": 1,
            "properties": {
                "i8": {"values": 0},
                "u8": {"values": 1},
                "i16": {"values": 2},
                "u16": {"values": 3},
                "i32": {"values": 4},
                "u32": {"values": 5},
                "i64": {"values": 6},
                "u64": {"values": 7},
                "f32": {"values": 8},
                "f64": {"values": 9},
                "flag": {"values": 10},
            }
        });
        let Value::Object(table_obj) = table_obj_value else {
            unreachable!()
        };

        let buffers: Vec<Vec<u8>> = vec![
            vec![(-5i8) as u8],                                   // i8 = -5
            vec![200u8],                                          // u8 = 200
            (-1000i16).to_le_bytes().to_vec(),                    // i16 = -1000
            60000u16.to_le_bytes().to_vec(),                      // u16 = 60000
            (-70000i32).to_le_bytes().to_vec(),                   // i32 = -70000
            4_000_000_000u32.to_le_bytes().to_vec(),              // u32 = 4_000_000_000
            (-9_000_000_000_000i64).to_le_bytes().to_vec(),       // i64
            18_000_000_000_000_000_000u64.to_le_bytes().to_vec(), // u64
            1.5f32.to_le_bytes().to_vec(),                        // f32 = 1.5
            2.25f64.to_le_bytes().to_vec(),                       // f64 = 2.25
            vec![0b0000_0001],                                    // flag row 0 = true
        ];

        let gltf =
            gltf_with_identity_buffer_views(&buffers.iter().map(|b| b.len()).collect::<Vec<_>>());
        let table = parse_property_table(&gltf, &table_obj, &schema, &buffers).unwrap();

        assert_eq!(
            table.properties["i8"].values[0],
            AttributeValue::Number((-5i64).into())
        );
        assert_eq!(
            table.properties["u8"].values[0],
            AttributeValue::Number(200u64.into())
        );
        assert_eq!(
            table.properties["i16"].values[0],
            AttributeValue::Number((-1000i64).into())
        );
        assert_eq!(
            table.properties["u16"].values[0],
            AttributeValue::Number(60000u64.into())
        );
        assert_eq!(
            table.properties["i32"].values[0],
            AttributeValue::Number((-70000i64).into())
        );
        assert_eq!(
            table.properties["u32"].values[0],
            AttributeValue::Number(4_000_000_000u64.into())
        );
        assert_eq!(
            table.properties["i64"].values[0],
            AttributeValue::Number((-9_000_000_000_000i64).into())
        );
        assert_eq!(
            table.properties["u64"].values[0],
            AttributeValue::Number(18_000_000_000_000_000_000u64.into())
        );
        assert_eq!(
            table.properties["f32"].values[0],
            AttributeValue::Number(serde_json::Number::from_f64(1.5).unwrap())
        );
        assert_eq!(
            table.properties["f64"].values[0],
            AttributeValue::Number(serde_json::Number::from_f64(2.25).unwrap())
        );
        assert_eq!(
            table.properties["flag"].values[0],
            AttributeValue::Bool(true)
        );
    }

    #[test]
    fn no_data_rows_are_omitted_from_feature_properties() {
        // Two features. "height" UINT32 noData = u32::MAX; row 1 is noData.
        // "name" STRING noData = "N/A"; row 1 is noData.
        let schema = serde_json::json!({
            "classes": {
                "Feature": {
                    "properties": {
                        "height": {"type": "SCALAR", "componentType": "UINT32", "noData": u32::MAX},
                        "name": {"type": "STRING", "noData": "N/A"}
                    }
                }
            }
        });
        let table_obj_value = serde_json::json!({
            "class": "Feature",
            "count": 2,
            "properties": {
                "height": {"values": 0},
                "name": {"values": 1, "stringOffsets": 2}
            }
        });
        let Value::Object(table_obj) = table_obj_value else {
            unreachable!()
        };

        // buffer 0: "height" UINT32 values: [10, u32::MAX]
        let height_buffer: Vec<u8> = [10u32, u32::MAX]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        // buffer 1: "name" STRING raw UTF-8 bytes: "a" + "N/A"
        let name_values_buffer: Vec<u8> = b"aN/A".to_vec();
        // buffer 2: "name" stringOffsets, UINT32 LE cumulative byte offsets
        // into buffer 1 (count + 1 = 3 entries): [0, 1, 4]
        let name_offsets_buffer: Vec<u8> = [0u32, 1u32, 4u32]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        let buffers = vec![height_buffer, name_values_buffer, name_offsets_buffer];

        let gltf =
            gltf_with_identity_buffer_views(&buffers.iter().map(|b| b.len()).collect::<Vec<_>>());
        let table = parse_property_table(&gltf, &table_obj, &schema, &buffers).unwrap();

        // noData rows decode to Null internally...
        assert_eq!(
            table.properties["height"].values[0],
            AttributeValue::Number(10u64.into())
        );
        assert_eq!(table.properties["height"].values[1], AttributeValue::Null);
        assert_eq!(
            table.properties["name"].values[0],
            AttributeValue::String("a".into())
        );
        assert_eq!(table.properties["name"].values[1], AttributeValue::Null);

        // ...and feature_properties skips them entirely rather than
        // surfacing the noData sentinel as a real attribute value.
        let tables = PropertyTables {
            schema,
            tables: vec![table],
        };
        let f0 = feature_properties(&tables, 0, 0);
        assert_eq!(
            f0.get("height"),
            Some(&AttributeValue::Number(10u64.into()))
        );
        assert_eq!(f0.get("name"), Some(&AttributeValue::String("a".into())));

        let f1 = feature_properties(&tables, 0, 1);
        assert_eq!(f1.get("height"), None, "noData height must be omitted");
        assert_eq!(f1.get("name"), None, "noData name must be omitted");
    }

    #[test]
    fn non_scalar_and_array_numeric_properties_are_not_mis_decoded() {
        // A VEC3 property and an array-flagged SCALAR property both carry a
        // `componentType`, exactly like a plain SCALAR does. Decoding them
        // with the flat-scalar reader would read the wrong stride and
        // produce garbage; they must be left as an empty column instead.
        let schema = serde_json::json!({
            "classes": {
                "Feature": {
                    "properties": {
                        "position": {"type": "VEC3", "componentType": "FLOAT32"},
                        "tags": {"type": "SCALAR", "componentType": "UINT8", "array": true, "count": 3}
                    }
                }
            }
        });
        let table_obj_value = serde_json::json!({
            "class": "Feature",
            "count": 1,
            "properties": {
                "position": {"values": 0},
                "tags": {"values": 1}
            }
        });
        let Value::Object(table_obj) = table_obj_value else {
            unreachable!()
        };

        // buffer 0: one VEC3<FLOAT32> = (1.0, 2.0, 3.0), 12 bytes.
        let position_buffer: Vec<u8> = [1.0f32, 2.0f32, 3.0f32]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        // buffer 1: one 3-element UINT8 array = [7, 8, 9].
        let tags_buffer: Vec<u8> = vec![7, 8, 9];
        let buffers = vec![position_buffer, tags_buffer];

        let gltf =
            gltf_with_identity_buffer_views(&buffers.iter().map(|b| b.len()).collect::<Vec<_>>());
        let table = parse_property_table(&gltf, &table_obj, &schema, &buffers).unwrap();

        assert!(
            table.properties["position"].values.is_empty(),
            "VEC3 must not be decoded as flat scalars"
        );
        assert!(
            table.properties["tags"].values.is_empty(),
            "array-flagged SCALAR must not be decoded as flat scalars"
        );

        // feature_properties over such a table simply has nothing to return
        // for these columns rather than surfacing corrupt values.
        let tables = PropertyTables {
            schema,
            tables: vec![table],
        };
        let f0 = feature_properties(&tables, 0, 0);
        assert_eq!(f0.get("position"), None);
        assert_eq!(f0.get("tags"), None);
    }

    /// Test-only fixtures for constructing minimal, hand-verified
    /// `EXT_structural_metadata` glTF documents without going through a real
    /// glTF exporter.
    mod fixtures {
        /// Builds a minimal glTF document carrying `EXT_structural_metadata`
        /// with two properties over two features:
        /// - "height": UINT32 = [10, 20]
        /// - "name": STRING = ["a", "b"]
        ///
        /// Declares one `bufferView` per buffer (an identity mapping, view `i`
        /// = the whole of buffer `i`), matching the `"values"`/`"stringOffsets"`
        /// indices below, so `read_structural_metadata`'s bufferView
        /// resolution has real views to resolve. Returns the parsed
        /// `gltf::Gltf` plus the `buffer_data` it expects (buffers are
        /// resolved separately; the JSON's own `buffers` entries only carry a
        /// `byteLength` since validation is skipped).
        pub fn metadata_glb_two_features() -> (gltf::Gltf, Vec<Vec<u8>>) {
            // buffer 0: "height" UINT32 values, little-endian: 10, 20
            let height_buffer: Vec<u8> = [10u32, 20u32]
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect();
            // buffer 1: "name" STRING raw UTF-8 bytes, concatenated: "a" + "b"
            let name_values_buffer: Vec<u8> = b"ab".to_vec();
            // buffer 2: "name" stringOffsets, UINT32 little-endian cumulative
            // byte offsets into buffer 1 (count + 1 entries): [0, 1, 2]
            let name_offsets_buffer: Vec<u8> = [0u32, 1u32, 2u32]
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect();
            let buffer_lens = [
                height_buffer.len(),
                name_values_buffer.len(),
                name_offsets_buffer.len(),
            ];

            let json = serde_json::json!({
                "asset": {"version": "2.0"},
                "extensionsUsed": ["EXT_structural_metadata"],
                "buffers": buffer_lens.iter().map(|len| serde_json::json!({"byteLength": len})).collect::<Vec<_>>(),
                "bufferViews": buffer_lens.iter().enumerate().map(|(i, len)| {
                    serde_json::json!({"buffer": i, "byteOffset": 0, "byteLength": len})
                }).collect::<Vec<_>>(),
                "extensions": {
                    "EXT_structural_metadata": {
                        "schema": {
                            "id": "TestSchema",
                            "classes": {
                                "Feature": {
                                    "properties": {
                                        "height": {"type": "SCALAR", "componentType": "UINT32"},
                                        "name": {"type": "STRING"}
                                    }
                                }
                            }
                        },
                        "propertyTables": [
                            {
                                "class": "Feature",
                                "count": 2,
                                "properties": {
                                    "height": {"values": 0},
                                    "name": {"values": 1, "stringOffsets": 2}
                                }
                            }
                        ]
                    }
                }
            });
            let json_bytes = serde_json::to_vec(&json).expect("fixture JSON is serializable");
            let gltf = gltf::Gltf::from_slice_without_validation(&json_bytes)
                .expect("fixture glTF JSON should parse");

            (
                gltf,
                vec![height_buffer, name_values_buffer, name_offsets_buffer],
            )
        }
    }
}
