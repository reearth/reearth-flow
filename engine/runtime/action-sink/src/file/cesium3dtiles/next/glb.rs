//! Minimal, hand-rolled GLB (binary glTF) emission: one mesh, one untextured
//! primitive, `POSITION` + `indices` only. No material, no normals, no UV — the
//! new-geometry writer's pass-1 scope is geometry only.
//!
//! Deliberately not built on `nusamai-gltf` or the existing `reearth-flow-gltf`
//! crate: both are shaped around materials/metadata this pass doesn't have.
//! This is small enough (glTF 2.0's JSON document + the 12-byte GLB header plus
//! two length-prefixed chunks) to write directly.

use serde_json::json;

const GLB_MAGIC: u32 = 0x46546C67; // "glTF"
const GLB_VERSION: u32 = 2;
const CHUNK_TYPE_JSON: u32 = 0x4E4F_534A; // "JSON"
const CHUNK_TYPE_BIN: u32 = 0x004E_4942; // "BIN\0"

const COMPONENT_TYPE_FLOAT: u32 = 5126;
const COMPONENT_TYPE_UNSIGNED_INT: u32 = 5125;
const TARGET_ARRAY_BUFFER: u32 = 34962;
const TARGET_ELEMENT_ARRAY_BUFFER: u32 = 34963;

/// Build a complete `.glb` byte stream for one mesh.
///
/// `positions` must already be localized (small deltas from some local origin,
/// not raw ECEF — see the module-level note in `next/mod.rs` on why) and cast
/// to `f32`, expressed in the same right-handed, Z-up axes as ECEF.
/// `translation` is that local origin, in full `f64` precision — a glTF node
/// `translation` is a plain JSON number array, so it round-trips exactly
/// regardless of how large the ECEF magnitude is.
///
/// 3D Tiles renderers apply a fixed Y-up-to-Z-up rotation — `(x, y, z) -> (x,
/// -z, y)` — to bare-glTF tile content before placing it via the tile
/// transform (confirmed against a known-good tile in
/// `testing/data/results`: its GLB translation only resolves to a sane
/// geographic position after that rotation is applied). Since the input here
/// is already Z-up (ECEF-relative), this writes the inverse, `(x, y, z) ->
/// (x, z, -y)`, so the renderer's rotation cancels out.
pub(super) fn write(positions: &[[f32; 3]], indices: &[[u32; 3]], translation: [f64; 3]) -> Vec<u8> {
    let gltf_positions: Vec<[f32; 3]> = positions.iter().map(|&[x, y, z]| [x, z, -y]).collect();
    let gltf_translation = [translation[0], translation[2], -translation[1]];

    let mut bin: Vec<u8> = Vec::with_capacity(gltf_positions.len() * 12 + indices.len() * 12);
    for p in &gltf_positions {
        for c in p {
            bin.extend_from_slice(&c.to_le_bytes());
        }
    }
    let positions_byte_length = bin.len();
    for tri in indices {
        for &i in tri {
            bin.extend_from_slice(&i.to_le_bytes());
        }
    }
    let indices_byte_length = bin.len() - positions_byte_length;

    let (min, max) = position_bounds(&gltf_positions);

    let json_doc = json!({
        "asset": {"version": "2.0"},
        "buffers": [{"byteLength": bin.len()}],
        "bufferViews": [
            {
                "buffer": 0,
                "byteOffset": 0,
                "byteLength": positions_byte_length,
                "target": TARGET_ARRAY_BUFFER,
            },
            {
                "buffer": 0,
                "byteOffset": positions_byte_length,
                "byteLength": indices_byte_length,
                "target": TARGET_ELEMENT_ARRAY_BUFFER,
            },
        ],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": COMPONENT_TYPE_FLOAT,
                "count": positions.len(),
                "type": "VEC3",
                "min": min,
                "max": max,
            },
            {
                "bufferView": 1,
                "componentType": COMPONENT_TYPE_UNSIGNED_INT,
                "count": indices.len() * 3,
                "type": "SCALAR",
            },
        ],
        // Explicit, double-sided material: a primitive with no material uses
        // glTF's default material, which has `doubleSided: false` (backface
        // culling on). This writer doesn't verify triangle winding order
        // anywhere upstream, so culling could hide a mesh whose winding ends
        // up reversed; double-siding removes that failure mode entirely.
        "materials": [{"doubleSided": true}],
        "meshes": [{
            "primitives": [{"attributes": {"POSITION": 0}, "indices": 1, "material": 0}],
        }],
        "nodes": [{"mesh": 0, "translation": gltf_translation}],
        "scenes": [{"nodes": [0]}],
        "scene": 0,
    });

    let mut json_bytes = serde_json::to_vec(&json_doc).expect("glTF JSON is always serializable");
    // GLB chunks must be 4-byte aligned; JSON pads with spaces, BIN with zeros.
    while json_bytes.len() % 4 != 0 {
        json_bytes.push(b' ');
    }
    while bin.len() % 4 != 0 {
        bin.push(0);
    }

    let total_len = 12 + 8 + json_bytes.len() + 8 + bin.len();

    let mut out = Vec::with_capacity(total_len);
    out.extend_from_slice(&GLB_MAGIC.to_le_bytes());
    out.extend_from_slice(&GLB_VERSION.to_le_bytes());
    out.extend_from_slice(&(total_len as u32).to_le_bytes());

    out.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(&CHUNK_TYPE_JSON.to_le_bytes());
    out.extend_from_slice(&json_bytes);

    out.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    out.extend_from_slice(&CHUNK_TYPE_BIN.to_le_bytes());
    out.extend_from_slice(&bin);

    out
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
