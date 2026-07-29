//! Material types.
//!
//! Both shading models are first-class native variants: Phong (CityGML
//! X3DMaterial, OBJ diffuse / specular / ambient / shininess) and PBR
//! metallic-roughness (glTF / 3D Tiles). A same-model path preserves the source
//! material exactly; conversion between the two runs only when a consumer or the
//! user asks for it, and is lossy in both directions.
//!
//! A `Material` is one self-contained shading description for a surface: a
//! coherent set of Phong or PBR parameters plus the maps that paint it. It is
//! theme-agnostic (the theme is supplied by the per-face binding, not embedded
//! here), and each map's `Texture` names the UV channel it samples, so a
//! material may be reused under several themes.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::texture::Texture;
use super::ChannelId;

/// One self-contained shading description: exactly one of the two shading
/// models.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "schema", schemars(title = "Material"))]
pub enum Material {
    /// CityGML X3DMaterial, OBJ illumination models 1-2.
    Phong(PhongMaterial),
    /// glTF / 3D Tiles metallic-roughness.
    Pbr(PbrMaterial),
}

/// Classic Phong / Blinn-Phong material.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "schema", schemars(title = "Phong material"))]
pub struct PhongMaterial {
    /// Kd / diffuseColor.
    #[cfg_attr(feature = "schema", schemars(title = "Diffuse color"))]
    pub diffuse: [f32; 3],
    /// Ks / specularColor.
    #[cfg_attr(feature = "schema", schemars(title = "Specular color"))]
    pub specular: [f32; 3],
    /// Ke / emissiveColor.
    #[cfg_attr(feature = "schema", schemars(title = "Emissive color"))]
    pub emissive: [f32; 3],
    /// Ka / ambientIntensity.
    #[cfg_attr(feature = "schema", schemars(title = "Ambient intensity"))]
    pub ambient_intensity: f32,
    /// Ns / shininess.
    #[cfg_attr(feature = "schema", schemars(title = "Shininess"))]
    pub shininess: f32,
    /// 0 = opaque (CityGML transparency / OBJ d, Tr).
    #[cfg_attr(feature = "schema", schemars(title = "Transparency"))]
    pub transparency: f32,
    /// ParameterizedTexture / map_Kd.
    #[cfg_attr(feature = "schema", schemars(title = "Diffuse map"))]
    pub diffuse_map: Option<Texture>,
    /// map_Ke.
    #[cfg_attr(feature = "schema", schemars(title = "Emissive map"))]
    pub emissive_map: Option<Texture>,
    /// OBJ norm / map_bump (CityGML carries none).
    #[cfg_attr(feature = "schema", schemars(title = "Normal map"))]
    pub normal_map: Option<Texture>,
}

impl Material {
    /// Whether this material has any textured map slot, and therefore samples a
    /// UV set. A material with no maps (colour / factors only) needs no UV; one
    /// with at least one map requires a UV set to sample.
    pub fn has_texture(&self) -> bool {
        match self {
            Material::Phong(m) => {
                m.diffuse_map.is_some() || m.emissive_map.is_some() || m.normal_map.is_some()
            }
            Material::Pbr(m) => {
                m.base_color_map.is_some()
                    || m.metallic_roughness_map.is_some()
                    || m.normal_map.is_some()
                    || m.occlusion_map.is_some()
                    || m.emissive_map.is_some()
            }
        }
    }

    /// The distinct UV channels this material's textured maps sample (empty if
    /// colour-only). A UV set must be supplied for each when attaching appearance;
    /// several maps sharing a channel collapse to one entry.
    pub fn referenced_channels(&self) -> BTreeSet<ChannelId> {
        let mut channels = BTreeSet::new();
        let mut add = |map: &Option<Texture>| {
            if let Some(texture) = map {
                channels.insert(texture.uv_channel);
            }
        };
        match self {
            Material::Phong(m) => {
                add(&m.diffuse_map);
                add(&m.emissive_map);
                add(&m.normal_map);
            }
            Material::Pbr(m) => {
                add(&m.base_color_map);
                add(&m.metallic_roughness_map);
                add(&m.normal_map);
                add(&m.occlusion_map);
                add(&m.emissive_map);
            }
        }
        channels
    }
}

/// glTF metallic-roughness PBR material.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "schema", schemars(title = "PBR material"))]
pub struct PbrMaterial {
    /// baseColorFactor, including alpha.
    #[cfg_attr(feature = "schema", schemars(title = "Base color"))]
    pub base_color: [f32; 4],
    /// metallicFactor.
    #[cfg_attr(feature = "schema", schemars(title = "Metallic"))]
    pub metallic: f32,
    /// roughnessFactor.
    #[cfg_attr(feature = "schema", schemars(title = "Roughness"))]
    pub roughness: f32,
    /// emissiveFactor.
    #[cfg_attr(feature = "schema", schemars(title = "Emissive color"))]
    pub emissive: [f32; 3],
    #[cfg_attr(feature = "schema", schemars(title = "Base color map"))]
    pub base_color_map: Option<Texture>,
    #[cfg_attr(feature = "schema", schemars(title = "Metallic-roughness map"))]
    pub metallic_roughness_map: Option<Texture>,
    #[cfg_attr(feature = "schema", schemars(title = "Normal map"))]
    pub normal_map: Option<Texture>,
    #[cfg_attr(feature = "schema", schemars(title = "Occlusion map"))]
    pub occlusion_map: Option<Texture>,
    #[cfg_attr(feature = "schema", schemars(title = "Emissive map"))]
    pub emissive_map: Option<Texture>,
    #[cfg_attr(feature = "schema", schemars(title = "Alpha mode"))]
    pub alpha_mode: AlphaMode,
    #[cfg_attr(feature = "schema", schemars(title = "Double sided"))]
    pub double_sided: bool,
}

/// How a material's alpha channel is interpreted.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum AlphaMode {
    Opaque,
    Mask { cutoff: f32 },
    Blend,
}
