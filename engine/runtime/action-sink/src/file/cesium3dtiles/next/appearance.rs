//! Resolve a triangulated mesh's [`Appearance`] into the flat, per-triangle /
//! per-corner form this writer consumes.
//!
//! A single-theme sink paints exactly one theme (see [`Appearance`]'s
//! `default_theme`) and, for now, only the front side. Resolving collapses the
//! appearance graph to three parallel arrays aligned to the triangulated mesh:
//! a small palette of [`ResolvedMaterial`]s (glTF PBR factors plus an optional
//! local base-colour texture), the per-triangle palette index each triangle
//! binds to, and the per-corner base-map UV. Phong materials are folded to PBR
//! the same lossy way the old writer's `X3DMaterial` path did (diffuse →
//! base colour, fixed metallic/roughness).

use std::collections::HashMap;
use std::path::PathBuf;

use reearth_flow_common::uri::Protocol;
use reearth_flow_geometry::appearance::{
    Appearance, ChannelId, FaceBinding, Material, Raster, Side, Texture, UvSource,
};

/// A material resolved for the writer: glTF PBR metallic-roughness factors plus
/// an optional base-colour texture drawn from a local file.
#[derive(Clone)]
pub(super) struct ResolvedMaterial {
    pub(super) base_color_factor: [f32; 4],
    pub(super) metallic_factor: f32,
    pub(super) roughness_factor: f32,
    /// `Some` only when the base map's raster is a readable local file *and* the
    /// theme supplies an explicit UV set for the channel it samples; otherwise
    /// the material renders colour-only (see [`resolve`]).
    pub(super) base_texture: Option<TextureSource>,
}

/// A base-colour texture backed by a local file, ready to feed the atlas packer.
#[derive(Clone)]
pub(super) struct TextureSource {
    pub(super) path: PathBuf,
}

/// A triangulated mesh's appearance, flattened onto its triangles and corners.
pub(super) struct ResolvedAppearance {
    /// Material palette; `triangle_material` indexes it.
    pub(super) materials: Vec<ResolvedMaterial>,
    /// Per output-triangle palette index; `None` = unbound (bare) face.
    pub(super) triangle_material: Vec<Option<u32>>,
    /// Per output-corner base-map UV, length `3 * triangle_count`. `[0.0, 0.0]`
    /// for a corner whose triangle is untextured.
    pub(super) corner_uv: Vec<[f64; 2]>,
}

/// Resolve `appearance` (already expanded onto the triangulated mesh, so its
/// bindings are per output triangle and its `Explicit` UV sets per output
/// corner) under the default theme, front side, for `triangle_count` triangles.
pub(super) fn resolve(appearance: &Appearance, triangle_count: usize) -> ResolvedAppearance {
    // Single-theme sink: paint the default theme, falling back to the first (a
    // sealed `Appearance` always has at least one) if it is somehow absent.
    let theme = appearance
        .themes()
        .iter()
        .find(|binding| binding.theme == *appearance.default_theme())
        .unwrap_or(&appearance.themes()[0]);

    // Front-side UV pool, keyed by channel. A `WorldToTexture` matrix has no
    // per-corner samples to atlas, so only `Explicit` sets are usable here.
    let front_channels: HashMap<ChannelId, &[[f64; 2]]> = theme
        .uv_sets
        .iter()
        .filter(|set| set.side == Side::Front)
        .filter_map(|set| match &set.uv {
            UvSource::Explicit(coords) => Some((set.channel, &coords[..])),
            UvSource::WorldToTexture(_) => None,
        })
        .collect();

    let mut materials = Vec::with_capacity(appearance.materials().len());
    // The channel each textured material samples, parallel to `materials`;
    // `None` for a colour-only material (drives `corner_uv` below).
    let mut material_channel: Vec<Option<ChannelId>> = Vec::with_capacity(materials.capacity());
    for material in appearance.materials() {
        let (resolved, channel) = convert_material(material, &front_channels);
        materials.push(resolved);
        material_channel.push(channel);
    }

    let triangle_material = resolve_binding(&theme.front, triangle_count);

    let mut corner_uv = vec![[0.0, 0.0]; triangle_count * 3];
    for (triangle, bound) in triangle_material.iter().enumerate() {
        let Some(channel) = bound.and_then(|mi| material_channel[mi as usize]) else {
            continue;
        };
        let coords = front_channels[&channel];
        for corner in triangle * 3..triangle * 3 + 3 {
            corner_uv[corner] = coords[corner];
        }
    }

    ResolvedAppearance {
        materials,
        triangle_material,
        corner_uv,
    }
}

/// Expand a per-triangle front binding to one palette index per triangle.
fn resolve_binding(binding: &FaceBinding, triangle_count: usize) -> Vec<Option<u32>> {
    match binding {
        FaceBinding::Uniform(index) => vec![Some(index.get()); triangle_count],
        FaceBinding::PerFace(faces) => {
            if faces.len() != triangle_count {
                tracing::error!(
                    "Cesium3DTilesWriter: per-face binding has {} entries but the mesh has {} \
                     triangles; unmatched triangles left unbound",
                    faces.len(),
                    triangle_count
                );
            }
            (0..triangle_count)
                .map(|t| faces.get(t).copied().flatten().map(|mi| mi.get()))
                .collect()
        }
    }
}

/// Convert one geometry [`Material`] to a [`ResolvedMaterial`], returning also
/// the UV channel its base map samples when that map is usable as a texture
/// here (readable local file + an `Explicit` UV set for its channel).
fn convert_material(
    material: &Material,
    front_channels: &HashMap<ChannelId, &[[f64; 2]]>,
) -> (ResolvedMaterial, Option<ChannelId>) {
    let (base_color_factor, metallic_factor, roughness_factor, base_map) = match material {
        Material::Phong(m) => (
            [
                m.diffuse[0],
                m.diffuse[1],
                m.diffuse[2],
                1.0 - m.transparency,
            ],
            // The old writer's Phong→PBR path fixed these; keep parity.
            0.0,
            0.9,
            &m.diffuse_map,
        ),
        Material::Pbr(m) => (m.base_color, m.metallic, m.roughness, &m.base_color_map),
    };

    let (base_texture, channel) = base_map
        .as_ref()
        .and_then(|tex| texture_source(tex, front_channels))
        .map_or((None, None), |(src, channel)| (Some(src), Some(channel)));

    (
        ResolvedMaterial {
            base_color_factor,
            metallic_factor,
            roughness_factor,
            base_texture,
        },
        channel,
    )
}

/// A base map's local-file texture source and sampled channel, or `None` when
/// the raster isn't a readable local file or the theme supplies no explicit UV
/// for its channel — in either case the material falls back to colour-only.
fn texture_source(
    texture: &Texture,
    front_channels: &HashMap<ChannelId, &[[f64; 2]]>,
) -> Option<(TextureSource, ChannelId)> {
    let channel = texture.uv_channel;
    if !front_channels.contains_key(&channel) {
        tracing::warn!(
            "Cesium3DTilesWriter: textured material samples UV channel {} but the theme has no \
             explicit UV set for it; rendering colour-only",
            channel.0
        );
        return None;
    }

    match &*texture.raster {
        Raster::Uri(uri) if uri.protocol() == Protocol::File => Some((
            TextureSource {
                path: uri.as_path(),
            },
            channel,
        )),
        Raster::Uri(uri) => {
            // Remote/in-memory rasters are a separate concern (a downloader
            // action materializes them locally first); the writer is local-only.
            tracing::error!(
                "Cesium3DTilesWriter: texture {uri} is not a local file ({:?}); rendering \
                 colour-only",
                uri.protocol()
            );
            None
        }
        Raster::InMemory(_) => {
            tracing::error!(
                "Cesium3DTilesWriter: in-memory rasters are not supported yet; rendering \
                 colour-only"
            );
            None
        }
    }
}
