use super::tree::{TileContent, TileTree};
use super::util::make_gltf;
use crate::errors::SinkError;
use itertools::Itertools;
use nusamai_gltf::nusamai_gltf_json::models::Node;
use nusamai_mvt::tileid::TileIdMethod;
use reearth_flow_common::gltf::{
    geometric_error, iter_x_slice, iter_y_slice, x_slice_range, x_step, y_slice_range,
    zxy_from_lng_lat,
};
use reearth_flow_common::uri::Uri;
use reearth_flow_geometry::algorithm::coords_iter::CoordsIter;
use reearth_flow_geometry::types::coordinate::Coordinate;
use reearth_flow_geometry::types::line_string::LineString;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon;
use reearth_flow_geometry::types::multi_polygon::MultiPolygon2D;
use reearth_flow_geometry::types::polygon::Polygon;
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
                SinkError::Cesium3DTilesWriterFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::Cesium3DTilesWriterFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(SinkError::Cesium3DTilesWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
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
#[serde(rename_all = "camelCase")]
pub struct Cesium3dtilesWriterParam {
    pub(super) output: Expr,
    pub(super) min_zoom: Option<u8>,
    pub(super) max_zoom: Option<u8>,
}

impl Sink for Cesium3dtilesWriter {
    fn initialize(&self, _ctx: NodeContext) {}

    fn name(&self) -> &str {
        "Cesium3DTilesWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let geometry = ctx.feature.geometry.as_ref().unwrap();
        let geometry_value = geometry.value.clone();
        match geometry_value {
            geomotry_types::GeometryValue::None => {
                return Err(Box::new(SinkError::Cesium3DTilesWriter(
                    "Unsupported input".to_string(),
                )));
            }
            geomotry_types::GeometryValue::CityGmlGeometry(city_gml) => {
                let storage_resolver = Arc::clone(&ctx.storage_resolver);
                let expr_engine = Arc::clone(&ctx.expr_engine);
                let output = self.params.output.clone();
                let scope = expr_engine.new_scope();
                let path = scope
                    .eval::<String>(output.as_ref())
                    .unwrap_or_else(|_| output.as_ref().to_string());
                let output = Uri::from_str(path.as_str())?;
                match handle_city_gml_geometry(
                    &output,
                    storage_resolver.clone(),
                    city_gml,
                    self.params.min_zoom,
                    self.params.max_zoom,
                    &mut self.contents,
                ) {
                    Ok(_) => {}
                    Err(e) => {
                        return Err(Box::new(SinkError::Cesium3DTilesWriter(format!(
                            "CityGmlGeometry handle Error: {:?}",
                            e
                        ))))
                    }
                };
            }
            geomotry_types::GeometryValue::FlowGeometry2D(_flow_geom_2d) => {
                return Err(Box::new(SinkError::Cesium3DTilesWriter(
                    "Unsupported input".to_string(),
                )));
            }
            geomotry_types::GeometryValue::FlowGeometry3D(_flow_geom_3d) => {
                return Err(Box::new(SinkError::Cesium3DTilesWriter(
                    "Unsupported input".to_string(),
                )));
            }
        }

        Ok(())
    }
    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        let mut tree = TileTree::default();
        for content in self.contents.lock().unwrap().drain(..) {
            tree.add_content(content);
        }

        let tileset = cesiumtiles::tileset::Tileset {
            asset: cesiumtiles::tileset::Asset {
                version: "1.1".to_string(),
                ..Default::default()
            },
            root: tree.into_tileset_root(None),
            geometric_error: 1e+100,
            ..Default::default()
        };

        let gltf_json = serde_json::to_value(&tileset).unwrap();
        let buf = gltf_json.to_string().as_bytes().to_owned();

        let storage_resolver = Arc::clone(&ctx.storage_resolver);
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let output = self.params.output.clone();
        let scope = expr_engine.new_scope();
        let path = scope
            .eval::<String>(output.as_ref())
            .unwrap_or_else(|_| output.as_ref().to_string());
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
    min_zoom: Option<u8>,
    max_zoom: Option<u8>,
    contents: &mut Arc<Mutex<Vec<TileContent>>>,
) -> Result<(), crate::errors::SinkError> {
    let features = city_gml.features.clone();
    for feature in features {
        match handle_feature(
            output,
            storage_resolver.clone(),
            &city_gml,
            feature,
            min_zoom,
            max_zoom,
            contents,
        ) {
            Ok(_) => {}
            Err(e) => {
                return Err(crate::errors::SinkError::file_writer(format!(
                    "Feature handle Error: {:?}",
                    e
                )))
            }
        }
    }
    Ok(())
}

fn handle_feature(
    output: &Uri,
    storage_resolver: Arc<StorageResolver>,
    city_gml: &geomotry_types::CityGmlGeometry,
    feature: geomotry_types::GeometryFeature,
    min_zoom: Option<u8>,
    max_zoom: Option<u8>,
    contents: &mut Arc<Mutex<Vec<TileContent>>>,
) -> Result<(), crate::errors::SinkError> {
    let typename = feature.ty.name();

    let ellipsoid = nusamai_projection::ellipsoid::wgs84();
    let tile_id_conv = TileIdMethod::Hilbert;

    let min_zoom = min_zoom.unwrap_or(12);
    let max_zoom = max_zoom.unwrap_or(18);

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

        for polygon in feature.polygons.clone() {
            let keys =
                match slice_polygon(zoom, polygon, city_gml.polygon_uv.as_ref().unwrap().clone()) {
                    Ok(keys) => keys,
                    Err(e) => {
                        return Err(crate::errors::SinkError::file_writer(format!(
                            "Failed to slice polygon: {}",
                            e
                        )));
                    }
                };

            for (z, x, y) in keys {
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
        }
    }
    Ok(())
}

/// Slice a polygon into tiles. The slicing algorithm is based on [geojson-vt](https://github.com/mapbox/geojson-vt).
fn slice_polygon(
    zoom: u8,
    poly: Polygon<f64>,
    poly_uv: MultiPolygon2D<f64>,
) -> Result<Vec<(u8, u32, u32)>, BoxedError> {
    if poly.exterior().into_iter().len() == 0 {
        return Ok(Vec::new());
    }

    let mut keys: Vec<(u8, u32, u32)> = Vec::new();

    let mut ring_buffer: Vec<[f64; 5]> = Vec::with_capacity(poly.exterior().into_iter().len() + 1);

    // Slice along Y-axis
    let y_range = {
        let (min_y, max_y) = poly
            .exterior()
            .into_iter()
            .fold((f64::MAX, f64::MIN), |(min_y, max_y), c| {
                (min_y.min(c.y), max_y.max(c.y))
            });
        iter_y_slice(zoom, min_y, max_y)
    };

    let mut y_sliced_polys: MultiPolygon = Default::default();

    for yi in y_range.clone() {
        let (k1, k2) = y_slice_range(zoom, yi);

        for poly_uv in poly_uv.iter() {
            for (ri, (ring, uv_ring)) in poly.rings().iter().zip_eq(poly_uv.rings()).enumerate() {
                if ring.coords_iter().len() == 0 {
                    continue;
                }

                ring_buffer.clear();
                ring.into_iter()
                    .zip_eq(uv_ring.into_iter())
                    .fold(None, |a, b| {
                        let Some((a, a_uv)) = a else { return Some(b) };
                        let (b, b_uv) = b;

                        if a.y < k1 {
                            if b.y > k1 {
                                let t = (k1 - a.y) / (b.y - a.y);
                                let x = (b.x - a.x) * t + a.x;
                                let z = (b.z - a.z) * t + a.z;
                                let u = (b_uv.x - a_uv.x) * t + a_uv.x;
                                let v = (b_uv.y - a_uv.y) * t + a_uv.y;
                                ring_buffer.push([x, k1, z, u, v])
                            }
                        } else if a.y > k2 {
                            if b.y < k2 {
                                let t = (k2 - a.y) / (b.y - a.y);
                                let x = (b.x - a.x) * t + a.x;
                                let z = (b.z - a.z) * t + a.z;
                                let u = (b_uv.x - a_uv.x) * t + a_uv.x;
                                let v = (b_uv.y - a_uv.y) * t + a_uv.y;
                                ring_buffer.push([x, k2, z, u, v])
                            }
                        } else {
                            ring_buffer.push([a.x, a.y, a.z, a_uv.x, a_uv.y])
                        }

                        if b.y < k1 && a.y > k1 {
                            let t = (k1 - a.y) / (b.y - a.y);
                            let x = (b.x - a.x) * t + a.x;
                            let z = (b.z - a.z) * t + a.z;
                            let u = (b_uv.x - a_uv.x) * t + a_uv.x;
                            let v = (b_uv.y - a_uv.y) * t + a_uv.y;
                            ring_buffer.push([x, k1, z, u, v])
                        } else if b.y > k2 && a.y < k2 {
                            let t = (k2 - a.y) / (b.y - a.y);
                            let x = (b.x - a.x) * t + a.x;
                            let z = (b.z - a.z) * t + a.z;
                            let u = (b_uv.x - a_uv.x) * t + a_uv.x;
                            let v = (b_uv.y - a_uv.y) * t + a_uv.y;
                            ring_buffer.push([x, k2, z, u, v])
                        }

                        Some((b, b_uv))
                    })
                    .unwrap();

                let ls: LineString = LineString::new(
                    ring_buffer
                        .iter()
                        .map(|c| Coordinate::new__(c[0], c[1], c[2]))
                        .collect(),
                );
                match ri {
                    0 => y_sliced_polys.add_exterior(ls),
                    _ => y_sliced_polys.add_interior(ls),
                }
            }
        }
    }

    // Slice along X-axis
    for (yi, y_sliced_poly) in y_range.zip_eq(y_sliced_polys.iter()) {
        let x_iter = {
            let (min_x, max_x) = y_sliced_poly
                .exterior()
                .into_iter()
                .fold((f64::MAX, f64::MIN), |(min_x, max_x), c| {
                    (min_x.min(c.x), max_x.max(c.x))
                });

            iter_x_slice(zoom, yi, min_x, max_x)
        };

        for (xi, _xs) in x_iter {
            let key = (
                zoom,
                xi.rem_euclid(1 << zoom) as u32, // handling geometry crossing the antimeridian
                yi,
            );

            keys.push(key);
        }
    }
    Ok(keys)
}
