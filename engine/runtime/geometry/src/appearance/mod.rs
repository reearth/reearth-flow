//! Surface appearance: a small graph of types hung off a surface geometry.
//!
//! Top-down: a geometry optionally has one `Appearance` and owns a pool of
//! `UvSet`s; an `Appearance` is a material palette plus, per theme, a per-side
//! (front / optional back) face-to-material binding; a `Material` is exactly one
//! of two shading models,
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

use std::num::NonZeroU32;
use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A dataset-global, stable theme name. Switching to a theme selects the same
/// theme across every feature, so this is an identity (a name), not a per-mesh
/// index (contrast [`ChannelId`]).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ThemeId(pub Arc<str>);

/// A material-local UV channel index. Carries no cross-theme meaning: channel 0
/// under one theme and channel 0 under another are different UV sets.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ChannelId(pub u32);

/// An index into [`Appearance::materials`].
///
/// A "non-max" `u32`: every value except `u32::MAX` is representable. Reserving
/// `u32::MAX` lets `Option<MaterialIndex>` reuse that slot as its niche, so
/// `Option<MaterialIndex>` is **4 bytes**, not the 8 of `Option<u32>`. `None`
/// is the bare / unbound face in [`FaceBinding::PerFace`]; a palette can never
/// reach `u32::MAX` entries, so nothing usable is lost.
///
/// Stored as `!index` in a [`NonZeroU32`]: `index == u32::MAX` would map to the
/// forbidden `0`, which is exactly what supplies the niche.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct MaterialIndex(NonZeroU32);

impl MaterialIndex {
    /// Wraps a palette index. Returns `None` iff `index == u32::MAX` (reserved
    /// as the niche, never a usable index).
    #[inline]
    pub const fn new(index: u32) -> Option<Self> {
        match NonZeroU32::new(!index) {
            Some(stored) => Some(Self(stored)),
            None => None,
        }
    }

    /// The palette index this refers to.
    #[inline]
    pub const fn get(self) -> u32 {
        !self.0.get()
    }
}

impl std::fmt::Debug for MaterialIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MaterialIndex({})", self.get())
    }
}

// Serialize as the logical index (a plain number), so the wire form is readable
// and `Option<MaterialIndex>` round-trips through `null` / number.
impl Serialize for MaterialIndex {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.get().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MaterialIndex {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let index = u32::deserialize(deserializer)?;
        Self::new(index)
            .ok_or_else(|| serde::de::Error::custom("material index u32::MAX is reserved"))
    }
}

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

/// One theme's per-side face-to-material binding.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ThemeBinding {
    pub theme: ThemeId,
    /// Front-side face-to-material binding.
    pub front: FaceBinding,
    /// Back-side binding; `None` = single-sided (only the front is painted).
    /// When `Some`, the back side has its own face-to-material mapping: it may
    /// bind different materials, or leave faces unbound.
    pub back: Option<FaceBinding>,
}

/// Which side of an oriented surface an appearance applies to. The front side
/// faces along the surface normal (from its winding). Side is a simultaneous
/// axis (both sides exist at once), distinct from a theme (mutually-exclusive
/// styling variants).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Side {
    #[default]
    Front,
    Back,
}

/// How a theme's faces map to the material palette.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum FaceBinding {
    /// One material for every face; the common case, and the only form a
    /// single-material theme or a `Polygon` ever needs. Indexes the material
    /// palette.
    Uniform(MaterialIndex),
    /// Per-face material index; length == face count; `None` = unbound (bare
    /// face). `Option<MaterialIndex>` is 4 bytes per face (see [`MaterialIndex`]).
    PerFace(Vec<Option<MaterialIndex>>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_index_round_trips() {
        for index in [0, 1, 42, u32::MAX - 1] {
            let mi = MaterialIndex::new(index).expect("non-max index is representable");
            assert_eq!(mi.get(), index);
        }
        assert_eq!(MaterialIndex::new(u32::MAX), None);
    }

    #[test]
    fn option_material_index_is_four_bytes() {
        // The whole point of the non-max niche: `None` (bare face) costs no
        // extra word, so `PerFace` is 4 bytes/face, not 8.
        assert_eq!(std::mem::size_of::<Option<MaterialIndex>>(), 4);
        assert_eq!(std::mem::size_of::<MaterialIndex>(), 4);
    }

    #[test]
    fn material_index_serializes_as_plain_number() {
        let mi = MaterialIndex::new(7).unwrap();
        assert_eq!(serde_json::to_string(&mi).unwrap(), "7");
        assert_eq!(serde_json::from_str::<MaterialIndex>("7").unwrap(), mi);

        // bare face -> null, and back
        let bare: Option<MaterialIndex> = None;
        assert_eq!(serde_json::to_string(&bare).unwrap(), "null");
        assert_eq!(
            serde_json::from_str::<Option<MaterialIndex>>("null").unwrap(),
            None
        );

        // the reserved sentinel is rejected on the wire, not silently accepted
        assert!(serde_json::from_str::<MaterialIndex>("4294967295").is_err());
    }
}
