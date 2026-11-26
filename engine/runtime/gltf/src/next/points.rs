//! Point clouds as a glTF `POINTS` primitive, with positions optionally
//! quantized to `u16` (`KHR_mesh_quantization`) and the position view
//! optionally compressed with `EXT_meshopt_compression` (vertex codec v0).
//!
//! Each point's feature ID shares the position view, at the first 4-byte
//! boundary after the position, as glTF requires of vertex attributes.

use gltf::json;
use gltf::json::validation::{Checked, USize64};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::glb::{Builder, MaterialDesc, PrimitiveHandle};

const KHR_MESH_QUANTIZATION: &str = "KHR_mesh_quantization";
const EXT_MESHOPT_COMPRESSION: &str = "EXT_meshopt_compression";

/// Quantization error, in metres, when none is given.
pub const DEFAULT_QUANTIZATION_ERROR: f64 = 0.01;

/// Whether point positions are quantized to 16-bit integers, and how precisely.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum PointCloudQuantization {
    /// # Disabled
    /// Write point positions as 32-bit floats.
    Disabled,
    /// # Enabled
    /// Quantize point positions to 16-bit integers.
    #[serde(rename_all = "camelCase")]
    Enabled {
        /// # Quantization Error
        /// Upper bound, in meters, on how far quantization may move a point.
        /// Must be positive. Defaults to 0.01. Where 16 bits cannot cover a
        /// tile's points at this error, the error is raised to what they can.
        #[serde(
            default,
            skip_serializing_if = "Option::is_none",
            deserialize_with = "crate::draco::deserialize_quantization_error"
        )]
        #[schemars(schema_with = "crate::draco::quantization_error_schema")]
        quantization_error: Option<f64>,
    },
}

impl Default for PointCloudQuantization {
    fn default() -> Self {
        Self::Enabled {
            quantization_error: None,
        }
    }
}

impl PointCloudQuantization {
    /// The maximum error, or `None` when disabled.
    fn error(self) -> Option<f64> {
        match self {
            Self::Disabled => None,
            Self::Enabled { quantization_error } => {
                Some(quantization_error.unwrap_or(DEFAULT_QUANTIZATION_ERROR))
            }
        }
    }
}

/// How point positions are encoded.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PointEncoding {
    pub quantization: PointCloudQuantization,
    /// Compress the position view with `EXT_meshopt_compression`.
    pub meshopt: bool,
}

/// One primitive's points.
pub struct Points<'a> {
    /// Positions in glTF axes, relative to the node's translation.
    pub positions: &'a [[f64; 3]],
    /// Per-point feature ID, parallel to `positions`.
    pub feature_ids: &'a [u32],
    /// Number of distinct feature IDs; picks the ID's component type.
    pub feature_count: usize,
    /// Per-point linear RGB as normalized `u16`, parallel to `positions`.
    pub colors: Option<&'a [[u16; 3]]>,
}

/// A pushed points primitive and the node transform its positions need.
pub struct PushedPoints {
    pub handle: PrimitiveHandle,
    /// Added to the node's translation.
    pub offset: [f64; 3],
    /// The node's scale; `None` for unit scale.
    pub scale: Option<[f64; 3]>,
}

/// Component type of the per-point feature ID.
#[derive(Clone, Copy)]
enum IdType {
    U16,
    F32,
}

impl IdType {
    fn for_count(feature_count: usize) -> Self {
        if feature_count <= u16::MAX as usize + 1 {
            IdType::U16
        } else {
            IdType::F32
        }
    }

    fn size(self) -> usize {
        match self {
            IdType::U16 => 2,
            IdType::F32 => 4,
        }
    }

    fn component(self) -> json::accessor::ComponentType {
        match self {
            IdType::U16 => json::accessor::ComponentType::U16,
            IdType::F32 => json::accessor::ComponentType::F32,
        }
    }

    fn write(self, id: u32, out: &mut [u8]) {
        match self {
            IdType::U16 => out.copy_from_slice(&(id as u16).to_le_bytes()),
            IdType::F32 => out.copy_from_slice(&(id as f32).to_le_bytes()),
        }
    }
}

/// Positions in their written form.
enum Positions {
    Quantized {
        values: Vec<[u16; 3]>,
        offset: [f64; 3],
        step: [f32; 3],
    },
    Float(Vec<[f32; 3]>),
}

impl Positions {
    fn size(&self) -> usize {
        match self {
            Positions::Quantized { .. } => 6,
            Positions::Float(_) => 12,
        }
    }

    fn write(&self, i: usize, out: &mut [u8]) {
        match self {
            Positions::Quantized { values, .. } => {
                for (axis, v) in values[i].iter().enumerate() {
                    out[axis * 2..axis * 2 + 2].copy_from_slice(&v.to_le_bytes());
                }
            }
            Positions::Float(values) => {
                for (axis, v) in values[i].iter().enumerate() {
                    out[axis * 4..axis * 4 + 4].copy_from_slice(&v.to_le_bytes());
                }
            }
        }
    }

    fn bounds(&self) -> (serde_json::Value, serde_json::Value) {
        fn fold<T: Copy + PartialOrd>(values: &[[T; 3]]) -> ([T; 3], [T; 3]) {
            let mut min = values[0];
            let mut max = values[0];
            for v in values {
                for axis in 0..3 {
                    if v[axis] < min[axis] {
                        min[axis] = v[axis];
                    }
                    if v[axis] > max[axis] {
                        max[axis] = v[axis];
                    }
                }
            }
            (min, max)
        }
        match self {
            Positions::Quantized { values, .. } => {
                let (min, max) = fold(values);
                (serde_json::json!(min), serde_json::json!(max))
            }
            Positions::Float(values) => {
                let (min, max) = fold(values);
                (serde_json::json!(min), serde_json::json!(max))
            }
        }
    }
}

/// Quantize `positions` to `u16` with at most `error` metres of error per
/// axis, or with the finest step 16 bits allow where that is coarser.
fn quantize(positions: &[[f64; 3]], error: f64) -> Positions {
    let mut min = [f64::INFINITY; 3];
    let mut max = [f64::NEG_INFINITY; 3];
    for p in positions {
        for axis in 0..3 {
            min[axis] = min[axis].min(p[axis]);
            max[axis] = max[axis].max(p[axis]);
        }
    }
    let wanted = 2.0 * error;
    let mut relaxed = 0.0f64;
    let step: [f32; 3] = std::array::from_fn(|axis| {
        let needed = (max[axis] - min[axis]) / u16::MAX as f64;
        if needed > wanted {
            relaxed = relaxed.max(needed / 2.0);
        }
        // Round towards whichever bound decides the step: down to stay within
        // the error, up to still span the extent.
        if needed > wanted {
            let step32 = needed as f32;
            if (step32 as f64) < needed {
                step32.next_up()
            } else {
                step32
            }
        } else {
            let step32 = wanted as f32;
            if (step32 as f64) > wanted {
                step32.next_down()
            } else {
                step32
            }
        }
    });
    if relaxed > 0.0 {
        tracing::warn!(
            "point cloud spans too far for 16-bit positions at {error} m; quantization error \
             raised to {relaxed:.4} m"
        );
    }
    let values = positions
        .iter()
        .map(|p| {
            std::array::from_fn(|axis| {
                ((p[axis] - min[axis]) / step[axis] as f64)
                    .round()
                    .clamp(0.0, u16::MAX as f64) as u16
            })
        })
        .collect();
    Positions::Quantized {
        values,
        offset: min,
        step,
    }
}

/// Encode `bytes`, `stride`-byte vertices, with meshopt vertex codec v0.
fn meshopt_v0(bytes: &[u8], stride: usize) -> Vec<u8> {
    fn encode<const N: usize>(bytes: &[u8]) -> Vec<u8> {
        let vertices = bytes.as_chunks::<N>().0;
        meshopt::encoding::encode_vertex_buffer_level(vertices, 2, Some(0))
            .expect("meshopt vertex encoding never fails on a sized buffer")
    }
    match stride {
        12 => encode::<12>(bytes),
        16 => encode::<16>(bytes),
        _ => unreachable!("point strides are 12 or 16 bytes"),
    }
}

fn accessor(
    view: json::Index<json::buffer::View>,
    byte_offset: usize,
    count: usize,
    component: json::accessor::ComponentType,
    type_: json::accessor::Type,
    normalized: bool,
    bounds: Option<(serde_json::Value, serde_json::Value)>,
) -> json::Accessor {
    let (min, max) = bounds.unzip();
    json::Accessor {
        buffer_view: Some(view),
        byte_offset: Some(USize64::from(byte_offset)),
        count: USize64::from(count),
        component_type: Checked::Valid(json::accessor::GenericComponentType(component)),
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        type_: Checked::Valid(type_),
        min,
        max,
        normalized,
        sparse: None,
    }
}

/// Push `points` as a `POINTS` primitive carrying `POSITION`, `_FEATURE_ID_0`
/// and, if given, `COLOR_0`. The node must be translated by the returned
/// `offset` on top of its origin and scaled by `scale`. `points` must not be
/// empty.
pub fn push_points(
    builder: &mut Builder,
    points: &Points<'_>,
    material: MaterialDesc,
    encoding: PointEncoding,
) -> PushedPoints {
    let count = points.positions.len();
    assert!(count > 0, "a points primitive needs at least one point");
    debug_assert_eq!(points.feature_ids.len(), count);

    let positions = match encoding.quantization.error() {
        Some(error) => quantize(points.positions, error),
        None => Positions::Float(
            points
                .positions
                .iter()
                .map(|p| p.map(|c| c as f32))
                .collect(),
        ),
    };
    let id_type = IdType::for_count(points.feature_count);
    // glTF requires every vertex attribute's byte offset and stride to be
    // multiples of 4.
    let id_offset = positions.size().next_multiple_of(4);
    let stride = (id_offset + id_type.size()).next_multiple_of(4);

    let mut bytes = vec![0u8; count * stride];
    for (i, (vertex, &id)) in bytes
        .chunks_exact_mut(stride)
        .zip(points.feature_ids)
        .enumerate()
    {
        positions.write(i, &mut vertex[..positions.size()]);
        id_type.write(id, &mut vertex[id_offset..id_offset + id_type.size()]);
    }

    let view = if encoding.meshopt {
        let encoded = meshopt_v0(&bytes, stride);
        let byte_offset = builder.push_bytes(&encoded, 4);
        builder.require_extension(EXT_MESHOPT_COMPRESSION);
        builder.push_fallback_view(
            bytes.len(),
            stride,
            EXT_MESHOPT_COMPRESSION,
            serde_json::json!({
                "buffer": 0,
                "byteOffset": byte_offset,
                "byteLength": encoded.len(),
                "byteStride": stride,
                "mode": "ATTRIBUTES",
                "count": count,
            }),
        )
    } else {
        builder.push_vertex_view(&bytes, stride)
    };

    let (component, offset, scale) = match &positions {
        Positions::Quantized { offset, step, .. } => {
            builder.require_extension(KHR_MESH_QUANTIZATION);
            (
                json::accessor::ComponentType::U16,
                *offset,
                Some(step.map(f64::from)),
            )
        }
        Positions::Float(_) => (json::accessor::ComponentType::F32, [0.0; 3], None),
    };
    let position = builder.push_accessor(accessor(
        view,
        0,
        count,
        component,
        json::accessor::Type::Vec3,
        false,
        Some(positions.bounds()),
    ));
    let feature_id = builder.push_accessor(accessor(
        view,
        id_offset,
        count,
        id_type.component(),
        json::accessor::Type::Scalar,
        false,
        None,
    ));
    let mut attributes = vec![
        (json::mesh::Semantic::Positions, position),
        (
            json::mesh::Semantic::Extras("FEATURE_ID_0".to_string()),
            feature_id,
        ),
    ];

    if let Some(colors) = points.colors {
        debug_assert_eq!(colors.len(), count);
        let mut bytes = vec![0u8; count * 8];
        for (vertex, rgb) in bytes.as_chunks_mut::<8>().0.iter_mut().zip(colors) {
            for (c, v) in rgb.iter().enumerate() {
                vertex[c * 2..c * 2 + 2].copy_from_slice(&v.to_le_bytes());
            }
        }
        let view = builder.push_vertex_view(&bytes, 8);
        let color = builder.push_accessor(accessor(
            view,
            0,
            count,
            json::accessor::ComponentType::U16,
            json::accessor::Type::Vec3,
            true,
            None,
        ));
        attributes.push((json::mesh::Semantic::Colors(0), color));
    }

    PushedPoints {
        handle: builder.push_points(attributes, material),
        offset,
        scale,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn material() -> MaterialDesc {
        MaterialDesc {
            base_color_factor: [1.0; 4],
            metallic_factor: 0.0,
            roughness_factor: 1.0,
            base_color_texture: None,
        }
    }

    /// The glb's JSON chunk and BIN chunk.
    fn parse(glb: &[u8]) -> (serde_json::Value, Vec<u8>) {
        let glb = gltf::binary::Glb::from_slice(glb).unwrap();
        (
            serde_json::from_slice(&glb.json).unwrap(),
            glb.bin.unwrap().into_owned(),
        )
    }

    fn sample() -> (Vec<[f64; 3]>, Vec<u32>) {
        let positions: Vec<[f64; 3]> = (0..100)
            .map(|i| {
                let t = i as f64;
                [t * 0.37, (t * 0.11).sin() * 5.0, 100.0 - t * 0.5]
            })
            .collect();
        let ids = (0..100).map(|i| (i / 30) as u32).collect();
        (positions, ids)
    }

    fn build(encoding: PointEncoding, colors: Option<&[[u16; 3]]>) -> (serde_json::Value, Vec<u8>) {
        let (positions, ids) = sample();
        let mut builder = Builder::new();
        let pushed = push_points(
            &mut builder,
            &Points {
                positions: &positions,
                feature_ids: &ids,
                feature_count: 4,
                colors,
            },
            material(),
            encoding,
        );
        parse(&builder.build_scaled(pushed.offset, pushed.scale))
    }

    /// Decode every point back to `f64` through the node transform, and its
    /// feature ID, from a glb holding one uncompressed points primitive.
    fn decode(json: &serde_json::Value, bin: &[u8]) -> Vec<([f64; 3], u32)> {
        let node = &json["nodes"][0];
        let t: Vec<f64> = serde_json::from_value(node["translation"].clone()).unwrap();
        let s: Vec<f64> = node
            .get("scale")
            .map(|s| serde_json::from_value(s.clone()).unwrap())
            .unwrap_or(vec![1.0; 3]);
        let attrs = &json["meshes"][0]["primitives"][0]["attributes"];
        let pos = &json["accessors"][attrs["POSITION"].as_u64().unwrap() as usize];
        let id = &json["accessors"][attrs["_FEATURE_ID_0"].as_u64().unwrap() as usize];
        let view = &json["bufferViews"][pos["bufferView"].as_u64().unwrap() as usize];
        let stride = view["byteStride"].as_u64().unwrap() as usize;
        let base = view["byteOffset"].as_u64().unwrap() as usize;
        let quantized = pos["componentType"] == 5123;
        let id_offset = id["byteOffset"].as_u64().unwrap() as usize;
        (0..pos["count"].as_u64().unwrap() as usize)
            .map(|i| {
                let v = &bin[base + i * stride..base + (i + 1) * stride];
                let p = std::array::from_fn(|a| {
                    let raw = if quantized {
                        u16::from_le_bytes([v[a * 2], v[a * 2 + 1]]) as f64
                    } else {
                        f32::from_le_bytes(v[a * 4..a * 4 + 4].try_into().unwrap()) as f64
                    };
                    raw * s[a] + t[a]
                });
                let id = u16::from_le_bytes([v[id_offset], v[id_offset + 1]]) as u32;
                (p, id)
            })
            .collect()
    }

    #[test]
    fn quantized_points_round_trip_within_the_error() {
        let (json, bin) = build(
            PointEncoding {
                quantization: PointCloudQuantization::Enabled {
                    quantization_error: Some(0.005),
                },
                meshopt: false,
            },
            None,
        );
        assert_eq!(
            json["extensionsRequired"],
            serde_json::json!(["KHR_mesh_quantization"])
        );
        assert_eq!(json["meshes"][0]["primitives"][0]["mode"], 0);
        let (positions, ids) = sample();
        for ((p, id), (want, want_id)) in decode(&json, &bin).iter().zip(positions.iter().zip(&ids))
        {
            for a in 0..3 {
                assert!((p[a] - want[a]).abs() <= 0.005 + 1e-9, "{p:?} vs {want:?}");
            }
            assert_eq!(id, want_id);
        }
        let pos = &json["accessors"][0];
        assert_eq!(pos["min"], serde_json::json!([0, 0, 0]));
    }

    #[test]
    fn unquantized_points_are_floats_with_unit_scale() {
        let (json, bin) = build(
            PointEncoding {
                quantization: PointCloudQuantization::Disabled,
                meshopt: false,
            },
            None,
        );
        assert!(json.get("extensionsRequired").is_none());
        assert!(json["nodes"][0].get("scale").is_none());
        let (positions, ids) = sample();
        for ((p, id), (want, want_id)) in decode(&json, &bin).iter().zip(positions.iter().zip(&ids))
        {
            for a in 0..3 {
                assert!((p[a] - want[a]).abs() < 1e-4);
            }
            assert_eq!(id, want_id);
        }
    }

    #[test]
    fn too_wide_an_extent_relaxes_the_step_to_fit_16_bits() {
        let positions = [[0.0, 0.0, 0.0], [10_000.0, 1.0, 0.0]];
        let Positions::Quantized { values, step, .. } = quantize(&positions, 0.01) else {
            panic!("quantized");
        };
        assert!(step[0] as f64 >= 10_000.0 / 65535.0);
        assert!(step[1] as f64 <= 0.02 && step[1] > 0.0199);
        assert_eq!(values[1][0], u16::MAX);
    }

    #[test]
    fn meshopt_view_decodes_to_the_uncompressed_bytes() {
        let encoding = |meshopt| PointEncoding {
            quantization: PointCloudQuantization::default(),
            meshopt,
        };
        let colors = vec![[65535u16, 0, 1000]; 100];
        let (plain_json, plain_bin) = build(encoding(false), Some(&colors));
        let (json, bin) = build(encoding(true), Some(&colors));

        assert_eq!(
            json["extensionsRequired"],
            serde_json::json!(["EXT_meshopt_compression", "KHR_mesh_quantization"])
        );
        assert_eq!(
            json["buffers"][1]["extensions"]["EXT_meshopt_compression"]["fallback"],
            true
        );
        let view = &json["bufferViews"][0];
        assert_eq!(view["buffer"], 1);
        let ext = &view["extensions"]["EXT_meshopt_compression"];
        assert_eq!(ext["mode"], "ATTRIBUTES");
        let start = ext["byteOffset"].as_u64().unwrap() as usize;
        let len = ext["byteLength"].as_u64().unwrap() as usize;
        let encoded = &bin[start..start + len];
        assert_eq!(meshopt::encoding::decode_vertex_version(encoded), Some(0));
        let decoded: Vec<[u8; 12]> = meshopt::encoding::decode_vertex_buffer(encoded, 100).unwrap();

        let plain_view = &plain_json["bufferViews"][0];
        let plain_start = plain_view["byteOffset"].as_u64().unwrap() as usize;
        assert_eq!(decoded.concat(), plain_bin[plain_start..plain_start + 1200]);

        // Colours stay in an uncompressed BIN view.
        let color = &json["meshes"][0]["primitives"][0]["attributes"]["COLOR_0"];
        let color_view = &json["accessors"][color.as_u64().unwrap() as usize]["bufferView"];
        assert_eq!(
            json["bufferViews"][color_view.as_u64().unwrap() as usize]["buffer"],
            0
        );
    }

    /// glTF requires every vertex attribute's byte offset and stride to be
    /// multiples of 4, in every position and feature-ID layout.
    #[test]
    fn vertex_attributes_are_4_byte_aligned() {
        let (positions, _) = sample();
        for quantization in [
            PointCloudQuantization::default(),
            PointCloudQuantization::Disabled,
        ] {
            for feature_count in [4, 70_000] {
                let ids: Vec<u32> = (0..positions.len() as u32).collect();
                let colors = vec![[0u16; 3]; positions.len()];
                let mut builder = Builder::new();
                let pushed = push_points(
                    &mut builder,
                    &Points {
                        positions: &positions,
                        feature_ids: &ids,
                        feature_count,
                        colors: Some(&colors),
                    },
                    material(),
                    PointEncoding {
                        quantization,
                        meshopt: false,
                    },
                );
                let (json, _) = parse(&builder.build_scaled(pushed.offset, pushed.scale));
                for accessor in json["accessors"].as_array().unwrap() {
                    let offset = accessor["byteOffset"].as_u64().unwrap();
                    assert_eq!(
                        offset % 4,
                        0,
                        "{quantization:?}, {feature_count}: {accessor}"
                    );
                }
                for view in json["bufferViews"].as_array().unwrap() {
                    let stride = view["byteStride"].as_u64().unwrap();
                    assert_eq!(stride % 4, 0, "{quantization:?}, {feature_count}: {view}");
                }
            }
        }
    }

    #[test]
    fn many_features_take_float_ids() {
        assert!(matches!(IdType::for_count(65_536), IdType::U16));
        assert!(matches!(IdType::for_count(65_537), IdType::F32));
    }
}
