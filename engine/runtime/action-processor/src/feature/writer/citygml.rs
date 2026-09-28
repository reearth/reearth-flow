use std::sync::Arc;

#[cfg(not(feature = "new-geometry"))]
use reearth_flow_action_sink::file::citygml::write_citygml_to_storage;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::diagnostics::NodeDiagnosticsHandle;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{lod::LodMask, Feature};

use super::FeatureProcessorError;

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

#[cfg(not(feature = "new-geometry"))]
#[allow(clippy::too_many_arguments)]
pub(super) fn write_citygml(
    output: &Uri,
    sandbox_root: &Uri,
    features: &[Feature],
    lod_mask: &LodMask,
    epsg_code: &Option<u32>,
    pretty_print: &bool,
    storage_resolver: &Arc<StorageResolver>,
    diagnostics: Option<&NodeDiagnosticsHandle>,
) -> Result<(), FeatureProcessorError> {
    write_citygml_to_storage(
        output,
        sandbox_root,
        features,
        lod_mask,
        *epsg_code,
        *pretty_print,
        storage_resolver,
        diagnostics,
    )
    .map_err(|e| FeatureProcessorError::FeatureWriter(format!("{e:?}")))
}
