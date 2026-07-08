//! Schema-agnostic GLB (binary glTF) assembly: a thin wrapper over the real
//! `gltf-rs/gltf` crate's `gltf_json` (the base document schema, already a
//! workspace dependency via the reader half of this crate) and
//! `gltf::binary::Glb` (the container). Knows nothing about CityGML,
//! PLATEAU, feature IDs, or any specific glTF extension: a caller pushes one
//! or more primitives, optionally attaches extra per-vertex attributes and
//! already-built extension payloads to whatever it got back, then calls
//! [`Builder::build`]. See `crate::next::metadata` for the feature-processing
//! layer that builds Cesium's metadata extensions on top of this.

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};

use gltf::json;
use gltf::json::validation::{Checked, USize64};

/// A material's PBR metallic-roughness factors; each primitive is untextured
/// and single-material.
pub struct MaterialDesc {
    pub base_color_factor: [f32; 4],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
}

/// Opaque handle to a primitive pushed via [`Builder::push_primitive`], for
/// attaching further attributes/extensions to it before [`Builder::build`].
#[derive(Clone, Copy)]
pub struct PrimitiveHandle(usize);

/// Target for [`Builder::extend`]: the document root, or a specific
/// primitive. `Builder::ROOT` is the only way to name the root; primitive
/// handles come from [`Builder::push_primitive`].
#[derive(Clone, Copy)]
pub enum Handle {
    Root,
    Primitive(PrimitiveHandle),
}

impl From<PrimitiveHandle> for Handle {
    fn from(handle: PrimitiveHandle) -> Self {
        Handle::Primitive(handle)
    }
}

struct Extension {
    name: &'static str,
    value: serde_json::Value,
}

struct PrimitiveBuilder {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
    /// `representative[out_vertex]` is the original (pre-dedup) vertex index
    /// it was first produced from, so [`Builder::set_attribute`] can map a
    /// caller-supplied per-original-vertex array onto the deduped layout
    /// after the fact.
    representative: Vec<u32>,
    material: json::Index<json::Material>,
    extra_attributes: Vec<(json::mesh::Semantic, Vec<u32>)>,
    extensions: Vec<Extension>,
}

/// Accumulates the GLB binary payload alongside the glTF document it
/// describes. `push_buffer_view` is public so a caller can place its own
/// domain-specific bytes (e.g. a property table's string values) in the same
/// shared buffer before referencing them from extension JSON.
#[derive(Default)]
pub struct Builder {
    bin: Vec<u8>,
    root: json::Root,
    primitives: Vec<PrimitiveBuilder>,
    root_extensions: Vec<Extension>,
}

impl Builder {
    pub const ROOT: Handle = Handle::Root;

    pub fn new() -> Self {
        Self::default()
    }

    /// Append `bytes` as a new, 4-byte-padded bufferView; returns its index.
    pub fn push_buffer_view(&mut self, bytes: &[u8]) -> usize {
        self.push_buffer_view_targeted(bytes, None).value()
    }

    fn push_buffer_view_targeted(
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
    fn push_vec3_f32(
        &mut self,
        data: &[[f32; 3]],
        with_bounds: bool,
    ) -> json::Index<json::Accessor> {
        let mut bytes = Vec::with_capacity(data.len() * 12);
        for p in data {
            for c in p {
                bytes.extend_from_slice(&c.to_le_bytes());
            }
        }
        let buffer_view =
            self.push_buffer_view_targeted(&bytes, Some(json::buffer::Target::ArrayBuffer));
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

    /// Append a `SCALAR` u32 accessor (triangle indices or an extra attribute).
    fn push_scalar_u32(
        &mut self,
        data: &[u32],
        target: json::buffer::Target,
    ) -> json::Index<json::Accessor> {
        let mut bytes = Vec::with_capacity(data.len() * 4);
        for &v in data {
            bytes.extend_from_slice(&v.to_le_bytes());
        }
        let buffer_view = self.push_buffer_view_targeted(&bytes, Some(target));
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

    /// Add a triangle-mesh primitive: derives one flat normal per triangle (a
    /// cross product of its own three corners) and splits any vertex shared
    /// by triangles that disagree on it, since no geometry in this pipeline
    /// ever carries source normals and nothing upstream verifies cross-face
    /// winding.
    ///
    /// `positions` must already be in the target coordinate convention and
    /// localized (small deltas from the translation given to
    /// [`Builder::build`], not raw ECEF).
    pub fn push_primitive(
        &mut self,
        positions: &[[f32; 3]],
        indices: &[[u32; 3]],
        material: MaterialDesc,
    ) -> PrimitiveHandle {
        let mut dedup: HashMap<(u32, u32, u32, u32, u32, u32), u32> = HashMap::new();
        let mut out_positions: Vec<[f32; 3]> = Vec::with_capacity(positions.len());
        let mut out_normals: Vec<[f32; 3]> = Vec::with_capacity(positions.len());
        let mut representative: Vec<u32> = Vec::with_capacity(positions.len());
        let mut out_indices: Vec<u32> = Vec::with_capacity(indices.len() * 3);
        for &[i0, i1, i2] in indices {
            let corners = [i0, i1, i2].map(|i| positions[i as usize]);
            let normal = triangle_normal(corners);
            for &orig in &[i0, i1, i2] {
                let p = positions[orig as usize];
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
                    representative.push(orig);
                    (out_positions.len() - 1) as u32
                });
                out_indices.push(idx);
            }
        }

        let material_index = self.root.push(json::Material {
            pbr_metallic_roughness: json::material::PbrMetallicRoughness {
                base_color_factor: json::material::PbrBaseColorFactor(material.base_color_factor),
                metallic_factor: json::material::StrengthFactor(material.metallic_factor),
                roughness_factor: json::material::StrengthFactor(material.roughness_factor),
                ..Default::default()
            },
            ..Default::default()
        });

        self.primitives.push(PrimitiveBuilder {
            positions: out_positions,
            normals: out_normals,
            indices: out_indices,
            representative,
            material: material_index,
            extra_attributes: Vec::new(),
            extensions: Vec::new(),
        });
        PrimitiveHandle(self.primitives.len() - 1)
    }

    /// Attach an extra per-vertex scalar attribute (e.g. a feature ID) to a
    /// primitive pushed earlier. `data` is indexed by *original* vertex index
    /// (as given to [`Builder::push_primitive`]), not the deduped layout.
    pub fn set_attribute(
        &mut self,
        primitive: PrimitiveHandle,
        semantic: json::mesh::Semantic,
        data: &[u32],
    ) {
        let primitive = &mut self.primitives[primitive.0];
        let out_data: Vec<u32> = primitive
            .representative
            .iter()
            .map(|&orig| data[orig as usize])
            .collect();
        primitive.extra_attributes.push((semantic, out_data));
    }

    /// Attach an already-built extension payload to the document root or a
    /// primitive, and register `name` in `extensionsUsed`. This module has
    /// no opinion on what the extension is.
    pub fn extend(
        &mut self,
        handle: impl Into<Handle>,
        name: &'static str,
        value: serde_json::Value,
    ) {
        match handle.into() {
            Handle::Root => self.root_extensions.push(Extension { name, value }),
            Handle::Primitive(handle) => {
                self.primitives[handle.0].extensions.push(Extension { name, value })
            }
        }
    }

    /// Finish assembling the `.glb`: builds accessors for every pushed
    /// primitive, attaches accumulated extensions, and serializes to bytes.
    ///
    /// `translation` carries the document's single node's origin, at full
    /// `f64` precision.
    pub fn build(mut self, translation: [f64; 3]) -> Vec<u8> {
        let primitives = std::mem::take(&mut self.primitives);
        let root_extensions = std::mem::take(&mut self.root_extensions);

        let mut json_primitives = Vec::with_capacity(primitives.len());
        for p in primitives {
            let position_accessor = self.push_vec3_f32(&p.positions, true);
            let normal_accessor = self.push_vec3_f32(&p.normals, false);
            let indices_accessor =
                self.push_scalar_u32(&p.indices, json::buffer::Target::ElementArrayBuffer);

            let mut attributes = BTreeMap::new();
            attributes.insert(
                Checked::Valid(json::mesh::Semantic::Positions),
                position_accessor,
            );
            attributes.insert(
                Checked::Valid(json::mesh::Semantic::Normals),
                normal_accessor,
            );
            for (semantic, data) in p.extra_attributes {
                let accessor = self.push_scalar_u32(&data, json::buffer::Target::ArrayBuffer);
                attributes.insert(Checked::Valid(semantic), accessor);
            }

            let mut prim_ext = json::extensions::mesh::Primitive::default();
            for ext in p.extensions {
                prim_ext.others.insert(ext.name.to_string(), ext.value);
                self.root.extensions_used.push(ext.name.to_string());
            }
            let extensions = (!prim_ext.others.is_empty()).then_some(prim_ext);

            json_primitives.push(json::mesh::Primitive {
                attributes,
                extensions,
                extras: Default::default(),
                indices: Some(indices_accessor),
                material: Some(p.material),
                mode: Checked::Valid(json::mesh::Mode::Triangles),
                targets: None,
            });
        }

        let mut root_ext = json::extensions::root::Root::default();
        for ext in root_extensions {
            root_ext.others.insert(ext.name.to_string(), ext.value);
            self.root.extensions_used.push(ext.name.to_string());
        }
        if !root_ext.others.is_empty() {
            self.root.extensions = Some(root_ext);
        }

        let mesh_index = self.root.push(json::Mesh {
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            primitives: json_primitives,
            weights: None,
        });
        let node_index = self.root.push(json::Node {
            mesh: Some(mesh_index),
            ..Default::default()
        });
        let scene_index = self.root.push(json::Scene {
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            nodes: vec![node_index],
        });
        self.root.scene = Some(scene_index);

        let mut json_value =
            serde_json::to_value(&self.root).expect("glTF JSON is always serializable");
        // `gltf_json::Node::translation` is `[f32; 3]`, too coarse for ECEF-scale
        // offsets (~6.4e6 m, where f32 error is already ~0.5-1 m) — the glTF spec
        // itself has no such precision limit on this field, so patch the real
        // `f64` value back in after serialization rather than truncate it.
        json_value["nodes"][0]["translation"] = serde_json::json!(translation);
        let json_bytes =
            serde_json::to_vec(&json_value).expect("glTF JSON is always serializable");

        let glb = gltf::binary::Glb {
            header: gltf::binary::Header {
                magic: *b"glTF",
                version: 2,
                length: 0, // recomputed by `to_vec`
            },
            json: Cow::Owned(json_bytes),
            bin: Some(Cow::Owned(self.bin)),
        };
        glb.to_vec().expect("GLB binary output is always writable")
    }
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
