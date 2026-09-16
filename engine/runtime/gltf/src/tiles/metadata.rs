//! Per-tile `EXT_structural_metadata` property table: one class per glb,
//! named after its features' CityGML feature type (every feature in a glb
//! shares one type — see `builder.rs`'s per-type content splitting), its
//! properties the union of every attribute path present across that glb's
//! features. Also the only place that knows the
//! `EXT_structural_metadata`/`EXT_mesh_features` JSON shapes, via [`encode`],
//! which attaches them to a `glb::Builder` directly.

use std::collections::BTreeMap;

use gltf::json;
use indexmap::IndexMap;
use nusamai_citygml::schema::{Map as SchemaMap, TypeRef};
use reearth_flow_types::{AttributeValue, Feature};
use serde::Serialize;

use super::glb::{Builder, PrimitiveHandle};
use crate::metadata::int_type_selector::{SignedIntCollector, UnsignedIntCollector};
use crate::{FLOAT_NO_DATA, STRING_NO_DATA};

const METADATA_SCHEMA_ID: &str = "Schema";
const DEFAULT_CLASS_NAME: &str = "Feature";

/// glTF property table class names must match `^[a-zA-Z_][a-zA-Z0-9_]*$`; a
/// CityGML feature type name such as `bldg:Building` isn't one, so sanitize
/// it into a valid identifier rather than dropping the per-type class name.
pub fn sanitize_class_name(feature_type: &str) -> String {
    feature_type.replace(':', "_")
}

/// Shared by every class; these exclusions reuse the parent writer's params.
#[derive(Debug, Clone, Copy, Default)]
pub struct MetadataOptions<'a> {
    pub schema_key: Option<&'a str>,
    pub skip_unexposed_attributes: bool,
    pub array_map_separator: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ColumnKind {
    SignedInt,
    UnsignedInt,
    Float64,
    String,
}

impl From<&TypeRef> for ColumnKind {
    fn from(type_ref: &TypeRef) -> Self {
        match type_ref {
            TypeRef::Integer => ColumnKind::SignedInt,
            TypeRef::NonNegativeInteger => ColumnKind::UnsignedInt,
            TypeRef::Double | TypeRef::Measure => ColumnKind::Float64,
            TypeRef::Boolean => ColumnKind::SignedInt,
            _ => ColumnKind::String,
        }
    }
}

/// `properties[i] = (raw attribute path, raw attribute path, column type)`;
/// `rows[feature][i]` is that feature's value for column `i` (`None` if the
/// feature doesn't carry that path — encoded as the column's no-data
/// sentinel).
pub struct PropertyTable {
    properties: Vec<(String, String, ColumnKind)>,
    rows: Vec<Vec<Option<AttributeValue>>>,
}

pub fn build_table(
    features: &[&Feature],
    schema_attrs: Option<&SchemaMap>,
    options: MetadataOptions,
) -> PropertyTable {
    let flattened: Vec<BTreeMap<String, AttributeValue>> = features
        .iter()
        .map(|feature| flatten_attributes(feature, options))
        .collect();

    let Some(schema_attrs) = schema_attrs else {
        return PropertyTable {
            properties: Vec::new(),
            rows: flattened.iter().map(|_| Vec::new()).collect(),
        };
    };

    // Property table keys are the schema-declared attribute name, unsanitized;
    // an attribute no feature actually carries is dropped rather than encoded
    // as an all-no-data column.
    let properties: Vec<(String, String, ColumnKind)> = schema_attrs
        .iter()
        .filter(|(name, _)| flattened.iter().any(|f| f.contains_key(name.as_str())))
        .map(|(name, attr)| (name.clone(), name.clone(), ColumnKind::from(&attr.type_ref)))
        .collect();

    let rows = flattened
        .iter()
        .map(|f| {
            properties
                .iter()
                .map(|(raw, _, _)| f.get(raw).cloned())
                .collect()
        })
        .collect();

    PropertyTable { properties, rows }
}

fn as_numeric(value: &AttributeValue) -> Option<f64> {
    match value {
        AttributeValue::Number(n) => n.as_f64(),
        AttributeValue::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    }
}

/// Attach `table` to `builder` as one `EXT_structural_metadata` property table
/// (built once) plus, on each `(primitive, feature_ids)` in `primitives`, an
/// `EXT_mesh_features` feature-ID attribute tagging each of that primitive's
/// vertices with `feature_ids[original_vertex]`. All primitives share the one
/// property table (reference `propertyTable` 0), but each carries its own
/// per-vertex `feature_ids` (their vertex buffers are independent). No-op if
/// `table` has no properties.
pub fn encode(
    table: &PropertyTable,
    class_name: &str,
    builder: &mut Builder,
    primitives: &[(PrimitiveHandle, &[u32])],
) {
    if table.properties.is_empty() {
        return;
    }
    let class_name = if class_name.is_empty() {
        DEFAULT_CLASS_NAME.to_string()
    } else {
        class_name.to_string()
    };

    let mut class_properties = IndexMap::new();
    let mut table_properties = IndexMap::new();
    for (col, (raw_name, id, kind)) in table.properties.iter().enumerate() {
        let values = table.rows.iter().map(|row| row[col].as_ref());
        let (class_property, table_property) = match kind {
            ColumnKind::String => encode_string_column(raw_name, values, builder),
            ColumnKind::Float64 => encode_float_column(raw_name, values, builder),
            ColumnKind::SignedInt => encode_signed_column(raw_name, values, builder),
            ColumnKind::UnsignedInt => encode_unsigned_column(raw_name, values, builder),
        };
        class_properties.insert(id.clone(), class_property);
        table_properties.insert(id.clone(), table_property);
    }

    let mut classes = IndexMap::new();
    classes.insert(
        class_name.clone(),
        MetadataClass {
            properties: class_properties,
        },
    );
    let ext_structural_metadata = ExtStructuralMetadata {
        schema: MetadataSchema {
            id: METADATA_SCHEMA_ID,
            classes,
        },
        property_tables: vec![MetadataPropertyTable {
            class: class_name,
            count: table.rows.len(),
            properties: table_properties,
        }],
    };
    builder.extend(
        Builder::ROOT,
        "EXT_structural_metadata",
        serde_json::to_value(&ext_structural_metadata)
            .expect("EXT_structural_metadata is always serializable"),
    );

    for &(primitive, feature_ids) in primitives {
        builder.extend(
            primitive,
            "EXT_mesh_features",
            serde_json::to_value(&ExtMeshFeatures {
                feature_ids: vec![FeatureId {
                    feature_count: table.rows.len(),
                    attribute: 0,
                    property_table: 0,
                }],
            })
            .expect("EXT_mesh_features is always serializable"),
        );

        // `Semantic::Extras`'s inner name excludes the glTF-spec-mandated
        // leading underscore; the crate adds it on (de)serialization.
        builder.set_attribute(
            primitive,
            json::mesh::Semantic::Extras("FEATURE_ID_0".to_string()),
            feature_ids,
        );
    }
}

fn encode_string_column<'a>(
    raw_name: &str,
    values: impl Iterator<Item = Option<&'a AttributeValue>>,
    builder: &mut Builder,
) -> (ClassProperty, MetadataPropertyTableProperty) {
    let mut value_bytes = Vec::new();
    let mut offsets: Vec<u32> = vec![0];
    for value in values {
        let s = value
            .map(|v| v.to_string())
            .unwrap_or_else(|| STRING_NO_DATA.to_string());
        value_bytes.extend_from_slice(s.as_bytes());
        offsets.push(value_bytes.len() as u32);
    }
    let values_bufferview = builder.push_buffer_view(&value_bytes);
    let offset_bytes: Vec<u8> = offsets.iter().flat_map(|o| o.to_le_bytes()).collect();
    let offsets_bufferview = builder.push_buffer_view(&offset_bytes);

    (
        ClassProperty {
            name: raw_name.to_string(),
            type_: "STRING",
            component_type: None,
            no_data: serde_json::json!(STRING_NO_DATA),
        },
        MetadataPropertyTableProperty {
            values: values_bufferview,
            string_offset_type: Some("UINT32"),
            string_offsets: Some(offsets_bufferview),
        },
    )
}

fn encode_float_column<'a>(
    raw_name: &str,
    values: impl Iterator<Item = Option<&'a AttributeValue>>,
    builder: &mut Builder,
) -> (ClassProperty, MetadataPropertyTableProperty) {
    let mut value_bytes = Vec::new();
    for value in values {
        let v = value.and_then(as_numeric).unwrap_or(FLOAT_NO_DATA);
        value_bytes.extend_from_slice(&v.to_le_bytes());
    }
    let values_bufferview = builder.push_buffer_view(&value_bytes);

    (
        ClassProperty {
            name: raw_name.to_string(),
            type_: "SCALAR",
            component_type: Some("FLOAT64"),
            no_data: serde_json::json!(FLOAT_NO_DATA),
        },
        MetadataPropertyTableProperty {
            values: values_bufferview,
            string_offset_type: None,
            string_offsets: None,
        },
    )
}

fn encode_signed_column<'a>(
    raw_name: &str,
    values: impl Iterator<Item = Option<&'a AttributeValue>>,
    builder: &mut Builder,
) -> (ClassProperty, MetadataPropertyTableProperty) {
    let mut collector = SignedIntCollector::new();
    for value in values {
        match value.and_then(as_numeric) {
            Some(n) => collector.push(n as i64),
            None => collector.push_no_data(),
        }
    }
    let finalized = collector.finalize();
    let mut value_bytes = Vec::new();
    finalized.encode_all(&mut value_bytes);
    let values_bufferview = builder.push_buffer_view(&value_bytes);

    (
        ClassProperty {
            name: raw_name.to_string(),
            type_: "SCALAR",
            component_type: Some(int_component_type(finalized.byte_size(), true)),
            no_data: finalized.no_data_json(),
        },
        MetadataPropertyTableProperty {
            values: values_bufferview,
            string_offset_type: None,
            string_offsets: None,
        },
    )
}

fn encode_unsigned_column<'a>(
    raw_name: &str,
    values: impl Iterator<Item = Option<&'a AttributeValue>>,
    builder: &mut Builder,
) -> (ClassProperty, MetadataPropertyTableProperty) {
    let mut collector = UnsignedIntCollector::new();
    for value in values {
        match value.and_then(as_numeric) {
            Some(n) => collector.push(n as u64),
            None => collector.push_no_data(),
        }
    }
    let finalized = collector.finalize();
    let mut value_bytes = Vec::new();
    finalized.encode_all(&mut value_bytes);
    let values_bufferview = builder.push_buffer_view(&value_bytes);

    (
        ClassProperty {
            name: raw_name.to_string(),
            type_: "SCALAR",
            component_type: Some(int_component_type(finalized.byte_size(), false)),
            no_data: finalized.no_data_json(),
        },
        MetadataPropertyTableProperty {
            values: values_bufferview,
            string_offset_type: None,
            string_offsets: None,
        },
    )
}

fn int_component_type(byte_size: usize, signed: bool) -> &'static str {
    match (byte_size, signed) {
        (1, true) => "INT8",
        (1, false) => "UINT8",
        (2, true) => "INT16",
        (2, false) => "UINT16",
        (4, true) => "INT32",
        (4, false) => "UINT32",
        (8, true) => "INT64",
        (8, false) => "UINT64",
        _ => unreachable!("int_type_selector only produces 1/2/4/8-byte widths"),
    }
}

#[derive(Serialize)]
struct ExtMeshFeatures {
    #[serde(rename = "featureIds")]
    feature_ids: Vec<FeatureId>,
}

#[derive(Serialize)]
struct FeatureId {
    #[serde(rename = "featureCount")]
    feature_count: usize,
    attribute: u32,
    #[serde(rename = "propertyTable")]
    property_table: u32,
}

#[derive(Serialize)]
struct ExtStructuralMetadata {
    schema: MetadataSchema,
    #[serde(rename = "propertyTables")]
    property_tables: Vec<MetadataPropertyTable>,
}

#[derive(Serialize)]
struct MetadataSchema {
    id: &'static str,
    classes: IndexMap<String, MetadataClass>,
}

#[derive(Serialize)]
struct MetadataClass {
    properties: IndexMap<String, ClassProperty>,
}

#[derive(Serialize)]
struct ClassProperty {
    name: String,
    #[serde(rename = "type")]
    type_: &'static str,
    #[serde(rename = "componentType", skip_serializing_if = "Option::is_none")]
    component_type: Option<&'static str>,
    #[serde(rename = "noData")]
    no_data: serde_json::Value,
}

#[derive(Serialize)]
struct MetadataPropertyTable {
    class: String,
    count: usize,
    properties: IndexMap<String, MetadataPropertyTableProperty>,
}

#[derive(Serialize)]
struct MetadataPropertyTableProperty {
    values: usize,
    #[serde(rename = "stringOffsetType", skip_serializing_if = "Option::is_none")]
    string_offset_type: Option<&'static str>,
    #[serde(rename = "stringOffsets", skip_serializing_if = "Option::is_none")]
    string_offsets: Option<usize>,
}

pub fn flatten_attributes(
    feature: &Feature,
    options: MetadataOptions,
) -> BTreeMap<String, AttributeValue> {
    let mut out = BTreeMap::new();
    for (key, value) in feature.attributes.iter() {
        let key = key.inner();
        if is_excluded(&key, options) {
            continue;
        }
        match options.array_map_separator {
            Some(sep) => flatten(key, value, sep, &mut out),
            None => {
                if !matches!(value, AttributeValue::Map(_) | AttributeValue::Array(_)) {
                    insert_leaf(key, value, &mut out);
                }
            }
        }
    }
    out
}

fn flatten(
    path: String,
    value: &AttributeValue,
    sep: &str,
    out: &mut BTreeMap<String, AttributeValue>,
) {
    match value {
        AttributeValue::Map(map) => {
            for (key, child) in map {
                flatten(format!("{path}{sep}{key}"), child, sep, out);
            }
        }
        AttributeValue::Array(items) => {
            for (i, child) in items.iter().enumerate() {
                flatten(format!("{path}{sep}{i}"), child, sep, out);
            }
        }
        leaf => insert_leaf(path, leaf, out),
    }
}

fn insert_leaf(path: String, leaf: &AttributeValue, out: &mut BTreeMap<String, AttributeValue>) {
    if out.insert(path.clone(), leaf.clone()).is_some() {
        tracing::warn!("Cesium3DTilesWriter: attribute path {path:?} collided; overwriting");
    }
}

fn is_excluded(key: &str, options: MetadataOptions) -> bool {
    (options.skip_unexposed_attributes && key.starts_with("__")) || options.schema_key == Some(key)
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;
    use nusamai_citygml::schema::Attribute as SchemaAttribute;

    use super::*;

    fn schema_map(entries: Vec<(&str, TypeRef)>) -> SchemaMap {
        let mut map = SchemaMap::default();
        for (name, type_ref) in entries {
            map.insert(
                name.to_string(),
                SchemaAttribute {
                    type_ref,
                    ..Default::default()
                },
            );
        }
        map
    }

    fn feature_with_nested() -> Feature {
        let mut attrs: IndexMap<String, AttributeValue> = IndexMap::new();
        attrs.insert("name".to_string(), AttributeValue::String("A".to_string()));
        attrs.insert(
            "addr".to_string(),
            AttributeValue::Map(
                [("city".to_string(), AttributeValue::String("X".to_string()))]
                    .into_iter()
                    .collect(),
            ),
        );
        attrs.insert(
            "heights".to_string(),
            AttributeValue::Array(vec![AttributeValue::String("1".to_string())]),
        );
        Feature::from(attrs)
    }

    fn raw_paths(table: &PropertyTable) -> Vec<&str> {
        table
            .properties
            .iter()
            .map(|(raw, _, _)| raw.as_str())
            .collect()
    }

    #[test]
    fn no_schema_entry_produces_no_properties() {
        let feature = feature_with_nested();
        let table = build_table(&[&feature], None, MetadataOptions::default());
        assert!(raw_paths(&table).is_empty());
    }

    #[test]
    fn none_separator_drops_maps_and_arrays() {
        let feature = feature_with_nested();
        let options = MetadataOptions {
            array_map_separator: None,
            ..Default::default()
        };
        let schema = schema_map(vec![("name", TypeRef::String)]);
        let table = build_table(&[&feature], Some(&schema), options);

        // Only the top-level scalar survives; the map and array contribute
        // no columns at all.
        assert_eq!(raw_paths(&table), vec!["name"]);
    }

    #[test]
    fn separator_flattens_nested_paths() {
        let feature = feature_with_nested();
        let options = MetadataOptions {
            array_map_separator: Some("."),
            ..Default::default()
        };
        let schema = schema_map(vec![
            ("name", TypeRef::String),
            ("addr.city", TypeRef::String),
            ("heights.0", TypeRef::String),
        ]);
        let table = build_table(&[&feature], Some(&schema), options);

        assert_eq!(raw_paths(&table), vec!["name", "addr.city", "heights.0"]);
    }

    fn number(n: f64) -> AttributeValue {
        AttributeValue::Number(serde_json::Number::from_f64(n).unwrap())
    }

    fn int_number(n: i64) -> AttributeValue {
        AttributeValue::Number(serde_json::Number::from(n))
    }

    #[test]
    fn typed_columns_round_trip_through_decode() {
        let feature1 = Feature::from(IndexMap::from([
            ("height".to_string(), number(11.4)),
            ("count".to_string(), int_number(3)),
            ("elevation_delta".to_string(), int_number(-12)),
            ("flag".to_string(), AttributeValue::Bool(true)),
            ("name".to_string(), AttributeValue::String("x".to_string())),
        ]));
        let feature2 = Feature::from(IndexMap::from([(
            "name".to_string(),
            AttributeValue::String("y".to_string()),
        )]));

        let schema = schema_map(vec![
            ("height", TypeRef::Double),
            ("count", TypeRef::Integer),
            ("elevation_delta", TypeRef::Integer),
            ("flag", TypeRef::Boolean),
            ("name", TypeRef::String),
        ]);
        let table = build_table(&[&feature1, &feature2], Some(&schema), MetadataOptions::default());
        let mut builder = Builder::new();
        encode(&table, "Feature", &mut builder, &[]);
        let glb = builder.build([0.0, 0.0, 0.0]);

        let gltf = crate::parse_gltf(&bytes::Bytes::from(glb)).unwrap();
        let features = crate::extract_feature_properties(&gltf).unwrap();

        assert_eq!(features[0].get("height"), Some(&serde_json::json!(11.4)));
        assert_eq!(features[0].get("count"), Some(&serde_json::json!(3)));
        assert_eq!(
            features[0].get("elevation_delta"),
            Some(&serde_json::json!(-12))
        );
        assert_eq!(features[0].get("flag"), Some(&serde_json::json!(1)));
        assert_eq!(
            features[0].get("name"),
            Some(&serde_json::Value::String("x".to_string()))
        );

        // feature2 carries none of height/count/flag — they must decode as
        // absent (no-data), not as zero/empty-string.
        assert_eq!(features[1].get("height"), None);
        assert_eq!(features[1].get("count"), None);
        assert_eq!(features[1].get("elevation_delta"), None);
        assert_eq!(features[1].get("flag"), None);
        assert_eq!(
            features[1].get("name"),
            Some(&serde_json::Value::String("y".to_string()))
        );
    }
}
