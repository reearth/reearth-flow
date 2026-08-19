//! The CityGML document model both geometry worlds converge on.
//!
//! A converter turns one `Feature` into a [`ConvertedCityObject`]; everything
//! downstream is shared. Nothing here names a geometry type, so this module
//! compiles unconditionally.
//!
//! Coordinates are `[f64; 3]` in the producing world's own ordinate order. Each
//! world exports its own `format_pos_list`, and the writer formats every
//! `gml:posList` and [`BoundingEnvelope`] corner through it.

use std::fmt;

use bytes::Bytes;
use reearth_flow_common::image::MimeType;
use reearth_flow_types::conversion::CrsCoverage;
use reearth_flow_types::material::X3DMaterial;

/// One feature's converted CityGML, plus what the shared shell needs around it.
#[derive(Debug, Clone)]
pub struct ConvertedCityObject {
    /// One entry per emitted geometry property, in source order.
    pub geometries: Vec<GeometryEntry>,
    pub appearance: AppearanceBundle,
    /// Folded over emitted coordinates only, so filtered geometry cannot widen it.
    pub envelope: Option<BoundingEnvelope>,
    /// Folded over the same leaves as the envelope; the shell turns it into `srsName`.
    pub crs: CrsCoverage,
    /// Images to stage beside the `.gml`.
    pub textures: Vec<TextureRef>,
    /// Geometry CityGML 2.0 has no place for. Reported, never silently dropped.
    pub omissions: Vec<GeometryOmission>,
}

/// One geometry property: its LOD, its source name, and the GML element carrying it.
#[derive(Debug, Clone)]
pub struct GeometryEntry {
    pub lod: u8,
    /// The source property's local name (`"lod0RoofEdge"`, `"tin"`), used as the
    /// wrapper element name. `None` falls back to LOD plus GML family.
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

/// A `gml:Solid`: one exterior shell, and one interior shell per void.
#[derive(Debug, Clone)]
pub struct GmlSolid {
    pub id: Option<String>,
    pub exterior: Vec<GmlSurface>,
    /// Always empty in the legacy world, whose reader discards interior shells.
    pub interiors: Vec<Vec<GmlSurface>>,
}

/// One `gml:Polygon` with its rings and appearance, so the writer can mint a
/// `gml:id` exactly when something targets it.
#[derive(Debug, Clone)]
pub struct GmlSurface {
    pub id: Option<String>,
    pub exterior: Vec<[f64; 3]>,
    pub interiors: Vec<Vec<[f64; 3]>>,
    /// Index into [`AppearanceBundle::materials`].
    pub material_idx: Option<u32>,
    /// Index into [`AppearanceBundle::textures`].
    pub texture_idx: Option<u32>,
    /// Parallel to `exterior`.
    pub uv_exterior: Vec<[f64; 2]>,
    /// Parallel to each ring in `interiors`.
    pub uv_interiors: Vec<Vec<[f64; 2]>>,
}

/// The palettes [`GmlSurface`]'s indices point into.
#[derive(Debug, Clone, Default)]
pub struct AppearanceBundle {
    /// Written as `app:theme`. `None` keeps the historical `rgbTexture` literal,
    /// which is all the legacy model can offer — it carries no theme name.
    pub theme: Option<String>,
    pub materials: Vec<X3DMaterial>,
    pub textures: Vec<GmlTexture>,
}

impl AppearanceBundle {
    pub fn has_content(&self) -> bool {
        !self.materials.is_empty() || !self.textures.is_empty()
    }
}

/// One `app:ParameterizedTexture`'s image.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GmlTexture {
    /// The [`TextureRef::key`] this image was staged under.
    pub key: String,
    /// What `app:imageURI` says when nothing was staged under [`key`](Self::key).
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

/// An image the document references, for the shell to stage and the writer to
/// rewrite `app:imageURI` by.
#[derive(Debug, Clone)]
pub struct TextureRef {
    /// For a URI-backed raster this *is* the source URI string, which keeps the
    /// rewrite identical to the legacy path's.
    pub key: String,
    pub source: TextureSource,
}

/// Where a referenced image's bytes come from.
#[derive(Debug, Clone)]
pub enum TextureSource {
    Uri(url::Url),
    /// Already decoded — an OBJ `map_Kd` or a glTF/GLB packed image, with no URI
    /// to read back from. The shell writes the bytes and names the file itself.
    InMemory {
        mime: MimeType,
        bytes: Bytes,
    },
}

impl TextureSource {
    /// The extension a staged copy gets. Only consulted for in-memory bytes; a
    /// URI-backed raster keeps its source's last segment.
    pub fn extension(mime: MimeType) -> &'static str {
        match mime {
            MimeType::ImageJpeg => "jpg",
            MimeType::ImagePng => "png",
            MimeType::ImageWebp => "webp",
        }
    }
}

/// One kind of geometry a conversion left out, aggregated per feature so a
/// collection of a thousand points is reported once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeometryOmission {
    pub geometry: &'static str,
    pub reason: &'static str,
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

/// The city-object class a feature is written as, fixing its element name,
/// namespace prefix, and minted `gml:id` prefix.
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
    /// Substring match, most specific first (`buildingpart` before `building`).
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
