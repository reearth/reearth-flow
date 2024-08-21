use super::tree::{TileContent, TileTree};
use super::util::make_gltf;
use crate::errors::SinkError;
use nusamai_gltf::nusamai_gltf_json::models::Node;
use nusamai_mvt::tileid::TileIdMethod;
use reearth_flow_common::gltf::{
    geometric_error, x_slice_range, x_step, y_slice_range, zxy_from_lng_lat,
};
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::geometry as geomotry_types;
use reearth_flow_types::Expr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::Mutex;
use std::vec;

#[derive(Debug, Clone, Default)]
pub struct Cesium3DTilesSinkFactory;

impl SinkFactory for Cesium3DTilesSinkFactory {
    fn name(&self) -> &str {
        "Cesium3DTilesWriter"
    }

    fn description(&self) -> &str {
        "Writes features to a file"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(Cesium3dtilesWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["File"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn prepare(&self) -> Result<(), BoxedError> {
        Ok(())
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Sink>, BoxedError> {
        let params = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SinkError::BuildFactory(format!("Failed to serialize `with` parameter: {}", e))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::BuildFactory(format!("Failed to deserialize `with` parameter: {}", e))
            })?
        } else {
            return Err(
                SinkError::BuildFactory("Missing required parameter `with`".to_string()).into(),
            );
        };

        let sink = Cesium3dtilesWriter {
            params,
            contents: Arc::new(Mutex::new(Vec::new())),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub struct Cesium3dtilesWriter {
    pub(super) params: Cesium3dtilesWriterParam,
    pub(super) contents: Arc<Mutex<Vec<TileContent>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Cesium3dtilesWriterCommonParam {
    pub(super) output: Expr,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase", tag = "format")]
pub enum Cesium3dtilesWriterParam {
    Cesium3dtiles {
        #[serde(flatten)]
        common_property: Cesium3dtilesWriterCommonParam,
    },
}

impl Cesium3dtilesWriterParam {
    pub fn to_common_param(&self) -> &Cesium3dtilesWriterCommonParam {
        match self {
            Cesium3dtilesWriterParam::Cesium3dtiles { common_property } => common_property,
        }
    }
}

impl Sink for Cesium3dtilesWriter {
    fn initialize(&self, _ctx: NodeContext) {}
    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let geometry = ctx.feature.geometry.as_ref().unwrap();
        let geometry_value = geometry.value.clone();
        match geometry_value {
            geomotry_types::GeometryValue::None => {
                return Err(Box::new(SinkError::FileWriter(
                    "Unsupported input".to_string(),
                )));
            }
            geomotry_types::GeometryValue::CityGmlGeometry(city_gml) => {
                let storage_resolver = Arc::clone(&ctx.storage_resolver);
                let expr_engine = Arc::clone(&ctx.expr_engine);
                let common_param = self.params.to_common_param();
                let scope = expr_engine.new_scope();
                let path = scope
                    .eval::<String>(common_param.output.as_ref())
                    .unwrap_or_else(|_| common_param.output.as_ref().to_string());
                let output = Uri::from_str(path.as_str())?;
                let contents =
                    match handle_city_gml_geometry(&output, storage_resolver.clone(), city_gml) {
                        Ok(contents) => contents,
                        Err(e) => {
                            return Err(Box::new(SinkError::FileWriter(format!(
                                "CityGmlGeometry handle Error: {:?}",
                                e
                            ))))
                        }
                    };
                self.contents
                    .lock()
                    .unwrap()
                    .extend(contents.lock().unwrap().iter().cloned());
            }
            geomotry_types::GeometryValue::FlowGeometry2D(_flow_geom_2d) => {
                return Err(Box::new(SinkError::FileWriter(
                    "Unsupported input".to_string(),
                )));
            }
            geomotry_types::GeometryValue::FlowGeometry3D(_flow_geom_3d) => {
                return Err(Box::new(SinkError::FileWriter(
                    "Unsupported input".to_string(),
                )));
            }
        }

        Ok(())
    }
    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        // Generate tileset.json
        let mut tree = TileTree::default();
        for content in self.contents.lock().unwrap().drain(..) {
            tree.add_content(content);
        }

        let tileset = cesiumtiles::tileset::Tileset {
            asset: cesiumtiles::tileset::Asset {
                version: "1.1".to_string(),
                ..Default::default()
            },
            root: tree.into_tileset_root(),
            geometric_error: 1e+100,
            ..Default::default()
        };

        let gltf_json = serde_json::to_value(&tileset).unwrap();
        let buf = gltf_json.to_string().as_bytes().to_owned();

        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let common_param = self.params.to_common_param();
        let scope = expr_engine.new_scope();
        let path = scope
            .eval::<String>(common_param.output.as_ref())
            .unwrap_or_else(|_| common_param.output.as_ref().to_string());
        let output = Uri::from_str(path.as_str())?;

        let storage = storage_resolver
            .resolve(&output)
            .map_err(crate::errors::SinkError::file_writer)?;

        let root_tileset_path = output.path().join(Path::new("tileset.json"));
        storage
            .put_sync(root_tileset_path.as_path(), bytes::Bytes::from(buf))
            .map_err(crate::errors::SinkError::file_writer)?;
        Ok(())
    }
}

fn handle_city_gml_geometry(
    output: &Uri,
    storage_resolver: Arc<StorageResolver>,
    city_gml: geomotry_types::CityGmlGeometry,
) -> Result<Arc<Mutex<std::vec::Vec<TileContent>>>, crate::errors::SinkError> {
    let ellipsoid = nusamai_projection::ellipsoid::wgs84();
    let tile_id_conv = TileIdMethod::Hilbert;

    let min_zoom = 12;
    let max_zoom = 18;

    let (lng_center, lat_center, approx_dx, approx_dy, approx_dh) = {
        let min_lng = f64::MAX;
        let max_lng = f64::MIN;
        let min_lat = f64::MAX;
        let max_lat = f64::MIN;
        let min_height = f64::MAX;
        let max_height = f64::MIN;
        let approx_dx =
            ellipsoid.a() * min_lat.to_radians().cos() * (max_lng - min_lng).to_radians();
        let approx_dy = ellipsoid.a() * (max_lng - min_lng).to_radians();
        let approx_dh = max_height - min_height;
        (
            (min_lng + max_lng) / 2.0,
            (min_lat + max_lat) / 2.0,
            approx_dx,
            approx_dy,
            approx_dh,
        )
    };

    let contents: Arc<Mutex<Vec<TileContent>>> = Default::default();

    for zoom in min_zoom..=max_zoom {
        if zoom < max_zoom {
            let geom_error = {
                let (_, _, y) = zxy_from_lng_lat(zoom, lng_center, lat_center);
                geometric_error(zoom, y)
            };
            let threshold = geom_error * 1.5;
            if approx_dx < threshold && approx_dy < threshold && approx_dh < threshold {
                continue;
            }
        }

        let (z, x, y) = zxy_from_lng_lat(zoom, lng_center, lat_center);
        let tile_id = tile_id_conv.zxy_to_id(z, x, y);
        // Tile information
        let (content, translation) = {
            let zxy = tile_id_conv.id_to_zxy(tile_id);
            let (tile_zoom, tile_x, tile_y) = zxy;
            let (min_lat, max_lat) = y_slice_range(tile_zoom, tile_y);
            let (min_lng, max_lng) =
                x_slice_range(tile_zoom, tile_x as i32, x_step(tile_zoom, tile_y));

            // Use the tile center as the translation of the glTF mesh
            let translation = {
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
            };

            let content_path = {
                let typename = "typename"; // TODO
                let normalized_typename = typename.replace(':', "_");
                format!("{tile_zoom}/{tile_x}/{tile_y}_{normalized_typename}.glb")
            };
            let content = TileContent {
                zxy,
                content_path,
                min_lng: f64::MAX,
                max_lng: f64::MIN,
                min_lat: f64::MAX,
                max_lat: f64::MIN,
                min_height: f64::MAX,
                max_height: f64::MIN,
            };

            (content, translation)
        };

        contents.lock().unwrap().push(content.clone());

        let mut gltf = match make_gltf(city_gml.clone()) {
            Ok(gltf) => gltf,
            Err(e) => {
                return Err(crate::errors::SinkError::file_writer(format!(
                    "Failed to create glTF: {}",
                    e
                )));
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

        gltf.nodes = vec![Node {
            mesh: gltf.nodes[0].mesh,
            translation,
            ..Default::default()
        }];

        gltf.extensions_used = extensions_used;

        let gltf_json = serde_json::to_value(&gltf).unwrap();
        let buf = gltf_json.to_string().as_bytes().to_owned();

        let content_path = content.content_path;
        let storage = storage_resolver
            .resolve(output)
            .map_err(crate::errors::SinkError::file_writer)?;
        let output_path = output.path().join(Path::new(&content_path));

        if let Some(dir) = output_path.parent() {
            fs::create_dir_all(dir).unwrap();
        }
        let path = Path::new(&output_path);

        storage
            .put_sync(path, bytes::Bytes::from(buf))
            .map_err(crate::errors::SinkError::file_writer)?;
    }

    //====================
    // やらなくてもいいかも？ (tile_id, typename, feats) で feats を sort するだけ。しかし使わなくてもよい？
    // let receiver_sliced = mpsc::sync_channel(2000); // TODO
    // let mut typename_to_seq: IndexSet<String, ahash::RandomState> = Default::default();

    // let config = kv_extsort::SortConfig::default()
    //     .max_chunk_bytes(256 * 1024 * 1024) // TODO: Configurable
    //     .set_cancel_flag(feedback.get_cancellation_flag());

    // let sorted_iter = kv_extsort::sort(
    //     receiver_sliced
    //         .into_iter()
    //         .map(|(tile_id, typename, body)| {
    //             let (idx, _) = typename_to_seq.insert_full(typename);
    //             let type_seq = idx as u64;
    //             std::result::Result::<_, Infallible>::Ok((SortKey { tile_id, type_seq }, body))
    //         }),
    //     config,
    // );

    // for ((_, key), grouped) in &sorted_iter.chunk_by(|feat| match feat {
    //     Ok((key, _)) => (false, *key),
    //     Err(_) => (true, SortKey::zeroed()),
    // }) {
    //     let grouped = grouped
    //         .into_iter()
    //         .map_ok(|(_, serialized_feats)| serialized_feats)
    //         .collect::<kv_extsort::Result<Vec<_>, _>>();
    //     match grouped {
    //         Ok(serialized_feats) => {
    //             feedback.ensure_not_canceled()?;
    //             let tile_id = key.tile_id;
    //             let typename = typename_to_seq[key.type_seq as usize].clone();
    //             if sender_sorted
    //                 .send((tile_id, typename, serialized_feats))
    //                 .is_err()
    //             {
    //                 return Err(PipelineError::Canceled);
    //             }
    //         }
    //         Err(kv_extsort::Error::Canceled) => {
    //             return Err(PipelineError::Canceled);
    //         }
    //         Err(err) => {
    //             return Err(PipelineError::Other(format!(
    //                 "Failed to sort features: {:?}",
    //                 err
    //             )));
    //         }
    //     }
    // }
    //====================

    Ok(contents)
}
