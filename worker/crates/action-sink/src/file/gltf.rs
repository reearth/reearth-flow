use super::util::make_gltf;
use crate::errors::SinkError;
use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::geometry as geomotry_types;
use reearth_flow_types::Feature;
use std::path::Path;
use std::sync::Arc;
use std::vec;

pub(super) fn write_gltf(
    output: &Uri,
    features: &[Feature],
    storage_resolver: Arc<StorageResolver>,
) -> Result<(), SinkError> {
    for feature in features {
        let geometry = feature.geometry.as_ref().unwrap();
        let geometry_value = geometry.value.clone();
        match geometry_value {
            geomotry_types::GeometryValue::None => {
                return Err(SinkError::FileWriter("Unsupported input".to_string()));
            }
            geomotry_types::GeometryValue::CityGmlGeometry(city_gml) => {
                match handle_city_gml_geometry(output, storage_resolver.clone(), city_gml) {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(SinkError::FileWriter(format!(
                            "CityGmlGeometry handle Error: {:?}",
                            e
                        )))
                    }
                }
            }
            geomotry_types::GeometryValue::FlowGeometry2D(_flow_geom_2d) => {
                return Err(SinkError::FileWriter("Unsupported input".to_string()));
            }
            geomotry_types::GeometryValue::FlowGeometry3D(_flow_geom_3d) => {
                return Err(SinkError::FileWriter("Unsupported input".to_string()));
            }
        }
    }
    Ok(())
}

fn handle_city_gml_geometry(
    output: &Uri,
    storage_resolver: Arc<StorageResolver>,
    city_gml: geomotry_types::CityGmlGeometry,
) -> Result<(), crate::errors::SinkError> {
    let mut gltf = match make_gltf(city_gml) {
        Ok(gltf) => gltf,
        Err(e) => {
            return Err(SinkError::FileWriter(format!(
                "Failed to make gltf: {:?}",
                e
            )))
        }
    };

    gltf.extensions_used = vec![
        "EXT_mesh_features".to_string(),
        "EXT_structural_metadata".to_string(),
        "EXT_texture_webp".to_string(),
    ];

    let gltf_json = serde_json::to_value(&gltf).unwrap();

    let buf = gltf_json.to_string().as_bytes().to_owned();

    let storage = storage_resolver
        .resolve(output)
        .map_err(crate::errors::SinkError::file_writer)?;
    let uri_path = output.path();
    let path = Path::new(&uri_path);

    storage
        .put_sync(path, bytes::Bytes::from(buf))
        .map_err(crate::errors::SinkError::file_writer)?;

    Ok(())
}
