//! Surface appearance: a small graph of types hung off a surface geometry.
//!
//! Top-down: a geometry optionally has one `Appearance`; an `Appearance` is a
//! material palette plus, per theme, a [`ThemeBinding`] — a per-side (front /
//! optional back) face-to-material binding together with that theme's own pool of
//! [`UvSet`]s; a `Material` is exactly one of two shading models, each with a
//! fixed set of texture slots; a `Texture` samples one `UvSet` from its theme's
//! pool, and several textures may share the same one.
//!
//! `Appearance` is a sealed type: its fields are private and it can only be built
//! through [`append_theme`] / [`Appearance::from_parts`], both crate-internal and
//! fed the host geometry's corner count so a `UvSet` can never outlive the
//! corner-buffer alignment it depends on.
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

use std::collections::{BTreeMap, BTreeSet};
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
#[derive(
    Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
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

/// Materials, themes, per-face bindings and per-theme UV for a surface geometry.
///
/// Sealed: the fields are private and every value is built through
/// [`append_theme`] or [`Appearance::from_parts`]. A `UvSet` inside a
/// [`ThemeBinding`] must stay length-matched to the host geometry's corner
/// buffer — a fact `Appearance` cannot check on its own — so construction is
/// confined to the geometry crate, where the corner count is in hand. Read
/// through [`materials`](Self::materials) / [`themes`](Self::themes) /
/// [`default_theme`](Self::default_theme).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Appearance {
    /// Palette; bindings index into it.
    materials: Vec<Material>,
    /// One independent binding per theme; length >= 1.
    themes: Vec<ThemeBinding>,
    /// Active theme for single-theme sinks (glTF / OBJ / CZML / 3D Tiles).
    default_theme: ThemeId,
}

impl Appearance {
    /// The material palette; bindings index into it.
    #[inline]
    pub fn materials(&self) -> &[Material] {
        &self.materials
    }

    /// The per-theme bindings; length >= 1.
    #[inline]
    pub fn themes(&self) -> &[ThemeBinding] {
        &self.themes
    }

    /// The active theme for single-theme sinks.
    #[inline]
    pub fn default_theme(&self) -> &ThemeId {
        &self.default_theme
    }

    /// Every UV set across all themes, in theme then set order. The theme each
    /// belongs to is lost; use [`themes`](Self::themes) when it matters.
    pub fn uv_iter(&self) -> impl Iterator<Item = &UvSet> {
        self.themes.iter().flat_map(|theme| theme.uv_sets.iter())
    }

    /// Assemble an appearance from already-validated parts.
    pub(crate) fn from_parts(
        materials: Vec<Material>,
        themes: Vec<ThemeBinding>,
        default_theme: ThemeId,
    ) -> Self {
        Appearance {
            materials,
            themes,
            default_theme,
        }
    }

    /// Decompose into `(materials, themes, default_theme)`.
    pub(crate) fn into_parts(self) -> (Vec<Material>, Vec<ThemeBinding>, ThemeId) {
        (self.materials, self.themes, self.default_theme)
    }
}

/// One theme's per-side face-to-material binding together with the UV sets that
/// theme's textured materials sample.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ThemeBinding {
    pub theme: ThemeId,
    /// Front-side face-to-material binding.
    pub front: FaceBinding,
    /// Back-side binding; `None` = single-sided (only the front is painted).
    /// When `Some`, the back side has its own face-to-material mapping: it may
    /// bind different materials, or leave faces unbound.
    pub back: Option<FaceBinding>,
    /// This theme's UV pool; one set per (side, channel) its materials reference.
    /// Each `Explicit` array is parallel to the host geometry's corner buffer.
    pub uv_sets: Vec<UvSet>,
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
    /// Shift every material index by `offset` into a merged palette, consuming the
    /// binding. Errors (`"material palette too large"`) if any index overflows
    /// `u32`. Handles both `Uniform` and `PerFace`, so it is the single offset
    /// routine the polygon, triangular-mesh and weld paths share.
    pub(crate) fn offset(self, offset: u32) -> Result<FaceBinding, Error> {
        let shift = |index: MaterialIndex| {
            index
                .get()
                .checked_add(offset)
                .and_then(MaterialIndex::new)
                .ok_or_else(|| Error::invalid_appearance("material palette too large"))
        };
        Ok(match self {
            FaceBinding::Uniform(index) => FaceBinding::Uniform(shift(index)?),
            FaceBinding::PerFace(faces) => FaceBinding::PerFace(
                faces
                    .into_iter()
                    .map(|opt| opt.map(shift).transpose())
                    .collect::<Result<_, _>>()?,
            ),
        })
    }

    /// Mark every palette index this binding references in `referenced`.
    ///
    /// An index past the palette can only arrive from an appearance installed
    /// through the unvalidated `appearance_mut` escape hatch; it references no
    /// real material, so it is skipped rather than indexed (which would panic).
    fn mark_referenced(&self, referenced: &mut [bool]) {
        let mut mark = |index: &MaterialIndex| {
            if let Some(slot) = referenced.get_mut(index.get() as usize) {
                *slot = true;
            }
        };
        match self {
            FaceBinding::Uniform(index) => mark(index),
            FaceBinding::PerFace(faces) => faces.iter().flatten().for_each(mark),
        }
    }

    /// Reindex every palette index through `remap` (old index → compacted index).
    /// Only indices previously marked by [`mark_referenced`] are looked up.
    fn reindex(&mut self, remap: &[Option<MaterialIndex>]) {
        let map = |index: MaterialIndex| match remap.get(index.get() as usize) {
            // In range: it was marked referenced, so it has a compacted slot.
            Some(slot) => slot.expect("a referenced material survives compaction"),
            // Out of range (malformed appearance, see `mark_referenced`): nothing
            // to remap to, so leave the dangling index untouched rather than panic.
            None => index,
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
        // A palette this large (only via `appearance_mut`) can't be reindexed
        // without truncating `kept.len() as u32`; leave the valid indices as-is.
        if self.materials.len() > u32::MAX as usize {
            return;
        }
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

/// Append one theme's already-validated appearance to a geometry's accumulated
/// `appearance`. `front` / `back` index `materials` locally; they are offset into
/// the running palette here. The theme's own `uv_sets` ride inside the appended
/// [`ThemeBinding`]. 
pub(crate) fn append_theme(
    appearance: &mut Option<Appearance>,
    theme: ThemeId,
    materials: Vec<Material>,
    front: FaceBinding,
    back: Option<FaceBinding>,
    uv_sets: Vec<UvSet>,
) -> Result<(), Error> {
    if let Some(app) = appearance.as_ref() {
        if app.themes.iter().any(|b| b.theme == theme) {
            return Err(Error::invalid_appearance(format!(
                "theme `{}` is already set",
                theme.0
            )));
        }
    }

    // Offset before mutating anything, so an overflow leaves the geometry unchanged.
    let offset = match appearance.as_ref() {
        Some(app) => u32::try_from(app.materials.len())
            .map_err(|_| Error::invalid_appearance("material palette too large"))?,
        None => 0,
    };
    let front = front.offset(offset)?;
    let back = back.map(|binding| binding.offset(offset)).transpose()?;

    let app = appearance.get_or_insert_with(|| Appearance {
        materials: Vec::new(),
        themes: Vec::new(),
        default_theme: theme.clone(),
    });
    app.materials.extend(materials);
    app.themes.push(ThemeBinding {
        theme,
        front,
        back,
        uv_sets,
    });
    Ok(())
}

/// Wrap an optional single UV as the default-channel entry of a channel map — the
/// single-`ParameterizedTexture` convenience the simple setters share. `None`
/// yields an empty map (a colour-only material).
pub(crate) fn single_channel_uv(uv: Option<UvSource>) -> BTreeMap<ChannelId, UvSource> {
    uv.into_iter()
        .map(|uv| (ChannelId::default(), uv))
        .collect()
}

/// Validate the texture/UV channel coupling and, for `Explicit` sources, the UV
/// length — the invariant every appearance setter shares. `referenced_channels`
/// is the set of UV channels the bound materials' textured maps sample; `uvs` must
/// supply exactly those channels — each textured channel needs a UV, and a channel
/// no material samples must not carry an orphan UV — and each `Explicit` UV must
/// have `corner_count` entries.
pub(crate) fn validate_uv_coupling(
    referenced_channels: &BTreeSet<ChannelId>,
    uvs: &BTreeMap<ChannelId, UvSource>,
    corner_count: usize,
) -> Result<(), Error> {
    for channel in referenced_channels {
        if !uvs.contains_key(channel) {
            return Err(Error::invalid_appearance(format!(
                "a textured material samples UV channel {} but no UV set was supplied for it",
                channel.0
            )));
        }
    }
    for (channel, uv) in uvs {
        if !referenced_channels.contains(channel) {
            return Err(Error::invalid_appearance(format!(
                "a UV set was supplied for channel {} but no material samples it (orphan UV)",
                channel.0
            )));
        }
        if let UvSource::Explicit(coords) = uv {
            if coords.len() != corner_count {
                return Err(Error::invalid_appearance(format!(
                    "UV length {} for channel {} does not match the corner count {corner_count}",
                    coords.len(),
                    channel.0
                )));
            }
        }
    }
    Ok(())
}

/// Strip all back-side appearance from a mesh's `appearance`: drop each theme's
/// back binding, remove every `Side::Back` UV set from that theme's pool, and
/// compact the palette so now-orphaned back-only materials are dropped.
pub(crate) fn make_front_only(appearance: &mut Option<Appearance>) {
    if let Some(app) = appearance {
        for binding in &mut app.themes {
            binding.back = None;
            binding.uv_sets.retain(|uv| uv.side != Side::Back);
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
        let uv = |side| UvSet {
            side,
            channel: ChannelId::default(),
            uv: UvSource::Explicit(Box::new([])),
        };
        // Material 0 is front-only, material 1 is back-only.
        let mut appearance = Some(Appearance {
            materials: vec![phong(1.0), phong(2.0)],
            themes: vec![ThemeBinding {
                theme: theme.clone(),
                front: FaceBinding::Uniform(MaterialIndex::new(0).unwrap()),
                back: Some(FaceBinding::Uniform(MaterialIndex::new(1).unwrap())),
                uv_sets: vec![uv(Side::Front), uv(Side::Back)],
            }],
            default_theme: theme.clone(),
        });

        make_front_only(&mut appearance);

        let app = appearance.unwrap();
        // The back binding is gone and its now-orphaned material dropped, leaving
        // only the front material (still reachable at the compacted index 0).
        assert_eq!(app.materials, vec![phong(1.0)]);
        assert!(app.themes[0].back.is_none());
        assert!(matches!(app.themes[0].front, FaceBinding::Uniform(i) if i.get() == 0));
        // The back UV set is stripped; only the front survives.
        assert_eq!(app.themes[0].uv_sets.len(), 1);
        assert_eq!(app.themes[0].uv_sets[0].side, Side::Front);
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
                uv_sets: Vec::new(),
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

    #[test]
    fn offset_overflow_is_rejected_not_panicked() {
        // A local index shifted by a near-`u32::MAX` palette offset overflows; the
        // shift must surface "material palette too large", not wrap (release) or
        // panic (debug) on a bare `index.get() + offset` add.
        let binding = FaceBinding::Uniform(MaterialIndex::new(2).unwrap());
        let err = binding.offset(u32::MAX - 1).unwrap_err();
        assert!(matches!(err, Error::InvalidAppearance(_)));
    }

    #[test]
    fn compact_materials_tolerates_out_of_range_binding() {
        let theme = ThemeId(Arc::from("rgb"));
        // A binding index past the palette can only arrive via the unvalidated
        // `appearance_mut` escape hatch. Compaction (now run on every `Solid`
        // shell) must not panic on it.
        let mut app = Appearance {
            materials: vec![phong(0.0)],
            themes: vec![ThemeBinding {
                theme: theme.clone(),
                front: FaceBinding::Uniform(MaterialIndex::new(5).unwrap()),
                back: None,
                uv_sets: Vec::new(),
            }],
            default_theme: theme,
        };

        app.compact_materials();

        // Nothing is referenced in range, so the palette empties; the dangling
        // index is left untouched rather than remapped or panicked on.
        assert!(app.materials.is_empty());
        assert!(matches!(app.themes[0].front, FaceBinding::Uniform(i) if i.get() == 5));
    }
}
