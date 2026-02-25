use std::collections::HashMap;
use std::f64::consts::FRAC_PI_2;
use std::io::BufWriter;
use std::sync::Mutex;
use std::{str::FromStr, sync::Arc};

use atlas_packer::export::JpegAtlasExporter;
use atlas_packer::pack::AtlasPacker;
use atlas_packer::texture::cache::{TextureCache, TextureSizeCache};
use flatgeom::{Polygon2, Polygon3};
use glam::{DMat4, DVec3, DVec4};
use indexmap::IndexSet;
use nusamai_projection::cartesian::geodetic_to_geocentric;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use reearth_flow_common::uri::Uri;
use reearth_flow_gltf::{BoundingVolume, MetadataEncoder};
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::material::{self, Material};
use reearth_flow_types::{Expr, GeometryType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tempfile::tempdir;

use crate::atlas::GltfFeature as ClassFeature;
use crate::atlas::{
    encode_metadata, load_textures_into_packer, process_geometry_with_atlas_export,
};
use crate::errors::SinkError;
use crate::zip_eq_logged::ZipEqLoggedExt;

#[derive(Debug, Clone, Default)]
pub struct GltfWriterSinkFactory;

impl SinkFactory for GltfWriterSinkFactory {
    fn name(&self) -> &str {
        "GltfWriter"
    }

    fn description(&self) -> &str {
        "Writes 3D features to GLTF format with optional texture attachment"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GltfWriterParam))
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
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Sink>, BoxedError> {
        let params: GltfWriterParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SinkError::BuildFactory(format!("Failed to serialize `with` parameter: {e}"))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SinkError::BuildFactory(format!("Failed to deserialize `with` parameter: {e}"))
            })?
        } else {
            return Err(
                SinkError::BuildFactory("Missing required parameter `with`".to_string()).into(),
            );
        };
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = expr_engine.new_scope();
        if let Some(with) = with {
            for (k, v) in with {
                scope.set(k.as_str(), v);
            }
        }
        let output = scope
            .eval::<String>(params.output.to_string().as_str())
            .map_err(|e| SinkError::BuildFactory(e.to_string()))?;
        let output = Uri::from_str(output.as_str())?;
        let sink = GltfWriter {
            output,
            attach_texture: params.attach_texture.unwrap_or(true),
            classified_features: Default::default(),
            draco_compression: params.draco_compression.unwrap_or(false),
        };
        Ok(Box::new(sink))
    }
}

type ClassifiedFeatures = HashMap<String, ClassFeatures>;

#[derive(Debug, Clone)]
struct ClassFeatures {
    feature_type: String,
    features: Vec<ClassFeature>,
    bounding_volume: BoundingVolume,
}

impl AsRef<ClassFeatures> for ClassFeatures {
    fn as_ref(&self) -> &ClassFeatures {
        self
    }
}

impl TryFrom<&ClassFeatures> for nusamai_citygml::schema::Schema {
    type Error = crate::errors::SinkError;

    fn try_from(v: &ClassFeatures) -> Result<Self, Self::Error> {
        let Some(first) = v.features.first() else {
            return Err(SinkError::GltfWriter("No features".to_string()));
        };
        let mut schema = nusamai_citygml::schema::Schema::default();
        let feature_type = v.feature_type.clone();
        let mut attributes = nusamai_citygml::schema::Map::default();
        for (k, v) in first
            .attributes
            .iter()
            .filter(|(_, v)| v.convertible_nusamai_type_ref())
        {
            attributes.insert(k.to_string(), v.clone().into());
        }
        schema.types.insert(
            feature_type,
            nusamai_citygml::schema::TypeDef::Feature(nusamai_citygml::schema::FeatureTypeDef {
                attributes,
                additional_attributes: true,
            }),
        );
        Ok(schema)
    }
}

#[derive(Debug, Clone)]
pub struct GltfWriter {
    output: Uri,
    classified_features: ClassifiedFeatures,
    attach_texture: bool,
    draco_compression: bool,
}

/// # GltfWriter Parameters
///
/// Configuration for writing features to GLTF 3D format.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GltfWriterParam {
    /// Output path or expression for the GLTF file to create
    output: Expr,
    /// Whether to attach texture information to the GLTF model
    attach_texture: Option<bool>,
    draco_compression: Option<bool>,
}

impl Sink for GltfWriter {
    fn name(&self) -> &str {
        "GltfWriter"
    }

    fn process(&mut self, ctx: ExecutorContext) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        match &feature.geometry.value {
            reearth_flow_types::geometry::GeometryValue::CityGmlGeometry(city_gml) => {
                self.process_citygml(city_gml, feature)?;
            }
            reearth_flow_types::geometry::GeometryValue::FlowGeometry3D(geo) => {
                self.process_flow_geometry_3d(geo, feature)?;
            }
            _ => {
                return Err(SinkError::GltfWriter(
                    "Unsupported geometry type. Expected CityGmlGeometry or FlowGeometry3D"
                        .to_string(),
                )
                .into());
            }
        }

        Ok(())
    }

    fn finish(&self, ctx: NodeContext) -> Result<(), BoxedError> {
        let ellipsoid = nusamai_projection::ellipsoid::wgs84();

        let global_bvol = self.compute_global_bounding_volume();
        let transform_matrix = compute_transform_matrix(&global_bvol, &ellipsoid);
        let _ = transform_matrix.inverse();

        let tileset_content_files = Mutex::new(Vec::new());

        self.classified_features
            .par_iter()
            .try_for_each(|(typename, features)| {
                let schema: nusamai_citygml::schema::Schema = features.try_into()?;

                let texture_cache = TextureCache::new(100_000_000);
                let texture_size_cache = TextureSizeCache::new();

                let mut metadata_encoder = MetadataEncoder::new(&schema);

                let binding = tempdir().unwrap();
                let folder_path = binding.path();
                let base_name = typename.replace(':', "_");

                let texture_folder_name = "textures";
                let atlas_dir = folder_path.join(texture_folder_name);
                std::fs::create_dir_all(&atlas_dir).map_err(|e| {
                    crate::errors::SinkError::GltfWriter(format!(
                        "Failed to create directory {atlas_dir:?} with : {e:?}"
                    ))
                })?;

                let packer = Mutex::new(AtlasPacker::default());

                let transformed_features = transform_features_to_local_enu(
                    features.features.clone(),
                    &transform_matrix,
                    &ellipsoid,
                );

                let filtered_features =
                    encode_metadata(&transformed_features, typename, &mut metadata_encoder);

                let (max_width, max_height) = load_textures_into_packer(
                    &filtered_features,
                    &packer,
                    &texture_size_cache,
                    &|feature_id, poly_count| {
                        generate_texture_id(&base_name, feature_id, poly_count)
                    },
                    1.0,   // geom_error (dummy value for non-tiled output)
                    false, // limit_texture_resolution (no downsampling for gltf)
                )?;

                // To reduce unnecessary draw calls, set the lower limit for max_width and max_height to 8192
                let (primitives, vertices) = process_geometry_with_atlas_export(
                    &filtered_features,
                    packer,
                    (max_width.max(8192), max_height.max(8192)),
                    JpegAtlasExporter::default(),
                    &atlas_dir,
                    &texture_cache,
                    |feature_id, poly_count| format!("{base_name}_{feature_id}_{poly_count}"),
                )?;

                let file_path = {
                    let filename = format!("{}.glb", typename.replace(':', "_"));
                    tileset_content_files.lock().unwrap().push(filename.clone());
                    self.output.join(filename).map_err(|e| {
                        crate::errors::SinkError::GltfWriter(format!(
                            "Failed to join uri with {e:?}"
                        ))
                    })?
                };

                let mut buffer = Vec::new();
                let writer = BufWriter::new(&mut buffer);

                reearth_flow_gltf::write_gltf_glb(
                    writer,
                    None,
                    vertices,
                    primitives,
                    filtered_features.len(),
                    metadata_encoder,
                    self.draco_compression,
                )
                .map_err(|e| {
                    crate::errors::SinkError::GltfWriter(format!(
                        "Failed to write_gltf_glb with : {e:?}"
                    ))
                })?;

                let storage = ctx
                    .storage_resolver
                    .resolve(&file_path)
                    .map_err(crate::errors::SinkError::gltf_writer)?;
                storage
                    .put_sync(file_path.as_path().as_path(), bytes::Bytes::from(buffer))
                    .map_err(crate::errors::SinkError::gltf_writer)?;

                Ok::<(), crate::errors::SinkError>(())
            })?;

        Ok(())
    }
}

fn generate_texture_id(folder_name: &str, feature_id: usize, poly_count: usize) -> String {
    format!("{folder_name}_{feature_id}_{poly_count}")
}

fn compute_transform_matrix(
    global_bvol: &BoundingVolume,
    ellipsoid: &nusamai_projection::ellipsoid::Ellipsoid,
) -> DMat4 {
    let center_lng = (global_bvol.min_lng + global_bvol.max_lng) / 2.0;
    let center_lat = (global_bvol.min_lat + global_bvol.max_lat) / 2.0;

    let psi = ((1. - ellipsoid.e_sq()) * center_lat.to_radians().tan()).atan();

    let (tx, ty, tz) = geodetic_to_geocentric(ellipsoid, center_lng, center_lat, 0.);
    let h = (tx * tx + ty * ty + tz * tz).sqrt();

    DMat4::from_translation(DVec3::new(0., -h, 0.))
        * DMat4::from_rotation_x(-(FRAC_PI_2 - psi))
        * DMat4::from_rotation_y((-center_lng - 90.).to_radians())
}

fn transform_features_to_local_enu(
    features: Vec<ClassFeature>,
    transform_matrix: &DMat4,
    ellipsoid: &nusamai_projection::ellipsoid::Ellipsoid,
) -> Vec<ClassFeature> {
    let mut features = features;
    features.iter_mut().for_each(|feature| {
        feature
            .polygons
            .transform_inplace(|&[lng, lat, height, u, v]| {
                // geographic to geocentric
                let (x, y, z) = geodetic_to_geocentric(ellipsoid, lng, lat, height);
                // z-up to y-up
                let v_xyz = DVec4::new(x, z, -y, 1.0);
                // local ENU coordinate
                let v_enu = transform_matrix * v_xyz;

                [v_enu[0], v_enu[1], v_enu[2], u, v]
            });
    });
    features
}

// Helper methods for GltfWriter
impl GltfWriter {
    fn entry(&mut self, feature_type: &str) -> &mut ClassFeatures {
        self.classified_features
            .entry(feature_type.to_string())
            .or_insert_with(|| ClassFeatures {
                feature_type: feature_type.to_string(),
                features: Vec::new(),
                bounding_volume: BoundingVolume::default(),
            })
    }

    fn compute_global_bounding_volume(&self) -> BoundingVolume {
        let mut global_bvol = BoundingVolume::default();
        for features in self.classified_features.values() {
            global_bvol.update(&features.bounding_volume);
        }
        global_bvol
    }

    fn process_citygml(
        &mut self,
        city_gml: &reearth_flow_types::geometry::CityGmlGeometry,
        feature: &reearth_flow_types::Feature,
    ) -> Result<(), BoxedError> {
        let Some(feature_type) = feature.feature_type() else {
            return Err(SinkError::GltfWriter("Feature type is missing".to_string()).into());
        };
        let mut materials: IndexSet<Material> = IndexSet::new();
        let default_material = reearth_flow_types::material::X3DMaterial::default();
        let mut local_bvol = BoundingVolume::default();
        let mut class_feature = ClassFeature {
            polygons: flatgeom::MultiPolygon::new(),
            attributes: feature
                .attributes
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
            polygon_material_ids: Default::default(),
            materials: Default::default(),
        };
        for entry in city_gml.gml_geometries.iter() {
            match entry.ty {
                GeometryType::Solid | GeometryType::Surface | GeometryType::Triangle => {
                    // for each polygon
                    for (((poly, poly_uv), poly_mat), poly_tex) in entry
                        .polygons
                        .iter()
                        .zip_eq_logged(
                            city_gml
                                .polygon_uvs
                                .range(entry.pos as usize..(entry.pos + entry.len) as usize)
                                .into_iter(),
                        )
                        .zip_eq_logged(
                            city_gml.polygon_materials
                                [entry.pos as usize..(entry.pos + entry.len) as usize]
                                .iter(),
                        )
                        .zip_eq_logged(
                            city_gml.polygon_textures
                                [entry.pos as usize..(entry.pos + entry.len) as usize]
                                .iter(),
                        )
                    {
                        let poly: Polygon3 = poly.clone().into();
                        let poly_uv: Polygon2 = poly_uv.into();
                        let mat = if self.attach_texture {
                            let orig_mat = poly_mat
                                .and_then(|idx| city_gml.materials.get(idx as usize))
                                .unwrap_or(&default_material)
                                .clone();
                            let orig_tex =
                                poly_tex.and_then(|idx| city_gml.textures.get(idx as usize));
                            Material {
                                base_color: orig_mat.diffuse_color.into(),
                                base_texture: orig_tex.map(|tex| material::Texture {
                                    uri: tex.uri.clone(),
                                }),
                            }
                        } else {
                            Material {
                                base_color: default_material.diffuse_color.into(),
                                base_texture: None,
                            }
                        };
                        let (mat_idx, _) = materials.insert_full(mat);
                        let mut ring_buffer: Vec<[f64; 5]> = Vec::new();
                        poly.rings()
                            .zip_eq_logged(poly_uv.rings())
                            .enumerate()
                            .for_each(|(ri, (ring, uv_ring))| {
                                ring.iter_closed()
                                    .zip_eq_logged(uv_ring.iter_closed())
                                    .for_each(|(c, uv)| {
                                        let [lng, lat, height] = c;
                                        ring_buffer.push([lng, lat, height, uv[0], uv[1]]);

                                        local_bvol.min_lng = local_bvol.min_lng.min(lng);
                                        local_bvol.max_lng = local_bvol.max_lng.max(lng);
                                        local_bvol.min_lat = local_bvol.min_lat.min(lat);
                                        local_bvol.max_lat = local_bvol.max_lat.max(lat);
                                        local_bvol.min_height = local_bvol.min_height.min(height);
                                        local_bvol.max_height = local_bvol.max_height.max(height);
                                    });
                                if ri == 0 {
                                    class_feature.polygons.add_exterior(ring_buffer.drain(..));
                                    class_feature.polygon_material_ids.push(mat_idx as u32);
                                } else {
                                    class_feature.polygons.add_interior(ring_buffer.drain(..));
                                }
                            });
                    }
                }
                GeometryType::Curve => {
                    unimplemented!()
                }
                GeometryType::Point => {
                    unimplemented!()
                }
            }
        }
        class_feature.materials = materials;
        {
            let feats = self.entry(&feature_type);
            feats.features.push(class_feature);
            feats.bounding_volume.update(&local_bvol);
        }
        Ok(())
    }

    fn process_flow_geometry_3d(
        &mut self,
        geo: &reearth_flow_geometry::types::geometry::Geometry3D<f64>,
        feature: &reearth_flow_types::Feature,
    ) -> Result<(), BoxedError> {
        use reearth_flow_geometry::types::geometry::Geometry3D;

        // Only support Solid, Polygon, and MultiPolygon for now
        match geo {
            Geometry3D::Solid(solid) => {
                self.convert_solid_to_gltf(solid, feature)?;
            }
            Geometry3D::Polygon(polygon) => {
                self.convert_polygon_to_gltf(polygon, feature)?;
            }
            Geometry3D::MultiPolygon(multi_polygon) => {
                for polygon in multi_polygon.iter() {
                    self.convert_polygon_to_gltf(polygon, feature)?;
                }
            }
            _ => {
                return Err(SinkError::GltfWriter(
                    "Only Solid, Polygon, and MultiPolygon are supported for FlowGeometry3D export"
                        .to_string(),
                )
                .into());
            }
        }

        Ok(())
    }

    fn convert_polygon_to_gltf(
        &mut self,
        polygon: &reearth_flow_geometry::types::polygon::Polygon3D<f64>,
        feature: &reearth_flow_types::Feature,
    ) -> Result<(), BoxedError> {
        use flatgeom::Polygon3 as FlatPolygon3;

        let feature_type = feature
            .feature_type()
            .unwrap_or_else(|| "Building".to_string());

        // Convert Polygon3D to flatgeom::Polygon format [x, y, z, u, v]
        let flat_polygon: FlatPolygon3 = polygon.clone().into();

        let mut multi_polygon = flatgeom::MultiPolygon::new();
        let mut local_bvol = BoundingVolume::default();

        // Add exterior ring
        let exterior_coords: Vec<[f64; 5]> = flat_polygon
            .exterior()
            .iter()
            .map(|coord| {
                let [lng, lat, height] = [coord[0], coord[1], coord[2]];
                // Update bounding volume
                local_bvol.min_lng = local_bvol.min_lng.min(lng);
                local_bvol.max_lng = local_bvol.max_lng.max(lng);
                local_bvol.min_lat = local_bvol.min_lat.min(lat);
                local_bvol.max_lat = local_bvol.max_lat.max(lat);
                local_bvol.min_height = local_bvol.min_height.min(height);
                local_bvol.max_height = local_bvol.max_height.max(height);
                [lng, lat, height, 0.0, 0.0]
            })
            .collect();
        multi_polygon.add_exterior(exterior_coords);

        // Add interior rings (holes)
        for interior in flat_polygon.interiors() {
            let interior_coords: Vec<[f64; 5]> = interior
                .iter()
                .map(|coord| {
                    let [lng, lat, height] = [coord[0], coord[1], coord[2]];
                    // Update bounding volume
                    local_bvol.min_lng = local_bvol.min_lng.min(lng);
                    local_bvol.max_lng = local_bvol.max_lng.max(lng);
                    local_bvol.min_lat = local_bvol.min_lat.min(lat);
                    local_bvol.max_lat = local_bvol.max_lat.max(lat);
                    local_bvol.min_height = local_bvol.min_height.min(height);
                    local_bvol.max_height = local_bvol.max_height.max(height);
                    [lng, lat, height, 0.0, 0.0]
                })
                .collect();
            multi_polygon.add_interior(interior_coords);
        }

        // Create default material for the polygon
        let default_material = Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: None,
        };
        let mut materials = IndexSet::new();
        materials.insert(default_material);

        let class_feature = ClassFeature {
            polygons: multi_polygon,
            polygon_material_ids: vec![0], // One material ID for the one polygon
            materials,
            attributes: feature
                .attributes
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
        };

        // Add to classified features and update bounding volume
        let feats = self.entry(&feature_type);
        feats.features.push(class_feature);
        feats.bounding_volume.update(&local_bvol);

        Ok(())
    }

    fn convert_solid_to_gltf(
        &mut self,
        solid: &reearth_flow_geometry::types::solid::Solid3D<f64>,
        feature: &reearth_flow_types::Feature,
    ) -> Result<(), BoxedError> {
        let feature_type = feature
            .feature_type()
            .unwrap_or_else(|| "Building".to_string());

        // Extract all faces from the solid
        let faces = solid.all_faces();

        if faces.is_empty() {
            return Ok(());
        }

        // Track bounding volume across all faces
        let mut local_bvol = BoundingVolume::default();

        // Create a single MultiPolygon containing all faces
        let mut multi_polygon = flatgeom::MultiPolygon::new();
        let mut polygon_count = 0;

        // Convert each face to a polygon and add it to the multi_polygon
        for face in faces.iter() {
            let coords = &face.0;

            if coords.len() < 3 {
                continue; // Skip degenerate faces
            }

            // Convert face coordinates to [x, y, z, u, v] format
            let face_coords: Vec<[f64; 5]> = coords
                .iter()
                .map(|coord| {
                    let lng = coord.x;
                    let lat = coord.y;
                    let height = coord.z;
                    // Update bounding volume
                    local_bvol.min_lng = local_bvol.min_lng.min(lng);
                    local_bvol.max_lng = local_bvol.max_lng.max(lng);
                    local_bvol.min_lat = local_bvol.min_lat.min(lat);
                    local_bvol.max_lat = local_bvol.max_lat.max(lat);
                    local_bvol.min_height = local_bvol.min_height.min(height);
                    local_bvol.max_height = local_bvol.max_height.max(height);
                    [lng, lat, height, 0.0, 0.0]
                })
                .collect();

            multi_polygon.add_exterior(face_coords);
            polygon_count += 1;
        }

        // Create default material
        let default_material = Material {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_texture: None,
        };
        let mut materials = IndexSet::new();
        materials.insert(default_material);

        // Create material IDs - one per face/polygon (all use material 0)
        let polygon_material_ids = vec![0; polygon_count];

        // Create a single ClassFeature for all faces of the solid
        let class_feature = ClassFeature {
            polygons: multi_polygon,
            polygon_material_ids,
            materials,
            attributes: feature
                .attributes
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
        };

        // Add to classified features and update bounding volume
        let feats = self.entry(&feature_type);
        feats.features.push(class_feature);
        feats.bounding_volume.update(&local_bvol);

        Ok(())
    }
}
