//! Per-feature tile-content cost estimate: geometry + texture + attribute bytes,
//! calibrated against the glb bytes the writer actually emits.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::Cursor;
use std::path::PathBuf;

use reearth_flow_gltf::next::metadata::{self, MetadataOptions};
use reearth_flow_types::Feature;

use super::super::sink::TextureCodec;
use super::appearance::TextureSource;
use super::mesh::ExtractedMesh;
use super::RenderOptions;

/// Output bytes per triangle with Draco compression. Draco quantizes and
/// predicts vertex attributes, so its output does not track the raw vertex
/// layout; measured at 3.4 on PLATEAU LOD2 building meshes, rounded up.
const DRACO_BYTES_PER_TRIANGLE: u64 = 5;

/// Bytes per triangle for `UINT32` indices.
const INDEX_BYTES_PER_TRIANGLE: u64 = 3 * 4;

/// Bytes per vertex for the `float32` vec3 `POSITION` attribute.
const POSITION_BYTES_PER_VERTEX: u64 = 3 * 4;

/// Bytes per vertex for the `float32` vec3 `NORMAL` attribute.
const NORMAL_BYTES_PER_VERTEX: u64 = 3 * 4;

/// Bytes per vertex for the `float32` vec2 `TEXCOORD_0` attribute.
const TEXCOORD_BYTES_PER_VERTEX: u64 = 2 * 4;

/// Bytes per vertex for the `float32` scalar feature ID attribute.
const FEATURE_ID_BYTES_PER_VERTEX: u64 = 4;

/// Flat, deliberately rough per-texture-pixel byte estimate; refine later.
const BYTES_PER_TEXTURE_PIXEL: f64 = 0.2;

/// Cache key for a texture's native pixel dimensions.
#[derive(PartialEq, Eq, Hash)]
enum TextureKey {
    File(PathBuf),
    EmbeddedHash(u64),
}

/// Shared across a whole `build` call so each distinct texture's dimensions
/// are read once (header only, not a full decode).
#[derive(Default)]
pub(super) struct CostCaches {
    dims: HashMap<TextureKey, Option<(u32, u32)>>,
}

/// Estimated glb bytes `feature` contributes to its cell's content. `render`
/// selects the geometry layout and whether textures are emitted at all;
/// `options` selects which attributes are written.
pub(super) fn estimate(
    feature: &Feature,
    mesh: &ExtractedMesh,
    caches: &mut CostCaches,
    render: RenderOptions,
    options: MetadataOptions,
) -> u64 {
    let texture = if matches!(render.texture_codec, TextureCodec::Untextured) {
        0
    } else {
        texture_bytes(mesh, caches)
    };
    geometry_bytes(mesh, render) + texture + attribute_bytes(feature, options)
}

/// Uncompressed size follows the glb vertex layout: `UINT32` indices per
/// triangle plus the per-vertex attributes the render options select. With
/// flat normals every polygon owns its corner vertices, so a polygon of `t`
/// triangles carries `t + 2` vertices; without them vertices weld across
/// polygons and the source vertex count applies.
fn geometry_bytes(mesh: &ExtractedMesh, render: RenderOptions) -> u64 {
    let triangles = mesh.indices.len() as u64;
    if render.draco {
        return triangles * DRACO_BYTES_PER_TRIANGLE;
    }
    let vertices = if render.compute_flat_normal {
        triangles + 2 * mesh.polygon_tris.len() as u64
    } else {
        mesh.ecef_vertices.len() as u64
    };
    let mut bytes_per_vertex = POSITION_BYTES_PER_VERTEX + FEATURE_ID_BYTES_PER_VERTEX;
    if render.compute_flat_normal {
        bytes_per_vertex += NORMAL_BYTES_PER_VERTEX;
    }
    if !matches!(render.texture_codec, TextureCodec::Untextured) {
        bytes_per_vertex += TEXCOORD_BYTES_PER_VERTEX;
    }
    triangles * INDEX_BYTES_PER_TRIANGLE + vertices * bytes_per_vertex
}

/// Native resolution × this feature's UV-bbox footprint, summed per material.
fn texture_bytes(mesh: &ExtractedMesh, caches: &mut CostCaches) -> u64 {
    let mut total = 0.0f64;
    for (material_index, material) in mesh.materials.iter().enumerate() {
        let Some(source) = &material.base_texture else {
            continue;
        };
        let Some((tw, th)) = texture_dims(source, caches) else {
            continue;
        };
        let Some(uv_area) = material_uv_area(mesh, material_index as u32) else {
            continue;
        };
        total += tw as f64 * th as f64 * uv_area * BYTES_PER_TEXTURE_PIXEL;
    }
    total as u64
}

/// This material's UV bounding-box area (fraction of the unit square), or
/// `None` if no triangle binds it.
fn material_uv_area(mesh: &ExtractedMesh, material_index: u32) -> Option<f64> {
    let (mut min_u, mut max_u) = (f64::INFINITY, f64::NEG_INFINITY);
    let (mut min_v, mut max_v) = (f64::INFINITY, f64::NEG_INFINITY);
    let mut found = false;
    for (tri, bound) in mesh.triangle_material.iter().enumerate() {
        if *bound != Some(material_index) {
            continue;
        }
        found = true;
        for c in 0..3 {
            let [u, v] = mesh.corner_uv[tri * 3 + c];
            min_u = min_u.min(u);
            max_u = max_u.max(u);
            min_v = min_v.min(v);
            max_v = max_v.max(v);
        }
    }
    found.then(|| ((max_u - min_u).max(0.0) * (max_v - min_v).max(0.0)).clamp(0.0, 1.0))
}

/// Native pixel dimensions from a header-only read, cached per distinct source.
fn texture_dims(source: &TextureSource, caches: &mut CostCaches) -> Option<(u32, u32)> {
    match source {
        TextureSource::File(path) => *caches
            .dims
            .entry(TextureKey::File(path.clone()))
            .or_insert_with(|| image::image_dimensions(path).ok()),
        TextureSource::Embedded(data) => {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            data.bytes.hash(&mut hasher);
            let key = TextureKey::EmbeddedHash(hasher.finish());
            *caches.dims.entry(key).or_insert_with(|| {
                image::ImageReader::new(Cursor::new(&data.bytes))
                    .with_guessed_format()
                    .ok()
                    .and_then(|r| r.into_dimensions().ok())
            })
        }
    }
}

/// Bytes of the property-table row this feature will occupy: only the
/// attributes the writer actually emits, as the strings it emits them as.
fn attribute_bytes(feature: &Feature, options: MetadataOptions) -> u64 {
    metadata::row_bytes(feature, options)
}
