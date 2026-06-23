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

use serde::{Deserialize, Serialize};

use super::texture::Texture;

/// One self-contained shading description: exactly one of the two shading
/// models.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum Material {
    /// CityGML X3DMaterial, OBJ illumination models 1-2.
    Phong(PhongMaterial),
    /// glTF / 3D Tiles metallic-roughness.
    Pbr(PbrMaterial),
}

/// Classic Phong / Blinn-Phong material.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PhongMaterial {
    /// Kd / diffuseColor.
    pub diffuse: [f32; 3],
    /// Ks / specularColor.
    pub specular: [f32; 3],
    /// Ke / emissiveColor.
    pub emissive: [f32; 3],
    /// Ka / ambientIntensity.
    pub ambient_intensity: f32,
    /// Ns / shininess.
    pub shininess: f32,
    /// 0 = opaque (CityGML transparency / OBJ d, Tr).
    pub transparency: f32,
    /// ParameterizedTexture / map_Kd.
    pub diffuse_map: Option<Texture>,
    /// map_Ke.
    pub emissive_map: Option<Texture>,
    /// OBJ norm / map_bump (CityGML carries none).
    pub normal_map: Option<Texture>,
}

/// glTF metallic-roughness PBR material.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PbrMaterial {
    /// baseColorFactor, including alpha.
    pub base_color: [f32; 4],
    /// metallicFactor.
    pub metallic: f32,
    /// roughnessFactor.
    pub roughness: f32,
    /// emissiveFactor.
    pub emissive: [f32; 3],
    pub base_color_map: Option<Texture>,
    pub metallic_roughness_map: Option<Texture>,
    pub normal_map: Option<Texture>,
    pub occlusion_map: Option<Texture>,
    pub emissive_map: Option<Texture>,
    pub alpha_mode: AlphaMode,
    pub double_sided: bool,
}

/// How a material's alpha channel is interpreted.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum AlphaMode {
    Opaque,
    Mask { cutoff: f32 },
    Blend,
}
