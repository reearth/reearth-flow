use std::path::PathBuf;

use reearth_flow_types::material::Material;

/// A material resolved for the writer: glTF PBR metallic-roughness factors plus
/// an optional base-colour texture drawn from a local file. Old-geometry
/// materials fold to PBR the same lossy way the old writer's `X3DMaterial`
/// path did (diffuse color -> base color, fixed metallic/roughness).
#[derive(Clone)]
pub(super) struct ResolvedMaterial {
    pub(super) base_color_factor: [f32; 4],
    pub(super) metallic_factor: f32,
    pub(super) roughness_factor: f32,
    pub(super) base_texture: Option<TextureSource>,
}

/// A base-colour texture ready to feed the atlas packer. CityGML textures are
/// always local files, so unlike the new-geometry writer this has no
/// in-memory/embedded variant.
#[derive(Clone)]
pub(super) enum TextureSource {
    File(PathBuf),
}

/// Convert an old-geometry `Material` palette (diffuse color + optional file
/// texture) into the writer's `ResolvedMaterial` palette, in the same order.
pub(super) fn resolve(materials: &indexmap::IndexSet<Material>) -> Vec<ResolvedMaterial> {
    materials
        .iter()
        .map(|material| {
            let base_texture = material.base_texture.as_ref().and_then(|texture| {
                match texture.uri.to_file_path() {
                    Ok(path) => Some(TextureSource::File(path)),
                    Err(_) => {
                        tracing::warn!(
                            uri = %texture.uri,
                            "Cesium3DTilesWriter: base_texture URI is not a local file; \
                             rendering colour-only"
                        );
                        None
                    }
                }
            });
            ResolvedMaterial {
                base_color_factor: material.base_color,
                metallic_factor: 0.0,
                roughness_factor: 0.9,
                base_texture,
            }
        })
        .collect()
}
