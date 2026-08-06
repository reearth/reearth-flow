//! The CityGML document model both geometry worlds converge on.
//!
//! A converter's whole job is to turn one `Feature` into a
//! [`ConvertedCityObject`]; everything downstream of that — XML serialization,
//! the sandboxed write, texture staging — is shared. Nothing here names a
//! geometry type, which is why this module is compiled unconditionally and
//! neither world's converter has to know the other exists.
//!
//! Coordinates are plain `[f64; 3]`, in whatever order the producing world
//! stores its ordinates. Getting them into the axis order CityGML declares is
//! the converter's business, not the writer's: each world exports its own
//! `format_pos_list`, and the writer formats every `gml:posList` — and every
//! [`BoundingEnvelope`] corner — through it.

use std::fmt;

use bytes::Bytes;
use reearth_flow_common::image::MimeType;
use reearth_flow_types::conversion::CrsCoverage;
use reearth_flow_types::material::X3DMaterial;

/// One feature's worth of converted CityGML, plus everything the shared shell
/// needs in order to write the document around it.
#[derive(Debug, Clone)]
pub struct ConvertedCityObject {
    /// One entry per emitted geometry property, in source order.
    pub geometries: Vec<GeometryEntry>,
    /// The materials and textures this object's surfaces index into.
    pub appearance: AppearanceBundle,
    /// Folded over the coordinates that were actually emitted, so geometry the
    /// LOD filter or an omission dropped cannot widen it. `None` when nothing
    /// was emitted.
    pub envelope: Option<BoundingEnvelope>,
    /// What the emitted coordinates are expressed in, folded over the same
    /// leaves as the envelope. The shell turns this into `srsName`.
    pub crs: CrsCoverage,
    /// The images this object's textures reference, for the shell to stage
    /// beside the `.gml`.
    pub textures: Vec<TextureRef>,
    /// Geometry that reached the converter but that CityGML 2.0 has no place
    /// for. Reported, never silently dropped.
    pub omissions: Vec<GeometryOmission>,
}

/// One geometry property of a city object: what LOD it belongs to, what the
/// source called it, and what GML element carries it.
#[derive(Debug, Clone)]
pub struct GeometryEntry {
    pub lod: u8,
    /// The source geometry property's local name (`"lod0RoofEdge"`, `"tin"`,
    /// …), which becomes the wrapper element name verbatim. `None` falls back
    /// to naming the wrapper after the LOD and the GML family.
    pub property: Option<String>,
    pub element: GmlElement,
}

/// The GML geometry families this writer can serialize.
#[derive(Debug, Clone)]
pub enum GmlElement {
    Solid(GmlSolid),
    MultiSolid {
        id: Option<String>,
        solids: Vec<GmlSolid>,
    },
    MultiSurface {
        id: Option<String>,
        surfaces: Vec<GmlSurface>,
    },
    MultiCurve {
        id: Option<String>,
        curves: Vec<Vec<[f64; 3]>>,
    },
}

/// A `gml:Solid`: one exterior shell and, for a solid with voids, one interior
/// shell per void.
#[derive(Debug, Clone)]
pub struct GmlSolid {
    pub id: Option<String>,
    /// The faces of the shell bounding the solid from outside, written as
    /// `gml:exterior/gml:CompositeSurface`.
    pub exterior: Vec<GmlSurface>,
    /// One `gml:interior/gml:CompositeSurface` per void shell. Empty for a
    /// solid without voids, which is every solid the legacy world can produce:
    /// its reader discards interior shells while parsing.
    pub interiors: Vec<Vec<GmlSurface>>,
}

/// One `gml:Polygon`, carrying its rings and its appearance together so the
/// writer can mint a `gml:id` for it exactly when something targets it.
#[derive(Debug, Clone)]
pub struct GmlSurface {
    pub id: Option<String>,
    pub exterior: Vec<[f64; 3]>,
    pub interiors: Vec<Vec<[f64; 3]>>,
    /// Index into `AppearanceBundle::materials` (None if no material)
    pub material_idx: Option<u32>,
    /// Index into `AppearanceBundle::textures` (None if no texture)
    pub texture_idx: Option<u32>,
    /// UV coords for exterior ring, parallel to `exterior` vertices
    pub uv_exterior: Vec<[f64; 2]>,
    /// UV coords for each interior ring, parallel to `interiors` vertices
    pub uv_interiors: Vec<Vec<[f64; 2]>>,
}

/// Appearance data for a feature: the palettes `GmlSurface::material_idx` and
/// `GmlSurface::texture_idx` index into.
#[derive(Debug, Clone, Default)]
pub struct AppearanceBundle {
    /// The name written as `app:theme`.
    ///
    /// `None` keeps the literal this writer has always emitted (`rgbTexture`):
    /// the legacy geometry model carries no theme name, so there is nothing
    /// truer to write. The unified world resolves a real
    /// [`ThemeId`](reearth_flow_geometry::appearance::ThemeId) and puts it here,
    /// because a wrong theme name breaks appearance selection downstream.
    pub theme: Option<String>,
    pub materials: Vec<X3DMaterial>,
    pub textures: Vec<GmlTexture>,
}

impl AppearanceBundle {
    pub fn has_content(&self) -> bool {
        !self.materials.is_empty() || !self.textures.is_empty()
    }
}

/// One `app:ParameterizedTexture`'s image, as the writer needs it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GmlTexture {
    /// The [`TextureRef::key`] this image was staged under, and therefore the
    /// key the writer looks the staged relative path up by.
    pub key: String,
    /// What `app:imageURI` says when nothing was staged under [`key`](Self::key)
    /// — the source URI, which is what the legacy build has always written for a
    /// texture whose file could not be copied.
    pub uri: String,
}

/// The `gml:boundedBy` extent of what was emitted.
#[derive(Debug, Clone)]
pub struct BoundingEnvelope {
    pub lower: [f64; 3],
    pub upper: [f64; 3],
}

impl BoundingEnvelope {
    pub fn merge(&mut self, other: &BoundingEnvelope) {
        for (corner, other) in self.lower.iter_mut().zip(other.lower.iter()) {
            *corner = corner.min(*other);
        }
        for (corner, other) in self.upper.iter_mut().zip(other.upper.iter()) {
            *corner = corner.max(*other);
        }
    }
}

/// An image the converted document references, for the shared shell to stage
/// beside the `.gml` and for the writer to rewrite `app:imageURI` by.
#[derive(Debug, Clone)]
pub struct TextureRef {
    /// The lookup key the writer rewrites `app:imageURI` by. For a URI-backed
    /// raster it *is* the source URI string, which is what keeps the rewrite
    /// identical to what the legacy path has always produced.
    pub key: String,
    pub source: TextureSource,
}

/// Where a referenced image's bytes come from.
#[derive(Debug, Clone)]
pub enum TextureSource {
    /// A raster the shell reads out of storage at this URI.
    Uri(url::Url),
    /// A raster that arrived already decoded into memory — an OBJ `map_Kd` or a
    /// glTF/GLB packed image — with no URI to read it back from. The shell
    /// writes the bytes itself, naming the file from the [`MimeType`].
    InMemory { mime: MimeType, bytes: Bytes },
}

impl TextureSource {
    /// The file extension a staged copy of this image gets.
    ///
    /// A URI-backed raster keeps whatever its source URI's last segment already
    /// says, so this is only consulted for in-memory bytes, whose format is the
    /// closed three-value [`MimeType`] and nothing else.
    pub fn extension(mime: MimeType) -> &'static str {
        match mime {
            MimeType::ImageJpeg => "jpg",
            MimeType::ImagePng => "png",
            MimeType::ImageWebp => "webp",
        }
    }
}

/// One kind of geometry a conversion left out of the document.
///
/// Aggregated per feature, so a collection of a thousand points is reported once
/// rather than a thousand times, and structured rather than a bare log line, so
/// the caller decides how loudly to say it. What it must never be is a silent
/// `continue`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeometryOmission {
    /// The leaf that was not written, named as its geometry world spells it.
    pub geometry: &'static str,
    /// Why CityGML 2.0 has nothing to write it as.
    pub reason: &'static str,
    /// How many leaves of this kind the feature held.
    pub count: usize,
}

impl fmt::Display for GeometryOmission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} omitted ({})",
            self.count, self.geometry, self.reason
        )
    }
}

/// The CityGML city-object class a feature is written as, which fixes its
/// element name, its namespace prefix, and the prefix of any `gml:id` minted
/// for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CityObjectType {
    Building,
    BuildingPart,
    Road,
    Railway,
    Track,
    Square,
    Bridge,
    BridgePart,
    Tunnel,
    TunnelPart,
    WaterBody,
    LandUse,
    SolitaryVegetationObject,
    PlantCover,
    CityFurniture,
    ReliefFeature,
    GenericCityObject,
}

impl CityObjectType {
    pub fn from_feature_type(feature_type: &str) -> Self {
        let normalized = feature_type.to_lowercase();
        if normalized.contains("buildingpart") {
            Self::BuildingPart
        } else if normalized.contains("building") {
            Self::Building
        } else if normalized.contains("railway") {
            Self::Railway
        } else if normalized.contains("road") {
            Self::Road
        } else if normalized.contains("track") {
            Self::Track
        } else if normalized.contains("square") {
            Self::Square
        } else if normalized.contains("bridgepart") {
            Self::BridgePart
        } else if normalized.contains("bridge") {
            Self::Bridge
        } else if normalized.contains("tunnelpart") {
            Self::TunnelPart
        } else if normalized.contains("tunnel") {
            Self::Tunnel
        } else if normalized.contains("waterbody") {
            Self::WaterBody
        } else if normalized.contains("landuse") {
            Self::LandUse
        } else if normalized.contains("solitaryvegetationobject") {
            Self::SolitaryVegetationObject
        } else if normalized.contains("plantcover") {
            Self::PlantCover
        } else if normalized.contains("cityfurniture") {
            Self::CityFurniture
        } else if normalized.contains("relieffeature") {
            Self::ReliefFeature
        } else {
            Self::GenericCityObject
        }
    }

    pub fn element_name(&self) -> &'static str {
        match self {
            Self::Building => "bldg:Building",
            Self::BuildingPart => "bldg:BuildingPart",
            Self::Road => "tran:Road",
            Self::Railway => "tran:Railway",
            Self::Track => "tran:Track",
            Self::Square => "tran:Square",
            Self::Bridge => "brid:Bridge",
            Self::BridgePart => "brid:BridgePart",
            Self::Tunnel => "tun:Tunnel",
            Self::TunnelPart => "tun:TunnelPart",
            Self::WaterBody => "wtr:WaterBody",
            Self::LandUse => "luse:LandUse",
            Self::SolitaryVegetationObject => "veg:SolitaryVegetationObject",
            Self::PlantCover => "veg:PlantCover",
            Self::CityFurniture => "frn:CityFurniture",
            Self::ReliefFeature => "dem:ReliefFeature",
            Self::GenericCityObject => "gen:GenericCityObject",
        }
    }

    pub fn namespace_prefix(&self) -> &'static str {
        match self {
            Self::Building | Self::BuildingPart => "bldg",
            Self::Road | Self::Railway | Self::Track | Self::Square => "tran",
            Self::Bridge | Self::BridgePart => "brid",
            Self::Tunnel | Self::TunnelPart => "tun",
            Self::WaterBody => "wtr",
            Self::LandUse => "luse",
            Self::SolitaryVegetationObject | Self::PlantCover => "veg",
            Self::CityFurniture => "frn",
            Self::ReliefFeature => "dem",
            Self::GenericCityObject => "gen",
        }
    }

    pub fn id_prefix(&self) -> &'static str {
        match self {
            Self::Building => "bldg",
            Self::BuildingPart => "bldg_part",
            Self::Road => "road",
            Self::Railway => "rail",
            Self::Track => "track",
            Self::Square => "square",
            Self::Bridge => "brid",
            Self::BridgePart => "brid_part",
            Self::Tunnel => "tun",
            Self::TunnelPart => "tun_part",
            Self::WaterBody => "wtr",
            Self::LandUse => "luse",
            Self::SolitaryVegetationObject => "veg_sol",
            Self::PlantCover => "veg_plant",
            Self::CityFurniture => "frn",
            Self::ReliefFeature => "dem",
            Self::GenericCityObject => "gen",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_city_object_type_from_feature_type() {
        assert_eq!(
            CityObjectType::from_feature_type("bldg:Building"),
            CityObjectType::Building
        );
        assert_eq!(
            CityObjectType::from_feature_type("tran:Road"),
            CityObjectType::Road
        );
    }

    /// Merging is a per-ordinate min/max, so an envelope grows to cover both
    /// operands and never shrinks.
    #[test]
    fn merging_envelopes_covers_both() {
        let mut envelope = BoundingEnvelope {
            lower: [0.0, 0.0, 0.0],
            upper: [1.0, 1.0, 1.0],
        };
        envelope.merge(&BoundingEnvelope {
            lower: [-1.0, 0.5, 0.0],
            upper: [0.5, 2.0, 3.0],
        });

        assert_eq!(envelope.lower, [-1.0, 0.0, 0.0]);
        assert_eq!(envelope.upper, [1.0, 2.0, 3.0]);
    }
}
