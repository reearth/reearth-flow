//! The unified world's half of the converter seam: `reearth_flow_geometry`'s
//! recursive hierarchy in, the shared [`super::model`] out.
//!
//! Handles every 3D leaf CityGML 2.0 has an element for — `Polygon`,
//! `LineString`, `PolygonMesh`, `TriangularMesh`, `Solid` — and collections of
//! them, each carrying whatever appearance the leaf held.
//!
//! # Not emitted
//!
//! Deliberate narrowings, each reported as a [`GeometryOmission`] rather than
//! silently dropped: `Point`, `PointCloud`, `Csg` and every 2D leaf; appearance
//! beyond one theme, one side and the diffuse map (see
//! [`super::appearance_next`]); `app:GeoreferencedTexture`; source geometry
//! `gml:id`s, which the unified model does not retain; feature attributes,
//! semantic surfaces and `xsi:schemaLocation`; CityGML 3.0.
//!
//! # Owned here, not by the legacy converter
//!
//! - **Axis order.** The new reader stores ordinates in the CRS's own declared
//!   order, so [`format_pos_list`] writes them back verbatim.
//! - **The CRS declaration.** `srsName` is folded over the leaves that reach the
//!   file: one CRS declared, a mixture an error.
//! - **Ring closure.** Rings arrive as stored, so they are closed here — and the
//!   duplicated corner is recorded so [`super::appearance_next`] extends the UV
//!   in the same step.

use std::ops::Range;

use reearth_flow_geometry::appearance::Appearance;
use reearth_flow_geometry::coordinate::CoordinateFrame;
use reearth_flow_geometry::polygon_mesh::FaceVisit;
use reearth_flow_geometry::solid::{Shell, Solid};
use reearth_flow_geometry::{Euclidean3DGeometry, Geometry, GeometryCollection};
use reearth_flow_types::conversion::CrsCoverage;
use reearth_flow_types::lod::LodMask;
use reearth_flow_types::{Attribute, AttributeValue, Attributes, CitygmlFeatureExt, Feature};

use super::appearance_next::{
    self, FaceCorners, LeafContext, Palette, RingCorners, SurfaceBinding,
};
use super::model::{
    BoundingEnvelope, ConvertedCityObject, GeometryEntry, GeometryOmission, GmlElement, GmlSolid,
    GmlSurface,
};
use crate::errors::SinkError;

/// Spelled out rather than imported: `action-sink` depends on `action-processor`
/// only as a dev-dependency. A unit test pins the two spellings together.
const MEMBER_LOD_KEY: &str = "lod";

/// The source geometry property's local name, as the reader records it.
const MEMBER_PROPERTY_KEY: &str = "citygmlProperty";

/// Matches the legacy `lod.unwrap_or(0)`. `lodFilter` applies to it like any other.
const DEFAULT_LOD: u8 = 0;

/// The highest LOD [`LodMask`] can represent; larger is rejected, not written.
const MAX_LOD: u8 = 4;

/// An unstageable texture aborts the write: this path knows exactly which images
/// the document points at, so a dangling `app:imageURI` would be inexcusable.
pub const STRICT_TEXTURE_STAGING: bool = true;

/// Serialize `coords` as a `gml:posList` body, or an envelope corner.
///
/// The identity formatter: the reader parses ordinates in the source's own axis
/// order and every leaf keeps them, so writing them back reproduces the source.
pub fn format_pos_list(coords: &[[f64; 3]]) -> String {
    coords
        .iter()
        .map(|c| format!("{} {} {}", c[0], c[1], c[2]))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The OGC CRS URI to declare for the given `coverage`.
///
/// `features` is unused: the unified world's CRS lives on each leaf and is
/// already folded into `coverage`, over exactly the leaves that were written.
/// `epsg_code` declares, it does not reproject.
pub fn srs_name(
    _features: &[Feature],
    epsg_code: Option<u32>,
    coverage: CrsCoverage,
) -> Result<String, SinkError> {
    let code = match coverage {
        CrsCoverage::Single(code) => {
            let code = u32::from(code.get());
            match epsg_code {
                Some(declared) if declared != code => {
                    return Err(SinkError::CityGmlWriter(format!(
                        "the `epsgCode` parameter declares EPSG:{declared} but every written \
                         coordinate is in EPSG:{code}; the writer does not reproject, so set \
                         `epsgCode` to {code} or reproject upstream"
                    )))
                }
                _ => code,
            }
        }
        CrsCoverage::Mixed { first, other } => {
            return Err(SinkError::CityGmlWriter(format!(
                "written coordinates are in both EPSG:{first} and EPSG:{other}; one CityGML \
                 document declares one `srsName`, so reproject to a single CRS upstream"
            )))
        }
        CrsCoverage::OutsideAnyCrs => match epsg_code {
            Some(declared) => declared,
            None => {
                return Err(SinkError::CityGmlWriter(
                    "a written coordinate is outside any CRS, so no `srsName` can be derived \
                     from the geometry; set the `epsgCode` parameter to declare one, or \
                     reproject upstream"
                        .to_string(),
                ))
            }
        },
        // Unreachable: the shell returns early when nothing was emitted.
        CrsCoverage::NoCoordinates => epsg_code.unwrap_or(DEFAULT_EPSG),
    };
    Ok(format!("http://www.opengis.net/def/crs/EPSG/0/{code}"))
}

/// Kept only for a document that declares a CRS no element references.
const DEFAULT_EPSG: u32 = 4326;

/// Convert one feature's geometry into the shared CityGML model. Geometry the
/// `lod_mask` filters out is skipped before the envelope or CRS coverage sees it.
pub fn convert_city_object(
    feature: &Feature,
    lod_mask: &LodMask,
) -> Result<ConvertedCityObject, SinkError> {
    let mut conversion = Conversion {
        context: FeatureContext {
            id: feature
                .feature_id()
                .unwrap_or_else(|| feature.id.to_string()),
        },
        ..Conversion::default()
    };
    conversion.convert_member(&feature.geometry, lod_mask, DEFAULT_LOD, None)?;
    Ok(conversion.finish())
}

/// What a failing conversion names so the error points at one feature.
#[derive(Default)]
struct FeatureContext {
    id: String,
}

/// One emitted leaf, before it is grouped into a GML family.
enum Piece {
    Solid(GmlSolid),
    Surface(GmlSurface),
    Curve(Vec<[f64; 3]>),
}

/// The running result of converting one feature.
#[derive(Default)]
struct Conversion {
    context: FeatureContext,
    geometries: Vec<GeometryEntry>,
    envelope: Option<BoundingEnvelope>,
    crs: CrsCoverage,
    /// One `app:Appearance` per feature; every leaf's bindings index into it.
    palette: Palette,
    omissions: Vec<GeometryOmission>,
}

impl Conversion {
    fn finish(self) -> ConvertedCityObject {
        ConvertedCityObject {
            geometries: self.geometries,
            appearance: self.palette.bundle,
            envelope: self.envelope,
            crs: self.crs,
            textures: self.palette.textures,
            omissions: self.omissions,
        }
    }

    /// Convert one collection member, or the feature's whole geometry.
    fn convert_member(
        &mut self,
        geometry: &Geometry,
        lod_mask: &LodMask,
        lod: u8,
        property: Option<&str>,
    ) -> Result<(), SinkError> {
        match geometry {
            Geometry::None => Ok(()),
            Geometry::Euclidean2D(_) => {
                self.omit(
                    "Euclidean2D",
                    "CityGML geometry is 3D; promoting a 2D leaf to Z=0 would fabricate elevation",
                );
                Ok(())
            }
            Geometry::Euclidean3D(geometry) => {
                // Top-level geometry carries no member attributes, so it arrives
                // at `DEFAULT_LOD` without passing `convert_collection`'s check.
                if !lod_mask.has_lod(lod) {
                    return Ok(());
                }
                let mut pieces = Vec::new();
                self.collect_pieces(geometry, &mut pieces)?;
                self.push_entries(lod, property, pieces);
                Ok(())
            }
            Geometry::GeometryCollection(collection) => {
                self.convert_collection(collection, lod_mask, lod, property)
            }
        }
    }

    /// Walk a collection, reading each member's LOD and property name off the
    /// parallel attribute record the reader filled in.
    fn convert_collection(
        &mut self,
        collection: &GeometryCollection,
        lod_mask: &LodMask,
        inherited_lod: u8,
        inherited_property: Option<&str>,
    ) -> Result<(), SinkError> {
        let attributes = collection.member_attributes();
        for (index, member) in collection.members().iter().enumerate() {
            let member_attributes = attributes.get(index);
            let lod = match member_attributes {
                Some(attributes) => self.member_lod(attributes, index)?.unwrap_or(inherited_lod),
                None => inherited_lod,
            };
            // Before accumulation, so a filtered member reaches neither the
            // envelope nor the CRS coverage.
            if !lod_mask.has_lod(lod) {
                continue;
            }
            let property = member_attributes
                .and_then(member_property)
                .or(inherited_property);
            self.convert_member(member, lod_mask, lod, property)?;
        }
        Ok(())
    }

    /// The LOD a member records, or `None`. A present-but-unusable value is an
    /// error: guessing would write geometry under the wrong LOD.
    fn member_lod(&self, attributes: &Attributes, index: usize) -> Result<Option<u8>, SinkError> {
        let Some(value) = attributes.get(&Attribute::new(MEMBER_LOD_KEY)) else {
            return Ok(None);
        };
        let lod = value
            .as_i64()
            .and_then(|lod| u8::try_from(lod).ok())
            .filter(|lod| *lod <= MAX_LOD)
            .ok_or_else(|| {
                SinkError::CityGmlWriter(format!(
                    "feature {}: geometry member {index} records an unusable `{MEMBER_LOD_KEY}` \
                     attribute {value:?}; expected a whole number in 0..={MAX_LOD}",
                    self.context.id
                ))
            })?;
        Ok(Some(lod))
    }

    /// Flatten one 3D geometry into leaves CityGML 2.0 can carry, reporting the rest.
    fn collect_pieces(
        &mut self,
        geometry: &Euclidean3DGeometry,
        out: &mut Vec<Piece>,
    ) -> Result<(), SinkError> {
        match geometry {
            // One face whose rings are concatenated exterior-first, which is
            // also the corner buffer its UV is parallel to.
            Euclidean3DGeometry::Polygon(polygon) => {
                self.fold_frame(polygon.frame());
                let mut corner = 0;
                let mut ring = |coords: &[[f64; 3]]| {
                    let start = corner;
                    corner += coords.len();
                    (coords.to_vec(), start..corner)
                };
                let face = FaceRings {
                    face: 0,
                    exterior: ring(polygon.exterior()),
                    interiors: polygon.interiors().map(ring).collect(),
                };
                let surfaces =
                    self.emit_surfaces(vec![face], polygon.appearance(), "a Polygon leaf")?;
                out.extend(surfaces.into_iter().map(Piece::Surface));
            }
            Euclidean3DGeometry::LineString(line_string) => {
                self.fold_frame(line_string.frame());
                let coords = line_string.coords().to_vec();
                self.fold_envelope(&coords);
                out.push(Piece::Curve(coords));
            }
            Euclidean3DGeometry::Collection(collection) => {
                for member in collection.members() {
                    self.collect_pieces(member, out)?;
                }
            }
            // Independent faces sharing a vertex pool — one `gml:Polygon` each.
            Euclidean3DGeometry::PolygonMesh(mesh) => {
                self.fold_frame(mesh.frame());
                let mut faces = Vec::with_capacity(mesh.num_faces());
                mesh.for_each_face(|face| faces.push(FaceRings::from(&face)));
                let surfaces = self.emit_surfaces(faces, mesh.appearance(), "a PolygonMesh")?;
                out.extend(surfaces.into_iter().map(Piece::Surface));
            }
            Euclidean3DGeometry::TriangularMesh(mesh) => {
                self.fold_frame(mesh.frame());
                let mut faces = Vec::with_capacity(mesh.num_triangles());
                mesh.for_each_face(|face| faces.push(FaceRings::from(&face)));
                let surfaces = self.emit_surfaces(faces, mesh.appearance(), "a TriangularMesh")?;
                out.extend(surfaces.into_iter().map(Piece::Surface));
            }
            // One `gml:CompositeSurface` per shell.
            Euclidean3DGeometry::Solid(solid) => {
                self.fold_frame(solid.frame());
                let solid = self.solid(solid)?;
                out.push(Piece::Solid(solid));
            }
            Euclidean3DGeometry::Point(_) => self.omit("Point", POINT_REASON),
            Euclidean3DGeometry::PointCloud(_) => self.omit("PointCloud", POINT_REASON),
            Euclidean3DGeometry::Csg(_) => self.omit(
                "Csg",
                "a boolean tree has no CityGML counterpart until it is evaluated, which the \
                 writer does not do",
            ),
        }
        Ok(())
    }

    /// One solid: its exterior shell's faces plus one face list per void. Each
    /// shell carries its own appearance over its own corner buffer.
    fn solid(&mut self, solid: &Solid) -> Result<GmlSolid, SinkError> {
        let exterior = self.shell(solid.exterior(), "a Solid's exterior shell")?;
        let mut interiors = Vec::with_capacity(solid.interiors().len());
        for shell in solid.interiors() {
            interiors.push(self.shell(shell, "a Solid's interior shell")?);
        }
        Ok(GmlSolid {
            id: None,
            exterior,
            interiors,
        })
    }

    /// One boundary shell's faces, whichever mesh kind it is.
    fn shell(&mut self, shell: &Shell, named: &'static str) -> Result<Vec<GmlSurface>, SinkError> {
        let mut faces = Vec::with_capacity(shell.num_faces());
        shell.for_each_face(|face| faces.push(FaceRings::from(&face)));
        self.emit_surfaces(faces, shell.appearance(), named)
    }

    /// Turn one leaf's faces into `gml:Polygon`s and paint them. Closing rings and
    /// resolving appearance is one step: the duplicated corner is only known here.
    fn emit_surfaces(
        &mut self,
        faces: Vec<FaceRings>,
        appearance: &Option<Appearance>,
        named: &'static str,
    ) -> Result<Vec<GmlSurface>, SinkError> {
        let mut surfaces = Vec::with_capacity(faces.len());
        let mut corners = Vec::with_capacity(faces.len());
        for face in faces {
            let (surface, face_corners) = self.surface(face);
            surfaces.push(surface);
            corners.push(face_corners);
        }

        let Some(appearance) = appearance else {
            return Ok(surfaces);
        };
        let context = LeafContext {
            feature: &self.context.id,
            geometry: named,
        };
        let resolved = appearance_next::resolve(appearance, &corners, &mut self.palette, &context)?;
        for (surface, binding) in surfaces.iter_mut().zip(resolved.bindings) {
            paint(surface, binding);
        }
        for omission in resolved.omissions {
            self.omit_n(omission.geometry, omission.reason, omission.count);
        }
        Ok(surfaces)
    }

    /// Build one `gml:Polygon`: close each ring, fold it into the envelope, and
    /// record where its texture coordinates live.
    fn surface(&mut self, face: FaceRings) -> (GmlSurface, FaceCorners) {
        let (exterior, exterior_corners) = self.ring(face.exterior);
        let mut interiors = Vec::with_capacity(face.interiors.len());
        let mut interior_corners = Vec::with_capacity(face.interiors.len());
        for ring in face.interiors {
            let (ring, corners) = self.ring(ring);
            interiors.push(ring);
            interior_corners.push(corners);
        }
        (
            GmlSurface {
                id: None,
                exterior,
                interiors,
                material_idx: None,
                texture_idx: None,
                uv_exterior: Vec::new(),
                uv_interiors: Vec::new(),
            },
            FaceCorners {
                face: face.face,
                exterior: exterior_corners,
                interiors: interior_corners,
            },
        )
    }

    /// Close one ring, fold it into the envelope, and locate its UV.
    fn ring(
        &mut self,
        (mut ring, corners): (Vec<[f64; 3]>, Range<usize>),
    ) -> (Vec<[f64; 3]>, RingCorners) {
        // The closing corner duplicates the ring's first, at `corners.start`.
        let closure = close_ring(&mut ring).map(|local| corners.start + local);
        self.fold_envelope(&ring);
        let len = ring.len();
        (
            ring,
            RingCorners {
                corners,
                closure,
                len,
            },
        )
    }

    /// Group one member's leaves by GML family, one entry per family, in
    /// descending dimension.
    fn push_entries(&mut self, lod: u8, property: Option<&str>, pieces: Vec<Piece>) {
        let mut solids = Vec::new();
        let mut surfaces = Vec::new();
        let mut curves = Vec::new();
        for piece in pieces {
            match piece {
                Piece::Solid(solid) => solids.push(solid),
                Piece::Surface(surface) => surfaces.push(surface),
                Piece::Curve(curve) => curves.push(curve),
            }
        }
        // One solid stays a `gml:Solid` so `lod1Solid` still names what it wraps.
        match solids.len() {
            0 => {}
            1 => self.geometries.push(GeometryEntry {
                lod,
                property: property.map(str::to_string),
                element: GmlElement::Solid(solids.pop().expect("one solid")),
            }),
            _ => self.geometries.push(GeometryEntry {
                lod,
                property: property.map(str::to_string),
                element: GmlElement::MultiSolid { id: None, solids },
            }),
        }
        if !surfaces.is_empty() {
            self.geometries.push(GeometryEntry {
                lod,
                property: property.map(str::to_string),
                element: GmlElement::MultiSurface { id: None, surfaces },
            });
        }
        if !curves.is_empty() {
            self.geometries.push(GeometryEntry {
                lod,
                property: property.map(str::to_string),
                element: GmlElement::MultiCurve { id: None, curves },
            });
        }
    }

    /// Fold one emitted leaf's frame into the document's CRS coverage.
    fn fold_frame(&mut self, frame: &CoordinateFrame) {
        let coverage = match frame {
            CoordinateFrame::Crs(code) => CrsCoverage::Single(*code),
            // A tangent plane's in-plane coordinates are not its base CRS's.
            CoordinateFrame::Euclidean | CoordinateFrame::Tangent(_) => CrsCoverage::OutsideAnyCrs,
        };
        self.crs = self.crs.and(coverage);
    }

    /// Grow the envelope to cover one emitted ring or curve.
    fn fold_envelope(&mut self, coords: &[[f64; 3]]) {
        for coord in coords {
            let point = BoundingEnvelope {
                lower: *coord,
                upper: *coord,
            };
            match &mut self.envelope {
                Some(envelope) => envelope.merge(&point),
                None => self.envelope = Some(point),
            }
        }
    }

    /// Record one unwritable leaf, aggregated by kind.
    fn omit(&mut self, geometry: &'static str, reason: &'static str) {
        self.omit_n(geometry, reason, 1);
    }

    /// As [`omit`](Self::omit), for a narrowing that already counted itself.
    fn omit_n(&mut self, geometry: &'static str, reason: &'static str, count: usize) {
        match self
            .omissions
            .iter_mut()
            .find(|omission| omission.geometry == geometry)
        {
            Some(omission) => omission.count += count,
            None => self.omissions.push(GeometryOmission {
                geometry,
                reason,
                count,
            }),
        }
    }
}

/// Apply one resolved binding to the surface it belongs to.
fn paint(surface: &mut GmlSurface, binding: SurfaceBinding) {
    surface.material_idx = binding.material_idx;
    surface.texture_idx = binding.texture_idx;
    surface.uv_exterior = binding.uv_exterior;
    surface.uv_interiors = binding.uv_interiors;
}

const POINT_REASON: &str =
    "this writer emits no gml:Point / gml:MultiPoint, matching the legacy build";

/// One face on its way out: each ring's coordinates plus its corner range.
///
/// Coordinates are copied because the visitor's borrows do not outlive its
/// callback; the ranges come along because they are unrecoverable afterwards and
/// are the only thing that lines a ring up with the theme's UV array.
struct FaceRings {
    face: usize,
    exterior: (Vec<[f64; 3]>, Range<usize>),
    interiors: Vec<(Vec<[f64; 3]>, Range<usize>)>,
}

impl From<&FaceVisit<'_>> for FaceRings {
    fn from(face: &FaceVisit<'_>) -> Self {
        Self {
            face: face.face,
            exterior: (face.exterior.coords.to_vec(), face.exterior.corners.clone()),
            interiors: face
                .interiors
                .iter()
                .map(|ring| (ring.coords.to_vec(), ring.corners.clone()))
                .collect(),
        }
    }
}

/// Close `ring`: a `gml:LinearRing` repeats its first corner, and triangle and
/// index-sourced faces arrive open, so this is not an edge case.
///
/// Returns the ring-local position of the duplicated corner — always `0` — or
/// `None` if it was already closed. The caller extends the UV by that corner.
fn close_ring(ring: &mut Vec<[f64; 3]>) -> Option<usize> {
    let first = *ring.first()?;
    if *ring.last()? == first {
        return None;
    }
    ring.push(first);
    Some(0)
}

/// The property name a member records, if it records one.
fn member_property(attributes: &Attributes) -> Option<&str> {
    match attributes.get(&Attribute::new(MEMBER_PROPERTY_KEY)) {
        Some(AttributeValue::String(property)) if !property.is_empty() => Some(property),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// No gating needed: this module compiles only under `new-geometry`.
    mod fixture {
        use std::str::FromStr;
        use std::sync::Arc;

        use reearth_flow_common::uri::Uri;
        use reearth_flow_geometry::appearance::{
            ChannelId, Material, PhongMaterial, Raster, Sampler, Texture, ThemeId, UvSource,
        };
        use reearth_flow_geometry::collection::Collection3D;
        use reearth_flow_geometry::coordinate::{
            BaseFrame, CoordinateFrame, EpsgCode, TangentPlane,
        };
        use reearth_flow_geometry::line_string::LineString3D;
        use reearth_flow_geometry::point::Point3D;
        use reearth_flow_geometry::polygon::{Polygon3D, PolygonFace};
        use reearth_flow_geometry::polygon_mesh::{PolygonMesh3D, PolygonMesh3DData};
        use reearth_flow_geometry::solid::{Shell, Solid};
        use reearth_flow_geometry::triangular_mesh::TriangularMesh3D;
        use reearth_flow_geometry::{Euclidean3DGeometry, Geometry, GeometryCollection};
        use reearth_flow_types::{Attribute, AttributeValue, Attributes, Feature};

        use super::{MEMBER_LOD_KEY, MEMBER_PROPERTY_KEY};

        pub fn crs(code: u16) -> CoordinateFrame {
            CoordinateFrame::Crs(EpsgCode::new(code))
        }

        pub fn tangent() -> CoordinateFrame {
            CoordinateFrame::Tangent(Box::new(TangentPlane {
                base: BaseFrame::Euclidean,
                origin: [0.0, 0.0, 0.0],
                u: [1.0, 0.0, 0.0],
                v: [0.0, 1.0, 0.0],
            }))
        }

        /// A closed triangle offset by `offset` on every ordinate, so two of them
        /// are distinguishable in an expected `posList`.
        pub fn triangle(frame: CoordinateFrame, offset: f64) -> Euclidean3DGeometry {
            let ring = vec![
                [35.0 + offset, 139.0 + offset, 0.0],
                [35.1 + offset, 139.0 + offset, 0.0],
                [35.0 + offset, 139.1 + offset, 0.0],
                [35.0 + offset, 139.0 + offset, 0.0],
            ];
            Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
                frame,
                ring,
                Vec::<Vec<[f64; 3]>>::new(),
            )))
        }

        pub fn line(frame: CoordinateFrame) -> Euclidean3DGeometry {
            Euclidean3DGeometry::LineString(LineString3D::from_coords(
                frame,
                vec![[35.0, 139.0, 0.0], [35.1, 139.1, 1.0]],
            ))
        }

        pub fn point(frame: CoordinateFrame) -> Euclidean3DGeometry {
            Euclidean3DGeometry::Point(Point3D::new(frame, [35.0, 139.0, 0.0]))
        }

        /// Two triangular faces sharing an edge, given as vertex-index faces so
        /// their rings are stored **open** — the shape ring closure has to fix.
        pub fn polygon_mesh(frame: CoordinateFrame) -> Euclidean3DGeometry {
            Euclidean3DGeometry::PolygonMesh(Box::new(
                PolygonMesh3D::from_parts(
                    frame,
                    mesh_vertices(),
                    vec![vec![0u32, 1, 2], vec![1, 3, 2]],
                )
                .unwrap(),
            ))
        }

        /// One square face with one square hole, as raw CSR — the only way to
        /// get a hole into a mesh without going through `Polygon`.
        pub fn polygon_mesh_with_a_hole(frame: CoordinateFrame) -> Euclidean3DGeometry {
            Euclidean3DGeometry::PolygonMesh(Box::new(
                PolygonMesh3D::from_raw_parts(
                    frame,
                    vec![
                        [35.0, 139.0, 0.0],
                        [35.4, 139.0, 0.0],
                        [35.4, 139.4, 0.0],
                        [35.0, 139.4, 0.0],
                        [35.1, 139.1, 0.0],
                        [35.3, 139.1, 0.0],
                        [35.3, 139.3, 0.0],
                        [35.1, 139.3, 0.0],
                    ],
                    vec![0, 1, 2, 3, 4, 5, 6, 7],
                    vec![],
                    vec![4],
                )
                .unwrap(),
            ))
        }

        /// The same two triangles as a triangle mesh; its faces are three
        /// corners each, always open.
        pub fn triangular_mesh(frame: CoordinateFrame) -> Euclidean3DGeometry {
            Euclidean3DGeometry::TriangularMesh(Box::new(
                TriangularMesh3D::from_parts(frame, mesh_vertices(), [0u32, 1, 2, 1, 3, 2])
                    .unwrap(),
            ))
        }

        /// A solid bounded by one two-face exterior shell and `voids` interior
        /// shells, each one face.
        pub fn solid(frame: CoordinateFrame, voids: usize) -> Euclidean3DGeometry {
            let exterior = Shell::PolygonMesh(
                PolygonMesh3DData::from_parts(mesh_vertices(), [[0u32, 1, 2], [1, 3, 2]]).unwrap(),
            );
            let interiors = (0..voids)
                .map(|n| {
                    let offset = 0.01 * (n as f64 + 1.0);
                    Shell::PolygonMesh(
                        PolygonMesh3DData::from_parts(
                            vec![
                                [35.0 + offset, 139.0 + offset, 0.0],
                                [35.1 + offset, 139.0 + offset, 0.0],
                                [35.0 + offset, 139.1 + offset, 0.0],
                            ],
                            [[0u32, 1, 2]],
                        )
                        .unwrap(),
                    )
                })
                .collect();
            Euclidean3DGeometry::Solid(Box::new(Solid::new(frame, exterior, interiors)))
        }

        /// A Phong material painted with the image at `uri`, sampling the
        /// default UV channel — the shape a CityGML `ParameterizedTexture`
        /// arrives in.
        pub fn textured(uri: &str) -> Material {
            Material::Phong(PhongMaterial {
                diffuse: [0.5, 0.5, 0.5],
                specular: [0.0; 3],
                emissive: [0.0; 3],
                ambient_intensity: 0.5,
                shininess: 0.0,
                transparency: 0.0,
                diffuse_map: Some(Texture {
                    raster: Arc::new(Raster::Uri(Uri::from_str(uri).unwrap())),
                    sampler: Sampler::default(),
                    transform: None,
                    uv_channel: ChannelId::default(),
                }),
                emissive_map: None,
                normal_map: None,
            })
        }

        /// The closed triangle of [`triangle`], painted, with one UV per stored
        /// corner (four, because the ring arrives closed).
        pub fn textured_triangle(frame: CoordinateFrame, uv: Vec<[f64; 2]>) -> Euclidean3DGeometry {
            let Euclidean3DGeometry::Polygon(mut polygon) = triangle(frame, 0.0) else {
                unreachable!("`triangle` builds a polygon");
            };
            polygon
                .set_appearance(
                    ThemeId(Arc::from("rgbTexture")),
                    textured("file:///textures/wall.png"),
                    Some(UvSource::Explicit(uv.into_boxed_slice())),
                )
                .unwrap();
            Euclidean3DGeometry::Polygon(polygon)
        }

        /// The two-triangle mesh of [`triangular_mesh`], painted, with one UV per
        /// corner — six, three per face, which is what makes a face's slice of
        /// them observable.
        pub fn textured_triangular_mesh(
            frame: CoordinateFrame,
            uv: Vec<[f64; 2]>,
        ) -> Euclidean3DGeometry {
            let Euclidean3DGeometry::TriangularMesh(mut mesh) = triangular_mesh(frame) else {
                unreachable!("`triangular_mesh` builds a triangular mesh");
            };
            mesh.set_appearance(
                ThemeId(Arc::from("rgbTexture")),
                textured("file:///textures/roof.png"),
                Some(UvSource::Explicit(uv.into_boxed_slice())),
            )
            .unwrap();
            Euclidean3DGeometry::TriangularMesh(mesh)
        }

        /// A triangle painted differently on each side — the case this writer
        /// narrows to its front side alone.
        pub fn two_sided_triangle(frame: CoordinateFrame) -> Euclidean3DGeometry {
            let Euclidean3DGeometry::Polygon(mut polygon) = triangle(frame, 0.0) else {
                unreachable!("`triangle` builds a polygon");
            };
            let plain = |diffuse: [f32; 3]| {
                Material::Phong(PhongMaterial {
                    diffuse,
                    specular: [0.0; 3],
                    emissive: [0.0; 3],
                    ambient_intensity: 0.5,
                    shininess: 0.0,
                    transparency: 0.0,
                    diffuse_map: None,
                    emissive_map: None,
                    normal_map: None,
                })
            };
            polygon
                .set_two_sided_appearance(
                    ThemeId(Arc::from("rgbTexture")),
                    PolygonFace::single(plain([1.0, 0.0, 0.0]), None),
                    PolygonFace::single(plain([0.0, 0.0, 1.0]), None),
                )
                .unwrap();
            Euclidean3DGeometry::Polygon(polygon)
        }

        /// The four corners the mesh fixtures share.
        fn mesh_vertices() -> Vec<[f64; 3]> {
            vec![
                [35.0, 139.0, 0.0],
                [35.1, 139.0, 0.0],
                [35.0, 139.1, 0.0],
                [35.1, 139.1, 0.0],
            ]
        }

        pub fn collection3d(members: Vec<Euclidean3DGeometry>) -> Euclidean3DGeometry {
            Euclidean3DGeometry::Collection(Collection3D::new(members))
        }

        /// Member attributes as the CityGML reader writes them.
        pub fn member_attrs(lod: Option<u8>, property: Option<&str>) -> Attributes {
            let mut attributes = Attributes::new();
            if let Some(lod) = lod {
                attributes.insert(
                    Attribute::new(MEMBER_LOD_KEY),
                    AttributeValue::Number(lod.into()),
                );
            }
            if let Some(property) = property {
                attributes.insert(
                    Attribute::new(MEMBER_PROPERTY_KEY),
                    AttributeValue::String(property.to_string()),
                );
            }
            attributes
        }

        /// A feature whose geometry is a reader-shaped collection: one member per
        /// source geometry property, each with its own attribute record.
        pub fn feature(members: Vec<(Attributes, Euclidean3DGeometry)>) -> Feature {
            let (attrs, geometries): (Vec<_>, Vec<_>) = members.into_iter().unzip();
            let members = geometries.into_iter().map(Geometry::Euclidean3D).collect();
            Feature::from(Geometry::GeometryCollection(
                GeometryCollection::with_attributes(members, attrs).unwrap(),
            ))
        }

        /// A feature whose geometry is one bare 3D leaf, with no collection and
        /// so no member metadata at all.
        pub fn bare(geometry: Euclidean3DGeometry) -> Feature {
            Feature::from(Geometry::Euclidean3D(geometry))
        }
    }

    use fixture::*;

    fn all_lods() -> LodMask {
        LodMask::all()
    }

    fn only_lod(lod: u8) -> LodMask {
        let mut mask = LodMask::default();
        mask.add_lod(lod);
        mask
    }

    fn surfaces(entry: &GeometryEntry) -> &[GmlSurface] {
        match &entry.element {
            GmlElement::MultiSurface { surfaces, .. } => surfaces,
            other => panic!("expected a MultiSurface, got {other:?}"),
        }
    }

    fn curves(entry: &GeometryEntry) -> &[Vec<[f64; 3]>] {
        match &entry.element {
            GmlElement::MultiCurve { curves, .. } => curves,
            other => panic!("expected a MultiCurve, got {other:?}"),
        }
    }

    fn solid_of(entry: &GeometryEntry) -> &GmlSolid {
        match &entry.element {
            GmlElement::Solid(solid) => solid,
            other => panic!("expected a Solid, got {other:?}"),
        }
    }

    fn multi_solid(entry: &GeometryEntry) -> &[GmlSolid] {
        match &entry.element {
            GmlElement::MultiSolid { solids, .. } => solids,
            other => panic!("expected a MultiSolid, got {other:?}"),
        }
    }

    /// Whether every ring of a surface repeats its first corner, which is what
    /// makes it a valid `gml:LinearRing`.
    fn is_closed(surface: &GmlSurface) -> bool {
        std::iter::once(&surface.exterior)
            .chain(&surface.interiors)
            .all(|ring| ring.first() == ring.last() && ring.len() >= 4)
    }

    /// The keys are spelled out in both crates, so this keeps them one fact.
    #[test]
    fn the_member_attribute_keys_match_the_readers() {
        use reearth_flow_action_processor::citygml_parser::pipeline::{
            MEMBER_LOD_KEY as READER_LOD, MEMBER_PROPERTY_KEY as READER_PROPERTY,
        };
        assert_eq!(MEMBER_LOD_KEY, READER_LOD);
        assert_eq!(MEMBER_PROPERTY_KEY, READER_PROPERTY);
    }

    // Coordinates and axis order

    /// The identity formatter: ordinates reach the file in the order the leaf
    /// stores them, which is the order the source declared.
    #[test]
    fn pos_list_is_written_in_the_stored_order() {
        assert_eq!(
            format_pos_list(&[[35.0, 139.0, 10.0], [35.1, 139.1, 11.0]]),
            "35 139 10 35.1 139.1 11"
        );
    }

    #[test]
    fn a_polygon_becomes_a_multi_surface_of_one_surface() {
        let feature = feature(vec![(
            member_attrs(Some(2), Some("lod2MultiSurface")),
            triangle(crs(6697), 0.0),
        )]);

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        assert_eq!(converted.geometries.len(), 1);
        assert_eq!(converted.geometries[0].lod, 2);
        assert_eq!(
            converted.geometries[0].property.as_deref(),
            Some("lod2MultiSurface")
        );
        let surfaces = surfaces(&converted.geometries[0]);
        assert_eq!(surfaces.len(), 1);
        assert_eq!(surfaces[0].exterior.len(), 4);
        assert!(surfaces[0].interiors.is_empty());
        assert!(converted.omissions.is_empty());
    }

    #[test]
    fn a_line_string_becomes_a_multi_curve_of_one_curve() {
        let feature = feature(vec![(
            member_attrs(Some(0), Some("lod0MultiCurve")),
            line(crs(6697)),
        )]);

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        assert_eq!(converted.geometries.len(), 1);
        assert_eq!(curves(&converted.geometries[0]).len(), 1);
        assert_eq!(curves(&converted.geometries[0])[0].len(), 2);
    }

    /// A `dem:tin` carries no LOD, so it lands at the default — and its property
    /// name is what still names the element it came from.
    #[test]
    fn a_member_with_no_lod_defaults_to_zero_and_keeps_its_property() {
        let feature = feature(vec![(
            member_attrs(None, Some("tin")),
            triangle(crs(6697), 0.0),
        )]);

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        assert_eq!(converted.geometries[0].lod, 0);
        assert_eq!(converted.geometries[0].property.as_deref(), Some("tin"));
    }

    /// Every LOD the mask can represent survives an all-LODs filter, each at its
    /// own level.
    #[test]
    fn every_lod_from_zero_to_four_is_carried_through() {
        let members = (0..=4)
            .map(|lod| {
                (
                    member_attrs(Some(lod), Some("lod2MultiSurface")),
                    triangle(crs(6697), f64::from(lod)),
                )
            })
            .collect();

        let converted = convert_city_object(&feature(members), &all_lods()).unwrap();

        let lods: Vec<u8> = converted.geometries.iter().map(|e| e.lod).collect();
        assert_eq!(lods, vec![0, 1, 2, 3, 4]);
    }

    /// The filter runs before accumulation, so a filtered-out member leaves no
    /// trace at all — not in the entries, and not in the envelope.
    #[test]
    fn a_filtered_out_member_contributes_nothing() {
        let feature = feature(vec![
            (member_attrs(Some(1), None), triangle(crs(6697), 0.0)),
            (member_attrs(Some(2), None), triangle(crs(6697), 10.0)),
        ]);

        let converted = convert_city_object(&feature, &only_lod(1)).unwrap();

        assert_eq!(converted.geometries.len(), 1);
        assert_eq!(converted.geometries[0].lod, 1);
        let envelope = converted.envelope.unwrap();
        assert_eq!(
            envelope.upper[0], 35.1,
            "the LOD 2 member is 10 degrees away"
        );
    }

    /// The default LOD is filtered like any other, so a `lodFilter` that excludes
    /// 0 excludes a member that recorded none.
    #[test]
    fn the_default_lod_is_subject_to_the_filter() {
        let feature = feature(vec![(
            member_attrs(None, Some("tin")),
            triangle(crs(6697), 0.0),
        )]);

        let converted = convert_city_object(&feature, &only_lod(2)).unwrap();

        assert!(converted.geometries.is_empty());
    }

    #[test]
    fn an_unusable_lod_attribute_is_an_error_naming_the_member() {
        let mut attributes = Attributes::new();
        attributes.insert(
            Attribute::new(MEMBER_LOD_KEY),
            AttributeValue::String("two".to_string()),
        );
        let feature = feature(vec![(attributes, triangle(crs(6697), 0.0))]);

        let error = convert_city_object(&feature, &all_lods()).unwrap_err();

        let message = error.to_string();
        assert!(message.contains("member 0"), "{message}");
        assert!(message.contains(MEMBER_LOD_KEY), "{message}");
    }

    #[test]
    fn an_out_of_range_lod_is_an_error() {
        let mut attributes = Attributes::new();
        attributes.insert(
            Attribute::new(MEMBER_LOD_KEY),
            AttributeValue::Number(9.into()),
        );
        let feature = feature(vec![(attributes, triangle(crs(6697), 0.0))]);

        assert!(convert_city_object(&feature, &all_lods()).is_err());
    }

    // Meshes

    /// A mesh is a set of independent faces over one vertex pool, so it becomes
    /// one `gml:MultiSurface` with one `gml:Polygon` per face — and each face's
    /// ring, stored open, is closed on the way out.
    #[test]
    fn a_polygon_mesh_becomes_one_surface_per_face() {
        let feature = feature(vec![(
            member_attrs(Some(2), Some("lod2MultiSurface")),
            polygon_mesh(crs(6697)),
        )]);

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        let surfaces = surfaces(&converted.geometries[0]);
        assert_eq!(surfaces.len(), 2);
        assert!(surfaces.iter().all(is_closed), "{surfaces:?}");
        assert_eq!(
            surfaces[0].exterior,
            vec![
                [35.0, 139.0, 0.0],
                [35.1, 139.0, 0.0],
                [35.0, 139.1, 0.0],
                // the closing corner, appended here
                [35.0, 139.0, 0.0],
            ]
        );
        assert!(converted.omissions.is_empty());
    }

    /// A mesh face's hole survives as a `gml:interior` ring of the same polygon,
    /// not as a face of its own.
    #[test]
    fn a_mesh_face_keeps_its_hole_as_an_interior_ring() {
        let feature = feature(vec![(
            member_attrs(Some(2), None),
            polygon_mesh_with_a_hole(crs(6697)),
        )]);

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        let surfaces = surfaces(&converted.geometries[0]);
        assert_eq!(surfaces.len(), 1);
        assert_eq!(surfaces[0].interiors.len(), 1);
        assert!(is_closed(&surfaces[0]));
        assert_eq!(surfaces[0].interiors[0][0], [35.1, 139.1, 0.0]);
    }

    /// A `dem:tin` reaches the sink as a triangle mesh, whose faces are always
    /// stored open — the case ring closure exists for.
    #[test]
    fn a_triangular_mesh_becomes_one_closed_surface_per_triangle() {
        let feature = feature(vec![(
            member_attrs(None, Some("tin")),
            triangular_mesh(crs(6697)),
        )]);

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        assert_eq!(converted.geometries[0].property.as_deref(), Some("tin"));
        let surfaces = surfaces(&converted.geometries[0]);
        assert_eq!(surfaces.len(), 2);
        assert!(surfaces.iter().all(is_closed), "{surfaces:?}");
        assert!(surfaces.iter().all(|s| s.exterior.len() == 4));
    }

    // Solids

    /// A lone solid stays a `gml:Solid`, so the source property that named it
    /// (`lod1Solid`) still names the element it wraps.
    #[test]
    fn a_solid_becomes_a_gml_solid_with_its_exterior_faces() {
        let feature = feature(vec![(
            member_attrs(Some(1), Some("lod1Solid")),
            solid(crs(6697), 0),
        )]);

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        assert_eq!(converted.geometries.len(), 1);
        assert_eq!(converted.geometries[0].lod, 1);
        assert_eq!(
            converted.geometries[0].property.as_deref(),
            Some("lod1Solid")
        );
        let solid = solid_of(&converted.geometries[0]);
        assert_eq!(solid.exterior.len(), 2);
        assert!(solid.interiors.is_empty());
        assert!(solid.exterior.iter().all(is_closed));
    }

    /// The void the unified reader deliberately kept, and the legacy one
    /// discarded, becomes one `gml:interior` shell per void.
    #[test]
    fn a_solids_voids_become_interior_shells() {
        let feature = feature(vec![(
            member_attrs(Some(1), Some("lod1Solid")),
            solid(crs(6697), 2),
        )]);

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        let solid = solid_of(&converted.geometries[0]);
        assert_eq!(solid.exterior.len(), 2);
        assert_eq!(solid.interiors.len(), 2);
        assert!(solid.interiors.iter().all(|shell| shell.len() == 1));
        assert_eq!(solid.interiors[0][0].exterior[0], [35.01, 139.01, 0.0]);
        assert_eq!(solid.interiors[1][0].exterior[0], [35.02, 139.02, 0.0]);
    }

    /// A collection of nothing but solids is one `gml:MultiSolid`, which is only
    /// nameable because the source property name is retained.
    #[test]
    fn a_homogeneous_collection_of_solids_coalesces_into_a_multi_solid() {
        let feature = feature(vec![(
            member_attrs(Some(2), Some("lod2MultiSolid")),
            collection3d(vec![solid(crs(6697), 0), solid(crs(6697), 1)]),
        )]);

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        assert_eq!(converted.geometries.len(), 1);
        assert_eq!(
            converted.geometries[0].property.as_deref(),
            Some("lod2MultiSolid")
        );
        let solids = multi_solid(&converted.geometries[0]);
        assert_eq!(solids.len(), 2);
        assert!(solids[0].interiors.is_empty());
        assert_eq!(solids[1].interiors.len(), 1);
    }

    /// Families never merge: a collection holding a solid and a surface yields
    /// one entry each, solid first.
    #[test]
    fn a_solid_beside_a_surface_yields_one_entry_per_family() {
        let feature = feature(vec![(
            member_attrs(Some(2), None),
            collection3d(vec![triangle(crs(6697), 0.0), solid(crs(6697), 0)]),
        )]);

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        assert_eq!(converted.geometries.len(), 2);
        assert_eq!(solid_of(&converted.geometries[0]).exterior.len(), 2);
        assert_eq!(surfaces(&converted.geometries[1]).len(), 1);
    }

    /// A solid's coordinates reach the file, so they have to reach the envelope
    /// and the CRS coverage too.
    #[test]
    fn a_solid_folds_its_coordinates_into_the_envelope_and_the_crs() {
        let feature = feature(vec![(member_attrs(Some(1), None), solid(crs(6697), 1))]);

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        let envelope = converted.envelope.unwrap();
        assert_eq!(envelope.lower, [35.0, 139.0, 0.0]);
        // The void's far corner, written as the arithmetic that produced it so
        // the assertion does not depend on a decimal literal rounding the same
        // way `139.1 + 0.01` does.
        assert_eq!(envelope.upper, [35.1 + 0.01, 139.1 + 0.01, 0.0]);
        assert_eq!(converted.crs, CrsCoverage::Single(6697.into()));
    }

    // Ring closure

    /// An open ring gains its first corner back, and says so: Phase 5 duplicates
    /// the UV of exactly that corner in the same step.
    #[test]
    fn closing_an_open_ring_appends_its_first_corner() {
        let mut ring = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];

        assert_eq!(close_ring(&mut ring), Some(0));
        assert_eq!(ring.len(), 4);
        assert_eq!(ring[3], [0.0, 0.0, 0.0]);
    }

    /// A ring that already closes is left exactly as it was — the CityGML→
    /// CityGML round trip must not grow a corner per ring.
    #[test]
    fn closing_an_already_closed_ring_changes_nothing() {
        let mut ring = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let before = ring.clone();

        assert_eq!(close_ring(&mut ring), None);
        assert_eq!(ring, before);
    }

    #[test]
    fn closing_an_empty_ring_is_a_no_op() {
        let mut ring: Vec<[f64; 3]> = Vec::new();

        assert_eq!(close_ring(&mut ring), None);
        assert!(ring.is_empty());
    }

    // Appearance

    /// The appearance path end to end on one leaf: palettes, surface indices,
    /// theme, and the image listed for the shell to stage.
    #[test]
    fn a_textured_polygon_reaches_the_document_with_its_material_texture_and_image() {
        let feature = feature(vec![(
            member_attrs(Some(2), Some("lod2MultiSurface")),
            textured_triangle(
                crs(6697),
                vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0], [0.0, 0.0]],
            ),
        )]);

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        let surfaces = surfaces(&converted.geometries[0]);
        assert_eq!(surfaces[0].material_idx, Some(0));
        assert_eq!(surfaces[0].texture_idx, Some(0));
        assert_eq!(converted.appearance.theme.as_deref(), Some("rgbTexture"));
        assert_eq!(converted.appearance.materials.len(), 1);
        assert_eq!(
            converted.appearance.textures[0].key,
            "file:///textures/wall.png"
        );
        assert_eq!(converted.textures.len(), 1);
        assert!(converted.omissions.is_empty());
    }

    /// The flip, at the level a reader→writer round trip sees it: what the
    /// CityGML reader turned into Flow's top-left origin comes back out in
    /// CityGML's bottom-left one.
    #[test]
    fn an_emitted_uv_is_flipped_back_to_citygmls_origin() {
        let feature = feature(vec![(
            member_attrs(Some(2), None),
            textured_triangle(
                crs(6697),
                vec![[0.0, 0.0], [1.0, 0.25], [0.5, 1.0], [0.0, 0.0]],
            ),
        )]);

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        assert_eq!(
            surfaces(&converted.geometries[0])[0].uv_exterior,
            vec![[0.0, 1.0], [1.0, 0.75], [0.5, 0.0], [0.0, 1.0]]
        );
    }

    /// Why the visitor hands back corner ranges: a mesh's UV is one flat array,
    /// so each ring slices its own part, closure corner included.
    #[test]
    fn each_mesh_face_takes_its_own_slice_of_the_meshs_uv() {
        // Six corners, three per triangle, each face's UVs distinguishable.
        let uv = vec![
            [0.0, 0.0],
            [0.1, 0.0],
            [0.2, 0.0],
            [0.5, 0.0],
            [0.6, 0.0],
            [0.7, 0.0],
        ];
        let feature = feature(vec![(
            member_attrs(None, Some("tin")),
            textured_triangular_mesh(crs(6697), uv),
        )]);

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        let surfaces = surfaces(&converted.geometries[0]);
        assert_eq!(surfaces.len(), 2);
        // `v` is 0 throughout, so every flipped ordinate is 1 and only the slice
        // boundary is under test.
        assert_eq!(
            surfaces[0].uv_exterior,
            vec![[0.0, 1.0], [0.1, 1.0], [0.2, 1.0], [0.0, 1.0]],
            "face 0 takes corners 0..3, closed by repeating corner 0"
        );
        assert_eq!(
            surfaces[1].uv_exterior,
            vec![[0.5, 1.0], [0.6, 1.0], [0.7, 1.0], [0.5, 1.0]],
            "face 1 takes corners 3..6, closed by repeating corner 3"
        );
        // A closed ring and its UV have the same length, which is the whole
        // point of closing and slicing in one step.
        assert!(surfaces
            .iter()
            .all(|surface| surface.exterior.len() == surface.uv_exterior.len()));
    }

    /// One image under two leaves is one `app:ParameterizedTexture` and one
    /// staged file, while the materials still merge with an index offset.
    #[test]
    fn palettes_merge_across_a_features_leaves() {
        let uv = || vec![[0.0, 0.0], [1.0, 0.0], [0.5, 1.0], [0.0, 0.0]];
        let feature = feature(vec![
            (
                member_attrs(Some(1), None),
                textured_triangle(crs(6697), uv()),
            ),
            (
                member_attrs(Some(2), None),
                textured_triangle(crs(6697), uv()),
            ),
        ]);

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        assert_eq!(converted.appearance.materials.len(), 2);
        assert_eq!(converted.appearance.textures.len(), 1);
        assert_eq!(converted.textures.len(), 1);
        assert_eq!(surfaces(&converted.geometries[0])[0].material_idx, Some(0));
        assert_eq!(surfaces(&converted.geometries[1])[0].material_idx, Some(1));
        assert_eq!(surfaces(&converted.geometries[1])[0].texture_idx, Some(0));
    }

    /// An appearance narrowing is reported through the same channel as a
    /// geometry one, so it reaches the feature's single warning line.
    #[test]
    fn an_appearance_narrowing_is_reported_with_the_geometry_omissions() {
        let feature = feature(vec![(
            member_attrs(Some(2), None),
            two_sided_triangle(crs(6697)),
        )]);

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        assert_eq!(converted.omissions.len(), 1);
        assert_eq!(
            converted.omissions[0].geometry,
            "back-side appearance binding"
        );
        // The front side is still painted, and only the front material is in
        // the palette.
        assert_eq!(converted.appearance.materials.len(), 1);
        assert_eq!(converted.appearance.materials[0].diffuse_color.r, 1.0);
    }

    /// A bare leaf stays bare: no appearance means no palettes, no images, and
    /// no `app:appearanceMember` in the document.
    #[test]
    fn a_leaf_with_no_appearance_produces_no_palettes() {
        let converted = convert_city_object(&bare(triangle(crs(6697), 0.0)), &all_lods()).unwrap();

        assert!(!converted.appearance.has_content());
        assert!(converted.appearance.theme.is_none());
        assert!(converted.textures.is_empty());
        assert_eq!(surfaces(&converted.geometries[0])[0].material_idx, None);
    }

    /// A bare leaf is written at `DEFAULT_LOD`, and `lodFilter` applies to that
    /// default like any other LOD — it is not exempt for having no metadata.
    #[test]
    fn a_bare_leaf_is_filtered_by_its_default_lod() {
        let feature = bare(triangle(crs(6697), 0.0));

        let kept = convert_city_object(&feature, &only_lod(DEFAULT_LOD)).unwrap();
        assert_eq!(kept.geometries.len(), 1);
        assert_eq!(kept.geometries[0].lod, DEFAULT_LOD);

        let dropped = convert_city_object(&feature, &only_lod(2)).unwrap();
        assert!(dropped.geometries.is_empty());
        assert!(
            dropped.envelope.is_none(),
            "a filtered leaf must not reach the envelope"
        );
        assert_eq!(
            dropped.crs,
            CrsCoverage::NoCoordinates,
            "a filtered leaf must not reach the CRS coverage"
        );
    }

    // Collections

    /// A nested `Collection3D` has no member metadata of its own, so it inherits
    /// the LOD and property of the member it descends from, and its surface
    /// members coalesce into one `gml:MultiSurface`.
    #[test]
    fn a_nested_collection_coalesces_and_inherits_its_members_metadata() {
        let feature = feature(vec![(
            member_attrs(Some(2), Some("lod2MultiSurface")),
            collection3d(vec![
                collection3d(vec![triangle(crs(6697), 0.0)]),
                triangle(crs(6697), 1.0),
            ]),
        )]);

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        assert_eq!(converted.geometries.len(), 1);
        assert_eq!(converted.geometries[0].lod, 2);
        assert_eq!(
            converted.geometries[0].property.as_deref(),
            Some("lod2MultiSurface")
        );
        assert_eq!(surfaces(&converted.geometries[0]).len(), 2);
    }

    /// A collection holding both families splits into one entry per family, both
    /// under the same LOD and property.
    #[test]
    fn a_mixed_family_collection_splits_into_one_entry_per_family() {
        let feature = feature(vec![(
            member_attrs(Some(1), Some("lod1Geometry")),
            collection3d(vec![triangle(crs(6697), 0.0), line(crs(6697))]),
        )]);

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        assert_eq!(converted.geometries.len(), 2);
        assert_eq!(surfaces(&converted.geometries[0]).len(), 1);
        assert_eq!(curves(&converted.geometries[1]).len(), 1);
        assert!(converted
            .geometries
            .iter()
            .all(|entry| entry.lod == 1 && entry.property.as_deref() == Some("lod1Geometry")));
    }

    /// A feature whose geometry is one bare leaf carries no member metadata, so
    /// it lands at the default LOD with the writer's family fallback naming it.
    #[test]
    fn a_bare_leaf_lands_at_the_default_lod_with_no_property() {
        let converted = convert_city_object(&bare(triangle(crs(6697), 0.0)), &all_lods()).unwrap();

        assert_eq!(converted.geometries.len(), 1);
        assert_eq!(converted.geometries[0].lod, 0);
        assert_eq!(converted.geometries[0].property, None);
    }

    // Omissions

    #[test]
    fn every_unsupported_leaf_is_reported_once_per_kind() {
        let feature = feature(vec![(
            member_attrs(Some(0), None),
            collection3d(vec![
                point(crs(6697)),
                point(crs(6697)),
                triangle(crs(6697), 0.0),
            ]),
        )]);

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        // The supported sibling still reaches the document.
        assert_eq!(surfaces(&converted.geometries[0]).len(), 1);
        assert_eq!(converted.omissions.len(), 1);
        assert_eq!(converted.omissions[0].geometry, "Point");
        assert_eq!(converted.omissions[0].count, 2);
    }

    /// An omitted leaf contributes neither coordinates nor a CRS, so a document
    /// made only of omissions writes nothing and declares nothing.
    #[test]
    fn an_omitted_leaf_contributes_no_envelope_and_no_crs() {
        let feature = feature(vec![(member_attrs(Some(0), None), point(crs(6697)))]);

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        assert!(converted.geometries.is_empty());
        assert!(converted.envelope.is_none());
        assert_eq!(converted.crs, CrsCoverage::NoCoordinates);
    }

    #[test]
    fn a_2d_geometry_is_omitted_rather_than_promoted() {
        use reearth_flow_geometry::line_string::LineString2D;
        use reearth_flow_geometry::{Euclidean2DGeometry, Geometry};

        let feature = Feature::from(Geometry::Euclidean2D(Euclidean2DGeometry::LineString(
            LineString2D::from_coords(crs(6668), vec![[35.0, 139.0], [35.1, 139.1]]),
        )));

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        assert!(converted.geometries.is_empty());
        assert_eq!(converted.omissions.len(), 1);
        assert_eq!(converted.omissions[0].geometry, "Euclidean2D");
    }

    // Envelope

    /// The envelope covers every emitted ordinate, holes included, and is folded
    /// in the leaf's own axis order so it reads like the geometry it bounds.
    #[test]
    fn the_envelope_covers_every_emitted_coordinate() {
        let feature = feature(vec![
            (member_attrs(Some(0), None), triangle(crs(6697), 0.0)),
            (member_attrs(Some(1), None), triangle(crs(6697), 1.0)),
        ]);

        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        let envelope = converted.envelope.unwrap();
        assert_eq!(envelope.lower, [35.0, 139.0, 0.0]);
        assert_eq!(envelope.upper, [36.1, 140.1, 0.0]);
    }

    // CRS coverage and srsName

    #[test]
    fn one_crs_across_every_leaf_is_declared() {
        let feature = feature(vec![(
            member_attrs(Some(0), None),
            triangle(crs(6697), 0.0),
        )]);
        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        assert_eq!(
            srs_name(&[], None, converted.crs).unwrap(),
            "http://www.opengis.net/def/crs/EPSG/0/6697"
        );
    }

    #[test]
    fn an_epsg_code_that_agrees_with_the_coverage_is_accepted() {
        let coverage = CrsCoverage::Single(6697.into());
        assert_eq!(
            srs_name(&[], Some(6697), coverage).unwrap(),
            "http://www.opengis.net/def/crs/EPSG/0/6697"
        );
    }

    #[test]
    fn an_epsg_code_that_disagrees_with_the_coverage_names_both() {
        let coverage = CrsCoverage::Single(6697.into());
        let message = srs_name(&[], Some(4326), coverage).unwrap_err().to_string();

        assert!(message.contains("4326"), "{message}");
        assert!(message.contains("6697"), "{message}");
    }

    #[test]
    fn a_mixture_of_crss_is_an_error_naming_both() {
        let feature = feature(vec![
            (member_attrs(Some(0), None), triangle(crs(6697), 0.0)),
            (member_attrs(Some(1), None), triangle(crs(6668), 0.0)),
        ]);
        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        let message = srs_name(&[], None, converted.crs).unwrap_err().to_string();

        assert!(message.contains("6697"), "{message}");
        assert!(message.contains("6668"), "{message}");
    }

    /// A leaf outside any CRS cannot be labelled by the geometry, so the writer
    /// fails rather than stamping a plausible `srsName` on it …
    #[test]
    fn coordinates_outside_any_crs_are_an_error_without_an_epsg_code() {
        let feature = feature(vec![(
            member_attrs(Some(0), None),
            triangle(tangent(), 0.0),
        )]);
        let converted = convert_city_object(&feature, &all_lods()).unwrap();

        assert_eq!(converted.crs, CrsCoverage::OutsideAnyCrs);
        assert!(srs_name(&[], None, converted.crs).is_err());
    }

    /// … but `epsgCode` is exactly the remedy for that case: a document with no
    /// `srsName` reads back as `Euclidean` on every leaf, and the user is the one
    /// who knows what it was.
    #[test]
    fn an_epsg_code_labels_coordinates_outside_any_crs() {
        assert_eq!(
            srs_name(&[], Some(6697), CrsCoverage::OutsideAnyCrs).unwrap(),
            "http://www.opengis.net/def/crs/EPSG/0/6697"
        );
    }

    /// Nothing that names the CRS is written when nothing was emitted, so this
    /// must not fail the run.
    #[test]
    fn a_document_with_no_emitted_coordinate_still_resolves_an_srs_name() {
        assert_eq!(
            srs_name(&[], None, CrsCoverage::NoCoordinates).unwrap(),
            "http://www.opengis.net/def/crs/EPSG/0/4326"
        );
        assert_eq!(
            srs_name(&[], Some(6697), CrsCoverage::NoCoordinates).unwrap(),
            "http://www.opengis.net/def/crs/EPSG/0/6697"
        );
    }
}
