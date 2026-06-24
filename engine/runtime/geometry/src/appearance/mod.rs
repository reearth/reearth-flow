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

use crate::error::Error;

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

impl FaceBinding {
    /// Mark every palette index this binding references in `referenced`.
    fn mark_referenced(&self, referenced: &mut [bool]) {
        match self {
            FaceBinding::Uniform(index) => referenced[index.get() as usize] = true,
            FaceBinding::PerFace(faces) => {
                for index in faces.iter().flatten() {
                    referenced[index.get() as usize] = true;
                }
            }
        }
    }

    /// Reindex every palette index through `remap` (old index → compacted index).
    /// Only indices previously marked by [`mark_referenced`] are looked up.
    fn reindex(&mut self, remap: &[Option<MaterialIndex>]) {
        let map = |index: MaterialIndex| {
            remap[index.get() as usize].expect("a referenced material survives compaction")
        };
        match self {
            FaceBinding::Uniform(index) => *index = map(*index),
            FaceBinding::PerFace(faces) => {
                for slot in faces.iter_mut().flatten() {
                    *slot = map(*slot);
                }
            }
        }
    }
}

impl Appearance {
    /// Remove palette entries unreferenced by any binding and reindex the
    /// survivors. Kept materials retain their relative order, so the compacted
    /// palette is a stable subsequence of the original.
    fn compact_materials(&mut self) {
        let mut referenced = vec![false; self.materials.len()];
        for binding in &self.themes {
            binding.front.mark_referenced(&mut referenced);
            if let Some(back) = &binding.back {
                back.mark_referenced(&mut referenced);
            }
        }

        let mut remap: Vec<Option<MaterialIndex>> = vec![None; self.materials.len()];
        let mut kept: Vec<Material> = Vec::new();
        for (old, material) in self.materials.iter().enumerate() {
            if referenced[old] {
                let new = MaterialIndex::new(kept.len() as u32)
                    .expect("a compacted palette is no larger than the original");
                remap[old] = Some(new);
                kept.push(material.clone());
            }
        }
        self.materials = kept;

        for binding in &mut self.themes {
            binding.front.reindex(&remap);
            if let Some(back) = &mut binding.back {
                back.reindex(&remap);
            }
        }
    }
}

/// Validate the texture/UV coupling and, for an `Explicit` source, the UV length —
/// the invariant every appearance setter shares. `references_texture` is whether
/// any bound material samples a texture: a textured binding requires exactly one UV
/// set, a colour-only one must not carry an orphan. `corner_count` is the number of
/// corners an `Explicit` UV must match.
pub(crate) fn validate_uv_coupling(
    references_texture: bool,
    uv: &Option<UvSource>,
    corner_count: usize,
) -> Result<(), Error> {
    match (references_texture, uv) {
        (true, None) => {
            return Err(Error::invalid_appearance(
                "a textured material requires a UV set, but none was supplied",
            ));
        }
        (false, Some(_)) => {
            return Err(Error::invalid_appearance(
                "a UV set was supplied but no material is textured (orphan UV)",
            ));
        }
        _ => {}
    }
    if let Some(UvSource::Explicit(coords)) = uv {
        if coords.len() != corner_count {
            return Err(Error::invalid_appearance(format!(
                "UV length {} does not match the corner count {corner_count}",
                coords.len()
            )));
        }
    }
    Ok(())
}

/// Strip all back-side appearance from a mesh's `(appearance, uv_sets)`: drop each
/// theme's back binding, remove every `Side::Back` UV set, and compact the palette
/// so now-orphaned back-only materials are dropped. Shared by the polygon-mesh and
/// triangular-mesh `make_front_only` shells — a [`Solid`](crate::solid::Solid)'s
/// back face is its interior (or the inside of a void), which is never rendered, so
/// back appearance is meaningless there; `Solid` construction calls this on every
/// shell.
pub(crate) fn make_front_only(appearance: &mut Option<Appearance>, uv_sets: &mut Vec<UvSet>) {
    uv_sets.retain(|uv| uv.side != Side::Back);
    if let Some(app) = appearance {
        for binding in &mut app.themes {
            binding.back = None;
        }
        app.compact_materials();
    }
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

    fn phong(diffuse: f32) -> Material {
        Material::Phong(PhongMaterial {
            diffuse: [diffuse, diffuse, diffuse],
            specular: [0.0; 3],
            emissive: [0.0; 3],
            ambient_intensity: 0.0,
            shininess: 0.0,
            transparency: 0.0,
            diffuse_map: None,
            emissive_map: None,
            normal_map: None,
        })
    }

    #[test]
    fn make_front_only_drops_back_and_compacts_palette() {
        let theme = ThemeId(Arc::from("rgb"));
        // Material 0 is front-only, material 1 is back-only.
        let mut appearance = Some(Appearance {
            materials: vec![phong(1.0), phong(2.0)],
            themes: vec![ThemeBinding {
                theme: theme.clone(),
                front: FaceBinding::Uniform(MaterialIndex::new(0).unwrap()),
                back: Some(FaceBinding::Uniform(MaterialIndex::new(1).unwrap())),
            }],
            default_theme: theme.clone(),
        });
        let uv = |side| UvSet {
            theme: Some(theme.clone()),
            side,
            channel: None,
            uv: UvSource::Explicit(Box::new([])),
        };
        let mut uv_sets = vec![uv(Side::Front), uv(Side::Back)];

        make_front_only(&mut appearance, &mut uv_sets);

        let app = appearance.unwrap();
        // The back binding is gone and its now-orphaned material dropped, leaving
        // only the front material (still reachable at the compacted index 0).
        assert_eq!(app.materials, vec![phong(1.0)]);
        assert!(app.themes[0].back.is_none());
        assert!(matches!(app.themes[0].front, FaceBinding::Uniform(i) if i.get() == 0));
        // The back UV set is stripped; only the front survives.
        assert_eq!(uv_sets.len(), 1);
        assert_eq!(uv_sets[0].side, Side::Front);
    }

    #[test]
    fn compact_materials_reindexes_perface_binding() {
        let theme = ThemeId(Arc::from("rgb"));
        // Palette entry 0 is unreferenced; faces bind 1 and 3, with a bare slot.
        let mut app = Appearance {
            materials: vec![phong(0.0), phong(1.0), phong(2.0), phong(3.0)],
            themes: vec![ThemeBinding {
                theme: theme.clone(),
                front: FaceBinding::PerFace(vec![
                    Some(MaterialIndex::new(1).unwrap()),
                    None,
                    Some(MaterialIndex::new(3).unwrap()),
                ]),
                back: None,
            }],
            default_theme: theme,
        };

        app.compact_materials();

        // Only the referenced materials survive, in their original relative order.
        assert_eq!(app.materials, vec![phong(1.0), phong(3.0)]);
        let FaceBinding::PerFace(faces) = &app.themes[0].front else {
            panic!("front binding should stay PerFace");
        };
        assert_eq!(faces[0].map(|i| i.get()), Some(0));
        assert_eq!(faces[1], None);
        assert_eq!(faces[2].map(|i| i.get()), Some(1));
    }
}
