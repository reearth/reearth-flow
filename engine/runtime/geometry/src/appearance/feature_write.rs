//! Lossless intermediate-data encoding for [`Appearance`].
//!
//! The stored `UvSource::Explicit` is one flat array running parallel to the host
//! geometry's corner buffer. The leaf wire forms decode that corner buffer into
//! explicit rings, so a flat UV array would be parallel to a buffer the wire form
//! no longer shows. Here it is nested to mirror those rings instead, which makes
//! the correspondence readable and checkable.
//!
//! Nesting needs the host's ring lengths, which an `Appearance` does not carry, so
//! every conversion takes a [`FaceRings`] layout supplied by the leaf.

use serde::{Deserialize, Serialize};

use crate::error::Error;

use super::{
    Appearance, ChannelId, FaceBinding, Material, Side, TexMatrix, ThemeBinding, ThemeId, UvSet,
    UvSource,
};

/// Corner counts of one host face: its exterior ring, then each hole ring in
/// order. Supplied by the leaf being encoded; never serialized.
pub(crate) struct FaceRings {
    pub(crate) exterior: usize,
    pub(crate) holes: Vec<usize>,
}

impl FaceRings {
    /// A face with no holes, e.g. a triangle.
    pub(crate) fn simple(exterior: usize) -> Self {
        FaceRings {
            exterior,
            holes: Vec::new(),
        }
    }

    fn corners(&self) -> usize {
        self.exterior + self.holes.iter().sum::<usize>()
    }
}

/// UV coordinates for one host face, nested to mirror that face's rings.
///
/// Each entry pairs up positionally with a corner of the host geometry: `exterior`
/// runs parallel to the face's exterior ring, and `holes[j]` to its `j`-th hole
/// ring. The face itself is identified by position, so the enclosing array runs
/// parallel to the host's own face list:
///
/// - `Polygon2D` / `Polygon3D` — exactly one entry, whose `exterior` matches the
///   polygon's `exterior` and whose `holes` match its `interiors`.
/// - `PolygonMesh2D` / `PolygonMesh3DData` — one entry per member of `faces`,
///   matching that face's `exterior` and `holes`.
/// - `TriangularMesh2D` / `TriangularMesh3DData` — one entry per member of
///   `triangles`, whose `exterior` holds that triangle's three corners and whose
///   `holes` is empty.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct FaceUvWire {
    /// UV for the face's exterior ring, one pair per corner.
    pub(crate) exterior: Vec<[f64; 2]>,
    /// UV for each hole ring, in the host's hole order, one pair per corner.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) holes: Vec<Vec<[f64; 2]>>,
}

/// Where a UV set's coordinates come from: sampled per corner, or evaluated from
/// a matrix.
///
/// # Coordinate convention
///
/// The UV origin is **top-left, with `v` increasing downward** — the glTF and
/// image-storage convention, in which `(0, 0)` is the first texel. Every producer
/// and consumer assumes it; a source using a bottom-left origin (CityGML) is
/// converted at ingest, and a consumer whose own convention differs must convert
/// at its own boundary.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) enum UvSourceWire {
    /// Per-corner coordinates, nested to mirror the host geometry's faces and
    /// rings. See [`FaceUvWire`] for how the nesting lines up with each geometry
    /// type.
    Explicit(Vec<FaceUvWire>),
    /// A 3x4 world-to-texture projective matrix, evaluated per position instead of
    /// stored per corner. Applies to the whole surface, so it carries no nesting.
    WorldToTexture(TexMatrix),
}

/// One UV set belonging to the theme that owns it.
///
/// A textured map does not point at a UV set directly; the set is found by
/// matching. To resolve the UV a face samples:
///
/// 1. Pick the [`ThemeBindingWire`] whose `theme` you are rendering.
/// 2. Look the face up in that binding's `front` (or `back`) to get a material
///    index, and remember which of the two sides you used.
/// 3. Index the appearance's `materials` palette with it.
/// 4. Read the `uv_channel` of the material map you are shading, then take the
///    entry of the same theme's `uv_sets` whose `side` and `channel` both match.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct UvSetWire {
    /// Surface side these coordinates parameterise; `Front` in the common
    /// single-sided case.
    pub(crate) side: Side,
    /// Material-local UV channel these coordinates serve, matched against a
    /// texture's `uv_channel`; `0` in the common single-map case. The value has no
    /// meaning outside its own theme and side.
    pub(crate) channel: ChannelId,
    pub(crate) uv: UvSourceWire,
}

/// One theme's face-to-material binding plus the UV sets that theme's textured
/// materials sample.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct ThemeBindingWire {
    /// This theme's name, unique within the appearance.
    pub(crate) theme: ThemeId,
    /// Front-side face-to-material binding.
    pub(crate) front: FaceBinding,
    /// Back-side binding; absent means single-sided, i.e. only the front is
    /// painted. When present the back side has its own mapping, which may bind
    /// different materials or leave faces unbound.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) back: Option<FaceBinding>,
    /// This theme's UV pool: one entry per `(side, channel)` its materials
    /// reference.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) uv_sets: Vec<UvSetWire>,
}

/// Materials, themes, per-face material bindings and per-theme UV for one surface
/// geometry.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct AppearanceWire {
    /// Material palette; the bindings in `themes` index into it by position.
    pub(crate) materials: Vec<Material>,
    /// One independent binding per theme; never empty.
    pub(crate) themes: Vec<ThemeBindingWire>,
    /// Which of `themes` a single-theme consumer (glTF / OBJ / CZML / 3D Tiles)
    /// should render.
    pub(crate) default_theme: ThemeId,
}

/// Split a flat, corner-parallel UV array into per-face rings.
fn nest_uv(flat: &[[f64; 2]], layout: &[FaceRings]) -> Result<Vec<FaceUvWire>, Error> {
    let expected: usize = layout.iter().map(FaceRings::corners).sum();
    if flat.len() != expected {
        return Err(Error::invalid_appearance(format!(
            "UV set has {} coordinates but the geometry has {expected} corners",
            flat.len()
        )));
    }
    let mut faces = Vec::with_capacity(layout.len());
    let mut at = 0usize;
    for face in layout {
        let exterior = flat[at..at + face.exterior].to_vec();
        at += face.exterior;
        let mut holes = Vec::with_capacity(face.holes.len());
        for &hole in &face.holes {
            holes.push(flat[at..at + hole].to_vec());
            at += hole;
        }
        faces.push(FaceUvWire { exterior, holes });
    }
    Ok(faces)
}

/// Concatenate per-face UV rings back into the flat, corner-parallel array,
/// checking the nesting against the host's rings as it goes.
fn flatten_uv(faces: &[FaceUvWire], layout: &[FaceRings]) -> Result<Box<[[f64; 2]]>, Error> {
    if faces.len() != layout.len() {
        return Err(Error::invalid_appearance(format!(
            "UV set covers {} faces but the geometry has {}",
            faces.len(),
            layout.len()
        )));
    }
    let mut flat = Vec::with_capacity(layout.iter().map(FaceRings::corners).sum());
    for (index, (face, rings)) in faces.iter().zip(layout).enumerate() {
        if face.exterior.len() != rings.exterior {
            return Err(Error::invalid_appearance(format!(
                "face {index}: UV exterior ring has {} coordinates but the geometry's has {}",
                face.exterior.len(),
                rings.exterior
            )));
        }
        if face.holes.len() != rings.holes.len() {
            return Err(Error::invalid_appearance(format!(
                "face {index}: UV covers {} hole rings but the geometry has {}",
                face.holes.len(),
                rings.holes.len()
            )));
        }
        flat.extend_from_slice(&face.exterior);
        for (hole_index, (hole, &expected)) in face.holes.iter().zip(&rings.holes).enumerate() {
            if hole.len() != expected {
                return Err(Error::invalid_appearance(format!(
                    "face {index} hole {hole_index}: UV ring has {} coordinates but the \
                     geometry's has {expected}",
                    hole.len()
                )));
            }
            flat.extend_from_slice(hole);
        }
    }
    Ok(flat.into_boxed_slice())
}

impl AppearanceWire {
    /// Encode an appearance against the host geometry's ring layout.
    pub(crate) fn encode(appearance: &Appearance, layout: &[FaceRings]) -> Result<Self, Error> {
        let themes = appearance
            .themes()
            .iter()
            .map(|binding| {
                let uv_sets = binding
                    .uv_sets
                    .iter()
                    .map(|set| {
                        let uv = match &set.uv {
                            UvSource::Explicit(flat) => {
                                UvSourceWire::Explicit(nest_uv(flat, layout)?)
                            }
                            UvSource::WorldToTexture(matrix) => {
                                UvSourceWire::WorldToTexture(*matrix)
                            }
                        };
                        Ok(UvSetWire {
                            side: set.side,
                            channel: set.channel,
                            uv,
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                Ok(ThemeBindingWire {
                    theme: binding.theme.clone(),
                    front: binding.front.clone(),
                    back: binding.back.clone(),
                    uv_sets,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(AppearanceWire {
            materials: appearance.materials().to_vec(),
            themes,
            default_theme: appearance.default_theme().clone(),
        })
    }

    /// Decode back into an appearance, checking every UV set against the host
    /// geometry's ring layout.
    pub(crate) fn decode(self, layout: &[FaceRings]) -> Result<Appearance, Error> {
        let themes = self
            .themes
            .into_iter()
            .map(|binding| {
                let uv_sets = binding
                    .uv_sets
                    .into_iter()
                    .map(|set| {
                        let uv = match set.uv {
                            UvSourceWire::Explicit(faces) => {
                                UvSource::Explicit(flatten_uv(&faces, layout)?)
                            }
                            UvSourceWire::WorldToTexture(matrix) => {
                                UvSource::WorldToTexture(matrix)
                            }
                        };
                        Ok(UvSet {
                            side: set.side,
                            channel: set.channel,
                            uv,
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                Ok(ThemeBinding {
                    theme: binding.theme,
                    front: binding.front,
                    back: binding.back,
                    uv_sets,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Appearance::from_parts(
            self.materials,
            themes,
            self.default_theme,
        ))
    }
}

/// Encode an optional appearance; `None` passes straight through.
pub(crate) fn encode_appearance(
    appearance: &Option<Appearance>,
    layout: &[FaceRings],
) -> Result<Option<AppearanceWire>, Error> {
    appearance
        .as_ref()
        .map(|a| AppearanceWire::encode(a, layout))
        .transpose()
}

/// Decode an optional appearance; `None` passes straight through.
pub(crate) fn decode_appearance(
    wire: Option<AppearanceWire>,
    layout: &[FaceRings],
) -> Result<Option<Appearance>, Error> {
    wire.map(|w| w.decode(layout)).transpose()
}
