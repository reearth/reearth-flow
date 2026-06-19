//! Surface appearance: a small graph of types hung off a surface geometry.
//!
//! Top-down: a geometry optionally has one `Appearance` and owns a pool of
//! `UvSet`s; an `Appearance` is a material palette plus, per theme, a
//! face-to-material binding; a `Material` is exactly one of two shading models,
//! each with a fixed set of texture slots; a `Texture` samples one `UvSet` from
//! the geometry's pool, and several textures may share the same one.
//!
//! Appearance attaches to `Polygon`, `PolygonMesh` and `TriangularMesh` as an
//! `Option<Appearance>`. `Solid` and `Csg` carry none of their own; their meshes
//! carry theirs. `Point`, `LineString` and `PointCloud` carry no appearance.

pub mod material;
pub mod texture;
pub mod uv;

pub use material::{AlphaMode, Material, PbrMaterial, PhongMaterial};
pub use texture::{Filter, Raster, RasterData, Sampler, Texture, TextureTransform, WrapMode};
pub use uv::{TexMatrix, UvSet, UvSource};

use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// A dataset-global, stable theme name. Switching to a theme selects the same
/// theme across every feature, so this is an identity (a name), not a per-mesh
/// index (contrast [`ChannelId`]).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ThemeId(pub Arc<str>);

/// A material-local UV channel index. Carries no cross-theme meaning: channel 0
/// under one theme and channel 0 under another are different UV sets.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ChannelId(pub u32);

/// Materials, themes and per-face bindings for a surface geometry.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Appearance {
    /// Palette; bindings index into it.
    pub materials: Vec<Material>,
    /// One independent binding per theme; length >= 1.
    pub themes: Vec<ThemeBinding>,
    /// Active theme for single-theme sinks (glTF / OBJ / CZML / 3D Tiles).
    pub default_theme: ThemeId,
}

/// One theme's face-to-material binding.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ThemeBinding {
    pub theme: ThemeId,
    pub binding: FaceBinding,
}

/// How a theme's faces map to the material palette.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum FaceBinding {
    /// One material for every face; the common case, and the only form a
    /// single-material theme or a `Polygon` ever needs. The value indexes the
    /// material palette.
    Uniform(u32),
    /// Per-face material index; length == face count; `None` = unbound (bare
    /// face).
    PerFace(Vec<Option<u32>>),
}
