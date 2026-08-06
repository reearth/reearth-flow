//! Resolve one geometry leaf's native [`Appearance`] onto the [`GmlSurface`]s
//! that leaf produced.
//!
//! [`GmlSurface`]: super::model::GmlSurface
//!
//! The native appearance graph is a per-theme, per-side, per-face binding over a
//! material palette, with UV sets parallel to the host geometry's corner buffer.
//! CityGML 2.0's `app:Appearance` is flatter: one theme, a list of
//! `app:X3DMaterial` / `app:ParameterizedTexture` surface data, each naming the
//! `gml:id`s it targets, with texture coordinates given per ring. Getting from
//! one to the other is a narrowing, and this module is where every narrowing is
//! decided and named:
//!
//! - **One theme.** The default theme is selected by equality against
//!   [`Appearance::default_theme`], falling back to the first (a sealed
//!   `Appearance` always holds at least one). Every other theme is dropped with
//!   an aggregated warning. Unlike glTF, CityGML 2.0 *could* express several
//!   themes; emitting them is a follow-up, not an impossibility.
//! - **Front side only.** A back-side binding is dropped with a warning.
//! - **`Explicit` UV only.** A `WorldToTexture` matrix has no per-corner samples
//!   to write into `app:textureCoordinates`, so a material sampling one is
//!   rendered colour-only.
//! - **Phong is the target model.** `app:X3DMaterial` carries diffuse colour,
//!   specular colour and ambient intensity, which is exactly a
//!   [`PhongMaterial`](reearth_flow_geometry::appearance::PhongMaterial)'s first
//!   three fields. A PBR material's base colour folds onto diffuse and the rest
//!   of it — metallic, roughness, emissive — has no CityGML counterpart.
//! - **One map per material.** Only the diffuse / base-colour map becomes an
//!   `app:ParameterizedTexture`; emissive, normal, occlusion and
//!   metallic-roughness maps are dropped with a warning.
//!
//! ## Texture coordinates
//!
//! Two things happen here that cannot happen anywhere else, and both would look
//! plausible in a diff if they were wrong:
//!
//! - **The `v` flip.** Flow's canonical UV origin is top-left with `v`
//!   increasing downward (see `reearth_flow_geometry::appearance::uv`); the
//!   CityGML reader flips `v` to `1 - v` at ingest for exactly that reason.
//!   CityGML's own origin is bottom-left, so the writer flips back. Miss it and
//!   every texture in the output is mirrored vertically.
//! - **Slicing by corner range.** A ring's UV is the slice of the theme's
//!   per-corner array covering the corner range that ring occupies — the range
//!   Phase 4's face visitor hands back. When the converter closed a ring it also
//!   recorded which corner the appended one duplicates, and that corner's UV is
//!   appended in the same step, so a ring and its UV can never drift apart.
//!   `uv.len() == ring.len()` is checked afterwards regardless, because the
//!   alternative to an error here is a document whose texture coordinates are
//!   silently off by one corner.
//!
//! Nothing in this module panics on malformed input: a per-face binding of the
//! wrong length, an out-of-range material index, and a UV array too short for
//! the corners it is supposed to cover are all errors naming the feature, the
//! geometry and the face.

use std::collections::HashMap;
use std::ops::Range;

use nusamai_citygml::Color;
use reearth_flow_geometry::appearance::{
    Appearance, ChannelId, FaceBinding, Material, Raster, Side, Texture, ThemeBinding, UvSource,
};
use reearth_flow_types::material::X3DMaterial;

use super::model::{AppearanceBundle, GeometryOmission, GmlTexture, TextureRef, TextureSource};
use crate::errors::SinkError;

/// Where one emitted ring's texture coordinates live in its leaf's corner
/// buffer.
#[derive(Debug, Clone)]
pub(super) struct RingCorners {
    /// The ring's half-open `[start, end)` range of corner positions, as the
    /// face visitor reported it — before closure, because the corner buffer and
    /// the UV array parallel to it never had the closing corner.
    pub(super) corners: Range<usize>,
    /// The absolute corner position whose UV the appended closing corner
    /// duplicates, or `None` when the ring already closed and nothing was
    /// appended.
    pub(super) closure: Option<usize>,
    /// The emitted ring's own length, closing corner included. Checking the UV
    /// against this rather than against `corners.len()` is what makes the check
    /// worth making: it cross-checks the corner bookkeeping against the
    /// coordinates that actually reached the document.
    pub(super) len: usize,
}

/// The corner ranges one emitted face's rings occupy, in the order the face's
/// `gml:Polygon` writes them.
#[derive(Debug, Clone)]
pub(super) struct FaceCorners {
    /// The face's position in its leaf's own face order — what a
    /// [`FaceBinding::PerFace`] entry indexes.
    pub(super) face: usize,
    pub(super) exterior: RingCorners,
    pub(super) interiors: Vec<RingCorners>,
}

/// What one emitted face binds, in the shared model's terms.
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct SurfaceBinding {
    pub(super) material_idx: Option<u32>,
    pub(super) texture_idx: Option<u32>,
    pub(super) uv_exterior: Vec<[f64; 2]>,
    pub(super) uv_interiors: Vec<Vec<[f64; 2]>>,
}

/// The feature-level palettes every leaf's bindings index into.
///
/// One `app:Appearance` is written per feature, but a feature's geometry is a
/// collection of leaves, each carrying its own `Appearance` with its own local
/// material indices. Merging them is what this holds: each leaf's materials are
/// appended and its indices shifted by the offset the append started at, so the
/// merged indices are a deterministic function of leaf order.
#[derive(Default)]
pub(super) struct Palette {
    /// The palettes the writer emits and `GmlSurface` indexes into.
    pub(super) bundle: AppearanceBundle,
    /// The images those textures reference, for the shell to stage beside the
    /// `.gml`. Deduplicated by [`TextureRef::key`], parallel to
    /// `bundle.textures`.
    pub(super) textures: Vec<TextureRef>,
    /// Texture key → its position in `bundle.textures`, so one image referenced
    /// by several leaves becomes one `app:ParameterizedTexture`.
    by_key: HashMap<String, u32>,
}

/// What a failing or narrowing resolution names, so a message points at one
/// leaf of one feature.
pub(super) struct LeafContext<'a> {
    /// The feature's `gml:id`, or its runtime id when it has none.
    pub(super) feature: &'a str,
    /// The leaf, named as its geometry world spells it (`"PolygonMesh"`,
    /// `"Solid exterior shell"`, …).
    pub(super) geometry: &'a str,
}

/// One leaf's appearance, resolved.
#[derive(Debug)]
pub(super) struct Resolved {
    /// One entry per face in `faces`, in the same order.
    pub(super) bindings: Vec<SurfaceBinding>,
    /// What this leaf's appearance carried that CityGML 2.0 has no place for,
    /// aggregated by kind for the caller to fold into the feature's warnings.
    pub(super) omissions: Vec<GeometryOmission>,
}

/// Resolve `appearance` onto the faces `faces` describes, merging its palettes
/// into `palette`.
///
/// `faces` is one entry per emitted face, in the leaf's own face order, so its
/// length is the face count a [`FaceBinding::PerFace`] must agree with.
pub(super) fn resolve(
    appearance: &Appearance,
    faces: &[FaceCorners],
    palette: &mut Palette,
    context: &LeafContext<'_>,
) -> Result<Resolved, SinkError> {
    // A sealed `Appearance` always holds at least one theme, but nothing here
    // depends on that being true: no theme simply means nothing to paint.
    let Some(theme) = select_theme(appearance) else {
        return Ok(Resolved {
            bindings: vec![SurfaceBinding::default(); faces.len()],
            omissions: Vec::new(),
        });
    };

    let mut resolver = Resolver {
        materials: appearance.materials(),
        front_channels: front_channels(theme),
        context,
        palette,
        omissions: Vec::new(),
        slots: HashMap::new(),
    };

    resolver.record_theme(&theme.theme.0);
    if appearance.themes().len() > 1 {
        resolver.omit(
            EXTRA_THEME,
            EXTRA_THEME_REASON,
            appearance.themes().len() - 1,
        );
    }
    if theme.back.is_some() {
        resolver.omit(BACK_SIDE, BACK_SIDE_REASON, 1);
    }
    let projective = theme
        .uv_sets
        .iter()
        .filter(|set| set.side == Side::Front)
        .filter(|set| matches!(set.uv, UvSource::WorldToTexture(_)))
        .count();
    if projective > 0 {
        resolver.omit(PROJECTIVE_UV, PROJECTIVE_UV_REASON, projective);
    }

    let bound = resolver.face_materials(&theme.front, faces.len())?;
    let mut bindings = Vec::with_capacity(faces.len());
    for (face, local) in faces.iter().zip(bound) {
        bindings.push(resolver.bind_face(face, local)?);
    }

    Ok(Resolved {
        bindings,
        omissions: resolver.omissions,
    })
}

/// The theme to paint: the default one, else the first one declared.
fn select_theme(appearance: &Appearance) -> Option<&ThemeBinding> {
    appearance
        .themes()
        .iter()
        .find(|binding| binding.theme == *appearance.default_theme())
        .or_else(|| appearance.themes().first())
}

/// The theme's front-side per-corner UV arrays, by channel. A `WorldToTexture`
/// set has no per-corner samples, so it is not in here and a material sampling
/// its channel comes out colour-only.
fn front_channels(theme: &ThemeBinding) -> HashMap<ChannelId, &[[f64; 2]]> {
    theme
        .uv_sets
        .iter()
        .filter(|set| set.side == Side::Front)
        .filter_map(|set| match &set.uv {
            UvSource::Explicit(coords) => Some((set.channel, &coords[..])),
            UvSource::WorldToTexture(_) => None,
        })
        .collect()
}

/// Flow's canonical UV origin is top-left; CityGML's is bottom-left. This is the
/// conversion at the writer's own boundary that `appearance::uv`'s coordinate
/// convention asks for, and the exact inverse of the flip the CityGML reader
/// applies on ingest.
fn to_citygml_uv(uv: [f64; 2]) -> [f64; 2] {
    [uv[0], 1.0 - uv[1]]
}

/// One merged-palette material, and the texture it paints with if any.
#[derive(Clone, Copy)]
struct MaterialSlot {
    material_idx: u32,
    /// The merged texture index and the UV channel it samples. `None` for a
    /// colour-only material, and for a textured one whose map this writer
    /// cannot carry.
    texture: Option<(u32, ChannelId)>,
}

/// The running state of resolving one leaf.
struct Resolver<'a> {
    materials: &'a [Material],
    front_channels: HashMap<ChannelId, &'a [[f64; 2]]>,
    context: &'a LeafContext<'a>,
    palette: &'a mut Palette,
    omissions: Vec<GeometryOmission>,
    /// Local palette index → its merged slot, filled on first use so a material
    /// no face binds is never converted and cannot fail the write.
    slots: HashMap<u32, MaterialSlot>,
}

impl Resolver<'_> {
    /// Record the theme this feature's `app:Appearance` is written under. The
    /// first leaf to resolve wins; a later leaf naming a different theme is a
    /// second theme, and one `app:Appearance` writes one.
    fn record_theme(&mut self, theme: &str) {
        match &self.palette.bundle.theme {
            None => self.palette.bundle.theme = Some(theme.to_string()),
            Some(existing) if existing != theme => self.omit(EXTRA_THEME, EXTRA_THEME_REASON, 1),
            Some(_) => {}
        }
    }

    /// Expand the front binding to one local material index per emitted face.
    ///
    /// A `PerFace` binding whose length disagrees with the face count means the
    /// appearance and the geometry describe different meshes; writing whichever
    /// faces happen to line up would paint the wrong surfaces, so it is an
    /// error rather than a truncation.
    fn face_materials(
        &self,
        binding: &FaceBinding,
        face_count: usize,
    ) -> Result<Vec<Option<u32>>, SinkError> {
        match binding {
            FaceBinding::Uniform(index) => {
                self.check_material(index.get(), None)?;
                Ok(vec![Some(index.get()); face_count])
            }
            FaceBinding::PerFace(faces) => {
                if faces.len() != face_count {
                    return Err(self.error(format!(
                        "its appearance binds {} faces but {face_count} were written",
                        faces.len()
                    )));
                }
                faces
                    .iter()
                    .enumerate()
                    .map(|(face, bound)| match bound {
                        Some(index) => {
                            self.check_material(index.get(), Some(face))?;
                            Ok(Some(index.get()))
                        }
                        None => Ok(None),
                    })
                    .collect()
            }
        }
    }

    /// Reject a palette index that names no material. The palette is validated
    /// on construction, but `appearance_mut` is an unvalidated escape hatch, so
    /// an out-of-range index can reach here — and indexing on it would panic.
    fn check_material(&self, index: u32, face: Option<usize>) -> Result<(), SinkError> {
        if (index as usize) < self.materials.len() {
            return Ok(());
        }
        let at = match face {
            Some(face) => format!(" on face {face}"),
            None => String::new(),
        };
        Err(self.error(format!(
            "its appearance binds material {index}{at} but its palette holds only {}",
            self.materials.len()
        )))
    }

    /// Resolve one face: what it binds, and the texture coordinates of each of
    /// its rings.
    fn bind_face(
        &mut self,
        face: &FaceCorners,
        local: Option<u32>,
    ) -> Result<SurfaceBinding, SinkError> {
        let Some(local) = local else {
            return Ok(SurfaceBinding::default());
        };
        let slot = self.slot(local)?;
        let Some((texture_idx, channel)) = slot.texture else {
            return Ok(SurfaceBinding {
                material_idx: Some(slot.material_idx),
                ..SurfaceBinding::default()
            });
        };
        let coords = self.front_channels[&channel];
        let uv_exterior = self.ring_uv(coords, &face.exterior, face.face, "exterior")?;
        let uv_interiors = face
            .interiors
            .iter()
            .enumerate()
            .map(|(n, ring)| self.ring_uv(coords, ring, face.face, &format!("interior {n}")))
            .collect::<Result<_, _>>()?;
        Ok(SurfaceBinding {
            material_idx: Some(slot.material_idx),
            texture_idx: Some(texture_idx),
            uv_exterior,
            uv_interiors,
        })
    }

    /// One ring's texture coordinates: its slice of the theme's per-corner
    /// array, flipped to CityGML's origin, extended by the corner ring closure
    /// duplicated.
    fn ring_uv(
        &self,
        coords: &[[f64; 2]],
        ring: &RingCorners,
        face: usize,
        which: &str,
    ) -> Result<Vec<[f64; 2]>, SinkError> {
        let Some(slice) = coords.get(ring.corners.clone()) else {
            return Err(self.error(format!(
                "its appearance supplies {} texture coordinates but face {face}'s {which} ring \
                 occupies corners {}..{}",
                coords.len(),
                ring.corners.start,
                ring.corners.end
            )));
        };
        let mut uv: Vec<[f64; 2]> = slice.iter().copied().map(to_citygml_uv).collect();
        // Closing the ring and extending its UV are one step: the appended
        // corner repeats a corner of the same ring, so its UV is that corner's.
        if let Some(closed_at) = ring.closure {
            let Some(repeated) = coords.get(closed_at) else {
                return Err(self.error(format!(
                    "face {face}'s {which} ring was closed by repeating corner {closed_at}, which \
                     its appearance supplies no texture coordinate for"
                )));
            };
            uv.push(to_citygml_uv(*repeated));
        }
        if uv.len() != ring.len {
            return Err(self.error(format!(
                "face {face}'s {which} ring has {} corners but {} texture coordinates",
                ring.len,
                uv.len()
            )));
        }
        Ok(uv)
    }

    /// The merged slot for a local palette index, converting the material on
    /// first use.
    fn slot(&mut self, local: u32) -> Result<MaterialSlot, SinkError> {
        if let Some(slot) = self.slots.get(&local) {
            return Ok(*slot);
        }
        // Read the slice out of `self` first: the borrow it hands back is the
        // appearance's, not this `&mut self`'s, so the palette can be mutated
        // while the material is still in hand.
        let materials = self.materials;
        let (x3d, map, unmapped) = narrow_material(&materials[local as usize]);
        if unmapped > 0 {
            self.omit(UNMAPPED_MAP, UNMAPPED_MAP_REASON, unmapped);
        }

        let texture = match map {
            Some(map) => self.texture_slot(map)?,
            None => None,
        };
        let material_idx = u32::try_from(self.palette.bundle.materials.len())
            .map_err(|_| self.error("its appearance palette is too large".to_string()))?;
        self.palette.bundle.materials.push(x3d);

        let slot = MaterialSlot {
            material_idx,
            texture,
        };
        self.slots.insert(local, slot);
        Ok(slot)
    }

    /// The merged texture index and sampled channel for one material's diffuse
    /// map, or `None` when this writer cannot carry it.
    fn texture_slot(&mut self, map: &Texture) -> Result<Option<(u32, ChannelId)>, SinkError> {
        let channel = map.uv_channel;
        if !self.front_channels.contains_key(&channel) {
            self.omit(UNSAMPLED_UV, UNSAMPLED_UV_REASON, 1);
            return Ok(None);
        }
        let reference = texture_ref(map)?;
        if let Some(index) = self.palette.by_key.get(&reference.key) {
            return Ok(Some((*index, channel)));
        }
        let index = u32::try_from(self.palette.bundle.textures.len())
            .map_err(|_| self.error("its appearance references too many images".to_string()))?;
        self.palette.bundle.textures.push(GmlTexture {
            key: reference.key.clone(),
            uri: fallback_uri(&reference),
        });
        self.palette.by_key.insert(reference.key.clone(), index);
        self.palette.textures.push(reference);
        Ok(Some((index, channel)))
    }

    /// Record one thing this leaf's appearance carried that the document has no
    /// place for, aggregated by kind.
    fn omit(&mut self, kind: &'static str, reason: &'static str, count: usize) {
        match self
            .omissions
            .iter_mut()
            .find(|omission| omission.geometry == kind)
        {
            Some(omission) => omission.count += count,
            None => self.omissions.push(GeometryOmission {
                geometry: kind,
                reason,
                count,
            }),
        }
    }

    /// An error naming the feature and the leaf it came from.
    fn error(&self, detail: String) -> SinkError {
        SinkError::CityGmlWriter(format!(
            "feature {}: {} {detail}",
            self.context.feature, self.context.geometry
        ))
    }
}

/// Narrow one native material to the `app:X3DMaterial` CityGML 2.0 can carry,
/// returning also its diffuse / base-colour map and how many other maps had to
/// be dropped.
fn narrow_material(material: &Material) -> (X3DMaterial, Option<&Texture>, usize) {
    match material {
        Material::Phong(phong) => (
            X3DMaterial {
                diffuse_color: color(phong.diffuse),
                specular_color: color(phong.specular),
                ambient_intensity: f64::from(phong.ambient_intensity),
            },
            phong.diffuse_map.as_ref(),
            [&phong.emissive_map, &phong.normal_map]
                .iter()
                .filter(|map| map.is_some())
                .count(),
        ),
        // PBR's base colour is the closest thing to a diffuse colour; its
        // metallic, roughness and emissive factors have no X3DMaterial slot, so
        // the specular colour and ambient intensity fall back to the values
        // CityGML readers use for a material that declares neither.
        Material::Pbr(pbr) => {
            let default = X3DMaterial::default();
            (
                X3DMaterial {
                    diffuse_color: Color::new(
                        f64::from(pbr.base_color[0]),
                        f64::from(pbr.base_color[1]),
                        f64::from(pbr.base_color[2]),
                    ),
                    specular_color: default.specular_color,
                    ambient_intensity: default.ambient_intensity,
                },
                pbr.base_color_map.as_ref(),
                [
                    &pbr.metallic_roughness_map,
                    &pbr.normal_map,
                    &pbr.occlusion_map,
                    &pbr.emissive_map,
                ]
                .iter()
                .filter(|map| map.is_some())
                .count(),
            )
        }
    }
}

fn color(rgb: [f32; 3]) -> Color {
    Color::new(f64::from(rgb[0]), f64::from(rgb[1]), f64::from(rgb[2]))
}

/// The staging reference for one texture's raster.
///
/// A URI-backed raster keys on its URI string, which is what keeps the writer's
/// `app:imageURI` rewrite identical to the legacy path's. An in-memory one — an
/// OBJ `map_Kd` or a glTF/GLB packed image — has no URI at all, so it keys on a
/// hash of its bytes: two leaves carrying the same image then stage it once, and
/// the name is stable across runs of the same input.
fn texture_ref(map: &Texture) -> Result<TextureRef, SinkError> {
    match &*map.raster {
        Raster::Uri(uri) => {
            let url = url::Url::parse(uri.as_str()).map_err(|e| {
                SinkError::CityGmlWriter(format!(
                    "texture image URI `{}` is not a URL that can be staged: {e}",
                    uri.as_str()
                ))
            })?;
            Ok(TextureRef {
                key: url.to_string(),
                source: TextureSource::Uri(url),
            })
        }
        Raster::InMemory(data) => Ok(TextureRef {
            key: content_key(data.bytes.as_ref()),
            source: TextureSource::InMemory {
                mime: data.mime_type,
                bytes: data.bytes.clone(),
            },
        }),
    }
}

/// What `app:imageURI` says if this image is never staged: its source URI, or —
/// for bytes that have no source — the name it would have been staged under, so
/// the reference is at least self-consistent.
fn fallback_uri(reference: &TextureRef) -> String {
    match &reference.source {
        TextureSource::Uri(url) => url.to_string(),
        TextureSource::InMemory { mime, .. } => {
            format!("{}.{}", reference.key, TextureSource::extension(*mime))
        }
    }
}

/// A short, stable identity for a block of image bytes.
///
/// Not cryptographic and not collision-proof: it only has to distinguish the
/// handful of images one document references, and to be the same on every run
/// over the same input so a staged file name is reproducible.
///
/// FNV-1a is spelled out here rather than taken from
/// [`DefaultHasher`](std::collections::hash_map::DefaultHasher) because this
/// value is *observable output*: it becomes the staged file's name and the
/// `app:imageURI` that points at it, and
/// `engine/testing/data/testcases/citygml_writer/gltf_textured/expected_output.gml`
/// names one. `DefaultHasher`'s algorithm is explicitly unspecified across Rust
/// releases, so a toolchain upgrade could have renamed every staged image and
/// broken that expectation; this one is fixed by its constants.
fn content_key(bytes: &[u8]) -> String {
    const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{hash:016x}")
}

const EXTRA_THEME: &str = "appearance theme";
const EXTRA_THEME_REASON: &str =
    "this writer emits one app:Appearance per feature, under the default theme";
const BACK_SIDE: &str = "back-side appearance binding";
const BACK_SIDE_REASON: &str =
    "CityGML back-side surface data is not emitted; only the front side is painted";
const PROJECTIVE_UV: &str = "world-to-texture UV set";
const PROJECTIVE_UV_REASON: &str =
    "a projective world-to-texture matrix has no per-corner texture coordinates to write into \
     app:textureCoordinates";
const UNMAPPED_MAP: &str = "material texture map";
const UNMAPPED_MAP_REASON: &str =
    "app:X3DMaterial carries no emissive, normal, occlusion or metallic-roughness map; only the \
     diffuse map becomes an app:ParameterizedTexture";
const UNSAMPLED_UV: &str = "textured material";
const UNSAMPLED_UV_REASON: &str =
    "its map samples a UV channel the theme supplies no explicit front-side coordinates for, so \
     it is written colour-only";

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::sync::Arc;

    use bytes::Bytes;
    use reearth_flow_common::image::MimeType;
    use reearth_flow_common::uri::Uri;
    use reearth_flow_geometry::appearance::{
        AlphaMode, MaterialIndex, PbrMaterial, PhongMaterial, RasterData, Sampler, TexMatrix,
        ThemeId,
    };
    use reearth_flow_geometry::coordinate::CoordinateFrame;
    use reearth_flow_geometry::polygon::{Polygon3D, PolygonFace};
    use reearth_flow_geometry::triangular_mesh::TriangularMesh3D;

    use super::*;

    fn theme() -> ThemeId {
        ThemeId(Arc::from("rgbTexture"))
    }

    fn context() -> LeafContext<'static> {
        LeafContext {
            feature: "bldg-1",
            geometry: "Polygon",
        }
    }

    /// Every component is a negative power of two, so `f32` → `f64` widening is
    /// exact and an assertion can be written as a decimal literal.
    fn phong(diffuse: [f32; 3], map: Option<Texture>) -> Material {
        Material::Phong(PhongMaterial {
            diffuse,
            specular: [0.125, 0.25, 0.5],
            emissive: [0.0; 3],
            ambient_intensity: 0.5,
            shininess: 0.0,
            transparency: 0.0,
            diffuse_map: map,
            emissive_map: None,
            normal_map: None,
        })
    }

    fn texture(uri: &str) -> Texture {
        Texture {
            raster: Arc::new(Raster::Uri(Uri::from_str(uri).unwrap())),
            sampler: Sampler::default(),
            transform: None,
            uv_channel: ChannelId::default(),
        }
    }

    fn in_memory_texture(mime: MimeType, bytes: &'static [u8]) -> Texture {
        Texture {
            raster: Arc::new(Raster::InMemory(RasterData {
                mime_type: mime,
                bytes: Bytes::from_static(bytes),
            })),
            sampler: Sampler::default(),
            transform: None,
            uv_channel: ChannelId::default(),
        }
    }

    /// A closed four-corner triangle carrying `material`, with `uv` per corner.
    fn polygon(material: Material, uv: Option<Vec<[f64; 2]>>) -> Polygon3D {
        let mut polygon = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        polygon
            .set_appearance(
                theme(),
                material,
                uv.map(|uv| UvSource::Explicit(uv.into_boxed_slice())),
            )
            .unwrap();
        polygon
    }

    /// The corner bookkeeping the converter records for a ring that was already
    /// closed: `len` corners, nothing appended.
    fn closed_ring(len: usize) -> RingCorners {
        RingCorners {
            corners: 0..len,
            closure: None,
            len,
        }
    }

    fn face(exterior: RingCorners) -> FaceCorners {
        FaceCorners {
            face: 0,
            exterior,
            interiors: Vec::new(),
        }
    }

    fn resolve_one(appearance: &Appearance, faces: &[FaceCorners]) -> (Palette, Resolved) {
        let mut palette = Palette::default();
        let resolved = resolve(appearance, faces, &mut palette, &context()).unwrap();
        (palette, resolved)
    }

    // Materials

    /// A Phong material's first three fields are exactly what `app:X3DMaterial`
    /// carries, so nothing is lost on the way out.
    #[test]
    fn a_phong_material_becomes_an_x3d_material() {
        let polygon = polygon(phong([0.25, 0.5, 0.75], None), None);

        let (palette, resolved) = resolve_one(
            polygon.appearance().as_ref().unwrap(),
            &[face(closed_ring(4))],
        );

        assert_eq!(palette.bundle.materials.len(), 1);
        let material = &palette.bundle.materials[0];
        assert_eq!(material.diffuse_color.r, 0.25);
        assert_eq!(material.diffuse_color.b, 0.75);
        assert_eq!(material.specular_color.g, 0.25);
        assert_eq!(material.ambient_intensity, 0.5);
        assert_eq!(resolved.bindings[0].material_idx, Some(0));
        assert_eq!(resolved.bindings[0].texture_idx, None);
        assert!(resolved.omissions.is_empty());
    }

    /// PBR's base colour is the only thing `app:X3DMaterial` has a slot for; the
    /// maps it cannot carry are reported rather than silently dropped.
    #[test]
    fn a_pbr_material_folds_its_base_colour_onto_diffuse_and_reports_its_maps() {
        let material = Material::Pbr(PbrMaterial {
            base_color: [0.125, 0.25, 0.75, 1.0],
            metallic: 0.5,
            roughness: 0.5,
            emissive: [0.0; 3],
            base_color_map: None,
            metallic_roughness_map: Some(texture("file:///t/mr.png")),
            normal_map: Some(texture("file:///t/n.png")),
            occlusion_map: None,
            emissive_map: None,
            alpha_mode: AlphaMode::Opaque,
            double_sided: false,
        });
        // The maps this writer cannot carry still make the material textured, so
        // its channel still needs a UV set to be well-formed.
        let polygon = polygon(
            material,
            Some(vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0], [0.0, 0.0]]),
        );

        let (palette, resolved) = resolve_one(
            polygon.appearance().as_ref().unwrap(),
            &[face(closed_ring(4))],
        );

        assert_eq!(palette.bundle.materials[0].diffuse_color.r, 0.125);
        assert_eq!(palette.bundle.materials[0].diffuse_color.b, 0.75);
        let unmapped = resolved
            .omissions
            .iter()
            .find(|o| o.geometry == UNMAPPED_MAP)
            .expect("the two unmappable maps are reported");
        assert_eq!(unmapped.count, 2);
    }

    // UV

    /// The one narrowing a diff cannot show: CityGML's texture origin is
    /// bottom-left and Flow's is top-left, so every `v` is mirrored on the way
    /// out. `u` is untouched.
    #[test]
    fn the_v_ordinate_is_flipped_back_to_citygmls_bottom_left_origin() {
        let uv = vec![[0.0, 0.0], [1.0, 0.25], [0.5, 1.0], [0.0, 0.0]];
        let polygon = polygon(
            phong([1.0; 3], Some(texture("file:///t/wall.png"))),
            Some(uv),
        );

        let (_, resolved) = resolve_one(
            polygon.appearance().as_ref().unwrap(),
            &[face(closed_ring(4))],
        );

        assert_eq!(
            resolved.bindings[0].uv_exterior,
            vec![[0.0, 1.0], [1.0, 0.75], [0.5, 0.0], [0.0, 1.0]],
        );
    }

    /// Flipping twice is the identity, which is the round-trip statement: the
    /// CityGML reader flips on ingest, this flips back, and a `v` that came out
    /// of a source document goes back into one unchanged.
    #[test]
    fn flipping_twice_returns_the_source_coordinate() {
        for v in [0.0, 0.25, 0.5, 1.0] {
            assert_eq!(to_citygml_uv(to_citygml_uv([0.3, v])), [0.3, v]);
        }
    }

    /// A ring the converter had to close gains one UV, and it is the UV of the
    /// corner the closing one duplicates — not a repeat of the last corner's.
    #[test]
    fn closing_a_ring_appends_the_duplicated_corners_uv() {
        // Three corners in the mesh's buffer; the emitted ring has four.
        let mut mesh = TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [0u32, 1, 2],
        )
        .unwrap();
        mesh.set_appearance(
            theme(),
            phong([1.0; 3], Some(texture("file:///t/wall.png"))),
            Some(UvSource::Explicit(
                vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]].into_boxed_slice(),
            )),
        )
        .unwrap();

        let faces = [face(RingCorners {
            corners: 0..3,
            closure: Some(0),
            len: 4,
        })];
        let (_, resolved) = resolve_one(mesh.appearance().as_ref().unwrap(), &faces);

        assert_eq!(
            resolved.bindings[0].uv_exterior,
            vec![[0.0, 1.0], [1.0, 1.0], [0.5, 0.0], [0.0, 1.0]],
            "the appended UV repeats corner 0's, flipped like the rest"
        );
    }

    /// A UV array that does not cover the corners a ring claims is an error
    /// naming the feature and the face, not a panic and not a short UV list.
    #[test]
    fn a_uv_array_too_short_for_the_ring_is_an_error() {
        let uv = vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0], [0.0, 0.0]];
        let polygon = polygon(
            phong([1.0; 3], Some(texture("file:///t/wall.png"))),
            Some(uv),
        );
        // Claim a fifth corner the theme has no coordinate for.
        let faces = [face(RingCorners {
            corners: 0..5,
            closure: None,
            len: 5,
        })];

        let mut palette = Palette::default();
        let message = resolve(
            polygon.appearance().as_ref().unwrap(),
            &faces,
            &mut palette,
            &context(),
        )
        .unwrap_err()
        .to_string();

        assert!(message.contains("bldg-1"), "{message}");
        assert!(message.contains("face 0"), "{message}");
    }

    /// The post-closure length check: a ring that says it emitted five corners
    /// but whose UV covers four is a bookkeeping bug, and writing it would shift
    /// every texture coordinate by one corner.
    #[test]
    fn a_uv_length_that_disagrees_with_the_emitted_ring_is_an_error() {
        let uv = vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0], [0.0, 0.0]];
        let polygon = polygon(
            phong([1.0; 3], Some(texture("file:///t/wall.png"))),
            Some(uv),
        );
        let faces = [face(RingCorners {
            corners: 0..4,
            closure: None,
            len: 5,
        })];

        let mut palette = Palette::default();
        let message = resolve(
            polygon.appearance().as_ref().unwrap(),
            &faces,
            &mut palette,
            &context(),
        )
        .unwrap_err()
        .to_string();

        assert!(message.contains("5 corners"), "{message}");
        assert!(message.contains("4 texture coordinates"), "{message}");
    }

    /// A retained world-to-texture matrix has no per-corner samples to write, so
    /// the material sampling it renders colour-only — a warning, not an error:
    /// the geometry is still correct and its colour is still right.
    #[test]
    fn a_projective_uv_set_downgrades_its_material_to_colour_only() {
        let mut mesh = TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [0u32, 1, 2],
        )
        .unwrap();
        // A textured material must be given *some* UV for its channel;
        // `WorldToTexture` satisfies the coupling without supplying corners.
        mesh.set_appearance(
            theme(),
            phong([1.0; 3], Some(texture("file:///t/wall.png"))),
            Some(UvSource::WorldToTexture(TexMatrix([[0.0; 4]; 3]))),
        )
        .unwrap();

        let (palette, resolved) =
            resolve_one(mesh.appearance().as_ref().unwrap(), &[face(closed_ring(3))]);

        assert_eq!(resolved.bindings[0].material_idx, Some(0));
        assert_eq!(resolved.bindings[0].texture_idx, None);
        assert!(palette.textures.is_empty(), "nothing to stage");
        assert!(resolved
            .omissions
            .iter()
            .any(|omission| omission.geometry == PROJECTIVE_UV));
        assert!(resolved
            .omissions
            .iter()
            .any(|omission| omission.geometry == UNSAMPLED_UV));
    }

    // Bindings

    /// A per-face binding paints each face independently, and its indices are
    /// shifted into the feature-level palette.
    #[test]
    fn a_per_face_binding_paints_each_face_independently() {
        let mut mesh = TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            [0u32, 1, 2, 1, 3, 2],
        )
        .unwrap();
        mesh.set_appearance_with_binding(
            theme(),
            vec![phong([1.0, 0.0, 0.0], None), phong([0.0, 1.0, 0.0], None)],
            FaceBinding::PerFace(vec![MaterialIndex::new(1), None]),
            Default::default(),
        )
        .unwrap();

        let faces = [
            FaceCorners {
                face: 0,
                exterior: closed_ring(3),
                interiors: Vec::new(),
            },
            FaceCorners {
                face: 1,
                exterior: closed_ring(3),
                interiors: Vec::new(),
            },
        ];
        let (palette, resolved) = resolve_one(mesh.appearance().as_ref().unwrap(), &faces);

        assert_eq!(resolved.bindings[0].material_idx, Some(0));
        assert_eq!(resolved.bindings[1].material_idx, None);
        // Only the bound material was converted; the unbound one costs nothing.
        assert_eq!(palette.bundle.materials.len(), 1);
        assert_eq!(palette.bundle.materials[0].diffuse_color.g, 1.0);
    }

    /// A per-face binding whose length disagrees with the emitted face count
    /// describes a different mesh; painting whichever faces line up would paint
    /// the wrong ones.
    #[test]
    fn a_per_face_binding_of_the_wrong_length_is_an_error() {
        let mut mesh = TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [0u32, 1, 2],
        )
        .unwrap();
        mesh.set_appearance_with_binding(
            theme(),
            vec![phong([1.0; 3], None)],
            FaceBinding::PerFace(vec![MaterialIndex::new(0)]),
            Default::default(),
        )
        .unwrap();

        let faces = [
            FaceCorners {
                face: 0,
                exterior: closed_ring(3),
                interiors: Vec::new(),
            },
            FaceCorners {
                face: 1,
                exterior: closed_ring(3),
                interiors: Vec::new(),
            },
        ];
        let mut palette = Palette::default();
        let message = resolve(
            mesh.appearance().as_ref().unwrap(),
            &faces,
            &mut palette,
            &context(),
        )
        .unwrap_err()
        .to_string();

        assert!(message.contains("binds 1 faces"), "{message}");
        assert!(message.contains("2 were written"), "{message}");
    }

    // Textures

    /// A URI-backed raster keys on its URI string, which is what makes the
    /// writer's `app:imageURI` rewrite identical to the legacy path's.
    #[test]
    fn a_uri_backed_raster_keys_on_its_uri() {
        let uv = vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0], [0.0, 0.0]];
        let polygon = polygon(
            phong([1.0; 3], Some(texture("file:///t/wall.png"))),
            Some(uv),
        );

        let (palette, resolved) = resolve_one(
            polygon.appearance().as_ref().unwrap(),
            &[face(closed_ring(4))],
        );

        assert_eq!(resolved.bindings[0].texture_idx, Some(0));
        assert_eq!(palette.textures.len(), 1);
        assert_eq!(palette.textures[0].key, "file:///t/wall.png");
        assert_eq!(palette.bundle.textures[0].uri, "file:///t/wall.png");
        assert!(matches!(palette.textures[0].source, TextureSource::Uri(_)));
    }

    /// An in-memory raster has no URI to key by, so it keys on its bytes and
    /// carries them through for the shell to write.
    #[test]
    fn an_in_memory_raster_keys_on_its_content_and_carries_its_bytes() {
        let uv = vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0], [0.0, 0.0]];
        let map = in_memory_texture(MimeType::ImageWebp, b"webp-bytes");
        let polygon = polygon(phong([1.0; 3], Some(map)), Some(uv));

        let (palette, _) = resolve_one(
            polygon.appearance().as_ref().unwrap(),
            &[face(closed_ring(4))],
        );

        assert_eq!(palette.textures[0].key, content_key(b"webp-bytes"));
        assert_eq!(
            palette.bundle.textures[0].uri,
            format!("{}.webp", content_key(b"webp-bytes"))
        );
        match &palette.textures[0].source {
            TextureSource::InMemory { mime, bytes } => {
                assert_eq!(*mime, MimeType::ImageWebp);
                assert_eq!(bytes.as_ref(), b"webp-bytes");
            }
            other => panic!("expected in-memory bytes, got {other:?}"),
        }
    }

    /// The content key is observable output — it names the staged file and the
    /// `app:imageURI` that points at it, and a workflow expectation
    /// (`citygml_writer/gltf_textured`) has one written into it. Pinning the
    /// literal here is what turns a change to the hash from a confusing
    /// end-to-end XML diff into a one-line unit failure that says so.
    #[test]
    fn the_content_key_is_pinned_to_its_algorithm() {
        assert_eq!(content_key(b"webp-bytes"), "a470fa1d13e58dd9");
        assert_eq!(content_key(b""), "cbf29ce484222325");
        assert_ne!(content_key(b"webp-bytes"), content_key(b"webp-byte"));
    }

    /// The same image under two leaves stages once and is one
    /// `app:ParameterizedTexture`.
    #[test]
    fn one_image_referenced_twice_enters_the_palette_once() {
        let uv = || vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0], [0.0, 0.0]];
        let first = polygon(
            phong([1.0; 3], Some(texture("file:///t/wall.png"))),
            Some(uv()),
        );
        let second = polygon(
            phong([0.5; 3], Some(texture("file:///t/wall.png"))),
            Some(uv()),
        );

        let mut palette = Palette::default();
        let a = resolve(
            first.appearance().as_ref().unwrap(),
            &[face(closed_ring(4))],
            &mut palette,
            &context(),
        )
        .unwrap();
        let b = resolve(
            second.appearance().as_ref().unwrap(),
            &[face(closed_ring(4))],
            &mut palette,
            &context(),
        )
        .unwrap();

        assert_eq!(a.bindings[0].texture_idx, Some(0));
        assert_eq!(b.bindings[0].texture_idx, Some(0));
        assert_eq!(palette.textures.len(), 1);
        // Materials still merge with an offset: two leaves, two materials.
        assert_eq!(a.bindings[0].material_idx, Some(0));
        assert_eq!(b.bindings[0].material_idx, Some(1));
    }

    // Theme selection

    /// The selected theme's own name is what the document declares, and every
    /// other theme is reported.
    #[test]
    fn the_selected_theme_is_recorded_and_the_others_reported() {
        let mut polygon = polygon(phong([1.0; 3], None), None);
        polygon
            .set_appearance(
                ThemeId(Arc::from("nightTexture")),
                phong([0.0; 3], None),
                None,
            )
            .unwrap();

        let (palette, resolved) = resolve_one(
            polygon.appearance().as_ref().unwrap(),
            &[face(closed_ring(4))],
        );

        // The first theme added is the default, so that is the one painted.
        assert_eq!(palette.bundle.theme.as_deref(), Some("rgbTexture"));
        let extra = resolved
            .omissions
            .iter()
            .find(|o| o.geometry == EXTRA_THEME)
            .expect("the second theme is reported");
        assert_eq!(extra.count, 1);
    }

    /// A back-side binding is dropped, with a warning rather than an error: the
    /// front side is still painted correctly.
    #[test]
    fn a_back_side_binding_is_reported_and_dropped() {
        let mut polygon = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        polygon
            .set_two_sided_appearance(
                theme(),
                PolygonFace::single(phong([1.0, 0.0, 0.0], None), None),
                PolygonFace::single(phong([0.0, 0.0, 1.0], None), None),
            )
            .unwrap();

        let (palette, resolved) = resolve_one(
            polygon.appearance().as_ref().unwrap(),
            &[face(closed_ring(4))],
        );

        // The front material, not the back one.
        assert_eq!(palette.bundle.materials[0].diffuse_color.r, 1.0);
        assert!(resolved.omissions.iter().any(|o| o.geometry == BACK_SIDE));
    }
}
