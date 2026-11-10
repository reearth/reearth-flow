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
#[derive(Debug, Clone, PartialEq)]
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
    /// The owner's `gml:id`. `None` when the member names no owner, or one
    /// without a `gml:id`.
    pub key: Option<String>,
    /// The owner's class as a qualified name, such as `bldg:WallSurface`.
    pub feature_type: Option<String>,
}

/// One city object to write, with the objects nested inside it.
#[derive(Debug, Clone)]
pub struct ObjectNode {
    pub class: CityClass,
    /// The key the object was grouped and joined by: its source `gml:id`.
    /// `None` for an object without one.
    pub key: Option<String>,
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
#[derive(Debug, Clone, PartialEq)]
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
    /// A `gml:TriangulatedSurface`, which PLATEAU terrain (`dem:tin`) is. A
    /// patch carries no `gml:id` and no appearance, and the surface has no id
    /// of its own, so only positions are kept.
    ///
    /// The converter also emits one for a bare triangle mesh before its
    /// property is known, so terrain never passes through a `GmlSurface` per
    /// triangle. The writer turns it into a `MultiSurface` when it settles
    /// anywhere other than a `dem:tin`, which loses nothing: it holds no ids
    /// or appearance to lose.
    #[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
    TriangulatedSurface {
        triangles: GmlTriangles,
    },
}

/// Triangles over one shared position pool: each is three indices into
/// `positions`, its open exterior ring in order. The pool is kept as the mesh
/// shared it, so a corner used by several triangles is stored once.
#[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
#[derive(Debug, Clone, Default, PartialEq)]
pub struct GmlTriangles {
    pub positions: Vec<[f64; 3]>,
    pub triangles: Vec<[u32; 3]>,
}

#[cfg_attr(not(feature = "new-geometry"), allow(dead_code))]
impl GmlTriangles {
    pub fn len(&self) -> usize {
        self.triangles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.triangles.is_empty()
    }

    /// Each triangle's three corners, in ring order.
    pub fn corners(&self) -> impl Iterator<Item = [[f64; 3]; 3]> + '_ {
        self.triangles
            .iter()
            .map(|t| t.map(|index| self.positions[index as usize]))
    }

    /// Add one triangle by its corners, with positions of its own.
    pub fn push(&mut self, corners: [[f64; 3]; 3]) {
        let first = self.next_index();
        self.positions.extend(corners);
        self.triangles.push([first, first + 1, first + 2]);
    }

    /// Append `other`'s triangles, re-indexed onto the end of this pool.
    pub fn extend(&mut self, other: GmlTriangles) {
        let offset = self.next_index();
        self.positions.extend(other.positions);
        self.triangles.extend(
            other
                .triangles
                .into_iter()
                .map(|t| t.map(|index| index + offset)),
        );
    }

    /// One `gml:Polygon` per triangle, its exterior closed and without
    /// appearance, for a property that holds a `gml:MultiSurface`.
    pub fn into_surfaces(self) -> Vec<GmlSurface> {
        self.corners()
            .map(|[a, b, c]| GmlSurface {
                id: None,
                exterior: vec![a, b, c, a],
                interiors: Vec::new(),
                material_idx: None,
                texture_idx: None,
                uv_exterior: Vec::new(),
                uv_interiors: Vec::new(),
            })
            .collect()
    }

    /// The index the next position pushed takes. A pool past `u32::MAX`
    /// positions would be some 100 GB of coordinates.
    fn next_index(&self) -> u32 {
        u32::try_from(self.positions.len()).expect("triangle pool exceeds u32 indices")
    }
}

/// A `gml:Solid`: one exterior shell, and one interior shell per void.
#[derive(Debug, Clone, PartialEq)]
pub struct GmlSolid {
    pub id: Option<String>,
    pub exterior: Vec<GmlSurface>,
    /// Always empty in the legacy world, whose reader discards interior shells.
    pub interiors: Vec<Vec<GmlSurface>>,
}

/// One `gml:Polygon` with its rings and appearance, so the writer can mint a
/// `gml:id` exactly when something targets it.
#[derive(Debug, Clone, PartialEq)]
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

    const A: [f64; 3] = [0.0, 0.0, 0.0];
    const B: [f64; 3] = [1.0, 0.0, 0.0];
    const C: [f64; 3] = [0.0, 1.0, 0.0];
    const D: [f64; 3] = [1.0, 1.0, 0.0];

    #[test]
    fn extending_triangles_reindexes_onto_the_pool() {
        let mut held = GmlTriangles {
            positions: vec![A, B, C, D],
            triangles: vec![[0, 1, 2], [1, 3, 2]],
        };
        held.extend(GmlTriangles {
            positions: vec![D, C, B],
            triangles: vec![[0, 1, 2]],
        });
        assert_eq!(held.triangles, [[0, 1, 2], [1, 3, 2], [4, 5, 6]]);
        assert_eq!(
            held.corners().collect::<Vec<_>>(),
            [[A, B, C], [B, D, C], [D, C, B]]
        );
    }

    #[test]
    fn triangles_become_closed_polygons_without_appearance() {
        let mut triangles = GmlTriangles::default();
        triangles.push([A, B, C]);
        let surfaces = triangles.into_surfaces();
        assert_eq!(surfaces.len(), 1);
        assert_eq!(surfaces[0].exterior, [A, B, C, A]);
        assert!(surfaces[0].interiors.is_empty());
        assert_eq!(surfaces[0].material_idx, None);
        assert_eq!(surfaces[0].texture_idx, None);
    }
}
