//! Per-tile `EXT_structural_metadata` property table: one implicit `Feature`
//! class per glb, its properties the union of every attribute path present
//! across that glb's features. Also the only place that knows the
//! `EXT_structural_metadata`/`EXT_mesh_features` JSON shapes, via [`encode`],
//! which attaches them to a `glb::Builder` directly.

use std::collections::{BTreeMap, BTreeSet, HashSet};

use gltf::json;
use indexmap::IndexMap;
use reearth_flow_types::{AttributeValue, Feature};
use serde::Serialize;

use super::glb::{Builder, PrimitiveHandle};

const METADATA_SCHEMA_ID: &str = "Schema";
const METADATA_CLASS_NAME: &str = "Feature";

/// No per-feature-type classing yet (single inlined `Feature` class,
/// string-typed properties only), but these two exclusions still apply,
/// reusing the parent writer's params.
#[derive(Debug, Clone, Copy, Default)]
pub struct MetadataOptions<'a> {
    pub schema_key: Option<&'a str>,
    pub skip_unexposed_attributes: bool,
}

/// `properties[i] = (raw attribute path, glTF-identifier-safe property id)`;
/// `rows[feature][i]` is that feature's value for column `i` (`""` if the
/// feature doesn't carry that path).
pub struct PropertyTable {
    pub properties: Vec<(String, String)>,
    pub rows: Vec<Vec<String>>,
}

pub fn build_table(features: &[&Feature], options: MetadataOptions) -> PropertyTable {
    let flattened: Vec<BTreeMap<String, String>> = features
        .iter()
        .map(|feature| flatten_attributes(feature, options))
        .collect();

    let mut raw_paths = BTreeSet::new();
    for f in &flattened {
        raw_paths.extend(f.keys().cloned());
    }

    let mut used_ids = HashSet::new();
    let properties: Vec<(String, String)> = raw_paths
        .into_iter()
        .map(|raw| {
            let id = sanitize_identifier(&raw, &mut used_ids);
            (raw, id)
        })
        .collect();

    let rows = flattened
        .iter()
        .map(|f| {
            properties
                .iter()
                .map(|(raw, _)| f.get(raw).cloned().unwrap_or_default())
                .collect()
        })
        .collect();

    PropertyTable { properties, rows }
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
    builder: &mut Builder,
    primitives: &[(PrimitiveHandle, &[u32])],
) {
    if table.properties.is_empty() {
        return;
    }

    // One STRING property per column: raw UTF-8 bytes in `values`,
    // cumulative byte offsets in `stringOffsets` (EXT_structural_metadata's
    // variable-length-array encoding).
    let mut class_properties = IndexMap::new();
    let mut table_properties = IndexMap::new();
    for (col, (raw_name, id)) in table.properties.iter().enumerate() {
        class_properties.insert(
            id.clone(),
            ClassProperty {
                name: raw_name.clone(),
                type_: "STRING",
            },
        );

        let mut value_bytes = Vec::new();
        let mut offsets: Vec<u32> = vec![0];
        for row in &table.rows {
            value_bytes.extend_from_slice(row[col].as_bytes());
            offsets.push(value_bytes.len() as u32);
        }
        let values_bufferview = builder.push_buffer_view(&value_bytes);
        let offset_bytes: Vec<u8> = offsets.iter().flat_map(|o| o.to_le_bytes()).collect();
        let offsets_bufferview = builder.push_buffer_view(&offset_bytes);

        table_properties.insert(
            id.clone(),
            MetadataPropertyTableProperty {
                values: values_bufferview,
                string_offset_type: "UINT32",
                string_offsets: offsets_bufferview,
            },
        );
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
    #[serde(rename = "stringOffsetType")]
    string_offset_type: &'static str,
    #[serde(rename = "stringOffsets")]
    string_offsets: usize,
}

fn flatten_attributes(feature: &Feature, options: MetadataOptions) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for (key, value) in feature.attributes.iter() {
        let key = key.inner();
        if is_excluded(&key, options) {
            continue;
        }
        flatten(key, value, &mut out);
    }
    out
}

/// Walks `value`, inserting one `path -> stringified leaf` entry per scalar
/// reached. `EXT_structural_metadata` has no arbitrary-nesting property type,
/// so a `Map`/`Array` contributes no entry of its own, only its descendants,
/// with `path` extended by `_<child key>` / `_<index>`.
fn flatten(path: String, value: &AttributeValue, out: &mut BTreeMap<String, String>) {
    match value {
        AttributeValue::Map(map) => {
            for (key, child) in map {
                flatten(format!("{path}_{key}"), child, out);
            }
        }
        AttributeValue::Array(items) => {
            for (i, child) in items.iter().enumerate() {
                flatten(format!("{path}_{i}"), child, out);
            }
        }
        leaf => {
            if out.insert(path.clone(), leaf.to_string()).is_some() {
                tracing::warn!(
                    "Cesium3DTilesWriter: attribute path {path:?} collided; overwriting"
                );
            }
        }
    }
}

fn is_excluded(key: &str, options: MetadataOptions) -> bool {
    (options.skip_unexposed_attributes && key.starts_with("__")) || options.schema_key == Some(key)
}

/// CityGML attribute keys are commonly namespace-prefixed (`bldg:measuredHeight`,
/// `uro:buildingIDAttribute`) and so routinely violate `EXT_structural_metadata`'s
/// identifier syntax; this maps a raw key to a valid, collision-free id, while
/// the raw key survives separately as the property's `name` for display.
fn sanitize_identifier(raw: &str, used: &mut HashSet<String>) -> String {
    let mut id: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if id.is_empty() || id.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        id.insert(0, '_');
    }
    if used.insert(id.clone()) {
        return id;
    }
    let mut n = 1;
    loop {
        let candidate = format!("{id}_{n}");
        if used.insert(candidate.clone()) {
            return candidate;
        }
        n += 1;
    }
}
