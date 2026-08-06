//! The unified world's half of the converter seam: `reearth_flow_geometry`'s
//! recursive hierarchy in, the shared [`super::model`] out.
//!
//! Scope: every 3D leaf CityGML 2.0 has an element for — `Polygon`,
//! `LineString`, `PolygonMesh`, `TriangularMesh`, `Solid` — and collections of
//! them. `Point`, `PointCloud`, `Csg` and the 2D leaves have no counterpart and
//! are reported as a [`GeometryOmission`] rather than silently skipped.
//! Appearance is still Phase 5's: surfaces come out bare.
//!
//! Three things this module owns that the legacy converter does not:
//!
//! - **Axis order.** The new reader stores `gml:posList` ordinates in the axis
//!   order the CRS itself declares, so [`format_pos_list`] writes them back
//!   verbatim. That reproduces the source `posList` byte for byte on a
//!   CityGML→CityGML round trip, and stays correct for projected frames where
//!   the legacy world's blind `y x z` transposition would not be.
//! - **The CRS declaration.** `srsName` is folded over the leaves that actually
//!   reach the file, so it is never a guess: one CRS is declared, a mixture is
//!   an error, and coordinates outside any CRS are only labelled when the user
//!   supplied an `epsgCode` to label them with.
//! - **Ring closure.** A `gml:LinearRing` must repeat its first corner, but the
//!   geometry crate stores rings as they arrived — a triangle-mesh face has
//!   three corners and no closing one. Every emitted ring is closed here, at
//!   the one point where the ring and (from Phase 5) its UV slice are both in
//!   hand, so the two cannot drift apart.

use reearth_flow_geometry::coordinate::CoordinateFrame;
use reearth_flow_geometry::polygon_mesh::FaceVisit;
use reearth_flow_geometry::solid::{Shell, Solid};
use reearth_flow_geometry::{Euclidean3DGeometry, Geometry, GeometryCollection};
use reearth_flow_types::conversion::CrsCoverage;
use reearth_flow_types::lod::LodMask;
use reearth_flow_types::{Attribute, AttributeValue, Attributes, CitygmlFeatureExt, Feature};

use super::model::{
    AppearanceBundle, BoundingEnvelope, ConvertedCityObject, GeometryEntry, GeometryOmission,
    GmlElement, GmlSolid, GmlSurface,
};
use crate::errors::SinkError;

/// The member-attribute key the CityGML reader records each collection member's
/// source LOD under. Declared here rather than imported because `action-sink`
/// depends on `action-processor` only as a dev-dependency; a unit test pins the
/// two spellings together.
const MEMBER_LOD_KEY: &str = "lod";

/// The member-attribute key the CityGML reader records the local name of the
/// geometry property each collection member was carved from under.
const MEMBER_PROPERTY_KEY: &str = "citygmlProperty";

/// The LOD a member with no recorded LOD is written at, matching the legacy
/// converter's `lod.unwrap_or(0)`. The `lodFilter` applies to it like any other,
/// so the default stays observable instead of becoming a hidden exemption.
const DEFAULT_LOD: u8 = 0;

/// The highest LOD [`LodMask`] can represent; a larger value could not be
/// filtered on, so it is rejected rather than silently written.
const MAX_LOD: u8 = 4;

/// Serialize `coords` as the body of a `gml:posList` — or of a
/// `gml:lowerCorner` / `gml:upperCorner`, which the writer formats the same way
/// so a document's envelope always reads in the same axis order as its geometry.
///
/// This is the identity formatter: the new reader parses `posList` into ordinate
/// triples in the source's own axis order and every leaf keeps them that way, so
/// writing them back unchanged is what reproduces the source. The legacy world
/// stores `x` as longitude/easting and therefore has its own, transposing,
/// formatter.
pub fn format_pos_list(coords: &[[f64; 3]]) -> String {
    coords
        .iter()
        .map(|c| format!("{} {} {}", c[0], c[1], c[2]))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The OGC CRS URI to declare for a document whose emitted coordinates have the
/// given `coverage`.
///
/// `features` is unused here: unlike the legacy world, which reads a
/// whole-feature EPSG off the first feature, the unified world's CRS lives on
/// each leaf and is already folded into `coverage` — over exactly the leaves
/// that were written, so filtered and omitted geometry cannot influence it.
///
/// `epsg_code` declares, it does not reproject: when the coverage names a CRS
/// the parameter must agree with it, and when the coverage names none the
/// parameter is the only thing that can label the coordinates truthfully.
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
        // Nothing that names the CRS is written for a document with no emitted
        // coordinate — no envelope, no geometry element — so this value never
        // reaches the file and cannot mislabel anything.
        CrsCoverage::NoCoordinates => epsg_code.unwrap_or(DEFAULT_EPSG),
    };
    Ok(format!("http://www.opengis.net/def/crs/EPSG/0/{code}"))
}

/// The code the legacy world falls back to, kept only for the degenerate
/// document that declares a CRS no element references.
const DEFAULT_EPSG: u32 = 4326;

/// Convert one feature's geometry into the shared CityGML model.
///
/// Members filtered out by `lod_mask` are skipped before anything is
/// accumulated, so neither the envelope nor the CRS coverage sees them.
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

/// One emitted leaf, before it is grouped into a GML family. Members of one
/// source property share LOD, property name, and appearance semantics by
/// construction, which is what makes coalescing them into one `gml:MultiSurface`
/// / `gml:MultiCurve` safe.
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
    omissions: Vec<GeometryOmission>,
}

impl Conversion {
    fn finish(self) -> ConvertedCityObject {
        ConvertedCityObject {
            geometries: self.geometries,
            appearance: AppearanceBundle {
                materials: Vec::new(),
                textures: Vec::new(),
            },
            envelope: self.envelope,
            crs: self.crs,
            textures: Vec::new(),
            omissions: self.omissions,
        }
    }

    /// Convert one collection member (or the feature's whole geometry, which is
    /// the same thing one level up), at the LOD and under the property name it
    /// inherited.
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
                let mut pieces = Vec::new();
                self.collect_pieces(geometry, &mut pieces);
                self.push_entries(lod, property, pieces);
                Ok(())
            }
            Geometry::GeometryCollection(collection) => {
                self.convert_collection(collection, lod_mask, lod, property)
            }
        }
    }

    /// Walk a heterogeneous collection, reading each member's LOD and property
    /// name off the parallel attribute record the CityGML reader filled in.
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
            // Filtering here, before anything is accumulated, is what keeps a
            // filtered-out member out of the envelope and the CRS coverage.
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

    /// The LOD a member records, or `None` when it records none.
    ///
    /// A present-but-unusable value is an error rather than a fallback to the
    /// default: it means the metadata channel is broken, and guessing would
    /// write geometry under the wrong LOD.
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

    /// Flatten one 3D geometry into the leaves CityGML 2.0 can carry, reporting
    /// the rest.
    fn collect_pieces(&mut self, geometry: &Euclidean3DGeometry, out: &mut Vec<Piece>) {
        match geometry {
            Euclidean3DGeometry::Polygon(polygon) => {
                self.fold_frame(polygon.frame());
                let exterior = polygon.exterior().to_vec();
                let interiors: Vec<Vec<[f64; 3]>> =
                    polygon.interiors().map(<[[f64; 3]]>::to_vec).collect();
                out.push(Piece::Surface(self.surface(exterior, interiors)));
            }
            Euclidean3DGeometry::LineString(line_string) => {
                self.fold_frame(line_string.frame());
                let coords = line_string.coords().to_vec();
                self.fold_envelope(&coords);
                out.push(Piece::Curve(coords));
            }
            Euclidean3DGeometry::Collection(collection) => {
                for member in collection.members() {
                    self.collect_pieces(member, out);
                }
            }
            // A mesh is a set of independent faces sharing a vertex pool, which
            // is exactly what `gml:MultiSurface` is: one `gml:Polygon` per face.
            Euclidean3DGeometry::PolygonMesh(mesh) => {
                self.fold_frame(mesh.frame());
                let mut faces = Vec::with_capacity(mesh.num_faces());
                mesh.for_each_face(|face| faces.push(face_rings(&face)));
                for (exterior, interiors) in faces {
                    out.push(Piece::Surface(self.surface(exterior, interiors)));
                }
            }
            Euclidean3DGeometry::TriangularMesh(mesh) => {
                self.fold_frame(mesh.frame());
                let mut faces = Vec::with_capacity(mesh.num_triangles());
                mesh.for_each_face(|face| faces.push(face_rings(&face)));
                for (exterior, interiors) in faces {
                    out.push(Piece::Surface(self.surface(exterior, interiors)));
                }
            }
            // One `gml:CompositeSurface` per shell: the exterior bounds the
            // volume, each interior bounds a void the unified reader kept and
            // the legacy one discarded.
            Euclidean3DGeometry::Solid(solid) => {
                self.fold_frame(solid.frame());
                out.push(Piece::Solid(self.solid(solid)));
            }
            // Parity-first omissions: CityGML 2.0 has no element for these, and
            // coercing them would fabricate geometry.
            Euclidean3DGeometry::Point(_) => self.omit("Point", POINT_REASON),
            Euclidean3DGeometry::PointCloud(_) => self.omit("PointCloud", POINT_REASON),
            Euclidean3DGeometry::Csg(_) => self.omit(
                "Csg",
                "a boolean tree has no CityGML counterpart until it is evaluated, which the \
                 writer does not do",
            ),
        }
    }

    /// One solid, as its exterior shell's faces plus one face list per void.
    fn solid(&mut self, solid: &Solid) -> GmlSolid {
        GmlSolid {
            id: None,
            exterior: self.shell(solid.exterior()),
            interiors: solid
                .interiors()
                .iter()
                .map(|shell| self.shell(shell))
                .collect(),
        }
    }

    /// One boundary shell's faces, whichever mesh kind it is.
    fn shell(&mut self, shell: &Shell) -> Vec<GmlSurface> {
        let mut faces = Vec::with_capacity(shell.num_faces());
        shell.for_each_face(|face| faces.push(face_rings(&face)));
        faces
            .into_iter()
            .map(|(exterior, interiors)| self.surface(exterior, interiors))
            .collect()
    }

    /// Build one `gml:Polygon` from a face's rings: close each ring, fold its
    /// coordinates into the envelope, and leave the appearance slots empty for
    /// Phase 5 to fill.
    fn surface(&mut self, exterior: Vec<[f64; 3]>, interiors: Vec<Vec<[f64; 3]>>) -> GmlSurface {
        let mut exterior = exterior;
        // The closing corner duplicates the ring's first, which is the corner
        // whose UV Phase 5 duplicates in the same step. Nothing reads it yet.
        let _closed_at = close_ring(&mut exterior);
        self.fold_envelope(&exterior);
        let interiors = interiors
            .into_iter()
            .map(|mut ring| {
                let _closed_at = close_ring(&mut ring);
                self.fold_envelope(&ring);
                ring
            })
            .collect();
        GmlSurface {
            id: None,
            exterior,
            interiors,
            material_idx: None,
            texture_idx: None,
            uv_exterior: Vec::new(),
            uv_interiors: Vec::new(),
        }
    }

    /// Group one member's emitted leaves by GML family: all solids into one
    /// `gml:Solid` or `gml:MultiSolid`, all surfaces into one
    /// `gml:MultiSurface`, all curves into one `gml:MultiCurve`. A member that
    /// produced several families yields one entry per family, in descending
    /// dimension.
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
        // One solid stays a `gml:Solid`, so a `lod1Solid` property still names
        // the element it wraps; several coalesce into one `gml:MultiSolid`,
        // which is only nameable because the source property name is retained.
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
            // A tangent plane's in-plane coordinates are not its base CRS's, so
            // neither frame names a CRS these coordinates are in.
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

    /// Record one leaf CityGML 2.0 has no place for, aggregated by kind so a
    /// collection of a thousand points is reported once.
    fn omit(&mut self, geometry: &'static str, reason: &'static str) {
        match self
            .omissions
            .iter_mut()
            .find(|omission| omission.geometry == geometry)
        {
            Some(omission) => omission.count += 1,
            None => self.omissions.push(GeometryOmission {
                geometry,
                reason,
                count: 1,
            }),
        }
    }
}

const POINT_REASON: &str =
    "this writer emits no gml:Point / gml:MultiPoint, matching the legacy build";

/// A visited face's rings as owned coordinates: the exterior first, then the
/// holes. Copied out because the visitor's borrows do not outlive the callback.
fn face_rings(face: &FaceVisit<'_>) -> (Vec<[f64; 3]>, Vec<Vec<[f64; 3]>>) {
    (
        face.exterior.coords.to_vec(),
        face.interiors
            .iter()
            .map(|ring| ring.coords.to_vec())
            .collect(),
    )
}

/// Close `ring` for GML: a `gml:LinearRing` repeats its first corner, and rings
/// reach here open whenever they came from a triangle (three corners, no
/// closing one) or from an index-sourced mesh, so this is not an edge case.
///
/// Returns the ring-local position of the corner the appended one duplicates —
/// always `0` — or `None` when the ring was already closed. Phase 5 extends the
/// ring's UV slice by exactly that corner, which is why closing a ring and
/// slicing its UV have to happen in the same step.
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

    /// The unified world is the only one that builds these, so the fixtures need
    /// no gating: this whole module compiles only under `new-geometry`.
    mod fixture {
        use reearth_flow_geometry::collection::Collection3D;
        use reearth_flow_geometry::coordinate::{
            BaseFrame, CoordinateFrame, EpsgCode, TangentPlane,
        };
        use reearth_flow_geometry::line_string::LineString3D;
        use reearth_flow_geometry::point::Point3D;
        use reearth_flow_geometry::polygon::Polygon3D;
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

    /// The reader writes these keys; the converter reads them. They are spelled
    /// out in both crates because the dependency runs the other way, so this is
    /// the assertion that keeps the two spellings one fact.
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
