//! Minimal, hand-rolled GLB (binary glTF) emission: one mesh, one untextured
//! primitive, `POSITION` + `indices`, plus an optional per-vertex
//! `_FEATURE_ID_0` and its `EXT_mesh_features` / `EXT_structural_metadata`
//! property table (`metadata::PropertyTable`) when the tile's features carry
//! any attributes. No material, no normals, no UV — the new-geometry
//! writer's pass-1 scope is geometry (+ this minimal metadata) only.
//!
//! Deliberately not built on `nusamai-gltf` or the existing `reearth-flow-gltf`
//! crate: both are shaped around a materials/metadata model this pass doesn't
//! use. This is small enough (glTF 2.0's JSON document + the 12-byte GLB
//! header plus two length-prefixed chunks) to write directly.

use serde_json::json;

use super::metadata::PropertyTable;

const GLB_MAGIC: u32 = 0x46546C67; // "glTF"
const GLB_VERSION: u32 = 2;
const CHUNK_TYPE_JSON: u32 = 0x4E4F_534A; // "JSON"
const CHUNK_TYPE_BIN: u32 = 0x004E_4942; // "BIN\0"

const COMPONENT_TYPE_FLOAT: u32 = 5126;
const COMPONENT_TYPE_UNSIGNED_INT: u32 = 5125;
const TARGET_ARRAY_BUFFER: u32 = 34962;
const TARGET_ELEMENT_ARRAY_BUFFER: u32 = 34963;

// `EXT_structural_metadata` requires a schema id; this writer has exactly one
// implicit class (no per-feature-type classing yet, see `metadata.rs`), so
// both are fixed constants rather than derived per tile.
const METADATA_SCHEMA_ID: &str = "flow_pass1";
const METADATA_CLASS_NAME: &str = "Feature";

/// Build a complete `.glb` byte stream for one mesh.
///
/// `positions` must already be localized (small deltas from some local origin,
/// not raw ECEF — see the module-level note in `next/mod.rs` on why) and cast
/// to `f32`, expressed in the same right-handed, Z-up axes as ECEF.
/// `translation` is that local origin, in full `f64` precision — a glTF node
/// `translation` is a plain JSON number array, so it round-trips exactly
/// regardless of how large the ECEF magnitude is. `feature_ids[i]` is the row
/// of `metadata` that vertex `i` belongs to; ignored when `metadata` has no
/// properties (nothing worth tagging feature ids against).
///
/// 3D Tiles renderers apply a fixed Y-up-to-Z-up rotation — `(x, y, z) -> (x,
/// -z, y)` — to bare-glTF tile content before placing it via the tile
/// transform (confirmed against a known-good tile in
/// `testing/data/results`: its GLB translation only resolves to a sane
/// geographic position after that rotation is applied). Since the input here
/// is already Z-up (ECEF-relative), this writes the inverse, `(x, y, z) ->
/// (x, z, -y)`, so the renderer's rotation cancels out.
pub(super) fn write(
    positions: &[[f32; 3]],
    indices: &[[u32; 3]],
    translation: [f64; 3],
    feature_ids: &[u32],
    metadata: &PropertyTable,
) -> Vec<u8> {
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

    let mut buffer_views = vec![
        json!({
            "buffer": 0, "byteOffset": 0, "byteLength": positions_byte_length,
            "target": TARGET_ARRAY_BUFFER,
        }),
        json!({
            "buffer": 0, "byteOffset": positions_byte_length, "byteLength": indices_byte_length,
            "target": TARGET_ELEMENT_ARRAY_BUFFER,
        }),
    ];
    let mut accessors = vec![
        json!({
            "bufferView": 0, "componentType": COMPONENT_TYPE_FLOAT, "count": positions.len(),
            "type": "VEC3", "min": min, "max": max,
        }),
        json!({
            "bufferView": 1, "componentType": COMPONENT_TYPE_UNSIGNED_INT,
            "count": indices.len() * 3, "type": "SCALAR",
        }),
    ];

    let mut primitive_attributes = json!({"POSITION": 0});
    let mut primitive_extensions = None;
    let mut root_extensions = None;

    if !metadata.properties.is_empty() {
        // `_FEATURE_ID_0`: one row index per vertex, parallel to POSITION.
        let feature_ids_bufferview = buffer_views.len();
        let feature_ids_byte_offset = bin.len();
        for &id in feature_ids {
            bin.extend_from_slice(&id.to_le_bytes());
        }
        pad_to_4(&mut bin);
        buffer_views.push(json!({
            "buffer": 0, "byteOffset": feature_ids_byte_offset,
            "byteLength": feature_ids.len() * 4, "target": TARGET_ARRAY_BUFFER,
        }));
        let feature_ids_accessor = accessors.len();
        accessors.push(json!({
            "bufferView": feature_ids_bufferview, "componentType": COMPONENT_TYPE_UNSIGNED_INT,
            "count": feature_ids.len(), "type": "SCALAR",
        }));
        primitive_attributes["_FEATURE_ID_0"] = json!(feature_ids_accessor);

        // One STRING property per column: raw UTF-8 bytes in a `values`
        // bufferView, `count + 1` cumulative byte offsets in a parallel
        // `stringOffsets` one (the variable-length-array encoding
        // `EXT_structural_metadata` uses for string columns).
        let mut schema_properties = serde_json::Map::new();
        let mut table_properties = serde_json::Map::new();
        for (col, (raw_name, id)) in metadata.properties.iter().enumerate() {
            schema_properties.insert(id.clone(), json!({"name": raw_name, "type": "STRING"}));

            let mut value_bytes = Vec::new();
            let mut offsets: Vec<u32> = vec![0];
            for row in &metadata.rows {
                value_bytes.extend_from_slice(row[col].as_bytes());
                offsets.push(value_bytes.len() as u32);
            }

            let values_bufferview = buffer_views.len();
            let values_byte_offset = bin.len();
            bin.extend_from_slice(&value_bytes);
            pad_to_4(&mut bin);
            buffer_views.push(json!({
                "buffer": 0, "byteOffset": values_byte_offset, "byteLength": value_bytes.len(),
            }));

            let offsets_bufferview = buffer_views.len();
            let offsets_byte_offset = bin.len();
            for &o in &offsets {
                bin.extend_from_slice(&o.to_le_bytes());
            }
            pad_to_4(&mut bin);
            buffer_views.push(json!({
                "buffer": 0, "byteOffset": offsets_byte_offset, "byteLength": offsets.len() * 4,
            }));

            table_properties.insert(
                id.clone(),
                json!({
                    "values": values_bufferview,
                    "stringOffsetType": "UINT32",
                    "stringOffsets": offsets_bufferview,
                }),
            );
        }

        root_extensions = Some(json!({
            "EXT_structural_metadata": {
                "schema": {
                    "id": METADATA_SCHEMA_ID,
                    "classes": {METADATA_CLASS_NAME: {"properties": schema_properties}},
                },
                "propertyTables": [{
                    "class": METADATA_CLASS_NAME,
                    "count": metadata.rows.len(),
                    "properties": table_properties,
                }],
            },
        }));
        primitive_extensions = Some(json!({
            "EXT_mesh_features": {
                "featureIds": [{
                    "featureCount": metadata.rows.len(), "attribute": 0, "propertyTable": 0,
                }],
            },
        }));
    }

    // Explicit, double-sided material: a primitive with no material uses
    // glTF's default material, which has `doubleSided: false` (backface
    // culling on). This writer doesn't verify triangle winding order
    // anywhere upstream, so culling could hide a mesh whose winding ends up
    // reversed; double-siding removes that failure mode entirely.
    let mut primitive = json!({
        "attributes": primitive_attributes, "indices": 1, "material": 0,
    });
    if let Some(extensions) = primitive_extensions {
        primitive["extensions"] = extensions;
    }

    let mut json_doc = json!({
        "asset": {"version": "2.0"},
        "buffers": [{"byteLength": bin.len()}],
        "bufferViews": buffer_views,
        "accessors": accessors,
        "materials": [{"doubleSided": true}],
        "meshes": [{"primitives": [primitive]}],
        "nodes": [{"mesh": 0, "translation": gltf_translation}],
        "scenes": [{"nodes": [0]}],
        "scene": 0,
    });
    if let Some(extensions) = root_extensions {
        json_doc["extensionsUsed"] = json!(["EXT_mesh_features", "EXT_structural_metadata"]);
        json_doc["extensions"] = extensions;
    }

    let mut json_bytes = serde_json::to_vec(&json_doc).expect("glTF JSON is always serializable");
    // GLB chunks must be 4-byte aligned; JSON pads with spaces, BIN with zeros.
    while !json_bytes.len().is_multiple_of(4) {
        json_bytes.push(b' ');
    }
    while !bin.len().is_multiple_of(4) {
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
