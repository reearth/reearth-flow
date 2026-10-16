//! Decoding of `KHR_draco_mesh_compression` primitives into plain accessors, so
//! the accessor readers in this crate work on Draco-compressed input.

use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt::Display;

use draco_oxide::core::attribute::{Attribute, ComponentDataType};
use draco_oxide::core::types::PointIdx;
use draco_oxide::Mesh;
use gltf::json;
use gltf::json::accessor::{ComponentType, GenericComponentType, Type};
use gltf::json::buffer::Target;
use gltf::json::validation::{Checked, USize64};
use serde_json::Value;

use crate::reader::GltfReaderError;
use crate::utils::write_glb;

const EXTENSION: &str = "KHR_draco_mesh_compression";

fn invalid(what: impl Display) -> GltfReaderError {
    GltfReaderError::Parse(format!("invalid {EXTENSION}: {what}"))
}

/// Whether `gltf` declares `KHR_draco_mesh_compression`. Safe to call before
/// validation.
pub fn uses_draco(gltf: &gltf::Gltf) -> bool {
    gltf.extensions_used().any(|e| e == EXTENSION)
}

/// Decodes every Draco primitive of `gltf`, whose loaded buffers are
/// `buffers`. Decoded data is appended to buffer 0, in `buffers` and in the
/// GLB blob alike. The result is unvalidated; pass it to
/// [`crate::validate_gltf`].
pub fn decode_draco(
    gltf: gltf::Gltf,
    buffers: &mut [Vec<u8>],
) -> Result<gltf::Gltf, GltfReaderError> {
    let mut root = gltf.document.into_json();
    let decoded = decode_root(&mut root, buffers)?;
    let document = gltf::Document::from_json_without_validation(root);
    let blob = match gltf.blob {
        Some(_) if decoded => Some(buffers[0].clone()),
        blob => blob,
    };
    Ok(gltf::Gltf { document, blob })
}

/// Rewrites a GLB with every Draco primitive decoded, or returns `None` when
/// it has none. The compressed buffer views are left in place, unreferenced.
pub fn decompress_glb(glb: &[u8]) -> Result<Option<Vec<u8>>, GltfReaderError> {
    let glb = gltf::binary::Glb::from_slice(glb)
        .map_err(|e| GltfReaderError::Parse(format!("Failed to parse GLB: {e}")))?;
    let mut root: json::Root = serde_json::from_slice(&glb.json)
        .map_err(|e| GltfReaderError::Parse(format!("Failed to parse glTF JSON: {e}")))?;
    let mut buffers = vec![glb.bin.map(Cow::into_owned).unwrap_or_default()];
    if !decode_root(&mut root, &mut buffers)? {
        return Ok(None);
    }
    let json = serde_json::to_vec(&root).expect("glTF JSON is always serializable");
    let [bin] = <[Vec<u8>; 1]>::try_from(buffers).unwrap();
    Ok(Some(write_glb(json, bin)))
}

/// Decodes the Draco primitives of `root` in place; returns whether there were
/// any.
fn decode_root(root: &mut json::Root, buffers: &mut [Vec<u8>]) -> Result<bool, GltfReaderError> {
    let mut compressed = Vec::new();
    for (mi, mesh) in root.meshes.iter_mut().enumerate() {
        for (pi, primitive) in mesh.primitives.iter_mut().enumerate() {
            let Some(extensions) = primitive.extensions.as_mut() else {
                continue;
            };
            let Some(ext) = extensions.others.remove(EXTENSION) else {
                continue;
            };
            // The decoded faces are written as a triangle list.
            if !matches!(primitive.mode, Checked::Valid(json::mesh::Mode::Triangles)) {
                return Err(invalid(format_args!(
                    "mesh {mi} primitive {pi} has mode {:?}; only TRIANGLES is supported",
                    primitive.mode
                )));
            }
            if extensions.others.is_empty() {
                primitive.extensions = None;
            }
            compressed.push((mi, pi, ext));
        }
    }
    if compressed.is_empty() {
        return Ok(false);
    }
    if buffers.is_empty() {
        return Err(invalid("document has no buffer"));
    }

    let mut written = HashSet::new();
    for (mi, pi, ext) in compressed {
        let mesh = decode_mesh(root, buffers, &ext)?;
        let num_points = mesh.attributes.first().map_or(0, Attribute::len);
        let primitive = &root.meshes[mi].primitives[pi];
        let indices_accessor = primitive.indices;
        let attribute_accessors: Vec<_> = primitive
            .attributes
            .iter()
            .map(|(semantic, accessor)| (semantic.to_string(), *accessor))
            .collect();

        let indices: Vec<u8> = mesh
            .faces
            .iter()
            .flatten()
            .flat_map(|&p| (usize::from(p) as u32).to_le_bytes())
            .collect();
        let view = push_view(root, buffers, &indices, None, Target::ElementArrayBuffer);
        let accessor = new_accessor(view, mesh.faces.len() * 3, ComponentType::U32, Type::Scalar);
        match indices_accessor {
            Some(index) => set_accessor(root, &mut written, index, accessor)?,
            None => {
                let index = root.push(accessor);
                root.meshes[mi].primitives[pi].indices = Some(index);
            }
        }

        let mapping = ext
            .get("attributes")
            .and_then(Value::as_object)
            .ok_or_else(|| invalid("attributes"))?;
        for (name, uid) in mapping {
            let uid = uid
                .as_u64()
                .ok_or_else(|| invalid(format_args!("attribute id of {name}")))?
                as usize;
            let attribute = mesh
                .attributes
                .iter()
                .find(|a| a.get_id().as_usize() == uid)
                .ok_or_else(|| invalid(format_args!("no decoded attribute {uid} ({name})")))?;
            if attribute.len() != num_points {
                return Err(invalid(format_args!("{name} does not cover every point")));
            }
            let index = attribute_accessors
                .iter()
                .find(|(semantic, _)| semantic == name)
                .map(|(_, accessor)| *accessor)
                .ok_or_else(|| invalid(format_args!("primitive has no {name} accessor")))?;
            let accessor = attribute_accessor(root, buffers, name, attribute, index)?;
            set_accessor(root, &mut written, index, accessor)?;
        }
    }

    root.buffers[0].byte_length = USize64::from(buffers[0].len());
    for list in [&mut root.extensions_used, &mut root.extensions_required] {
        list.retain(|e| e != EXTENSION);
    }
    Ok(true)
}

fn decode_mesh(
    root: &json::Root,
    buffers: &[Vec<u8>],
    ext: &Value,
) -> Result<Mesh, GltfReaderError> {
    let view = ext
        .get("bufferView")
        .and_then(Value::as_u64)
        .and_then(|i| root.buffer_views.get(i as usize))
        .ok_or_else(|| invalid("bufferView"))?;
    let offset = view.byte_offset.map_or(0, |o| o.0 as usize);
    let bytes = buffers
        .get(view.buffer.value())
        .and_then(|b| b.get(offset..offset + view.byte_length.0 as usize))
        .ok_or_else(|| invalid("bufferView is out of bounds"))?;
    draco_oxide::decode::decode_mesh(bytes).map_err(|e| invalid(format_args!("{e:?}")))
}

/// Writes `attribute`'s per-point values as a new buffer view and returns the
/// accessor describing them. The decoded component type must be the one the
/// placeholder accessor `index` declares, whose `normalized`, `min` and `max`
/// carry over.
fn attribute_accessor(
    root: &mut json::Root,
    buffers: &mut [Vec<u8>],
    name: &str,
    attribute: &Attribute,
    index: json::Index<json::Accessor>,
) -> Result<json::Accessor, GltfReaderError> {
    let declared = root
        .accessors
        .get(index.value())
        .ok_or_else(|| invalid(format_args!("{name} accessor does not exist")))?;
    let (Checked::Valid(GenericComponentType(component_type)), Checked::Valid(type_)) =
        (declared.component_type, declared.type_)
    else {
        return Err(invalid(format_args!("{name} accessor type")));
    };
    let num_components = attribute.get_num_components();
    if type_.multiplicity() != num_components {
        return Err(invalid(format_args!(
            "{name} accessor is {type_:?} but {num_components} components were decoded"
        )));
    }
    let decoded = attribute.get_component_type();
    if gl_component_type(decoded) != Some(component_type) {
        return Err(invalid(format_args!(
            "{name} accessor is {component_type:?} but {decoded:?} was decoded"
        )));
    }

    // Vertex attribute elements must start on 4-byte boundaries.
    let element = component_type.size() * num_components;
    let stride = element.next_multiple_of(4);
    let values = attribute.get_data_as_bytes();
    let mut bytes = vec![0u8; attribute.len() * stride];
    for (p, out) in bytes.chunks_exact_mut(stride).enumerate() {
        let v = usize::from(attribute.get_unique_val_idx(PointIdx::from(p)));
        out[..element].copy_from_slice(&values[v * element..][..element]);
    }

    let stride = (stride != element).then_some(stride);
    let count = attribute.len();
    let view = push_view(root, buffers, &bytes, stride, Target::ArrayBuffer);
    let mut accessor = new_accessor(view, count, component_type, type_);
    let declared = &root.accessors[index.value()];
    accessor.normalized = declared.normalized;
    accessor.min = declared.min.clone();
    accessor.max = declared.max.clone();
    Ok(accessor)
}

fn gl_component_type(t: ComponentDataType) -> Option<ComponentType> {
    match t {
        ComponentDataType::I8 => Some(ComponentType::I8),
        ComponentDataType::U8 => Some(ComponentType::U8),
        ComponentDataType::I16 => Some(ComponentType::I16),
        ComponentDataType::U16 => Some(ComponentType::U16),
        ComponentDataType::U32 => Some(ComponentType::U32),
        ComponentDataType::F32 => Some(ComponentType::F32),
        _ => None,
    }
}

/// Appends `bytes` to buffer 0 as a new buffer view.
fn push_view(
    root: &mut json::Root,
    buffers: &mut [Vec<u8>],
    bytes: &[u8],
    stride: Option<usize>,
    target: Target,
) -> json::Index<json::buffer::View> {
    let buffer = &mut buffers[0];
    buffer.resize(buffer.len().next_multiple_of(4), 0);
    let byte_offset = buffer.len();
    buffer.extend_from_slice(bytes);
    root.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: USize64::from(bytes.len()),
        byte_offset: Some(USize64::from(byte_offset)),
        byte_stride: stride.map(json::buffer::Stride),
        name: None,
        target: Some(Checked::Valid(target)),
        extensions: Default::default(),
        extras: Default::default(),
    })
}

fn new_accessor(
    view: json::Index<json::buffer::View>,
    count: usize,
    component_type: ComponentType,
    type_: Type,
) -> json::Accessor {
    json::Accessor {
        buffer_view: Some(view),
        byte_offset: None,
        count: USize64::from(count),
        component_type: Checked::Valid(GenericComponentType(component_type)),
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        type_: Checked::Valid(type_),
        min: None,
        max: None,
        normalized: false,
        sparse: None,
    }
}

/// Replaces the placeholder accessor `index` with `accessor`, keeping its
/// name, extensions and extras.
fn set_accessor(
    root: &mut json::Root,
    written: &mut HashSet<usize>,
    index: json::Index<json::Accessor>,
    accessor: json::Accessor,
) -> Result<(), GltfReaderError> {
    if !written.insert(index.value()) {
        return Err(invalid(format_args!(
            "accessor {} is shared by several primitives",
            index.value()
        )));
    }
    let target = root
        .accessors
        .get_mut(index.value())
        .ok_or_else(|| invalid(format_args!("accessor {} does not exist", index.value())))?;
    *target = json::Accessor {
        name: target.name.take(),
        extensions: target.extensions.take(),
        extras: std::mem::take(&mut target.extras),
        ..accessor
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use draco_oxide::io::gltf::transcoder::GltfTranscoder;

    const POSITIONS: [[f32; 3]; 6] = [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [5.0, 5.0, 5.0],
        [6.0, 5.0, 5.0],
        [5.0, 6.0, 5.0],
    ];
    const FEATURE_IDS: [f32; 6] = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0];

    /// An uncompressed GLB of two triangles carrying `_FEATURE_ID_0`.
    fn uncompressed_glb() -> Vec<u8> {
        let mut bin = Vec::new();
        bin.extend(POSITIONS.iter().flatten().flat_map(|x| x.to_le_bytes()));
        bin.extend(FEATURE_IDS.iter().flat_map(|x| x.to_le_bytes()));
        bin.extend((0u32..6).flat_map(|i| i.to_le_bytes()));
        let json = serde_json::json!({
            "asset": {"version": "2.0"},
            "buffers": [{"byteLength": bin.len()}],
            "bufferViews": [
                {"buffer": 0, "byteOffset": 0, "byteLength": 72},
                {"buffer": 0, "byteOffset": 72, "byteLength": 24},
                {"buffer": 0, "byteOffset": 96, "byteLength": 24},
            ],
            "accessors": [
                {"bufferView": 0, "componentType": 5126, "count": 6, "type": "VEC3",
                 "min": [0.0, 0.0, 0.0], "max": [6.0, 6.0, 5.0]},
                {"bufferView": 1, "componentType": 5126, "count": 6, "type": "SCALAR"},
                {"bufferView": 2, "componentType": 5125, "count": 6, "type": "SCALAR"},
            ],
            "meshes": [{"primitives": [{
                "attributes": {"POSITION": 0, "_FEATURE_ID_0": 1},
                "indices": 2,
            }]}],
            "nodes": [{"mesh": 0}],
            "scenes": [{"nodes": [0]}],
            "scene": 0,
        });
        write_glb(serde_json::to_vec(&json).unwrap(), bin)
    }

    fn compressed_glb() -> Vec<u8> {
        let (glb, _) = GltfTranscoder::default()
            .transcode_to_glb(&uncompressed_glb())
            .unwrap();
        assert!(String::from_utf8_lossy(&glb).contains(EXTENSION));
        glb
    }

    /// The triangles of the first primitive as (feature id, sorted corners),
    /// ordered by feature id.
    fn triangles(gltf: &gltf::Gltf, buffers: &[Vec<u8>]) -> Vec<(u32, Vec<[f32; 3]>)> {
        let primitive = gltf.meshes().next().unwrap().primitives().next().unwrap();
        let reader = primitive.reader(|b| buffers.get(b.index()).map(Vec::as_slice));
        let positions: Vec<[f32; 3]> = reader.read_positions().unwrap().collect();
        let indices: Vec<u32> = reader.read_indices().unwrap().into_u32().collect();
        let accessor = primitive
            .get(&gltf::Semantic::Extras("FEATURE_ID_0".into()))
            .unwrap();
        let feature_ids = crate::metadata::decode::read_feature_id_accessor(&accessor, buffers)
            .unwrap()
            .unwrap();

        let mut triangles: Vec<_> = indices
            .chunks_exact(3)
            .map(|t| {
                let fid = feature_ids[t[0] as usize];
                assert!(t.iter().all(|&i| feature_ids[i as usize] == fid));
                let mut corners: Vec<_> = t.iter().map(|&i| positions[i as usize]).collect();
                corners.sort_by(|a, b| a.partial_cmp(b).unwrap());
                (fid, corners)
            })
            .collect();
        triangles.sort_by_key(|t| t.0);
        triangles
    }

    fn assert_original_triangles(triangles: &[(u32, Vec<[f32; 3]>)]) {
        assert_eq!(triangles.len(), 2);
        for (t, (fid, corners)) in triangles.iter().enumerate() {
            assert_eq!(*fid as f32, FEATURE_IDS[t * 3]);
            let mut expected = POSITIONS[t * 3..t * 3 + 3].to_vec();
            expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
            for (got, want) in corners.iter().zip(&expected) {
                for c in 0..3 {
                    assert!((got[c] - want[c]).abs() < 1e-2, "{got:?} vs {want:?}");
                }
            }
        }
    }

    #[test]
    fn uncompressed_glb_is_left_alone() {
        assert!(decompress_glb(&uncompressed_glb()).unwrap().is_none());
    }

    #[test]
    fn decode_draco_restores_triangles() {
        let gltf = crate::parse_gltf(&compressed_glb().into()).unwrap();
        let mut buffers = vec![gltf.blob.clone().unwrap()];
        assert!(uses_draco(&gltf));
        let gltf = crate::validate_gltf(decode_draco(gltf, &mut buffers).unwrap()).unwrap();
        assert!(!uses_draco(&gltf));
        assert_original_triangles(&triangles(&gltf, &buffers));
    }

    #[test]
    fn decompress_glb_restores_triangles() {
        let glb = decompress_glb(&compressed_glb()).unwrap().unwrap();
        // The rewritten GLB passes validation as-is.
        let gltf = gltf::Gltf::from_slice(&glb).unwrap();
        let buffers = vec![gltf.blob.clone().unwrap()];
        assert_original_triangles(&triangles(&gltf, &buffers));
    }
}
