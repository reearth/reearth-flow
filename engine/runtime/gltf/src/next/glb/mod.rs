//! GLB (binary glTF) emission: one mesh, one untextured primitive,
//! `POSITION` + `NORMAL` + `indices`, plus an optional `_FEATURE_ID_0` /
//! `EXT_mesh_features` / `EXT_structural_metadata` property table.
//!
//! Built on `gltf_json` (the real, upstream `gltf-rs/gltf` crate, already a
//! workspace dependency via the reader half of this crate) for the base glTF
//! document and `gltf::binary::Glb` for the container. The two Cesium 3D
//! Tiles extensions aren't part of that crate's typed vocabulary, so they're
//! modeled locally in `extensions` and attached through the base crate's
//! generic per-object `extensions` bag.

mod extensions;

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};

use gltf::json;
use gltf::json::validation::{Checked, USize64};
use indexmap::IndexMap;

use self::extensions::{
    ClassProperty, ExtMeshFeatures, ExtStructuralMetadata, FeatureId, MetadataClass,
    MetadataPropertyTable, MetadataPropertyTableProperty, MetadataSchema,
};
use super::metadata::PropertyTable;

const METADATA_SCHEMA_ID: &str = "flow_pass1";
const METADATA_CLASS_NAME: &str = "Feature";

/// Accumulates the GLB binary payload alongside the glTF document it
/// describes, so each attribute is appended in one call instead of the
/// caller hand-tracking byte offsets and accessor indices at each site.
#[derive(Default)]
struct GltfBuffer {
    bin: Vec<u8>,
    root: json::Root,
}

impl GltfBuffer {
    /// Append `bytes` as a new, 4-byte-padded bufferView; returns its index.
    fn push_buffer_view(
        &mut self,
        bytes: &[u8],
        target: Option<json::buffer::Target>,
    ) -> json::Index<json::buffer::View> {
        let byte_offset = self.bin.len();
        self.bin.extend_from_slice(bytes);
        pad_to_4(&mut self.bin);
        self.root.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: USize64::from(bytes.len()),
            byte_offset: Some(USize64::from(byte_offset)),
            byte_stride: None,
            name: None,
            target: target.map(Checked::Valid),
            extensions: Default::default(),
            extras: Default::default(),
        })
    }

    /// Append a `VEC3` f32 accessor (positions or normals); `with_bounds`
    /// also records `min`/`max` (glTF requires this on `POSITION`).
    fn push_vec3_f32(&mut self, data: &[[f32; 3]], with_bounds: bool) -> json::Index<json::Accessor> {
        let mut bytes = Vec::with_capacity(data.len() * 12);
        for p in data {
            for c in p {
                bytes.extend_from_slice(&c.to_le_bytes());
            }
        }
        let buffer_view = self.push_buffer_view(&bytes, Some(json::buffer::Target::ArrayBuffer));
        let (min, max) = if with_bounds {
            let (min, max) = position_bounds(data);
            (Some(serde_json::json!(min)), Some(serde_json::json!(max)))
        } else {
            (None, None)
        };
        self.root.push(json::Accessor {
            buffer_view: Some(buffer_view),
            byte_offset: None,
            count: USize64::from(data.len()),
            component_type: Checked::Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            type_: Checked::Valid(json::accessor::Type::Vec3),
            min,
            max,
            normalized: false,
            sparse: None,
        })
    }

    /// Append a `SCALAR` u32 accessor (triangle indices or feature IDs).
    fn push_scalar_u32(
        &mut self,
        data: &[u32],
        target: json::buffer::Target,
    ) -> json::Index<json::Accessor> {
        let mut bytes = Vec::with_capacity(data.len() * 4);
        for &v in data {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        let buffer_view = self.push_buffer_view(&bytes, Some(target));
        self.root.push(json::Accessor {
            buffer_view: Some(buffer_view),
            byte_offset: None,
            count: USize64::from(data.len()),
            component_type: Checked::Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::U32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            type_: Checked::Valid(json::accessor::Type::Scalar),
            min: None,
            max: None,
            normalized: false,
            sparse: None,
        })
    }
}

/// Build a complete `.glb` byte stream for one mesh.
///
/// `positions` must be localized (small deltas from `translation`, not raw
/// ECEF) and cast to `f32`; `translation` carries the local origin at full
/// `f64` precision. `feature_ids[i]` is the `metadata` row for vertex `i`,
/// ignored when `metadata` has no properties.
pub fn write(
    positions: &[[f32; 3]],
    indices: &[[u32; 3]],
    translation: [f64; 3],
    feature_ids: &[u32],
    metadata: &PropertyTable,
) -> Vec<u8> {
    // 3D Tiles renderers rotate bare-glTF content Y-up -> Z-up on load; our
    // input is already Z-up (ECEF-relative), so pre-apply the inverse here
    // and the renderer's rotation cancels out.
    let gltf_positions: Vec<[f32; 3]> = positions.iter().map(|&[x, y, z]| [x, z, -y]).collect();
    let gltf_translation = [translation[0], translation[2], -translation[1]];

    // No source normals and no cross-face winding guarantee, so derive one flat
    // normal per triangle and split any vertex shared by triangles that disagree
    // on it (Cesium doesn't compute this itself).
    let has_feature_ids = !metadata.properties.is_empty();
    let mut dedup: HashMap<(u32, u32, u32, u32, u32, u32), u32> = HashMap::new();
    let mut out_positions: Vec<[f32; 3]> = Vec::with_capacity(gltf_positions.len());
    let mut out_normals: Vec<[f32; 3]> = Vec::with_capacity(gltf_positions.len());
    let mut out_feature_ids: Vec<u32> = Vec::with_capacity(feature_ids.len());
    let mut out_indices: Vec<u32> = Vec::with_capacity(indices.len() * 3);
    for &[i0, i1, i2] in indices {
        let corners = [i0, i1, i2].map(|i| gltf_positions[i as usize]);
        let normal = triangle_normal(corners);
        for &orig in &[i0, i1, i2] {
            let p = gltf_positions[orig as usize];
            let key = (
                p[0].to_bits(),
                p[1].to_bits(),
                p[2].to_bits(),
                normal[0].to_bits(),
                normal[1].to_bits(),
                normal[2].to_bits(),
            );
            let idx = *dedup.entry(key).or_insert_with(|| {
                out_positions.push(p);
                out_normals.push(normal);
                if has_feature_ids {
                    out_feature_ids.push(feature_ids[orig as usize]);
                }
                (out_positions.len() - 1) as u32
            });
            out_indices.push(idx);
        }
    }

    let mut buf = GltfBuffer::default();
    let position_accessor = buf.push_vec3_f32(&out_positions, true);
    let normal_accessor = buf.push_vec3_f32(&out_normals, false);
    let indices_accessor =
        buf.push_scalar_u32(&out_indices, json::buffer::Target::ElementArrayBuffer);

    let mut attributes = BTreeMap::new();
    attributes.insert(
        Checked::Valid(json::mesh::Semantic::Positions),
        position_accessor,
    );
    attributes.insert(
        Checked::Valid(json::mesh::Semantic::Normals),
        normal_accessor,
    );

    let mut primitive_extensions: Option<json::extensions::mesh::Primitive> = None;

    if !metadata.properties.is_empty() {
        // `_FEATURE_ID_0`: one row index per vertex, parallel to POSITION.
        let feature_ids_accessor =
            buf.push_scalar_u32(&out_feature_ids, json::buffer::Target::ArrayBuffer);
        attributes.insert(
            Checked::Valid(json::mesh::Semantic::Extras("FEATURE_ID_0".to_string())),
            feature_ids_accessor,
        );

        // One STRING property per column: raw UTF-8 bytes in `values`,
        // cumulative byte offsets in `stringOffsets` (EXT_structural_metadata's
        // variable-length-array encoding). These are raw bufferViews, not
        // accessors: the extension references them by bufferView index directly.
        let mut class_properties = IndexMap::new();
        let mut table_properties = IndexMap::new();
        for (col, (raw_name, id)) in metadata.properties.iter().enumerate() {
            class_properties.insert(
                id.clone(),
                ClassProperty {
                    name: raw_name.clone(),
                    type_: "STRING",
                },
            );

            let mut value_bytes = Vec::new();
            let mut offsets: Vec<u32> = vec![0];
            for row in &metadata.rows {
                value_bytes.extend_from_slice(row[col].as_bytes());
                offsets.push(value_bytes.len() as u32);
            }
            let values_bufferview = buf.push_buffer_view(&value_bytes, None);
            let offset_bytes: Vec<u8> = offsets.iter().flat_map(|o| o.to_le_bytes()).collect();
            let offsets_bufferview = buf.push_buffer_view(&offset_bytes, None);

            table_properties.insert(
                id.clone(),
                MetadataPropertyTableProperty {
                    values: values_bufferview.value(),
                    string_offset_type: "UINT32",
                    string_offsets: offsets_bufferview.value(),
                },
            );
        }

        let mut classes = IndexMap::new();
        classes.insert(
            METADATA_CLASS_NAME,
            MetadataClass {
                properties: class_properties,
            },
        );
        let ext_structural_metadata = ExtStructuralMetadata {
            schema: MetadataSchema {
                id: METADATA_SCHEMA_ID,
                classes,
            },
            property_tables: vec![MetadataPropertyTable {
                class: METADATA_CLASS_NAME,
                count: metadata.rows.len(),
                properties: table_properties,
            }],
        };
        let mut root_ext = json::extensions::root::Root::default();
        root_ext.others.insert(
            "EXT_structural_metadata".to_string(),
            serde_json::to_value(&ext_structural_metadata)
                .expect("EXT_structural_metadata is always serializable"),
        );
        buf.root.extensions = Some(root_ext);
        buf.root
            .extensions_used
            .push("EXT_structural_metadata".to_string());

        let mut prim_ext = json::extensions::mesh::Primitive::default();
        prim_ext.others.insert(
            "EXT_mesh_features".to_string(),
            serde_json::to_value(&ExtMeshFeatures {
                feature_ids: vec![FeatureId {
                    feature_count: metadata.rows.len(),
                    attribute: 0,
                    property_table: 0,
                }],
            })
            .expect("EXT_mesh_features is always serializable"),
        );
        primitive_extensions = Some(prim_ext);
        buf.root
            .extensions_used
            .push("EXT_mesh_features".to_string());
    }

    let material_index = buf.root.push(json::Material {
        pbr_metallic_roughness: json::material::PbrMetallicRoughness {
            // No appearance data is read yet, so every feature would otherwise
            // get the glTF spec's white default, making adjacent buildings
            // visually merge together. Flat gray (matching the old writer's
            // X3DMaterial default) keeps features distinguishable until real
            // appearance support lands.
            base_color_factor: json::material::PbrBaseColorFactor([0.7, 0.7, 0.7, 1.0]),
            metallic_factor: json::material::StrengthFactor(0.0),
            roughness_factor: json::material::StrengthFactor(0.9),
            ..Default::default()
        },
        ..Default::default()
    });

    let primitive = json::mesh::Primitive {
        attributes,
        extensions: primitive_extensions,
        extras: Default::default(),
        indices: Some(indices_accessor),
        material: Some(material_index),
        mode: Checked::Valid(json::mesh::Mode::Triangles),
        targets: None,
    };
    let mesh_index = buf.root.push(json::Mesh {
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        primitives: vec![primitive],
        weights: None,
    });
    let node_index = buf.root.push(json::Node {
        mesh: Some(mesh_index),
        ..Default::default()
    });
    let scene_index = buf.root.push(json::Scene {
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        nodes: vec![node_index],
    });
    buf.root.scene = Some(scene_index);

    let mut json_value =
        serde_json::to_value(&buf.root).expect("glTF JSON is always serializable");
    // `gltf_json::Node::translation` is `[f32; 3]`, too coarse for ECEF-scale
    // offsets (~6.4e6 m, where f32 error is already ~0.5-1 m) — the glTF spec
    // itself has no such precision limit on this field, so patch the real
    // `f64` value back in after serialization rather than truncate it.
    json_value["nodes"][0]["translation"] = serde_json::json!(gltf_translation);
    let json_bytes = serde_json::to_vec(&json_value).expect("glTF JSON is always serializable");

    let glb = gltf::binary::Glb {
        header: gltf::binary::Header {
            magic: *b"glTF",
            version: 2,
            length: 0, // recomputed by `to_vec`
        },
        json: Cow::Owned(json_bytes),
        bin: Some(Cow::Owned(buf.bin)),
    };
    glb.to_vec().expect("GLB binary output is always writable")
}

/// Unit normal of the plane through three points, via cross product; `[0, 0, 1]`
/// for a degenerate (near-zero-area) triangle.
fn triangle_normal([a, b, c]: [[f32; 3]; 3]) -> [f32; 3] {
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len < 1e-12 {
        [0.0, 0.0, 1.0]
    } else {
        [n[0] / len, n[1] / len, n[2] / len]
    }
}

fn pad_to_4(buf: &mut Vec<u8>) {
    while !buf.len().is_multiple_of(4) {
        buf.push(0);
    }
}

/// glTF requires an accessor's `min`/`max`; compute them directly rather than
/// pulling in a bounding-box dependency for two floats per axis.
fn position_bounds(positions: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];
    for p in positions {
        for i in 0..3 {
            min[i] = min[i].min(p[i]);
            max[i] = max[i].max(p[i]);
        }
    }
    (min, max)
}
