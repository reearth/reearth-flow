//! Per-tile `EXT_structural_metadata` property table: one implicit `Feature`
//! class per glb, its properties the union of every attribute path present
//! across that glb's features. Also the only place that knows the
//! `EXT_structural_metadata`/`EXT_mesh_features` JSON shapes, via [`encode`],
//! which attaches them to a `glb::Builder` directly.

use std::collections::{BTreeMap, BTreeSet};

use indexmap::IndexMap;
use reearth_flow_types::{AttributeValue, Feature};
use serde::Serialize;

use super::glb::{Builder, PrimitiveHandle};
use crate::metadata::int_type_selector::SignedIntCollector;
use crate::{FLOAT_NO_DATA, STRING_NO_DATA};

const METADATA_SCHEMA_ID: &str = "Schema";
const METADATA_CLASS_NAME: &str = "Feature";

/// No per-feature-type classing yet (single inlined `Feature` class), but
/// these two exclusions still apply, reusing the parent writer's params.
#[derive(Debug, Clone, Copy, Default)]
pub struct MetadataOptions<'a> {
    pub schema_key: Option<&'a str>,
    pub skip_unexposed_attributes: bool,
    pub array_map_separator: Option<&'a str>,
}

// Decided per column from the values actually present (no schema here); `Bool` counts as numeric.
// Variants are declared in widening order so a column's kind is the `max` over its values'
// kinds: int -> float -> string, each able to represent everything below it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ColumnKind {
    Int,
    Float64,
    String,
}

/// `properties[i] = (raw attribute path, raw attribute path, column type)`;
/// `rows[feature][i]` is that feature's value for column `i` (`None` if the
/// feature doesn't carry that path — encoded as the column's no-data
/// sentinel).
pub struct PropertyTable {
    properties: Vec<(String, String, ColumnKind)>,
    rows: Vec<Vec<Option<AttributeValue>>>,
}

pub fn build_table(features: &[&Feature], options: MetadataOptions) -> PropertyTable {
    let flattened: Vec<BTreeMap<String, AttributeValue>> = features
        .iter()
        .map(|feature| flatten_attributes(feature, options))
        .collect();

    let mut raw_paths = BTreeSet::new();
    for f in &flattened {
        raw_paths.extend(f.keys().cloned());
    }

    // Property table keys are the raw attribute path, unsanitized.
    let properties: Vec<(String, String, ColumnKind)> = raw_paths
        .into_iter()
        .map(|raw| {
            let kind = column_kind(&flattened, &raw);
            (raw.clone(), raw, kind)
        })
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

fn as_int(value: &AttributeValue) -> Option<i64> {
    value.as_i64().or_else(|| value.as_bool().map(i64::from))
}

fn as_float(value: &AttributeValue) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_bool().map(|b| if b { 1.0 } else { 0.0 }))
}

/// The narrowest kind that can hold `value` on its own. `Bool` rides along as
/// 0/1; anything nonnumeric only fits as a string.
fn value_kind(value: &AttributeValue) -> ColumnKind {
    match value {
        AttributeValue::Number(n) if n.is_i64() => ColumnKind::Int,
        AttributeValue::Number(_) => ColumnKind::Float64,
        AttributeValue::Bool(_) => ColumnKind::Int,
        _ => ColumnKind::String,
    }
}

/// Widen across every value in the column, so a single nonnumeric value forces
/// `String` no matter where it sits and a single float forces `Float64` over
/// its integer neighbours. A column with no values at all encodes as `String`.
fn column_kind(flattened: &[BTreeMap<String, AttributeValue>], path: &str) -> ColumnKind {
    flattened
        .iter()
        .filter_map(|f| f.get(path))
        .map(value_kind)
        .max()
        .unwrap_or(ColumnKind::String)
}

/// Attach `table` to `builder` as one `EXT_structural_metadata` property table
/// (built once) plus, on each primitive, an `EXT_mesh_features` declaration
/// reading the `FEATURE_ID_0` attribute the caller pushed with it. All
/// primitives share the one property table (reference `propertyTable` 0).
/// No-op if `table` has no properties.
pub fn encode(table: &PropertyTable, builder: &mut Builder, primitives: &[PrimitiveHandle]) {
    if table.properties.is_empty() {
        return;
    }

    let mut class_properties = IndexMap::new();
    let mut table_properties = IndexMap::new();
    for (col, (raw_name, id, kind)) in table.properties.iter().enumerate() {
        let values = table.rows.iter().map(|row| row[col].as_ref());
        let (class_property, table_property) = match kind {
            ColumnKind::String => encode_string_column(raw_name, values, builder),
            ColumnKind::Float64 => encode_float_column(raw_name, values, builder),
            ColumnKind::Int => encode_int_column(raw_name, values, builder),
        };
        class_properties.insert(id.clone(), class_property);
        table_properties.insert(id.clone(), table_property);
    }

    let mut classes = IndexMap::new();
    classes.insert(
        METADATA_CLASS_NAME,
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
            class: METADATA_CLASS_NAME,
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

    for &primitive in primitives {
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
        let v = value.and_then(as_float).unwrap_or(FLOAT_NO_DATA);
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

fn encode_int_column<'a>(
    raw_name: &str,
    values: impl Iterator<Item = Option<&'a AttributeValue>>,
    builder: &mut Builder,
) -> (ClassProperty, MetadataPropertyTableProperty) {
    let mut collector = SignedIntCollector::new();
    for value in values {
        match value.and_then(as_int) {
            Some(n) => collector.push(n),
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
            component_type: Some(int_component_type(finalized.byte_size())),
            no_data: finalized.no_data_json(),
        },
        MetadataPropertyTableProperty {
            values: values_bufferview,
            string_offset_type: None,
            string_offsets: None,
        },
    )
}

fn int_component_type(byte_size: usize) -> &'static str {
    match byte_size {
        1 => "INT8",
        2 => "INT16",
        4 => "INT32",
        8 => "INT64",
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
    classes: IndexMap<&'static str, MetadataClass>,
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
    class: &'static str,
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

    use super::*;
    use reearth_flow_types::Attribute;

    fn feature_with_nested() -> Feature {
        let mut attrs: IndexMap<String, AttributeValue> = IndexMap::new();
        attrs.insert("name".to_string(), AttributeValue::String("A".to_string()));
        attrs.insert(
            "addr".to_string(),
            AttributeValue::Map(
                [(
                    Attribute::new("city"),
                    AttributeValue::String("X".to_string()),
                )]
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
    fn none_separator_drops_maps_and_arrays() {
        let feature = feature_with_nested();
        let options = MetadataOptions {
            array_map_separator: None,
            ..Default::default()
        };
        let table = build_table(&[&feature], options);

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
        let table = build_table(&[&feature], options);

        assert_eq!(raw_paths(&table), vec!["addr.city", "heights.0", "name"]);
    }

    fn number(n: f64) -> AttributeValue {
        AttributeValue::Number(serde_json::Number::from_f64(n).unwrap())
    }

    fn int_number(n: i64) -> AttributeValue {
        AttributeValue::Number(serde_json::Number::from(n))
    }

    #[test]
    fn column_kind_picks_narrowest_matching_type() {
        let all_nonneg_ints = [BTreeMap::from([("k".to_string(), int_number(3))])];
        assert_eq!(column_kind(&all_nonneg_ints, "k"), ColumnKind::Int);

        let has_negative = [BTreeMap::from([("k".to_string(), int_number(-3))])];
        assert_eq!(column_kind(&has_negative, "k"), ColumnKind::Int);

        let is_f64_typed_even_though_whole = [BTreeMap::from([("k".to_string(), number(3.0))])];
        assert_eq!(
            column_kind(&is_f64_typed_even_though_whole, "k"),
            ColumnKind::Float64
        );

        let has_fraction = [BTreeMap::from([("k".to_string(), number(1.5))])];
        assert_eq!(column_kind(&has_fraction, "k"), ColumnKind::Float64);

        let bool_only = [BTreeMap::from([(
            "k".to_string(),
            AttributeValue::Bool(true),
        )])];
        assert_eq!(column_kind(&bool_only, "k"), ColumnKind::Int);

        let has_string = [BTreeMap::from([
            ("k".to_string(), int_number(1)),
            ("k2".to_string(), AttributeValue::String("s".to_string())),
        ])];
        assert_eq!(column_kind(&has_string, "k2"), ColumnKind::String);
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

        let table = build_table(&[&feature1, &feature2], MetadataOptions::default());
        let mut builder = Builder::new();
        encode(&table, &mut builder, &[]);
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
