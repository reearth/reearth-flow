//! Intermediate-data encoding for [`Appearance`], with UV nested to mirror the
//! host geometry's faces and rings. Every conversion takes the host's
//! [`FaceRings`] layout, which an `Appearance` does not itself carry.

use serde::{Deserialize, Serialize};

use crate::error::Error;

use super::{ChannelId, FaceBinding, Material, Side, TexMatrix, ThemeId};

/// Corner counts of one host face.
pub(crate) struct FaceRings {
    /// Corners in the exterior ring.
    pub(crate) exterior: usize,
    /// Corners in each hole ring, in the host's hole order.
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

    /// Total corners across all rings.
    fn corners(&self) -> usize {
        self.exterior + self.holes.iter().sum::<usize>()
    }
}

/// UV coordinates for one host face, nested to mirror that face's rings.
///
/// The enclosing array runs parallel to the host's own face list: one entry for
/// a polygon, one per member of `faces`, or one per member of `triangles`.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "schema", schemars(title = "Per-face UV"))]
pub(crate) struct FaceUv {
    /// UV for the face's exterior ring, one pair per corner.
    #[cfg_attr(feature = "schema", schemars(title = "Exterior ring UV"))]
    pub(crate) exterior: Vec<[f64; 2]>,
    /// UV for each hole ring, in the host's hole order, one pair per corner.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "schema", schemars(title = "Hole ring UV"))]
    pub(crate) holes: Vec<Vec<[f64; 2]>>,
}

/// Wire form of [`UvSource`]; see it for the UV coordinate convention.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) enum UvSource {
    /// Per-corner coordinates, nested to mirror the host geometry's faces and
    /// rings.
    Explicit(Vec<FaceUv>),
    /// A 3x4 world-to-texture projective matrix, applying to the whole surface.
    WorldToTexture(TexMatrix),
}

/// One UV set, owned by the theme it belongs to. A map finds its set by matching
/// both `side` and `channel`.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "schema", schemars(title = "UV set"))]
pub(crate) struct UvSet {
    /// Surface side these coordinates parameterise.
    #[cfg_attr(feature = "schema", schemars(title = "Surface side"))]
    pub(crate) side: Side,
    /// Material-local UV channel these coordinates serve, matched against a
    /// texture's `uv_channel`.
    #[cfg_attr(feature = "schema", schemars(title = "UV channel"))]
    pub(crate) channel: ChannelId,
    #[cfg_attr(feature = "schema", schemars(title = "UV source"))]
    pub(crate) uv: UvSource,
}

/// One theme's face-to-material binding plus the UV sets that theme's textured
/// materials sample.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "schema", schemars(title = "Theme binding"))]
pub(crate) struct ThemeBinding {
    /// This theme's name, unique within the appearance.
    #[cfg_attr(feature = "schema", schemars(title = "Theme name"))]
    pub(crate) theme: ThemeId,
    /// Front-side face-to-material binding.
    #[cfg_attr(feature = "schema", schemars(title = "Front face materials"))]
    pub(crate) front: FaceBinding,
    /// Back-side binding; absent means single-sided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "schema", schemars(title = "Back face materials"))]
    pub(crate) back: Option<FaceBinding>,
    /// This theme's UV pool: one entry per `(side, channel)` its materials
    /// reference.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[cfg_attr(feature = "schema", schemars(title = "UV sets"))]
    pub(crate) uv_sets: Vec<UvSet>,
}

/// Materials, themes, per-face material bindings and per-theme UV for one surface
/// geometry.
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct Appearance {
    /// Material palette; the bindings in `themes` index into it by position.
    #[cfg_attr(feature = "schema", schemars(title = "Material palette"))]
    pub(crate) materials: Vec<Material>,
    /// One independent binding per theme; never empty.
    #[cfg_attr(feature = "schema", schemars(title = "Themes"))]
    pub(crate) themes: Vec<ThemeBinding>,
    /// Which of `themes` a single-theme consumer (glTF / OBJ / CZML / 3D Tiles)
    /// should render.
    #[cfg_attr(feature = "schema", schemars(title = "Default theme"))]
    pub(crate) default_theme: ThemeId,
}

/// Split a flat, corner-parallel UV array into per-face rings. `flat` must hold
/// exactly as many coordinates as `layout` has corners.
fn nest_uv(flat: &[[f64; 2]], layout: &[FaceRings]) -> Result<Vec<FaceUv>, Error> {
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
        faces.push(FaceUv { exterior, holes });
    }
    Ok(faces)
}

/// Concatenate per-face UV rings back into the flat, corner-parallel array.
/// `faces` must match `layout` face for face and ring for ring.
fn flatten_uv(faces: &[FaceUv], layout: &[FaceRings]) -> Result<Box<[[f64; 2]]>, Error> {
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

impl Appearance {
    /// Encode an appearance against the host geometry's ring layout.
    pub(crate) fn encode(
        appearance: &super::Appearance,
        layout: &[FaceRings],
    ) -> Result<Self, Error> {
        let themes = appearance
            .themes()
            .iter()
            .map(|binding| {
                let uv_sets = binding
                    .uv_sets
                    .iter()
                    .map(|set| {
                        let uv = match &set.uv {
                            super::UvSource::Explicit(flat) => {
                                UvSource::Explicit(nest_uv(flat, layout)?)
                            }
                            super::UvSource::WorldToTexture(matrix) => {
                                UvSource::WorldToTexture(*matrix)
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
                    theme: binding.theme.clone(),
                    front: binding.front.clone(),
                    back: binding.back.clone(),
                    uv_sets,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(Appearance {
            materials: appearance.materials().to_vec(),
            themes,
            default_theme: appearance.default_theme().clone(),
        })
    }

    /// Decode back into an appearance. Every UV set must match `layout`.
    pub(crate) fn decode(self, layout: &[FaceRings]) -> Result<super::Appearance, Error> {
        let themes = self
            .themes
            .into_iter()
            .map(|binding| {
                let uv_sets = binding
                    .uv_sets
                    .into_iter()
                    .map(|set| {
                        let uv = match set.uv {
                            UvSource::Explicit(faces) => {
                                super::UvSource::Explicit(flatten_uv(&faces, layout)?)
                            }
                            UvSource::WorldToTexture(matrix) => {
                                super::UvSource::WorldToTexture(matrix)
                            }
                        };
                        Ok(super::UvSet {
                            side: set.side,
                            channel: set.channel,
                            uv,
                        })
                    })
                    .collect::<Result<Vec<_>, Error>>()?;
                Ok(super::ThemeBinding {
                    theme: binding.theme,
                    front: binding.front,
                    back: binding.back,
                    uv_sets,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        Ok(super::Appearance::from_parts(
            self.materials,
            themes,
            self.default_theme,
        ))
    }
}

/// Encode an optional appearance; `None` passes straight through.
pub(crate) fn encode_appearance(
    appearance: &Option<super::Appearance>,
    layout: &[FaceRings],
) -> Result<Option<Appearance>, Error> {
    appearance
        .as_ref()
        .map(|a| Appearance::encode(a, layout))
        .transpose()
}

/// Decode an optional appearance; `None` passes straight through.
pub(crate) fn decode_appearance(
    wire: Option<Appearance>,
    layout: &[FaceRings],
) -> Result<Option<super::Appearance>, Error> {
    wire.map(|w| w.decode(layout)).transpose()
}
