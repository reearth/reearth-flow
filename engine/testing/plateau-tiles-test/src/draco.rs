//! Decodes `KHR_draco_mesh_compression` primitives in GLB files, so the tile
//! comparisons can read them through plain accessors.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use draco_oxide::core::attribute::{Attribute, ComponentDataType};
use draco_oxide::core::types::PointIdx;
use draco_oxide::io::gltf::glb::{parse_glb, write_glb};
use draco_oxide::Mesh;
use serde_json::{json, Map, Value};

const EXTENSION: &str = "KHR_draco_mesh_compression";
const ARRAY_BUFFER: u32 = 34962;
const ELEMENT_ARRAY_BUFFER: u32 = 34963;
const UNSIGNED_INT: u64 = 5125;
const FLOAT: u64 = 5126;

/// Rewrites the GLB at `path` in place with every Draco primitive decoded.
/// A file without Draco primitives is left untouched.
pub fn decompress_glb_file(path: &Path) -> Result<(), String> {
    let data = fs::read(path).map_err(|e| e.to_string())?;
    if let Some(out) = decompress_glb(&data)? {
        fs::write(path, out).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Returns `data` with every Draco primitive decoded into uncompressed
/// accessors, or `None` when it has no Draco primitive. The compressed
/// buffer views are left in place, unreferenced.
pub fn decompress_glb(data: &[u8]) -> Result<Option<Vec<u8>>, String> {
    let glb = parse_glb(data).map_err(|e| e.to_string())?;
    let mut json: Value = serde_json::from_slice(&glb.json).map_err(|e| e.to_string())?;
    let mut buffer = glb.buffer;

    let mut compressed = Vec::new();
    if let Some(meshes) = json.get_mut("meshes").and_then(Value::as_array_mut) {
        for (mi, mesh) in meshes.iter_mut().enumerate() {
            let Some(primitives) = mesh.get_mut("primitives").and_then(Value::as_array_mut) else {
                continue;
            };
            for (pi, primitive) in primitives.iter_mut().enumerate() {
                let Some(extensions) = primitive
                    .get_mut("extensions")
                    .and_then(Value::as_object_mut)
                else {
                    continue;
                };
                let Some(ext) = extensions.remove(EXTENSION) else {
                    continue;
                };
                if extensions.is_empty() {
                    primitive.as_object_mut().unwrap().remove("extensions");
                }
                compressed.push((mi, pi, ext));
            }
        }
    }
    if compressed.is_empty() {
        return Ok(None);
    }

    let mut written = HashSet::new();
    for (mi, pi, ext) in &compressed {
        let primitive = json["meshes"][mi]["primitives"][pi].clone();
        let mesh = decode_primitive(&json, &buffer, ext)?;
        let num_points = mesh.attributes.first().map_or(0, Attribute::len);

        let indices: Vec<u8> = mesh
            .faces
            .iter()
            .flatten()
            .flat_map(|&p| (usize::from(p) as u32).to_le_bytes())
            .collect();
        let view = push_view(&mut json, &mut buffer, indices, None, ELEMENT_ARRAY_BUFFER);
        let accessor = json!({
            "bufferView": view,
            "componentType": UNSIGNED_INT,
            "count": mesh.faces.len() * 3,
            "type": "SCALAR",
        });
        match primitive["indices"].as_u64() {
            Some(idx) => set_accessor(&mut json, &mut written, idx as usize, accessor)?,
            None => {
                let idx = push_accessor(&mut json, accessor);
                json["meshes"][mi]["primitives"][pi]["indices"] = json!(idx);
            }
        }

        let mapping = ext["attributes"]
            .as_object()
            .ok_or("draco extension has no attributes")?;
        for (name, uid) in mapping {
            let uid = uid.as_u64().ok_or("draco attribute id is not an integer")? as usize;
            let attribute = mesh
                .attributes
                .iter()
                .find(|a| a.get_id().as_usize() == uid)
                .ok_or_else(|| format!("draco stream has no attribute {uid} ({name})"))?;
            if attribute.len() != num_points {
                return Err(format!("attribute {name} does not cover every point"));
            }
            let idx = primitive["attributes"][name]
                .as_u64()
                .ok_or_else(|| format!("primitive has no accessor for {name}"))?
                as usize;
            let declared = json["accessors"][idx].clone();
            let accessor = attribute_accessor(&mut json, &mut buffer, name, attribute, &declared)?;
            set_accessor(&mut json, &mut written, idx, accessor)?;
        }
    }

    json["buffers"][0]["byteLength"] = json!(buffer.len());
    for key in ["extensionsUsed", "extensionsRequired"] {
        let Some(list) = json.get_mut(key).and_then(Value::as_array_mut) else {
            continue;
        };
        list.retain(|e| e != EXTENSION);
        if list.is_empty() {
            json.as_object_mut().unwrap().remove(key);
        }
    }

    let json = serde_json::to_vec(&json).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    write_glb(&mut out, &json, &buffer).map_err(|e| e.to_string())?;
    Ok(Some(out))
}

fn decode_primitive(json: &Value, buffer: &[u8], ext: &Value) -> Result<Mesh, String> {
    let view_idx = ext["bufferView"]
        .as_u64()
        .ok_or("draco extension has no bufferView")? as usize;
    let view = &json["bufferViews"][view_idx];
    if view["buffer"].as_u64() != Some(0) {
        return Err("draco buffer view is not in the GLB binary chunk".into());
    }
    let offset = view["byteOffset"].as_u64().unwrap_or(0) as usize;
    let length = view["byteLength"]
        .as_u64()
        .ok_or("draco buffer view has no byteLength")? as usize;
    let bytes = buffer
        .get(offset..offset + length)
        .ok_or("draco buffer view is out of bounds")?;
    draco_oxide::decode::decode_mesh(bytes).map_err(|e| format!("draco decode failed: {e:?}"))
}

/// Writes `attribute`'s per-point values, converted to the component type of
/// the placeholder accessor `declared`, to a new buffer view and returns the
/// accessor describing them. Draco may store an attribute in another type
/// than the accessor declares (e.g. float feature ids as integers).
fn attribute_accessor(
    json: &mut Value,
    buffer: &mut Vec<u8>,
    name: &str,
    attribute: &Attribute,
    declared: &Value,
) -> Result<Value, String> {
    let gl_component_type = declared["componentType"]
        .as_u64()
        .ok_or_else(|| format!("{name}: accessor has no componentType"))?;
    let size = gl_component_size(gl_component_type)
        .ok_or_else(|| format!("{name}: unsupported componentType {gl_component_type}"))?;
    let gl_type = declared["type"].as_str().unwrap_or_default();
    let num_components = attribute.get_num_components();
    let declared_components = match gl_type {
        "SCALAR" => 1,
        "VEC2" => 2,
        "VEC3" => 3,
        "VEC4" => 4,
        _ => 0,
    };
    if declared_components != num_components {
        return Err(format!(
            "{name}: accessor type {gl_type} does not match {num_components} decoded components"
        ));
    }
    let decoded = attribute.get_component_type();
    let decoded_size = decoded.size();
    if decoded_size == 0 {
        return Err(format!("{name}: decoded component type is invalid"));
    }
    let same_type = gl_component_type_of(decoded) == Some(gl_component_type);
    // Float values bound for a normalized integer accessor are scaled to its range.
    let scale = declared["normalized"].as_bool() == Some(true) && decoded.is_float();

    // Vertex attribute elements must start on 4-byte boundaries.
    let element = size * num_components;
    let stride = element.next_multiple_of(4);
    let values = attribute.get_data_as_bytes();
    let count = attribute.len();
    let mut bytes = vec![0u8; count * stride];
    for (p, out) in bytes.chunks_exact_mut(stride).enumerate() {
        let v = usize::from(attribute.get_unique_val_idx(PointIdx::from(p)));
        let src = &values[v * num_components * decoded_size..][..num_components * decoded_size];
        if same_type {
            out[..element].copy_from_slice(src);
            continue;
        }
        for (c, src) in src.chunks_exact(decoded_size).enumerate() {
            let x = read_component(decoded, src);
            write_component(gl_component_type, scale, x, &mut out[c * size..][..size]);
        }
    }

    let mut accessor = json!({
        "componentType": gl_component_type,
        "count": count,
        "type": gl_type,
    });
    if name == "POSITION" && gl_component_type == FLOAT {
        let (min, max) = f32_bounds(&bytes, stride, num_components);
        accessor["min"] = json!(min);
        accessor["max"] = json!(max);
    }
    let stride = (stride != element).then_some(stride);
    accessor["bufferView"] = json!(push_view(json, buffer, bytes, stride, ARRAY_BUFFER));
    Ok(accessor)
}

fn gl_component_size(gl_component_type: u64) -> Option<usize> {
    match gl_component_type {
        5120 | 5121 => Some(1),
        5122 | 5123 => Some(2),
        5125 | 5126 => Some(4),
        _ => None,
    }
}

fn gl_component_type_of(t: ComponentDataType) -> Option<u64> {
    match t {
        ComponentDataType::I8 => Some(5120),
        ComponentDataType::U8 => Some(5121),
        ComponentDataType::I16 => Some(5122),
        ComponentDataType::U16 => Some(5123),
        ComponentDataType::U32 => Some(5125),
        ComponentDataType::F32 => Some(5126),
        _ => None,
    }
}

fn read_component(t: ComponentDataType, b: &[u8]) -> f64 {
    match t {
        ComponentDataType::I8 => b[0] as i8 as f64,
        ComponentDataType::U8 => b[0] as f64,
        ComponentDataType::I16 => i16::from_le_bytes(b.try_into().unwrap()) as f64,
        ComponentDataType::U16 => u16::from_le_bytes(b.try_into().unwrap()) as f64,
        ComponentDataType::I32 => i32::from_le_bytes(b.try_into().unwrap()) as f64,
        ComponentDataType::U32 => u32::from_le_bytes(b.try_into().unwrap()) as f64,
        ComponentDataType::I64 => i64::from_le_bytes(b.try_into().unwrap()) as f64,
        ComponentDataType::U64 => u64::from_le_bytes(b.try_into().unwrap()) as f64,
        ComponentDataType::F32 => f32::from_le_bytes(b.try_into().unwrap()) as f64,
        ComponentDataType::F64 => f64::from_le_bytes(b.try_into().unwrap()),
        ComponentDataType::Invalid => unreachable!("rejected before conversion"),
    }
}

fn write_component(gl_component_type: u64, scale: bool, x: f64, out: &mut [u8]) {
    let scaled = |max: f64| if scale { (x * max).round() } else { x };
    match gl_component_type {
        5120 => out.copy_from_slice(&(scaled(127.0) as i8).to_le_bytes()),
        5121 => out.copy_from_slice(&(scaled(255.0) as u8).to_le_bytes()),
        5122 => out.copy_from_slice(&(scaled(32767.0) as i16).to_le_bytes()),
        5123 => out.copy_from_slice(&(scaled(65535.0) as u16).to_le_bytes()),
        5125 => out.copy_from_slice(&(x as u32).to_le_bytes()),
        _ => out.copy_from_slice(&(x as f32).to_le_bytes()),
    }
}

/// Component-wise bounds of the values written per point.
fn f32_bounds(bytes: &[u8], stride: usize, num_components: usize) -> (Vec<f32>, Vec<f32>) {
    let mut min = vec![f32::INFINITY; num_components];
    let mut max = vec![f32::NEG_INFINITY; num_components];
    for element in bytes.chunks_exact(stride) {
        for c in 0..num_components {
            let x = f32::from_le_bytes(element[c * 4..c * 4 + 4].try_into().unwrap());
            min[c] = min[c].min(x);
            max[c] = max[c].max(x);
        }
    }
    (min, max)
}

/// Appends `bytes` to the binary chunk as a new buffer view, returning its index.
fn push_view(
    json: &mut Value,
    buffer: &mut Vec<u8>,
    bytes: Vec<u8>,
    stride: Option<usize>,
    target: u32,
) -> usize {
    buffer.resize(buffer.len().next_multiple_of(4), 0);
    let mut view = json!({
        "buffer": 0,
        "byteOffset": buffer.len(),
        "byteLength": bytes.len(),
        "target": target,
    });
    if let Some(stride) = stride {
        view["byteStride"] = json!(stride);
    }
    buffer.extend_from_slice(&bytes);
    let views = json["bufferViews"].as_array_mut().unwrap();
    views.push(view);
    views.len() - 1
}

fn push_accessor(json: &mut Value, accessor: Value) -> usize {
    let accessors = json["accessors"].as_array_mut().unwrap();
    accessors.push(accessor);
    accessors.len() - 1
}

/// Replaces the placeholder accessor `idx` with `accessor`, keeping any
/// fields of the original (name, extras) that the decode does not determine.
fn set_accessor(
    json: &mut Value,
    written: &mut HashSet<usize>,
    idx: usize,
    accessor: Value,
) -> Result<(), String> {
    if !written.insert(idx) {
        return Err(format!(
            "accessor {idx} is shared by several draco primitives"
        ));
    }
    let target = json["accessors"]
        .get_mut(idx)
        .and_then(Value::as_object_mut)
        .ok_or_else(|| format!("accessor {idx} does not exist"))?;
    for key in ["byteOffset", "sparse", "min", "max"] {
        target.remove(key);
    }
    let Value::Object(fields) = accessor else {
        unreachable!()
    };
    target.extend::<Map<String, Value>>(fields);
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
        let json = json!({
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
        let mut out = Vec::new();
        write_glb(&mut out, &serde_json::to_vec(&json).unwrap(), &bin).unwrap();
        out
    }

    #[test]
    fn uncompressed_glb_is_left_alone() {
        assert!(decompress_glb(&uncompressed_glb()).unwrap().is_none());
    }

    #[test]
    fn draco_round_trip_restores_triangles() {
        let (compressed, _) = GltfTranscoder::default()
            .transcode_to_glb(&uncompressed_glb())
            .unwrap();
        assert!(String::from_utf8_lossy(&compressed).contains(EXTENSION));

        let decompressed = decompress_glb(&compressed).unwrap().unwrap();
        let json: Value = serde_json::from_slice(&parse_glb(&decompressed).unwrap().json).unwrap();
        assert!(!json.to_string().contains(EXTENSION));

        let gltf = gltf::Gltf::from_slice(&decompressed).unwrap();
        let blob = gltf.blob.as_deref().unwrap();
        let primitive = gltf.meshes().next().unwrap().primitives().next().unwrap();
        let reader = primitive.reader(|_| Some(blob));
        let positions: Vec<[f32; 3]> = reader.read_positions().unwrap().collect();
        let indices: Vec<u32> = reader.read_indices().unwrap().into_u32().collect();
        let accessor = primitive
            .get(&gltf::Semantic::Extras("FEATURE_ID_0".into()))
            .expect("_FEATURE_ID_0 is missing");
        let view = accessor.view().unwrap();
        let start = view.offset() + accessor.offset();
        let stride = view.stride().unwrap_or(accessor.size());
        // The transcoder may retype feature ids; read them as declared.
        let feature_ids: Vec<f32> = (0..accessor.count())
            .map(|i| {
                let o = start + i * stride;
                match accessor.data_type() {
                    gltf::accessor::DataType::U16 => {
                        u16::from_le_bytes(blob[o..o + 2].try_into().unwrap()) as f32
                    }
                    gltf::accessor::DataType::F32 => {
                        f32::from_le_bytes(blob[o..o + 4].try_into().unwrap())
                    }
                    other => panic!("unexpected feature id type {other:?}"),
                }
            })
            .collect();

        assert_eq!(indices.len(), 6);
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
        triangles.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        for (t, (fid, corners)) in triangles.iter().enumerate() {
            assert_eq!(*fid, FEATURE_IDS[t * 3]);
            let mut expected = POSITIONS[t * 3..t * 3 + 3].to_vec();
            expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
            for (got, want) in corners.iter().zip(&expected) {
                for c in 0..3 {
                    assert!((got[c] - want[c]).abs() < 1e-2, "{got:?} vs {want:?}");
                }
            }
        }
    }
}
