extern crate nusamai_gltf;
use super::types::material::{Image, Material, Texture};
use super::types::metadata::MetadataEncoder;
use crate::errors::SinkError;
use ahash::HashSet;
use byteorder::{ByteOrder, LittleEndian};
use indexmap::IndexSet;
use itertools::Itertools;
use nusamai_gltf::nusamai_gltf_json;
use nusamai_gltf::nusamai_gltf_json::extensions;
use nusamai_gltf::nusamai_gltf_json::extensions::mesh::ext_mesh_features;
use nusamai_gltf::nusamai_gltf_json::models::{
    Accessor, AccessorType, Buffer, BufferView, BufferViewTarget, ComponentType, Gltf, Mesh,
    MeshPrimitive, Node, PrimitiveMode, Scene,
};
use nusamai_mvt::TileZXY;
use reearth_flow_common::gltf::{iter_x_slice, iter_y_slice, x_slice_range, y_slice_range};
use reearth_flow_geometry::types::polygon::{Polygon2D, Polygon3D};
use reearth_flow_types::geometry as geomotry_types;
use std::collections::HashMap;
use std::io::Write;
use std::vec;

#[derive(Default)]
pub(super) struct PrimitiveInfo {
    pub indices: Vec<u32>,
    pub feature_ids: HashSet<u32>,
}

pub(super) type Primitives = HashMap<Material, PrimitiveInfo>;

pub(super) fn make_gltf(
    _city_gml: geomotry_types::CityGmlGeometry,
) -> Result<(Gltf, Vec<u8>), SinkError> {
    Err(SinkError::FileWriter("Unsupported input".to_string()))
}

pub(super) fn slice_polygon(
    zoom: u8,
    poly: &Polygon3D<f64>,
    poly_uv: &Polygon2D<f64>,
    mut send_polygon: impl FnMut(TileZXY, &flatgeom::Polygon<'static, [f64; 5]>),
) {
    if poly.exterior().is_empty() {
        return;
    }

    let mut ring_buffer: Vec<[f64; 5]> = Vec::with_capacity(poly.exterior().len() + 1);

    // Slice along Y-axis
    let y_range = {
        let (min_y, max_y) = poly
            .exterior()
            .iter()
            .fold((f64::MAX, f64::MIN), |(min_y, max_y), c| {
                (min_y.min(c.y), max_y.max(c.y))
            });
        iter_y_slice(zoom, min_y, max_y)
    };

    let mut y_sliced_polys = flatgeom::MultiPolygon::new();

    for yi in y_range.clone() {
        let (k1, k2) = y_slice_range(zoom, yi);

        // todo?: check interior bbox to optimize

        for (ri, (ring, uv_ring)) in poly.rings().iter().zip_eq(poly_uv.rings()).enumerate() {
            if ring.coords().collect_vec().is_empty() {
                continue;
            }

            ring_buffer.clear();
            ring.iter()
                .zip_eq(uv_ring.iter())
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

            match ri {
                0 => y_sliced_polys.add_exterior(ring_buffer.drain(..)),
                _ => y_sliced_polys.add_interior(ring_buffer.drain(..)),
            }
        }
    }

    // Slice along X-axis
    let mut poly_buf: flatgeom::Polygon<[f64; 5]> = flatgeom::Polygon::new();
    for (yi, y_sliced_poly) in y_range.zip_eq(y_sliced_polys.iter()) {
        let x_iter = {
            let (min_x, max_x) = y_sliced_poly
                .exterior()
                .iter()
                .fold((f64::MAX, f64::MIN), |(min_x, max_x), c| {
                    (min_x.min(c[0]), max_x.max(c[0]))
                });

            iter_x_slice(zoom, yi, min_x, max_x)
        };

        for (xi, xs) in x_iter {
            let (k1, k2) = x_slice_range(zoom, xi, xs);

            // todo?: check interior bbox to optimize ...

            let key = (
                zoom,
                xi.rem_euclid(1 << zoom) as u32, // handling geometry crossing the antimeridian
                yi,
            );
            poly_buf.clear();

            for ring in y_sliced_poly.rings() {
                if ring.raw_coords().is_empty() {
                    continue;
                }

                ring_buffer.clear();
                ring.iter_closed()
                    .fold(None, |a, b| {
                        let Some(a) = a else { return Some(b) };

                        if a[0] < k1 {
                            if b[0] > k1 {
                                let t = (k1 - a[0]) / (b[0] - a[0]);
                                let y = (b[1] - a[1]) * t + a[1];
                                let z = (b[2] - a[2]) * t + a[2];
                                let u = (b[3] - a[3]) * t + a[3];
                                let v = (b[4] - a[4]) * t + a[4];
                                ring_buffer.push([k1, y, z, u, v])
                            }
                        } else if a[0] > k2 {
                            if b[0] < k2 {
                                let t = (k2 - a[0]) / (b[0] - a[0]);
                                let y = (b[1] - a[1]) * t + a[1];
                                let z = (b[2] - a[2]) * t + a[2];
                                let u = (b[3] - a[3]) * t + a[3];
                                let v = (b[4] - a[4]) * t + a[4];
                                ring_buffer.push([k2, y, z, u, v])
                            }
                        } else {
                            ring_buffer.push(a)
                        }

                        if b[0] < k1 && a[0] > k1 {
                            let t = (k1 - a[0]) / (b[0] - a[0]);
                            let y = (b[1] - a[1]) * t + a[1];
                            let z = (b[2] - a[2]) * t + a[2];
                            let u = (b[3] - a[3]) * t + a[3];
                            let v = (b[4] - a[4]) * t + a[4];
                            ring_buffer.push([k1, y, z, u, v])
                        } else if b[0] > k2 && a[0] < k2 {
                            let t = (k2 - a[0]) / (b[0] - a[0]);
                            let y = (b[1] - a[1]) * t + a[1];
                            let z = (b[2] - a[2]) * t + a[2];
                            let u = (b[3] - a[3]) * t + a[3];
                            let v = (b[4] - a[4]) * t + a[4];
                            ring_buffer.push([k2, y, z, u, v])
                        }

                        Some(b)
                    })
                    .unwrap();

                poly_buf.add_ring(ring_buffer.drain(..))
            }

            send_polygon(key, &poly_buf);
        }
    }
}

pub fn write_gltf_glb<W: Write>(
    writer: W,
    translation: [f64; 3],
    vertices: impl IntoIterator<Item = [u32; 9]>,
    primitives: Primitives,
    num_features: usize,
    metadata_encoder: MetadataEncoder,
) -> Result<(), crate::errors::SinkError> {
    // The buffer for the BIN part
    let mut bin_content: Vec<u8> = Vec::new();
    let mut gltf_buffer_views = vec![];
    let mut gltf_accessors = vec![];

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
            bin_content
                .write_all(&buf)
                .map_err(crate::errors::SinkError::file_writer)?;
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
        for (mat_idx, (mat, primitive)) in primitives.iter().enumerate() {
            let mut indices_count = 0;
            for idx in &primitive.indices {
                bin_content
                    .write_all(&idx.to_le_bytes())
                    .map_err(crate::errors::SinkError::file_writer)?;
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
            // TODO: For no-texture data, it's better to exclude u, v from the vertex buffer
            if mat.base_texture.is_some() {
                attributes.push(("TEXCOORD_0".to_string(), 2));
            }
            attributes.push(("_FEATURE_ID_0".to_string(), 3));

            gltf_primitives.push(MeshPrimitive {
                attributes: attributes.into_iter().collect(),
                indices: Some(gltf_accessors.len() as u32 - 1),
                material: Some(mat_idx as u32), // TODO
                mode: PrimitiveMode::Triangles,
                extensions: extensions::mesh::MeshPrimitive {
                    ext_mesh_features: ext_mesh_features::ExtMeshFeatures {
                        feature_ids: vec![ext_mesh_features::FeatureId {
                            feature_count: num_features as u32, // primitive.feature_ids.len() as u32,
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

    let mut image_set: IndexSet<Image, ahash::RandomState> = Default::default();
    let mut texture_set: IndexSet<Texture, ahash::RandomState> = Default::default();

    // materials
    let gltf_materials = primitives
        .keys()
        .map(|material| material.to_gltf(&mut texture_set))
        .collect();

    let gltf_textures: Vec<_> = texture_set
        .into_iter()
        .map(|t| t.to_gltf(&mut image_set))
        .collect();

    let gltf_images = image_set
        .into_iter()
        .map(|img| {
            img.to_gltf(&mut gltf_buffer_views, &mut bin_content)
                .map_err(SinkError::file_writer)
        })
        .collect::<Result<Vec<nusamai_gltf::nusamai_gltf_json::Image>, SinkError>>()?;

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
        extensions_used: vec![
            "EXT_mesh_features".to_string(),
            "EXT_structural_metadata".to_string(),
        ],
        ..Default::default()
    };

    // Write glb to the writer
    nusamai_gltf::glb::Glb {
        json: serde_json::to_vec(&gltf).unwrap().into(),
        bin: Some(bin_content.into()),
    }
    .to_writer_with_alignment(writer, 8)
    .map_err(SinkError::file_writer)?;

    Ok(())
}
