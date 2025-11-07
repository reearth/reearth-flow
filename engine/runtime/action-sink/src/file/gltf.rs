use std::collections::HashMap;
use std::f64::consts::FRAC_PI_2;
use std::io::BufWriter;
use std::sync::Mutex;
use std::{str::FromStr, sync::Arc};

use ahash::RandomState;
use atlas_packer::export::{AtlasExporter, JpegAtlasExporter};
use atlas_packer::pack::AtlasPacker;
use atlas_packer::place::{GuillotineTexturePlacer, TexturePlacerConfig};
use atlas_packer::texture::cache::{TextureCache, TextureSizeCache};
use atlas_packer::texture::{DownsampleFactor, PolygonMappedTexture};
use earcut::utils3d::project3d_to_2d;
use earcut::Earcut;
use flatgeom::{MultiPolygon, Polygon3};
use glam::{DMat4, DVec3, DVec4};
use indexmap::IndexSet;
use itertools::Itertools;
use nusamai_projection::cartesian::geodetic_to_geocentric;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use reearth_flow_gltf::{calculate_normal, BoundingVolume, MetadataEncoder, Primitives};
use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_runtime::event::EventHub;
use reearth_flow_runtime::executor_operation::{ExecutorContext, NodeContext};
use reearth_flow_runtime::node::{Port, Sink, SinkFactory, DEFAULT_PORT};
use reearth_flow_types::material::{self, Material};
use reearth_flow_types::{AttributeValue, Expr, GeometryType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use reearth_flow_common::uri::Uri;
use serde_json::Value;
use tempfile::tempdir;
use url::Url;

use crate::errors::SinkError;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClassFeature {
    // polygons [x, y, z, u, v]
    pub polygons: MultiPolygon<'static, [f64; 5]>,
    // material ids for each polygon
    pub polygon_material_ids: Vec<u32>,
    // materials
    pub materials: IndexSet<Material>,
    // attribute values
    pub attributes: HashMap<String, AttributeValue>,
    // feature_id
    pub feature_id: Option<u32>,
    // feature type
    pub feature_type: String,
}

type ClassifiedFeatures = HashMap<String, ClassFeatures>;

#[derive(Default, Debug, Clone)]
struct ClassFeatures {
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
        let feature_type = first.feature_type.clone();
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
        // Bounding volume for the entire dataset
        let global_bvol = {
            let mut global_bvol = BoundingVolume::default();
            for features in self.classified_features.values() {
                global_bvol.update(&features.bounding_volume);
            }
            global_bvol
        };

        let tileset_content_files = Mutex::new(Vec::new());

        let transform_matrix = {
            let bounds = &global_bvol;
            let center_lng = (bounds.min_lng + bounds.max_lng) / 2.0;
            let center_lat = (bounds.min_lat + bounds.max_lat) / 2.0;

            let psi = ((1. - ellipsoid.e_sq()) * center_lat.to_radians().tan()).atan();

            let (tx, ty, tz) = geodetic_to_geocentric(&ellipsoid, center_lng, center_lat, 0.);
            let h = (tx * tx + ty * ty + tz * tz).sqrt();

            DMat4::from_translation(DVec3::new(0., -h, 0.))
                * DMat4::from_rotation_x(-(FRAC_PI_2 - psi))
                * DMat4::from_rotation_y((-center_lng - 90.).to_radians())
        };
        let _ = transform_matrix.inverse();

        self.classified_features
            .par_iter()
            .try_for_each(|(typename, features)| {
                let schema: nusamai_citygml::schema::Schema = features.try_into()?;
                // The decoded image file is cached
                let texture_cache = TextureCache::new(100_000_000);
                // The image size is cached to avoid unnecessary decoding
                let texture_size_cache = TextureSizeCache::new();

                let mut vertices: IndexSet<[u32; 9], RandomState> = IndexSet::default(); // [x, y, z, nx, ny, nz, u, v, feature_id]
                let mut primitives: Primitives = Default::default();

                let mut metadata_encoder = MetadataEncoder::new(&schema);

                // Use a temporary directory for embedding in glb.
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

                // Check the size of all the textures and calculate the power of 2 of the largest size
                let mut max_width = 0;
                let mut max_height = 0;
                for feature in features.features.iter() {
                    for (_, orig_mat_id) in feature
                        .polygons
                        .iter()
                        .zip_eq(feature.polygon_material_ids.iter())
                    {
                        let mat = feature.materials[*orig_mat_id as usize].clone();
                        let t = mat.base_texture.clone();
                        if let Some(base_texture) = t {
                            let texture_uri = base_texture.uri.to_file_path().unwrap();
                            let texture_size = texture_size_cache.get_or_insert(&texture_uri);
                            max_width = max_width.max(texture_size.0);
                            max_height = max_height.max(texture_size.1);
                        }
                    }
                }
                let max_width = max_width.next_power_of_two();
                let max_height = max_height.next_power_of_two();

                // initialize texture packer
                // To reduce unnecessary draw calls, set the lower limit for max_width and max_height to 8192
                let config = TexturePlacerConfig {
                    width: max_width.max(8192),
                    height: max_height.max(8192),
                    padding: 0,
                };

                let packer = Mutex::new(AtlasPacker::default());

                // Transform features
                let features = {
                    let mut features = features.features.clone();
                    features.iter_mut().for_each(|feature| {
                        feature
                            .polygons
                            .transform_inplace(|&[lng, lat, height, u, v]| {
                                // geographic to geocentric
                                let (x, y, z) =
                                    geodetic_to_geocentric(&ellipsoid, lng, lat, height);
                                // z-up to y-up
                                let v_xyz = DVec4::new(x, z, -y, 1.0);
                                // local ENU coordinate
                                let v_enu = transform_matrix * v_xyz;

                                [v_enu[0], v_enu[1], v_enu[2], u, v]
                            });
                    });
                    features
                };

                // Encode metadata
                let features = features
                    .iter()
                    .filter(|feature| {
                        metadata_encoder
                            .add_feature(typename, &feature.attributes)
                            .is_ok()
                    })
                    .collect::<Vec<_>>();

                // A unique ID used when planning the atlas layout
                //  and when obtaining the UV coordinates after the layout has been completed
                let generate_texture_id =
                    |folder_name: &str, feature_id: usize, poly_count: usize| {
                        format!("{folder_name}_{feature_id}_{poly_count}")
                    };

                // Load all textures into the Packer
                for (feature_id, feature) in features.iter().enumerate() {
                    for (poly_count, (mat, poly)) in feature
                        .polygons
                        .iter()
                        .zip_eq(feature.polygon_material_ids.iter())
                        .map(move |(poly, orig_mat_id)| {
                            (feature.materials[*orig_mat_id as usize].clone(), poly)
                        })
                        .enumerate()
                    {
                        let t = mat.base_texture.clone();
                        if let Some(base_texture) = t {
                            // texture packing
                            let original_vertices = poly
                                .raw_coords()
                                .iter()
                                .map(|[x, y, z, u, v]| (*x, *y, *z, *u, *v))
                                .collect::<Vec<(f64, f64, f64, f64, f64)>>();

                            let uv_coords = original_vertices
                                .iter()
                                .map(|(_, _, _, u, v)| (*u, *v))
                                .collect::<Vec<(f64, f64)>>();

                            let texture_uri = base_texture.uri.to_file_path().unwrap();
                            let texture_size = texture_size_cache.get_or_insert(&texture_uri);

                            let downsample_scale = 1.0;

                            let downsample_factor = DownsampleFactor::new(&downsample_scale);

                            let texture = PolygonMappedTexture::new(
                                &texture_uri,
                                texture_size,
                                &uv_coords,
                                downsample_factor,
                            );

                            // Unique id required for placement in atlas

                            let texture_id =
                                generate_texture_id(&base_name, feature_id, poly_count);

                            packer.lock().unwrap().add_texture(texture_id, texture);
                        }
                    }
                }

                let placer = GuillotineTexturePlacer::new(config.clone());
                let packer = packer.into_inner().unwrap();

                // Packing the loaded textures into an atlas
                let packed = packer.pack(placer);

                let exporter = JpegAtlasExporter::default();
                let ext = exporter.clone().get_extension().to_string();

                // Obtain the UV coordinates placed in the atlas by specifying the ID
                //  and apply them to the original polygon.
                for (feature_id, feature) in features.iter().enumerate() {
                    for (poly_count, (mut mat, mut poly)) in feature
                        .polygons
                        .iter()
                        .zip_eq(feature.polygon_material_ids.iter())
                        .map(move |(poly, orig_mat_id)| {
                            (feature.materials[*orig_mat_id as usize].clone(), poly)
                        })
                        .enumerate()
                    {
                        let original_vertices = poly
                            .raw_coords()
                            .iter()
                            .map(|[x, y, z, u, v]| (*x, *y, *z, *u, *v))
                            .collect::<Vec<(f64, f64, f64, f64, f64)>>();

                        let texture_id = generate_texture_id(&base_name, feature_id, poly_count);

                        if let Some(info) = packed.get_texture_info(&texture_id) {
                            // Place the texture in the atlas
                            let atlas_placed_uv_coords = info
                                .placed_uv_coords
                                .iter()
                                .map(|(u, v)| ({ *u }, { *v }))
                                .collect::<Vec<(f64, f64)>>();
                            let updated_vertices = original_vertices
                                .iter()
                                .zip(atlas_placed_uv_coords.iter())
                                .map(|((x, y, z, _, _), (u, v))| (*x, *y, *z, *u, *v))
                                .collect::<Vec<(f64, f64, f64, f64, f64)>>();

                            // Apply the UV coordinates placed in the atlas to the original polygon
                            poly.transform_inplace(|&[x, y, z, _, _]| {
                                let (u, v) = updated_vertices
                                    .iter()
                                    .find(|(x_, y_, z_, _, _)| {
                                        (*x_ - x).abs() < 1e-6
                                            && (*y_ - y).abs() < 1e-6
                                            && (*z_ - z).abs() < 1e-6
                                    })
                                    .map(|(_, _, _, u, v)| (*u, *v))
                                    .unwrap();
                                [x, y, z, u, v]
                            });

                            let atlas_file_name = info.atlas_id.to_string();

                            let atlas_uri =
                                atlas_dir.join(atlas_file_name).with_extension(ext.clone());

                            // update material
                            mat = material::Material {
                                base_color: mat.base_color,
                                base_texture: Some(material::Texture {
                                    uri: Url::from_file_path(atlas_uri).unwrap(),
                                }),
                            };
                        }

                        let primitive = primitives.entry(mat).or_default();
                        primitive.feature_ids.insert(feature_id as u32);

                        if let Some((nx, ny, nz)) =
                            calculate_normal(poly.exterior().iter().map(|v| [v[0], v[1], v[2]]))
                        {
                            let num_outer_points = match poly.hole_indices().first() {
                                Some(&v) => v as usize,
                                None => poly.raw_coords().len(),
                            };
                            let mut earcutter = Earcut::new();
                            let mut buf3d: Vec<[f64; 3]> = Vec::new();
                            let mut buf2d: Vec<[f64; 2]> = Vec::new();
                            let mut index_buf: Vec<u32> = Vec::new();

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
                                        // flip the texture v-coordinate
                                        ((1.0 - v) as f32).to_bits(),
                                        (feature_id as f32).to_bits(), // UNSIGNED_INT can't be used for vertex attribute
                                    ];
                                    let (index, _) = vertices.insert_full(vbits);
                                    index as u32
                                }));
                            }
                        }
                    }
                }

                packed.export(
                    exporter,
                    &atlas_dir,
                    &texture_cache,
                    config.width,
                    config.height,
                );

                // Write glTF (.glb)
                let file_path = {
                    let filename = format!("{}.glb", typename.replace(':', "_"));
                    // Save the filename to the content list of the tileset.json (3D Tiles)
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
                    features.len(),
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

// Helper methods for GltfWriter
impl GltfWriter {
    fn process_citygml(
        &mut self,
        city_gml: &reearth_flow_types::geometry::CityGmlGeometry,
        feature: &reearth_flow_types::Feature,
    ) -> Result<(), BoxedError> {
        let Some(feature_type) = feature.metadata.feature_type.clone() else {
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
            feature_id: None,
            feature_type: feature_type.clone(),
        };
        for entry in city_gml.gml_geometries.iter() {
            match entry.ty {
                GeometryType::Solid | GeometryType::Surface | GeometryType::Triangle => {
                    // for each polygon
                    for (((poly, poly_uv), poly_mat), poly_tex) in entry
                        .polygons
                        .iter()
                        .zip_eq(
                            city_gml
                                .polygon_uvs
                                .iter_range(entry.pos as usize..(entry.pos + entry.len) as usize),
                        )
                        .zip_eq(
                            city_gml.polygon_materials
                                [entry.pos as usize..(entry.pos + entry.len) as usize]
                                .iter(),
                        )
                        .zip_eq(
                            city_gml.polygon_textures
                                [entry.pos as usize..(entry.pos + entry.len) as usize]
                                .iter(),
                        )
                    {
                        let poly: Polygon3 = poly.clone().into();
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
                        poly.rings().zip_eq(poly_uv.rings()).enumerate().for_each(
                            |(ri, (ring, uv_ring))| {
                                ring.iter_closed().zip_eq(uv_ring.iter_closed()).for_each(
                                    |(c, uv)| {
                                        let [lng, lat, height] = c;
                                        ring_buffer.push([lng, lat, height, uv[0], uv[1]]);

                                        local_bvol.min_lng = local_bvol.min_lng.min(lng);
                                        local_bvol.max_lng = local_bvol.max_lng.max(lng);
                                        local_bvol.min_lat = local_bvol.min_lat.min(lat);
                                        local_bvol.max_lat = local_bvol.max_lat.max(lat);
                                        local_bvol.min_height = local_bvol.min_height.min(height);
                                        local_bvol.max_height = local_bvol.max_height.max(height);
                                    },
                                );
                                if ri == 0 {
                                    class_feature.polygons.add_exterior(ring_buffer.drain(..));
                                    class_feature.polygon_material_ids.push(mat_idx as u32);
                                } else {
                                    class_feature.polygons.add_interior(ring_buffer.drain(..));
                                }
                            },
                        );
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
            let feats = self.classified_features.entry(feature_type).or_default();
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
            .metadata
            .feature_type
            .clone()
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

        let class_feature = ClassFeature {
            polygons: multi_polygon,
            polygon_material_ids: vec![],
            materials: Default::default(),
            attributes: feature
                .attributes
                .iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect(),
            feature_id: None,
            feature_type: feature_type.clone(),
        };

        // Add to classified features and update bounding volume
        let feats = self.classified_features.entry(feature_type).or_default();
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
            .metadata
            .feature_type
            .clone()
            .unwrap_or_else(|| "Building".to_string());

        // Extract all faces from the solid
        let faces = solid.all_faces();

        // Track bounding volume across all faces
        let mut local_bvol = BoundingVolume::default();

        // Convert each face to a polygon and add it
        for face in faces.iter() {
            // Face is essentially a LineString of coordinates
            // We need to convert it to a flatgeom polygon
            let coords = &face.0;

            if coords.len() < 3 {
                continue; // Skip degenerate faces
            }

            let mut multi_polygon = flatgeom::MultiPolygon::new();

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

            let class_feature = ClassFeature {
                polygons: multi_polygon,
                polygon_material_ids: vec![],
                materials: Default::default(),
                attributes: feature
                    .attributes
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.clone()))
                    .collect(),
                feature_id: None,
                feature_type: feature_type.clone(),
            };

            // Add to classified features
            self.classified_features
                .entry(feature_type.clone())
                .or_default()
                .features
                .push(class_feature);
        }

        // Update bounding volume for the entire solid
        if !faces.is_empty() {
            self.classified_features
                .entry(feature_type)
                .or_default()
                .bounding_volume
                .update(&local_bvol);
        }

        Ok(())
    }
}
