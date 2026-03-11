use std::sync::Arc;

use reearth_flow_action_sink::file::citygml::write_citygml_to_storage;
use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{lod::LodMask, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::FeatureProcessorError;

/// # CityGmlWriter Parameters
///
/// Configuration for writing features in CityGML 2.0 format.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct CityGmlWriterParam {
    /// LOD levels to include (e.g., [0, 1, 2]). If empty, includes all LODs.
    #[serde(default)]
    pub(super) lod_filter: Option<Vec<u8>>,
    /// EPSG code for coordinate reference system
    #[serde(default)]
    pub(super) epsg_code: Option<u32>,
    /// Whether to format output with indentation (default: true)
    #[serde(default = "default_pretty_print")]
    pub(super) pretty_print: Option<bool>,
}

fn default_pretty_print() -> Option<bool> {
    Some(true)
}

pub(super) fn build_lod_mask(lod_filter: &Option<Vec<u8>>) -> LodMask {
    match lod_filter {
        Some(lods) if !lods.is_empty() => {
            let mut mask = LodMask::default();
            for lod in lods {
                mask.add_lod(*lod);
            }
            mask
        }
        _ => LodMask::all(),
    }
}

pub(super) fn write_citygml(
    output: &Uri,
    features: &[Feature],
    lod_mask: &LodMask,
    epsg_code: &Option<u32>,
    pretty_print: &bool,
    storage_resolver: &Arc<StorageResolver>,
) -> Result<(), FeatureProcessorError> {
    write_citygml_to_storage(
        output,
        features,
        lod_mask,
        *epsg_code,
        *pretty_print,
        storage_resolver,
    )
    .map_err(|e| FeatureProcessorError::FeatureWriter(format!("{e:?}")))
}
