use earcut::utils3d::project3d_to_2d;
use earcut::Earcut;
use indexmap::IndexSet;
use nusamai_projection::cartesian::geodetic_to_geocentric;
use nusamai_projection::vshift::Jgd2011ToWgs84;
use std::collections::HashMap;
use std::fs;
use std::hash::RandomState;
use std::io::BufWriter;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::vec;

use itertools::Itertools;
use reearth_flow_common::gltf::{
    calculate_normal, geometric_error, x_slice_range, x_step, y_slice_range, zxy_from_lng_lat,
};
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::Expr;
use reearth_flow_types::{geometry as geomotry_types, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::types::material::{Material, Texture};
use super::types::metadata::MetadataEncoder;
use super::types::slice::SlicedFeature;
use super::types::tree::{TileContent, TileTree};
use super::util::{slice_polygon, write_gltf_glb, Primitives};
use crate::errors::SinkError;

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
            jgd2wgs: Jgd2011ToWgs84::default(),
            contents: Default::default(),
        };
        Ok(Box::new(sink))
    }
}

#[derive(Debug, Clone)]
pub struct Cesium3dtilesWriter {
    pub(super) params: Cesium3dtilesWriterParam,
    pub(super) contents: HashMap<(u8, u32, u32), TileContent>,
    pub(super) jgd2wgs: Jgd2011ToWgs84,
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
                let min_zoom = self.params.min_zoom.unwrap_or(12);
                let max_zoom = self.params.max_zoom.unwrap_or(18);
                let mut city_gml = city_gml.clone();
                city_gml.gml_geometries.iter_mut().for_each(|entry| {
                    entry.transform_inplace(&self.jgd2wgs);
                });
                match self.handle_city_gml_geometry(
                    &ctx.feature,
                    &output,
                    storage_resolver.clone(),
                    city_gml,
                    min_zoom,
                    max_zoom,
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
        for content in self.contents.values() {
            tree.add_content(content.clone());
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

impl Cesium3dtilesWriter {
    fn handle_city_gml_geometry(
        &mut self,
        feature: &Feature,
        output: &Uri,
        storage_resolver: Arc<StorageResolver>,
        city_gml: geomotry_types::CityGmlGeometry,
        min_zoom: u8,
        max_zoom: u8,
    ) -> Result<(), crate::errors::SinkError> {
        let ellipsoid = nusamai_projection::ellipsoid::wgs84();


        let default_material = reearth_flow_types::Material::default();
        let mut sliced_tiles: HashMap<(u8, u32, u32), SlicedFeature> = HashMap::new();
        let mut materials: IndexSet<Material> = IndexSet::new();

        let (lng_center, lat_center, approx_dx, approx_dy, approx_dh) = {
            let vertice = city_gml.max_min_vertice();

            let approx_dx = ellipsoid.a()
                * vertice.min_lat.to_radians().cos()
                * (vertice.max_lng - vertice.min_lng).to_radians();
            let approx_dy = ellipsoid.a() * (vertice.max_lng - vertice.min_lng).to_radians();
            let approx_dh = vertice.max_height - vertice.min_height;
            (
                (vertice.min_lng + vertice.max_lng) / 2.0,
                (vertice.min_lat + vertice.max_lat) / 2.0,
                approx_dx,
                approx_dy,
                approx_dh,
            )
        };
        for entry in city_gml.gml_geometries.iter() {
            for (((poly, poly_uv), poly_mat), poly_tex) in entry
                .polygons
                .iter()
                .zip_eq(
                    city_gml
                        .polygon_uvs
                        .range(entry.pos as usize..(entry.pos + entry.len) as usize)
                        .iter(),
                )
                .zip_eq(
                    city_gml.polygon_materials
                        [entry.pos as usize..(entry.pos + entry.len) as usize]
                        .iter(),
                )
                .zip_eq(
                    city_gml.polygon_textures[entry.pos as usize..(entry.pos + entry.len) as usize]
                        .iter(),
                )
            {
                let orig_mat = poly_mat
                    .and_then(|idx| city_gml.materials.get(idx as usize))
                    .unwrap_or(&default_material)
                    .clone();
                let orig_tex = poly_tex.and_then(|idx| city_gml.textures.get(idx as usize));
                let mat = Material {
                    base_color: orig_mat.diffuse_color.into(),
                    base_texture: orig_tex.map(|tex| Texture {
                        uri: tex.uri.clone(),
                    }),
                };
                let (mat_idx, _) = materials.insert_full(mat);

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
                    slice_polygon(zoom, poly, poly_uv, |(z, x, y), poly| {
                        let sliced_feature = sliced_tiles.entry((z, x, y)).or_insert_with(|| {
                            SlicedFeature {
                                typename: entry.name().to_string(),
                                polygons: Default::default(),
                                attributes: feature.attributes.clone(),
                                polygon_material_ids: Default::default(),
                                materials: Default::default(), // set later
                            }
                        });
                        sliced_feature.polygons.push(poly);
                        sliced_feature.polygon_material_ids.push(mat_idx as u32);
                    });
                }
            }
        }
        let mut feature_id = 0;
        for ((tile_zoom, tile_x, tile_y), mut sliced_feature) in sliced_tiles {
            sliced_feature.materials.clone_from(&materials);
            // Tile information
            let (mut content, translation) = {
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
                    let normalized_typename = sliced_feature.typename.replace(':', "_");
                    format!("{tile_zoom}/{tile_x}/{tile_y}_{normalized_typename}.glb")
                };
                let content = TileContent {
                    zxy: (tile_zoom, tile_x, tile_y),
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
            sliced_feature
                .polygons
                .transform_inplace(|&[lng, lat, height, u, v]| {
                    // Update tile boundary
                    content.min_lng = content.min_lng.min(lng);
                    content.max_lng = content.max_lng.max(lng);
                    content.min_lat = content.min_lat.min(lat);
                    content.max_lat = content.max_lat.max(lat);
                    content.min_height = content.min_height.min(height);
                    content.max_height = content.max_height.max(height);

                    let (x, y, z) = geodetic_to_geocentric(&ellipsoid, lng, lat, height);
                    [
                        x - translation[0],
                        z - translation[1],
                        -y - translation[2],
                        u,
                        1.0 - v,
                    ]
                });
            let content_path = content.content_path.clone();
            let exist = self
                .contents
                .entry((tile_zoom, tile_x, tile_y))
                .or_insert(content.clone());
            if exist.min_lng > content.min_lng {
                exist.min_lng = content.min_lng;
            }
            if exist.max_lng < content.max_lng {
                exist.max_lng = content.max_lng;
            }
            if exist.min_lat > content.min_lat {
                exist.min_lat = content.min_lat;
            }
            if exist.max_lat < content.max_lat {
                exist.max_lat = content.max_lat;
            }
            if exist.min_height > content.min_height {
                exist.min_height = content.min_height;
            }
            if exist.max_height < content.max_height {
                exist.max_height = content.max_height;
            }

            let mut earcutter = Earcut::new();
            let mut buf3d: Vec<[f64; 3]> = Vec::new();
            let mut buf2d: Vec<[f64; 2]> = Vec::new(); // 2d-projected [x, y]
            let mut index_buf: Vec<u32> = Vec::new();

            let mut vertices: IndexSet<[u32; 9], RandomState> = IndexSet::default(); // [x, y, z, u, v, feature_id]
            let mut primitives: Primitives = Default::default();

            let metadata_encoder = MetadataEncoder::new();
            // TODO support metadata_encoder

            for (poly, orig_mat_id) in sliced_feature
                .polygons
                .iter()
                .zip_eq(sliced_feature.polygon_material_ids.iter())
            {
                let num_outer_points = match poly.hole_indices().first() {
                    Some(&v) => v as usize,
                    None => poly.raw_coords().len(),
                };

                let mat = sliced_feature.materials[*orig_mat_id as usize].clone();
                let primitive = primitives.entry(mat).or_default();
                primitive.feature_ids.insert(feature_id as u32);

                if let Some((nx, ny, nz)) =
                    calculate_normal(poly.exterior().iter().map(|v| [v[0], v[1], v[2]]))
                {
                    buf3d.clear();
                    buf3d.extend(poly.raw_coords().iter().map(|c| [c[0], c[1], c[2]]));

                    if project3d_to_2d(&buf3d, num_outer_points, &mut buf2d) {
                        // earcut
                        earcutter.earcut(
                            buf2d.iter().cloned(),
                            poly.hole_indices(),
                            &mut index_buf,
                        );

                        // collect triangles
                        primitive.indices.extend(index_buf.iter().map(|&idx| {
                            let [x, y, z, u, v] = poly.raw_coords()[idx as usize];
                            let vbits = [
                                (x as f32).to_bits(),
                                (y as f32).to_bits(),
                                (z as f32).to_bits(),
                                (nx as f32).to_bits(),
                                (ny as f32).to_bits(),
                                (nz as f32).to_bits(),
                                (u as f32).to_bits(),
                                (v as f32).to_bits(),
                                (feature_id as f32).to_bits(), // UNSIGNED_INT can't be used for vertex attribute
                            ];
                            let (index, _) = vertices.insert_full(vbits);
                            index as u32
                        }));
                    }
                }
                feature_id += 1;
            }

            let mut buffer = Vec::new();
            let writer = BufWriter::new(&mut buffer);

            write_gltf_glb(
                writer,
                translation,
                vertices,
                primitives,
                feature_id, // number of features
                metadata_encoder,
            )?;

            let storage = storage_resolver
                .resolve(output)
                .map_err(crate::errors::SinkError::file_writer)?;
            let output_path = output.path().join(Path::new(&content_path));

            if let Some(dir) = output_path.parent() {
                fs::create_dir_all(dir).unwrap();
            }
            let path = Path::new(&output_path);

            storage
                .put_sync(path, bytes::Bytes::from(buffer))
                .map_err(crate::errors::SinkError::file_writer)?;
        }
        Ok(())
    }
}
