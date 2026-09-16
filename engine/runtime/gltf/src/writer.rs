use std::io::Write;

use ahash::{HashMap, HashSet};
use byteorder::{ByteOrder, LittleEndian};
use indexmap::IndexSet;
use nusamai_gltf::nusamai_gltf_json::extensions::mesh::ext_mesh_features;
use reearth_flow_types::material;

use crate::metadata::MetadataEncoder;

#[derive(Default)]
pub struct PrimitiveInfo {
    pub indices: Vec<u32>,
    pub feature_ids: HashSet<u32>,
}

pub type Primitives = HashMap<material::Material, PrimitiveInfo>;

/// Draco geometry compression setting for a written glb.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum DracoCompression {
    /// Positions are written uncompressed.
    #[default]
    Disabled,
    /// Positions are compressed. The value bounds how far quantization may move a
    /// vertex, in the unit of the vertex coordinates, and must be positive; `None`
    /// leaves the encoder's default resolution.
    Enabled(Option<f64>),
}

/// Upper bound, in texels of the referenced texture, on how far texture coordinate
/// quantization may move a vertex.
const TEXCOORD_MAX_ERROR_TEXELS: f64 = 0.5;

/// Writes the mesh as a glb, optionally Draco-compressed.
///
/// A position error bound is resolved against the bounding box of all `vertices`, so
/// every primitive in the glb is quantized at the same resolution. `texture_size` is
/// the largest dimension, in pixels, of any texture the `primitives` reference and
/// bounds the texture coordinate quantization; `None` leaves the encoder's default.
#[allow(clippy::too_many_arguments)]
pub fn write_gltf_glb<W: Write>(
    mut writer: W,
    translation: Option<[f64; 3]>,
    vertices: impl IntoIterator<Item = [u32; 9]>,
    primitives: Primitives,
    num_features: usize,
    metadata_encoder: MetadataEncoder,
    draco_compression: DracoCompression,
    texture_size: Option<u32>,
) -> crate::errors::Result<()> {
    use nusamai_gltf::nusamai_gltf_json::*;

    // The buffer for the BIN part
    let mut bin_content: Vec<u8> = Vec::new();
    let mut gltf_buffer_views = vec![];
    let mut gltf_accessors = vec![];

    let mut position_max = [f64::MIN; 3];
    let mut position_min = [f64::MAX; 3];

    // vertices
    {
        let mut vertices_count = 0;

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
                .map_err(crate::errors::Error::writer)?;
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
                    .map_err(crate::errors::Error::writer)?;
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

    let mut image_set: IndexSet<material::Image, ahash::RandomState> = Default::default();
    let mut texture_set: IndexSet<material::Texture, ahash::RandomState> = Default::default();

    // materials
    let gltf_materials = primitives
        .keys()
        .map(|material| material.to_gltf(&mut texture_set))
        .collect();

    // Create a single sampler with proper mipmap filtering
    let gltf_samplers = vec![nusamai_gltf::nusamai_gltf_json::Sampler {
        mag_filter: Some(nusamai_gltf::nusamai_gltf_json::MagFilter::Linear),
        min_filter: Some(nusamai_gltf::nusamai_gltf_json::MinFilter::NearestMipmapLinear),
        wrap_s: nusamai_gltf::nusamai_gltf_json::WrappingMode::Repeat,
        wrap_t: nusamai_gltf::nusamai_gltf_json::WrappingMode::Repeat,
        ..Default::default()
    }];

    let gltf_textures: Vec<_> = texture_set
        .into_iter()
        .map(|t| t.to_gltf(&mut image_set))
        .collect();

    let gltf_images = image_set
        .into_iter()
        .map(|img| {
            img.to_gltf(&mut gltf_buffer_views, &mut bin_content)
                .map_err(crate::errors::Error::writer)
        })
        .collect::<Result<Vec<Image>, crate::errors::Error>>()?;

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
            .is_some_and(|_| true)
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

    // Build the JSON part of glTF
    let gltf = Gltf {
        scenes: vec![Scene {
            nodes: Some(vec![0]),
            ..Default::default()
        }],
        nodes: vec![Node {
            mesh: (!primitives.is_empty()).then_some(0),
            translation: translation.unwrap_or_default(),
            ..Default::default()
        }],
        meshes: gltf_meshes,
        materials: gltf_materials,
        samplers: gltf_samplers,
        textures: gltf_textures,
        images: gltf_images,
        accessors: gltf_accessors,
        buffer_views: gltf_buffer_views,
        buffers: gltf_buffers,
        extensions: nusamai_gltf::nusamai_gltf_json::extensions::gltf::Gltf {
            ext_structural_metadata: structural_metadata,
            ..Default::default()
        }
        .into(),
        extensions_used,
        ..Default::default()
    };

    // Write glb to the writer
    if draco_compression != DracoCompression::Disabled {
        let mut tmp_buffer = Vec::new();
        let indirect_writer = IndirectWriter {
            buffer: &mut tmp_buffer,
        };

        nusamai_gltf::glb::Glb {
            json: serde_json::to_vec(&gltf).unwrap().into(),
            bin: Some(bin_content.into()),
        }
        .to_writer_with_alignment(indirect_writer, 8)
        .map_err(crate::errors::Error::writer)?;
        tmp_buffer.flush()?;

        // Now the glb data is in `tmp_buffer`. We compress it.
        let transcoder = draco_oxide::io::gltf::transcoder::GltfTranscoder::new(
            draco_oxide::io::gltf::transcoder::TranscoderConfig {
                draco: draco_config(draco_compression, texture_size, position_min, position_max),
            },
        );
        let (buff, warnings) = transcoder.transcode_to_glb(&tmp_buffer)?;
        for warning in warnings {
            tracing::warn!("Draco warning: {}", warning);
        }
        writer
            .write_all(&buff)
            .map_err(crate::errors::Error::writer)?;
        writer.flush()?;
    } else {
        nusamai_gltf::glb::Glb {
            json: serde_json::to_vec(&gltf).unwrap().into(),
            bin: Some(bin_content.into()),
        }
        .to_writer_with_alignment(writer, 8)
        .map_err(crate::errors::Error::writer)?;
    }

    Ok(())
}

/// Builds the Draco encoder configuration. A requested position error bound is
/// resolved against the bounding box spanned by `position_min`..`position_max`, and
/// `texture_size` bounds the texture coordinate error to
/// [`TEXCOORD_MAX_ERROR_TEXELS`].
///
/// Draco sizes its lattice so that the quantization *step* stays within the value it
/// is given, while a vertex is snapped to the nearest lattice point and so moves by at
/// most half a step. Each requested bound is therefore doubled on the way in.
///
/// The texture coordinate bound is resolved against each primitive's own coordinate
/// range, so a primitive whose coordinates tile the texture keeps the same accuracy
/// in texels.
fn draco_config(
    draco: DracoCompression,
    texture_size: Option<u32>,
    position_min: [f64; 3],
    position_max: [f64; 3],
) -> draco_oxide::encode::Config {
    use draco_oxide::encode::{AttributeConfig, Quantization};
    use draco_oxide::{AttributeType, ConfigType};

    let mut config = <draco_oxide::encode::Config as ConfigType>::default();

    let position_bound = match draco {
        DracoCompression::Enabled(Some(max_error)) => {
            let box_is_valid = position_min
                .iter()
                .zip(&position_max)
                .all(|(lo, hi)| lo <= hi);
            (max_error > 0.0 && box_is_valid).then_some(max_error)
        }
        _ => None,
    };
    if let Some(max_error) = position_bound {
        let min = position_min.map(|v| v as f32);
        let max = position_max.map(|v| v as f32);
        config = config.with_attribute(
            AttributeType::Position,
            AttributeConfig {
                quantization: Some(Quantization::from_bounding_box(
                    &min,
                    &max,
                    (max_error * 2.0) as f32,
                )),
                ..Default::default()
            },
        );
    }

    if let Some(texture_size) = texture_size.filter(|size| *size > 0) {
        let max_error = 2.0 * TEXCOORD_MAX_ERROR_TEXELS / texture_size as f64;
        config = config.with_attribute(
            AttributeType::TextureCoordinate,
            AttributeConfig {
                quantization: Some(Quantization::MaxError(max_error as f32)),
                ..Default::default()
            },
        );
    }

    config
}

// A struct that writes data to the buffer. The buffer, which is of type `Vec<u8>`, does not die even if
// A struct that writes data to a buffer. The buffer persists even when the writer is dropped.
struct IndirectWriter<'a> {
    buffer: &'a mut Vec<u8>,
}

impl<'a> std::io::Write for IndirectWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.buffer.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use draco_oxide::encode::Quantization;
    use pretty_assertions::assert_eq;

    fn resolved_quantization(
        draco: DracoCompression,
        min: [f64; 3],
        max: [f64; 3],
    ) -> Option<Quantization> {
        draco_config(draco, None, min, max)
            .attribute_config(draco_oxide::AttributeType::Position)
            .quantization
    }

    #[test]
    fn error_bound_resolves_against_the_whole_bounding_box() {
        let quantization = resolved_quantization(
            DracoCompression::Enabled(Some(0.001)),
            [-500.0, -20.0, -500.0],
            [500.0, 80.0, 500.0],
        )
        .unwrap();
        // Draco bounds the step, a vertex moves by at most half of one.
        assert_eq!(
            quantization,
            Quantization::Bounded {
                range: 1000.0,
                max_error: 0.002,
            }
        );

        // A primitive covering only part of the box quantizes no coarser than the bound.
        let bits = quantization.resolve(0.0);
        assert!(100.0 / ((1u64 << bits) - 1) as f32 / 2.0 <= 0.001);
    }

    #[test]
    fn no_error_bound_keeps_the_encoder_default() {
        assert_eq!(
            resolved_quantization(
                DracoCompression::Enabled(None),
                [0.0, 0.0, 0.0],
                [1.0, 1.0, 1.0]
            ),
            None
        );
    }

    fn vertex(position: [f32; 3]) -> [u32; 9] {
        let [x, y, z] = position;
        [
            x.to_bits(),
            y.to_bits(),
            z.to_bits(),
            0f32.to_bits(),
            1f32.to_bits(),
            0f32.to_bits(),
            0f32.to_bits(),
            0f32.to_bits(),
            0f32.to_bits(),
        ]
    }

    /// Decodes the positions of every Draco primitive in a transcoded glb.
    fn decoded_positions(glb: &[u8]) -> Vec<[f32; 3]> {
        use draco_oxide::core::types::{NdVector, Vector};

        let parsed = draco_oxide::io::gltf::glb::parse_glb(glb).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&parsed.json).unwrap();
        let views = json["bufferViews"].as_array().unwrap();

        let mut positions = Vec::new();
        for mesh in json["meshes"].as_array().unwrap() {
            for primitive in mesh["primitives"].as_array().unwrap() {
                let view = primitive["extensions"]["KHR_draco_mesh_compression"]["bufferView"]
                    .as_u64()
                    .unwrap() as usize;
                let offset = views[view]["byteOffset"].as_u64().unwrap_or(0) as usize;
                let length = views[view]["byteLength"].as_u64().unwrap() as usize;
                let decoded =
                    draco_oxide::decode::decode_mesh(&parsed.buffer[offset..offset + length])
                        .unwrap();
                let attribute = decoded
                    .get_attributes()
                    .iter()
                    .find(|a| a.get_attribute_type() == draco_oxide::AttributeType::Position)
                    .unwrap();
                for i in 0..attribute.num_unique_values() {
                    let v: NdVector<3, f32> = attribute.get_unique_val(i.into());
                    positions.push([*v.get(0), *v.get(1), *v.get(2)]);
                }
            }
        }
        positions
    }

    #[test]
    fn error_bound_holds_through_a_glb_round_trip() {
        // Two triangles a kilometer apart, each small enough that its own extent
        // would otherwise buy it a much finer grid than the tile deserves.
        let originals = [
            [-500.0, 0.0, -500.0],
            [-499.0, 0.0, -500.0],
            [-500.0, 0.5, -499.0],
            [500.0, 0.0, 500.0],
            [499.0, 0.0, 500.0],
            [500.0, 0.5, 499.0],
        ];
        let mut primitives = Primitives::default();
        primitives.insert(
            material::Material::default(),
            PrimitiveInfo {
                indices: vec![0, 1, 2, 3, 4, 5],
                feature_ids: Default::default(),
            },
        );

        let schema = nusamai_citygml::schema::Schema::default();
        let mut glb = Vec::new();
        write_gltf_glb(
            &mut glb,
            None,
            originals.iter().map(|p| vertex(*p)),
            primitives,
            1,
            MetadataEncoder::new(&schema),
            DracoCompression::Enabled(Some(0.001)),
            None,
        )
        .unwrap();

        let decoded = decoded_positions(&glb);
        assert_eq!(decoded.len(), originals.len());
        for position in decoded {
            let error = originals
                .iter()
                .map(|o| {
                    ((o[0] - position[0]).powi(2)
                        + (o[1] - position[1]).powi(2)
                        + (o[2] - position[2]).powi(2))
                    .sqrt()
                })
                .fold(f32::INFINITY, f32::min);
            assert!(error <= 0.001, "position error {error} exceeds 1mm");
        }
    }

    #[test]
    fn empty_bounding_box_keeps_the_encoder_default() {
        assert_eq!(
            resolved_quantization(
                DracoCompression::Enabled(Some(0.001)),
                [f64::MAX; 3],
                [f64::MIN; 3]
            ),
            None
        );
    }
}
