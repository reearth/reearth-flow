//! Per-tile `EXT_structural_metadata` property table: one implicit `Feature`
//! class per glb, its properties the union of every attribute path present
//! across that glb's features. Also the only place that knows the
//! `EXT_structural_metadata`/`EXT_mesh_features` JSON shapes, via [`encode`],
//! which attaches them to a `glb::Builder` directly.

use std::collections::BTreeMap;

use indexmap::{IndexMap, IndexSet};
use nusamai_citygml::schema::{Map as SchemaMap, TypeRef};
use reearth_flow_types::{AttributeValue, Feature};
use serde::Serialize;

use super::glb::{Builder, PrimitiveHandle};
use crate::metadata::int_type_selector::SignedIntCollector;
use crate::{FLOAT_NO_DATA, STRING_NO_DATA};

const METADATA_SCHEMA_ID: &str = "Schema";
const METADATA_CLASS_NAME: &str = "Feature";

/// No per-feature-type classing yet (single inlined `Feature` class), but
/// this exclusion still applies, reusing the parent writer's param.
#[derive(Debug, Clone, Copy, Default)]
pub struct MetadataOptions {
    pub skip_unexposed_attributes: bool,
}

// Variants are declared in widening order so a column's kind is the `max` over its values'
// kinds: int -> float -> string, each able to represent everything below it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ColumnKind {
    Int,
    Float64,
    String,
}

impl From<&TypeRef> for ColumnKind {
    fn from(type_ref: &TypeRef) -> Self {
        match type_ref {
            TypeRef::Integer => ColumnKind::Int,
            TypeRef::NonNegativeInteger => ColumnKind::Int,
            TypeRef::Double | TypeRef::Measure => ColumnKind::Float64,
            TypeRef::Boolean => ColumnKind::Int,
            _ => ColumnKind::String,
        }
    }
}

/// A column's kind and the `(min, max)` of the numbers encoded into it
/// (`None` if none were).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColumnStats {
    Int(Option<(i64, i64)>),
    Float64(Option<(f64, f64)>),
    String,
}

impl ColumnStats {
    pub fn empty(kind: ColumnKind) -> Self {
        match kind {
            ColumnKind::Int => ColumnStats::Int(None),
            ColumnKind::Float64 => ColumnStats::Float64(None),
            ColumnKind::String => ColumnStats::String,
        }
    }

    /// Widen to the more general kind of the two, with the union of their ranges.
    pub fn merge(self, other: ColumnStats) -> ColumnStats {
        match (self, other) {
            (ColumnStats::String, _) | (_, ColumnStats::String) => ColumnStats::String,
            (ColumnStats::Int(a), ColumnStats::Int(b)) => ColumnStats::Int(union(a, b)),
            (a, b) => ColumnStats::Float64(union(a.float_range(), b.float_range())),
        }
    }

    fn float_range(self) -> Option<(f64, f64)> {
        match self {
            ColumnStats::Int(range) => range.map(|(min, max)| (min as f64, max as f64)),
            ColumnStats::Float64(range) => range,
            ColumnStats::String => None,
        }
    }
}

fn union<T: PartialOrd + Copy>(a: Option<(T, T)>, b: Option<(T, T)>) -> Option<(T, T)> {
    match (a, b) {
        (Some((a_min, a_max)), Some((b_min, b_max))) => Some((
            if b_min < a_min { b_min } else { a_min },
            if b_max > a_max { b_max } else { a_max },
        )),
        (range, None) | (None, range) => range,
    }
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

/// Flattened features and their declared schemas (parallel) -> column name to column kind.
fn column_kinds(
    flattened: &[BTreeMap<String, AttributeValue>],
    schemas: &[Option<&SchemaMap>],
) -> IndexMap<String, ColumnKind> {
    let mut kinds: IndexMap<String, ColumnKind> = IndexMap::new();
    let mut widen = |name: String, kind: ColumnKind| {
        kinds
            .entry(name)
            .and_modify(|held| *held = (*held).max(kind))
            .or_insert(kind);
    };

    for (flattened, schema_attrs) in flattened.iter().zip(schemas) {
        match schema_attrs {
            Some(schema_attrs) => {
                for (name, attr) in schema_attrs.iter() {
                    // Skip unused properties
                    if !flattened.contains_key(name) {
                        continue;
                    }
                    widen(name.clone(), ColumnKind::from(&attr.type_ref));
                }
            }
            None => {
                for (path, value) in flattened {
                    widen(path.clone(), value_kind(value));
                }
            }
        }
    }
    kinds
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
    schemas: &[Option<&SchemaMap>],
    options: MetadataOptions,
) -> PropertyTable {
    let flattened: Vec<BTreeMap<String, AttributeValue>> = features
        .iter()
        .map(|feature| flatten_attributes(feature, options))
        .collect();

    // Property table keys are the attribute name, unsanitized.
    let properties: Vec<(String, String, ColumnKind)> = column_kinds(&flattened, schemas)
        .into_iter()
        .map(|(name, kind)| (name.clone(), name, kind))
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
    match value {
        AttributeValue::String(s) => s.parse().ok(),
        _ => value.as_i64().or_else(|| value.as_bool().map(i64::from)),
    }
}

fn as_float(value: &AttributeValue) -> Option<f64> {
    match value {
        AttributeValue::String(s) => s.parse().ok(),
        _ => value
            .as_f64()
            .or_else(|| value.as_bool().map(|b| if b { 1.0 } else { 0.0 })),
    }
}

/// Attach `table` to `builder` as one `EXT_structural_metadata` property table
/// (built once) plus, on each primitive, an `EXT_mesh_features` declaration
/// reading the `FEATURE_ID_0` attribute the caller pushed with it. All
/// primitives share the one property table (reference `propertyTable` 0).
/// No-op if `table` has no properties. Returns each column's [`ColumnStats`].
pub fn encode(
    table: &PropertyTable,
    builder: &mut Builder,
    primitives: &[PrimitiveHandle],
) -> IndexMap<String, ColumnStats> {
    let mut stats = IndexMap::new();
    if table.properties.is_empty() {
        return stats;
    }

    let mut class_properties = IndexMap::new();
    let mut table_properties = IndexMap::new();
    let mut enums = IndexMap::new();
    for (col, (raw_name, id, kind)) in table.properties.iter().enumerate() {
        let values = table.rows.iter().map(|row| row[col].as_ref());
        let (class_property, table_property, column_stats) = match kind {
            ColumnKind::String => {
                let enum_id = format!("Enum{col}");
                let (class_property, table_property, enum_def) =
                    encode_string_column(raw_name, &enum_id, values, builder);
                if let Some(enum_def) = enum_def {
                    enums.insert(enum_id, enum_def);
                }
                (class_property, table_property, ColumnStats::String)
            }
            ColumnKind::Float64 => encode_float_column(raw_name, values, builder),
            ColumnKind::Int => encode_int_column(raw_name, values, builder),
        };
        class_properties.insert(id.clone(), class_property);
        table_properties.insert(id.clone(), table_property);
        stats.insert(raw_name.clone(), column_stats);
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
            enums,
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
    stats
}

/// Upper bound on an `ENUM` column's JSON and padding, excluding its enum
/// definition and `enum_id`.
const ENUM_OVERHEAD: usize = 39;

/// Upper bound on the JSON and padding cost of narrower-than-`UINT32` string
/// offsets (`UINT32` is the default, so its `stringOffsetType` is omitted).
const NARROW_OFFSETS_OVERHEAD: usize = 35;

/// Encodes as `STRING` or, when that certainly shrinks the glb, as an `ENUM`
/// whose definition is returned for the schema under `enum_id`.
fn encode_string_column<'a>(
    raw_name: &str,
    enum_id: &str,
    values: impl Iterator<Item = Option<&'a AttributeValue>>,
    builder: &mut Builder,
) -> (
    ClassProperty,
    MetadataPropertyTableProperty,
    Option<serde_json::Value>,
) {
    let strings: Vec<String> = values
        .map(|v| v.map_or_else(|| STRING_NO_DATA.to_string(), |v| v.to_string()))
        .collect();
    let byte_len: usize = strings.iter().map(String::len).sum();
    let offset_count = strings.len() + 1;
    let narrow_size = uint_size(byte_len);
    let offset_size = if (4 - narrow_size) * offset_count > NARROW_OFFSETS_OVERHEAD {
        narrow_size
    } else {
        4
    };

    // The no-data name takes value 0.
    let mut names = IndexSet::from([STRING_NO_DATA]);
    names.extend(strings.iter().map(String::as_str));
    let index_size = uint_size(names.len() - 1);
    let enum_def = EnumDef {
        value_type: uint_type(index_size),
        values: names
            .iter()
            .enumerate()
            .map(|(value, &name)| EnumValue { name, value })
            .collect(),
    };
    let enum_json_len = serde_json::to_vec(&enum_def)
        .expect("an enum definition is always serializable")
        .len();

    // Leaves out the `STRING` JSON an `ENUM` would drop, so `ENUM` only wins clearly.
    let string_cost = byte_len + offset_size * offset_count;
    let enum_cost = index_size * strings.len() + enum_json_len + 2 * enum_id.len() + ENUM_OVERHEAD;
    // Enum names need at least one character: https://github.com/CesiumGS/3d-tiles/blob/main/specification/schema/Schema/enum.value.schema.json
    if enum_cost < string_cost && !names.contains("") {
        let value_bytes: Vec<u8> = strings
            .iter()
            .flat_map(|s| {
                let index = names
                    .get_index_of(s.as_str())
                    .expect("every value is named");
                (index as u32).to_le_bytes().into_iter().take(index_size)
            })
            .collect();
        let values_bufferview = builder.push_buffer_view(&value_bytes, index_size);
        return (
            ClassProperty {
                name: raw_name.to_string(),
                type_: "ENUM",
                component_type: None,
                enum_type: Some(enum_id.to_string()),
                no_data: serde_json::json!(STRING_NO_DATA),
            },
            MetadataPropertyTableProperty {
                values: values_bufferview,
                string_offset_type: None,
                string_offsets: None,
            },
            Some(
                serde_json::to_value(&enum_def).expect("an enum definition is always serializable"),
            ),
        );
    }

    let value_bytes: Vec<u8> = strings.iter().flat_map(|s| s.bytes()).collect();
    let mut offset = 0u32;
    let offset_bytes: Vec<u8> = std::iter::once(0)
        .chain(strings.iter().map(|s| {
            offset += s.len() as u32;
            offset
        }))
        .flat_map(|o| o.to_le_bytes().into_iter().take(offset_size))
        .collect();
    let values_bufferview = builder.push_buffer_view(&value_bytes, 1);
    let offsets_bufferview = builder.push_buffer_view(&offset_bytes, offset_size);

    (
        ClassProperty {
            name: raw_name.to_string(),
            type_: "STRING",
            component_type: None,
            enum_type: None,
            no_data: serde_json::json!(STRING_NO_DATA),
        },
        MetadataPropertyTableProperty {
            values: values_bufferview,
            string_offset_type: (offset_size != 4).then(|| uint_type(offset_size)),
            string_offsets: Some(offsets_bufferview),
        },
        None,
    )
}

/// Byte size of the narrowest unsigned integer holding `max`.
fn uint_size(max: usize) -> usize {
    if max <= u8::MAX as usize {
        1
    } else if max <= u16::MAX as usize {
        2
    } else {
        4
    }
}

fn uint_type(size: usize) -> &'static str {
    match size {
        1 => "UINT8",
        2 => "UINT16",
        _ => "UINT32",
    }
}

fn encode_float_column<'a>(
    raw_name: &str,
    values: impl Iterator<Item = Option<&'a AttributeValue>>,
    builder: &mut Builder,
) -> (ClassProperty, MetadataPropertyTableProperty, ColumnStats) {
    let mut value_bytes = Vec::new();
    let mut range = None;
    for value in values {
        let v = match value {
            Some(value) => match as_float(value) {
                Some(v) => {
                    if !matches!(value, AttributeValue::Bool(_)) && v.is_finite() {
                        range = union(range, Some((v, v)));
                    }
                    v
                }
                None => {
                    tracing::error!(
                        "Cesium3DTilesWriter: {raw_name:?} is a FLOAT64 column but {value:?} is \
                         not convertible; encoding it as no-data"
                    );
                    FLOAT_NO_DATA
                }
            },
            None => FLOAT_NO_DATA,
        };
        value_bytes.extend_from_slice(&v.to_le_bytes());
    }
    let values_bufferview = builder.push_buffer_view(&value_bytes, 8);

    (
        ClassProperty {
            name: raw_name.to_string(),
            type_: "SCALAR",
            component_type: Some("FLOAT64"),
            enum_type: None,
            no_data: serde_json::json!(FLOAT_NO_DATA),
        },
        MetadataPropertyTableProperty {
            values: values_bufferview,
            string_offset_type: None,
            string_offsets: None,
        },
        ColumnStats::Float64(range),
    )
}

fn encode_int_column<'a>(
    raw_name: &str,
    values: impl Iterator<Item = Option<&'a AttributeValue>>,
    builder: &mut Builder,
) -> (ClassProperty, MetadataPropertyTableProperty, ColumnStats) {
    let mut collector = SignedIntCollector::new();
    let mut range = None;
    for value in values {
        match value.and_then(as_int) {
            Some(n) => {
                if !matches!(value, Some(AttributeValue::Bool(_))) {
                    range = union(range, Some((n, n)));
                }
                collector.push(n)
            }
            None => {
                if let Some(value) = value {
                    tracing::error!(
                        "Cesium3DTilesWriter: {raw_name:?} is an integer column but {value:?} is \
                         not convertible; encoding it as no-data"
                    );
                }
                collector.push_no_data()
            }
        }
    }
    let finalized = collector.finalize();
    let mut value_bytes = Vec::new();
    finalized.encode_all(&mut value_bytes);
    let values_bufferview = builder.push_buffer_view(&value_bytes, finalized.byte_size());

    (
        ClassProperty {
            name: raw_name.to_string(),
            type_: "SCALAR",
            component_type: Some(int_component_type(finalized.byte_size())),
            enum_type: None,
            no_data: finalized.no_data_json(),
        },
        MetadataPropertyTableProperty {
            values: values_bufferview,
            string_offset_type: None,
            string_offsets: None,
        },
        ColumnStats::Int(range),
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
    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    enums: IndexMap<String, serde_json::Value>,
}

#[derive(Serialize)]
struct EnumDef<'a> {
    #[serde(rename = "valueType")]
    value_type: &'static str,
    values: Vec<EnumValue<'a>>,
}

#[derive(Serialize)]
struct EnumValue<'a> {
    name: &'a str,
    value: usize,
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
    #[serde(rename = "enumType", skip_serializing_if = "Option::is_none")]
    enum_type: Option<String>,
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
        if !matches!(value, AttributeValue::Map(_) | AttributeValue::Array(_)) {
            insert_leaf(key, value, &mut out);
        }
    }
    out
}

fn insert_leaf(path: String, leaf: &AttributeValue, out: &mut BTreeMap<String, AttributeValue>) {
    if out.insert(path.clone(), leaf.clone()).is_some() {
        tracing::warn!("Cesium3DTilesWriter: attribute path {path:?} collided; overwriting");
    }
}

fn is_excluded(key: &str, options: MetadataOptions) -> bool {
    options.skip_unexposed_attributes && key.starts_with("__")
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use super::*;

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

    fn schema_map(attributes: &[(&str, TypeRef)]) -> SchemaMap {
        let mut map = SchemaMap::default();
        for (name, type_ref) in attributes {
            map.insert(
                (*name).to_string(),
                nusamai_citygml::schema::Attribute {
                    type_ref: type_ref.clone(),
                    ..Default::default()
                },
            );
        }
        map
    }

    fn raw_paths(table: &PropertyTable) -> Vec<&str> {
        table
            .properties
            .iter()
            .map(|(raw, _, _)| raw.as_str())
            .collect()
    }

    #[test]
    fn maps_and_arrays_are_dropped() {
        let feature = feature_with_nested();
        let schema = schema_map(&[("name", TypeRef::String)]);
        let table = build_table(&[&feature], &[Some(&schema)], MetadataOptions::default());

        // Only the top-level scalar survives; the map and array contribute
        // no columns at all.
        assert_eq!(raw_paths(&table), vec!["name"]);
    }

    fn number(n: f64) -> AttributeValue {
        AttributeValue::Number(serde_json::Number::from_f64(n).unwrap())
    }

    fn int_number(n: i64) -> AttributeValue {
        AttributeValue::Number(serde_json::Number::from(n))
    }

    #[test]
    fn a_path_declared_twice_widens_to_the_more_general_type() {
        let feature = Feature::from(IndexMap::from([("k".to_string(), int_number(3))]));
        let kind_of = |schemas: &[Option<&SchemaMap>]| {
            let features = vec![&feature; schemas.len()];
            build_table(&features, schemas, MetadataOptions::default()).properties[0].2
        };

        let unsigned = schema_map(&[("k", TypeRef::NonNegativeInteger)]);
        let signed = schema_map(&[("k", TypeRef::Integer)]);
        let double = schema_map(&[("k", TypeRef::Double)]);
        let string = schema_map(&[("k", TypeRef::String)]);

        assert_eq!(kind_of(&[Some(&unsigned)]), ColumnKind::Int);
        assert_eq!(kind_of(&[Some(&unsigned), Some(&signed)]), ColumnKind::Int);
        assert_eq!(kind_of(&[Some(&signed), Some(&unsigned)]), ColumnKind::Int);
        assert_eq!(
            kind_of(&[Some(&unsigned), Some(&double)]),
            ColumnKind::Float64
        );
        assert_eq!(
            kind_of(&[Some(&double), Some(&signed)]),
            ColumnKind::Float64
        );
        assert_eq!(kind_of(&[Some(&string), Some(&double)]), ColumnKind::String);
    }

    #[test]
    fn declared_type_overrides_inferred_type() {
        let declared = Feature::from(IndexMap::from([("k".to_string(), int_number(3))]));
        let undeclared = Feature::from(IndexMap::from([("j".to_string(), int_number(3))]));
        let schema = schema_map(&[("k", TypeRef::String)]);

        let table = build_table(
            &[&declared, &undeclared],
            &[Some(&schema), None],
            MetadataOptions::default(),
        );
        let mut builder = Builder::new();
        encode(&table, &mut builder, &[]);
        let glb = builder.build([0.0, 0.0, 0.0]);

        let gltf = crate::parse_gltf(&bytes::Bytes::from(glb)).unwrap();
        let features = crate::extract_feature_properties(&gltf).unwrap();

        assert_eq!(
            features[0].get("k"),
            Some(&serde_json::Value::String("3".to_string()))
        );
        assert_eq!(features[1].get("j"), Some(&serde_json::json!(3)));
    }

    #[test]
    fn numeric_string_under_declared_number_decodes_as_number() {
        let feature = Feature::from(IndexMap::from([
            ("i".to_string(), AttributeValue::String("7".to_string())),
            ("f".to_string(), AttributeValue::String("7.5".to_string())),
        ]));
        let schema = schema_map(&[("i", TypeRef::Integer), ("f", TypeRef::Double)]);

        let table = build_table(&[&feature], &[Some(&schema)], MetadataOptions::default());
        let mut builder = Builder::new();
        encode(&table, &mut builder, &[]);
        let glb = builder.build([0.0, 0.0, 0.0]);

        let gltf = crate::parse_gltf(&bytes::Bytes::from(glb)).unwrap();
        let features = crate::extract_feature_properties(&gltf).unwrap();

        assert_eq!(features[0].get("i"), Some(&serde_json::json!(7)));
        assert_eq!(features[0].get("f"), Some(&serde_json::json!(7.5)));
    }

    #[test]
    fn stats_cover_only_encoded_numbers() {
        let features: Vec<Feature> = [
            int_number(3),
            AttributeValue::String("7".to_string()),
            number(9.5),
            AttributeValue::Bool(true),
        ]
        .into_iter()
        .map(|v| Feature::from(IndexMap::from([("k".to_string(), v)])))
        .collect();
        let schema = schema_map(&[("k", TypeRef::Integer)]);

        let table = build_table(
            &features.iter().collect::<Vec<_>>(),
            &vec![Some(&schema); features.len()],
            MetadataOptions::default(),
        );
        let stats = encode(&table, &mut Builder::new(), &[]);

        assert_eq!(stats["k"], ColumnStats::Int(Some((3, 7))));
    }

    #[test]
    fn merged_stats_widen_and_a_string_column_has_no_range() {
        let int = ColumnStats::Int(Some((1, 4)));
        let float = ColumnStats::Float64(Some((2.5, 3.0)));
        assert_eq!(int.merge(float), ColumnStats::Float64(Some((1.0, 4.0))));
        assert_eq!(int.merge(ColumnStats::String), ColumnStats::String);
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

        let schema = schema_map(&[
            ("height", TypeRef::Double),
            ("count", TypeRef::NonNegativeInteger),
            ("elevation_delta", TypeRef::Integer),
            ("flag", TypeRef::Boolean),
            ("name", TypeRef::String),
        ]);
        let table = build_table(
            &[&feature1, &feature2],
            &[Some(&schema)],
            MetadataOptions::default(),
        );
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

    #[test]
    fn repeated_strings_encode_as_enum_and_unique_ones_as_string() {
        let owned: Vec<Feature> = (0..20)
            .map(|i| {
                let mut attrs = IndexMap::from([(
                    "id".to_string(),
                    AttributeValue::String(format!("bldg_{i:08}")),
                )]);
                if i > 0 {
                    attrs.insert(
                        "usage".to_string(),
                        AttributeValue::String("residential".to_string()),
                    );
                }
                Feature::from(attrs)
            })
            .collect();
        let features: Vec<&Feature> = owned.iter().collect();

        let table = build_table(
            &features,
            &vec![None; features.len()],
            MetadataOptions::default(),
        );
        let mut builder = Builder::new();
        encode(&table, &mut builder, &[]);
        let glb = builder.build([0.0, 0.0, 0.0]);

        let gltf = crate::parse_gltf(&bytes::Bytes::from(glb)).unwrap();
        let schema = &gltf.extension_value("EXT_structural_metadata").unwrap()["schema"];
        let properties = &schema["classes"]["Feature"]["properties"];
        assert_eq!(properties["id"]["type"], "STRING");
        assert_eq!(properties["usage"]["type"], "ENUM");

        let decoded = crate::extract_feature_properties(&gltf).unwrap();
        assert_eq!(decoded[0].get("usage"), None);
        assert_eq!(decoded[1]["usage"], "residential");
        assert_eq!(decoded[7]["id"], "bldg_00000007");
    }

    #[test]
    fn repeated_strings_with_empty_one_encode_as_string() {
        let owned: Vec<Feature> = (0..20)
            .map(|i| {
                let usage = if i == 0 { "" } else { "residential" };
                Feature::from(IndexMap::from([
                    (
                        "usage".to_string(),
                        AttributeValue::String(usage.to_string()),
                    ),
                    (
                        "kind".to_string(),
                        AttributeValue::String("building".to_string()),
                    ),
                ]))
            })
            .collect();
        let features: Vec<&Feature> = owned.iter().collect();

        let table = build_table(
            &features,
            &vec![None; features.len()],
            MetadataOptions::default(),
        );
        let mut builder = Builder::new();
        encode(&table, &mut builder, &[]);
        let glb = builder.build([0.0, 0.0, 0.0]);

        let gltf = crate::parse_gltf(&bytes::Bytes::from(glb)).unwrap();
        let schema = &gltf.extension_value("EXT_structural_metadata").unwrap()["schema"];
        let properties = &schema["classes"]["Feature"]["properties"];
        assert_eq!(properties["usage"]["type"], "STRING");
        assert_eq!(properties["kind"]["type"], "ENUM");

        let decoded = crate::extract_feature_properties(&gltf).unwrap();
        assert_eq!(decoded[0]["usage"], "");
        assert_eq!(decoded[1]["usage"], "residential");
    }
}
