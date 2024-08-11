extern crate nusamai_gltf;
use super::metadata::MetadataEncoder;
use crate::errors::SinkError;
use ahash::{HashSet, RandomState};
use byteorder::{ByteOrder, LittleEndian};
use indexmap::IndexSet;
use nusamai_gltf::nusamai_gltf_json;
use nusamai_gltf::nusamai_gltf_json::extensions;
use nusamai_gltf::nusamai_gltf_json::extensions::mesh::ext_mesh_features;
use nusamai_gltf::nusamai_gltf_json::models::{
    Accessor, AccessorType, Buffer, BufferView, BufferViewTarget, ComponentType, Gltf, Image, Mesh,
    MeshPrimitive, Node, PrimitiveMode, Scene,
};
use nusamai_mvt::tileid::TileIdMethod;
use rayon::iter::{ParallelBridge, ParallelIterator};
use reearth_flow_common::uri::Uri;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::geometry as geomotry_types;
use reearth_flow_types::geometry::{Image as geometryImage, Material, Texture};
use reearth_flow_types::Feature;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc;
use std::sync::Arc;
use std::vec;

#[derive(Default)]
pub struct PrimitiveInfo {
    pub indices: Vec<u32>,
    pub feature_ids: HashSet<u32>,
}

pub type Primitives = HashMap<Material, PrimitiveInfo>;

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
    let mut bin_content: Vec<u8> = Vec::new();
    let mut gltf_buffer_views = vec![];
    let mut gltf_accessors = vec![];

    let mut vertices: IndexSet<[u32; 9], RandomState> = IndexSet::default();
    let mut primitives: Primitives = Default::default();

    let metadata_encoder = MetadataEncoder::new();

    let materials = city_gml.materials;
    let features = city_gml.features;
    let polygon_uv = city_gml.polygon_uv;

    let mut u: f64 = 0.0;
    let mut v: f64 = 0.0;

    if let Some(polygon_uv) = polygon_uv {
        polygon_uv.into_iter().for_each(|c| {
            c.exterior().into_iter().for_each(|c| {
                u = c.x;
                v = c.y;
            });
        });
    }

    for (index, feature) in features.iter().enumerate() {
        for poly in feature.polygons.iter() {
            let mat = materials.get(index).unwrap().clone();
            let primitive = primitives.entry(mat).or_default();
            primitive.feature_ids.insert(index as u32);

            if let Some((nx, ny, nz)) =
                calculate_normal(poly.exterior().into_iter().map(|c| [c.x, c.y, c.z]))
            {
                poly.exterior().into_iter().for_each(|c| {
                    let x = c.x;
                    let y = c.y;
                    let z = c.z;
                    let vbits = [
                        (x as f32).to_bits(),
                        (y as f32).to_bits(),
                        (z as f32).to_bits(),
                        (nx as f32).to_bits(),
                        (ny as f32).to_bits(),
                        (nz as f32).to_bits(),
                        (u as f32).to_bits(),
                        (v as f32).to_bits(),
                        (index as f32).to_bits(),
                    ];
                    let (_, _) = vertices.insert_full(vbits);
                });
            }
        }
    }

    // vertices
    {
        let mut vertices_count = 0;
        let mut position_max = [f64::MIN; 3];
        let mut position_min = [f64::MAX; 3];

        const VERTEX_BYTE_STRIDE: usize = 4 * 9; // 4-bytes (f32) x 9

        let buffer_offset = bin_content.len();
        let mut buf = [0; VERTEX_BYTE_STRIDE];
        for v in vertices {
            let [x, y, z, nx, ny, nz, u, v, feature_id] = v;
            position_min = [
                f64::min(position_min[0], f32::from_bits(x) as f64),
                f64::min(position_min[1], f32::from_bits(y) as f64),
                f64::min(position_min[2], f32::from_bits(z) as f64),
            ];
            position_max = [
                f64::max(position_max[0], f32::from_bits(x) as f64),
                f64::max(position_max[1], f32::from_bits(y) as f64),
                f64::max(position_max[2], f32::from_bits(z) as f64),
            ];

            LittleEndian::write_u32_into(&[x, y, z, nx, ny, nz, u, v, feature_id], &mut buf);
            let _ = bin_content.write_all(&buf);
            vertices_count += 1;
        }

        let len_vertices = bin_content.len() - buffer_offset;
        if len_vertices > 0 {
            gltf_buffer_views.push(BufferView {
                name: Some("vertices".to_string()),
                byte_offset: buffer_offset as u32,
                byte_length: len_vertices as u32,
                byte_stride: Some(VERTEX_BYTE_STRIDE as u8),
                target: Some(BufferViewTarget::ArrayBuffer),
                ..Default::default()
            });

            // accessor (positions)
            gltf_accessors.push(Accessor {
                name: Some("positions".to_string()),
                buffer_view: Some(gltf_buffer_views.len() as u32 - 1),
                component_type: ComponentType::Float,
                count: vertices_count,
                min: Some(position_min.to_vec()),
                max: Some(position_max.to_vec()),
                type_: AccessorType::Vec3,
                ..Default::default()
            });

            // accessor (normal)
            gltf_accessors.push(Accessor {
                name: Some("normals".to_string()),
                buffer_view: Some(gltf_buffer_views.len() as u32 - 1),
                byte_offset: 4 * 3,
                component_type: ComponentType::Float,
                count: vertices_count,
                type_: AccessorType::Vec3,
                ..Default::default()
            });

            // accessor (texcoords)
            gltf_accessors.push(Accessor {
                name: Some("texcoords".to_string()),
                buffer_view: Some(gltf_buffer_views.len() as u32 - 1),
                byte_offset: 4 * 6,
                component_type: ComponentType::Float,
                count: vertices_count,
                type_: AccessorType::Vec2,
                ..Default::default()
            });

            // accessor (feature_id)
            gltf_accessors.push(Accessor {
                name: Some("_feature_ids".to_string()),
                buffer_view: Some(gltf_buffer_views.len() as u32 - 1),
                byte_offset: 4 * 8,
                component_type: ComponentType::Float,
                count: vertices_count,
                type_: AccessorType::Scalar,
                ..Default::default()
            });
        }
    }

    let mut gltf_primitives = vec![];

    let structural_metadata =
        metadata_encoder.into_metadata(&mut bin_content, &mut gltf_buffer_views);

    // indices
    {
        let indices_offset = bin_content.len();

        let mut byte_offset = 0;
        for (mat_idx, (_, primitive)) in primitives.iter().enumerate() {
            let mut indices_count = 0;
            for idx in &primitive.indices {
                let _ = bin_content.write_all(&idx.to_le_bytes());
                indices_count += 1;
            }

            gltf_accessors.push(Accessor {
                name: Some("indices".to_string()),
                buffer_view: Some(gltf_buffer_views.len() as u32),
                byte_offset,
                component_type: ComponentType::UnsignedInt,
                count: indices_count,
                type_: AccessorType::Scalar,
                ..Default::default()
            });

            let mut attributes = vec![("POSITION".to_string(), 0), ("NORMAL".to_string(), 1)];
            attributes.push(("_FEATURE_ID_0".to_string(), 3));

            gltf_primitives.push(MeshPrimitive {
                attributes: attributes.into_iter().collect(),
                indices: Some(gltf_accessors.len() as u32 - 1),
                material: Some(mat_idx as u32),
                mode: PrimitiveMode::Triangles,
                extensions: extensions::mesh::MeshPrimitive {
                    ext_mesh_features: ext_mesh_features::ExtMeshFeatures {
                        feature_ids: vec![ext_mesh_features::FeatureId {
                            feature_count: primitive.feature_ids.len() as u32,
                            attribute: Some(0),
                            property_table: Some(0),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }
                    .into(),
                    ..Default::default()
                }
                .into(),
                ..Default::default()
            });

            byte_offset += indices_count * 4;
        }

        let indices_len = bin_content.len() - indices_offset;
        if indices_len > 0 {
            gltf_buffer_views.push(BufferView {
                name: Some("indices".to_string()),
                byte_offset: indices_offset as u32,
                byte_length: indices_len as u32,
                target: Some(BufferViewTarget::ElementArrayBuffer),
                ..Default::default()
            })
        }
    }

    let mut image_set: IndexSet<geometryImage, ahash::RandomState> = Default::default();
    let mut texture_set: IndexSet<Texture, ahash::RandomState> = Default::default();

    // materials
    let gltf_materials = primitives
        .iter()
        .enumerate()
        .map(|(idx, (material, _))| {
            let texture = city_gml.textures.get(idx);
            material.to_gltf(&mut texture_set, texture)
        })
        .collect();

    // textures
    let gltf_textures: Vec<_> = texture_set
        .into_iter()
        .map(|t| t.to_gltf(&mut image_set))
        .collect();

    // images
    let gltf_images = image_set
        .into_iter()
        .map(|img| img.to_gltf(&mut gltf_buffer_views, &mut bin_content))
        .collect::<Result<Vec<Image>, std::io::Error>>()
        .map_err(|e| {
            crate::errors::SinkError::file_writer(format!(
                "Failed to convert image to GLTF: {:?}",
                e
            ))
        })?;

    let mut gltf_meshes = vec![];
    if !gltf_primitives.is_empty() {
        gltf_meshes.push(Mesh {
            primitives: gltf_primitives,
            ..Default::default()
        });
    }

    let gltf_buffers = {
        let mut buffers = vec![];
        if !bin_content.is_empty() {
            buffers.push(Buffer {
                byte_length: bin_content.len() as u32,
                ..Default::default()
            });
        }
        buffers
    };

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

    // Build the JSON part of glTF
    let gltf = Gltf {
        scenes: vec![Scene {
            nodes: Some(vec![0]),
            ..Default::default()
        }],
        nodes: vec![Node {
            mesh: (!primitives.is_empty()).then_some(0),
            translation,
            ..Default::default()
        }],
        meshes: gltf_meshes,
        materials: gltf_materials,
        textures: gltf_textures,
        images: gltf_images,
        accessors: gltf_accessors,
        buffer_views: gltf_buffer_views,
        buffers: gltf_buffers,
        extensions: nusamai_gltf_json::extensions::gltf::Gltf {
            ext_structural_metadata: structural_metadata,
            ..Default::default()
        }
        .into(),
        extensions_used,
        ..Default::default()
    };

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

#[inline]
fn cross((ax, ay, az): (f64, f64, f64), (bx, by, bz): (f64, f64, f64)) -> (f64, f64, f64) {
    (ay * bz - az * by, az * bx - ax * bz, ax * by - ay * bx)
}

pub fn calculate_normal(
    vertex_iter: impl IntoIterator<Item = [f64; 3]>,
) -> Option<(f64, f64, f64)> {
    let mut iter = vertex_iter.into_iter();
    let first = iter.next()?;
    let mut prev = first;

    let mut sum = (0., 0., 0.);

    for data in iter {
        // ..
        let (x, y, z) = (data[0], data[1], data[2]);
        let c = cross(
            (prev[0] - x, prev[1] - y, prev[2] - z),
            (prev[0] + x, prev[1] + y, prev[2] + z),
        );
        sum.0 += c.0;
        sum.1 += c.1;
        sum.2 += c.2;
        prev = [x, y, z];
    }

    {
        let (x, y, z) = (first[0], first[1], first[2]);
        let c = cross(
            (prev[0] - x, prev[1] - y, prev[2] - z),
            (prev[0] + x, prev[1] + y, prev[2] + z),
        );
        sum.0 += c.0;
        sum.1 += c.1;
        sum.2 += c.2;
    }

    match (sum.0 * sum.0 + sum.1 * sum.1 + sum.2 * sum.2).sqrt() {
        d if d < 1e-30 => None,
        d => Some((sum.0 / d, sum.1 / d, sum.2 / d)),
    }
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

fn y_slice_range(z: u8, y: u32) -> (f64, f64) {
    let (_, y_size) = size_for_z(z);
    let y = y as f64;
    let north = 90.0 - 180.0 * y / y_size as f64;
    let south = 90.0 - 180.0 * (y + 1.0) / y_size as f64;
    (south, north)
}

fn x_slice_range(z: u8, x: i32, xs: u32) -> (f64, f64) {
    let (x_size, _) = size_for_z(z);
    let west = -180.0 + 360.0 * x as f64 / x_size as f64;
    let east = -180.0 + 360.0 * (x + xs as i32) as f64 / x_size as f64;
    (west, east)
}

fn x_step(z: u8, y: u32) -> u32 {
    match z {
        0 | 1 => 1,
        _ => {
            let zz = 1 << z;
            if y < zz / 4 {
                u32::max(1, zz / (1 << msb(y))) / 4
            } else {
                u32::max(1, zz / (1 << msb(zz / 2 - y - 1))) / 4
            }
        }
    }
}

fn msb(d: u32) -> u32 {
    u32::BITS - d.leading_zeros()
}

fn size_for_z(z: u8) -> (u32, u32) {
    match z {
        0 => (1, 1),
        1 => (2, 2),
        _ => (1 << z, 1 << (z - 1)),
    }
}
