//! Per-feature tile-content cost estimate: geometry + texture + attribute bytes.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::Cursor;
use std::path::PathBuf;

use reearth_flow_types::{AttributeValue, Feature};

use super::appearance::TextureSource;
use super::mesh::ExtractedMesh;

/// Flat, deliberately rough per-triangle byte estimate; refine later.
const BYTES_PER_TRIANGLE: u64 = 100;

/// Flat, deliberately rough per-texture-pixel byte estimate; refine later.
const BYTES_PER_TEXTURE_PIXEL: f64 = 1.0;

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

pub(super) fn estimate(feature: &Feature, mesh: &ExtractedMesh, caches: &mut CostCaches) -> u64 {
    geometry_bytes(mesh) + texture_bytes(mesh, caches) + attribute_bytes(feature)
}

fn geometry_bytes(mesh: &ExtractedMesh) -> u64 {
    mesh.indices.len() as u64 * BYTES_PER_TRIANGLE
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

fn attribute_bytes(feature: &Feature) -> u64 {
    feature
        .attributes
        .iter()
        .map(|(k, v)| k.as_ref().len() as u64 + attribute_value_bytes(v))
        .sum()
}

fn attribute_value_bytes(value: &AttributeValue) -> u64 {
    match value {
        AttributeValue::Null => 0,
        AttributeValue::Bool(_) => 1,
        AttributeValue::Number(_) => 8,
        AttributeValue::String(s) => s.len() as u64,
        AttributeValue::DateTime(_) => 8,
        AttributeValue::Array(items) => items.iter().map(attribute_value_bytes).sum(),
        AttributeValue::Map(items) => items
            .iter()
            .map(|(k, v)| k.len() as u64 + attribute_value_bytes(v))
            .sum(),
        AttributeValue::Bytes(b) => b.len() as u64,
    }
}
