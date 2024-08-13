use super::util::make_gltf;
use crate::errors::SinkError;
use nusamai_gltf::nusamai_gltf_json::models::Node;
use nusamai_mvt::tileid::TileIdMethod;
use rayon::iter::{ParallelBridge, ParallelIterator};
use reearth_flow_common::gltf::{x_slice_range, x_step, y_slice_range};
use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::geometry as geomotry_types;
use reearth_flow_types::Feature;
use std::path::Path;
use std::sync::mpsc;
use std::sync::Arc;
use std::vec;

pub(super) fn write_cesium3dtiles(
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

    let gltf_textures = &gltf.textures;

    let has_webp = gltf_textures.iter().any(|texture| {
        texture
            .extensions
            .as_ref()
            .and_then(|ext| ext.ext_texture_webp.as_ref())
            .map_or(false, |_| true)
    });

    let extensions_used = {
        let mut extensions_used = vec![
            "EXT_mesh_features".to_string(),
            "EXT_structural_metadata".to_string(),
        ];

        // Add "EXT_texture_webp" extension if WebP textures are present
        if has_webp {
            extensions_used.push("EXT_texture_webp".to_string());
        }

        extensions_used
    };

    let translation = translation();

    gltf.nodes = vec![Node {
        mesh: gltf.nodes[0].mesh,
        translation,
        ..Default::default()
    }];

    gltf.extensions_used = extensions_used;

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

fn translation() -> [f64; 3] {
    let (_, receiver_sorted) = mpsc::sync_channel(2000);
    let ellipsoid = nusamai_projection::ellipsoid::wgs84();
    let tile_id_conv = TileIdMethod::Hilbert;

    let translation: Vec<[f64; 3]> = receiver_sorted
        .into_iter()
        .par_bridge()
        .map(|tile_id| {
            // Tile information
            let zxy = tile_id_conv.id_to_zxy(tile_id);
            let (tile_zoom, tile_x, tile_y) = zxy;
            let (min_lat, max_lat) = y_slice_range(tile_zoom, tile_y);
            let (min_lng, max_lng) =
                x_slice_range(tile_zoom, tile_x as i32, x_step(tile_zoom, tile_y));

            // Use the tile center as the translation of the glTF mesh
            let (tx, ty, tz) = nusamai_projection::cartesian::geodetic_to_geocentric(
                &ellipsoid,
                (min_lng + max_lng) / 2.0,
                (min_lat + max_lat) / 2.0,
                0.,
            );
            // z-up to y-up
            let [tx, ty, tz] = [tx, tz, -ty];
            // double-precision to single-precision
            [(tx as f32) as f64, (ty as f32) as f64, (tz as f32) as f64]
        })
        .collect();

    translation[0]
}
