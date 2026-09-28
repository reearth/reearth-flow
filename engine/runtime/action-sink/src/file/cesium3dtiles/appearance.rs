use std::path::PathBuf;

use reearth_flow_types::material::Material;

/// A material resolved for the writer: glTF PBR metallic-roughness factors plus
/// an optional base-colour texture drawn from a local file.
#[derive(Clone)]
pub(super) struct ResolvedMaterial {
    pub(super) base_color_factor: [f32; 4],
    pub(super) metallic_factor: f32,
    pub(super) roughness_factor: f32,
    pub(super) base_texture: Option<TextureSource>,
}

/// A base-colour texture ready to feed the atlas packer: a local file (e.g. a
/// CityGML texture on disk).
#[derive(Clone)]
pub(super) enum TextureSource {
    File(PathBuf),
}

pub(super) fn resolve(materials: &indexmap::IndexSet<Material>) -> Vec<ResolvedMaterial> {
    materials
        .iter()
        .map(|material| {
            let base_texture = material.base_texture.as_ref().and_then(|texture| {
                match texture.uri.to_file_path() {
                    Ok(path) => Some(TextureSource::File(path)),
                    Err(_) => {
                        tracing::error!(
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
                // The old writer's Phong->PBR path fixed these; keep parity.
                metallic_factor: 0.0,
                roughness_factor: 0.9,
                base_texture,
            }
        })
        .collect()
}
