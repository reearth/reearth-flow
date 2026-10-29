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
use reearth_flow_citygml::schema::QName;
use reearth_flow_common::image::MimeType;
use reearth_flow_types::conversion::CrsCoverage;
use reearth_flow_types::material::X3DMaterial;

use super::classes::CityClass;
use super::placement::Placed;

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
    /// The object this geometry belongs to.
    pub owner: EntryOwner,
}

/// The object a [`GeometryEntry`]'s geometry belongs to, as its member records
/// it. Entries with the same `key` belong to one object.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EntryOwner {
    /// The owner's `gml:id`, or its synthetic owner key when it has no id.
    /// `None` when the member names no owner.
    pub key: Option<String>,
    /// The owner's class as a qualified name, such as `bldg:WallSurface`.
    pub feature_type: Option<String>,
}

/// One city object to write, with the objects nested inside it.
#[derive(Debug, Clone)]
pub struct ObjectNode {
    pub class: CityClass,
    /// The key the object was grouped and joined by: its source `gml:id`, or
    /// its synthetic owner key. `None` for an object with neither.
    pub key: Option<String>,
    /// Whether `key` is a synthetic owner key rather than a source `gml:id`.
    pub synthetic: bool,
    /// The candidate `gml:id`; the writer claims it or mints one.
    pub gml_id: Option<String>,
    pub geometries: Vec<GeometryEntry>,
    pub properties: Vec<Placed>,
    pub children: Vec<NestedObject>,
    /// The index of the feature whose palette and id this object reports with.
    pub source: usize,
}

impl ObjectNode {
    /// Whether this object or any object nested in it has geometry.
    pub fn has_geometry(&self) -> bool {
        !self.geometries.is_empty()
            || self
                .children
                .iter()
                .any(|child| child.object.has_geometry())
    }
}

/// A city object written inside its parent's `property`, which occupies the
/// parent class's slot `slot`.
#[derive(Debug, Clone)]
pub struct NestedObject {
    pub slot: usize,
    pub property: QName,
    pub object: ObjectNode,
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
    /// Written as `app:theme`. `None` writes the writer's fallback theme, which
    /// is all the legacy model can offer: it carries no theme name.
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

#[cfg(test)]
mod tests {
    use super::*;

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
