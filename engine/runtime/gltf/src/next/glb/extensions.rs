//! Local typing for the two Cesium 3D Tiles glTF extensions this writer
//! emits (`EXT_mesh_features`, `EXT_structural_metadata`). Not part of the
//! base glTF spec, so kept separate from it and glued in through its generic
//! `extensions` bag rather than depending on a 3D-Tiles-specific crate.

use indexmap::IndexMap;
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct ExtMeshFeatures {
    #[serde(rename = "featureIds")]
    pub(super) feature_ids: Vec<FeatureId>,
}

#[derive(Serialize)]
pub(super) struct FeatureId {
    #[serde(rename = "featureCount")]
    pub(super) feature_count: usize,
    pub(super) attribute: u32,
    #[serde(rename = "propertyTable")]
    pub(super) property_table: u32,
}

#[derive(Serialize)]
pub(super) struct ExtStructuralMetadata {
    pub(super) schema: MetadataSchema,
    #[serde(rename = "propertyTables")]
    pub(super) property_tables: Vec<MetadataPropertyTable>,
}

#[derive(Serialize)]
pub(super) struct MetadataSchema {
    pub(super) id: &'static str,
    pub(super) classes: IndexMap<&'static str, MetadataClass>,
}

#[derive(Serialize)]
pub(super) struct MetadataClass {
    pub(super) properties: IndexMap<String, ClassProperty>,
}

#[derive(Serialize)]
pub(super) struct ClassProperty {
    pub(super) name: String,
    #[serde(rename = "type")]
    pub(super) type_: &'static str,
}

#[derive(Serialize)]
pub(super) struct MetadataPropertyTable {
    pub(super) class: &'static str,
    pub(super) count: usize,
    pub(super) properties: IndexMap<String, MetadataPropertyTableProperty>,
}

#[derive(Serialize)]
pub(super) struct MetadataPropertyTableProperty {
    pub(super) values: usize,
    #[serde(rename = "stringOffsetType")]
    pub(super) string_offset_type: &'static str,
    #[serde(rename = "stringOffsets")]
    pub(super) string_offsets: usize,
}
