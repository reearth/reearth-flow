//! Shapefile shape conversion. Builds `reearth_flow_geometry::Geometry` (per-leaf
//! `CoordinateFrame`) from the shapes the `shapefile` crate yields.
//!
//! A shapefile winds its outer rings clockwise and its holes counter-clockwise,
//! and shows a multipatch surface's front to a viewer who sees its vertices go
//! clockwise; a geometry winds each face the other way round, so every ring and
//! triangle is reversed on the way in.
//!
//! Measures (the `M` channel) are not represented: an M-bearing shape converts to
//! its unmeasured counterpart and the measures are discarded, reported once per
//! read by [`ShapeConverter::report_discarded_measures`].

use std::cell::Cell;

use reearth_flow_geometry::{
    collection::{Collection2D, Collection3D},
    coordinate::{CoordinateFrame, EpsgCode},
    line_string::{LineString2D, LineString3D},
    point::{Point2D, Point3D},
    polygon::{Polygon2D, Polygon3D},
    polygon_mesh::PolygonMesh3D,
    triangular_mesh::TriangularMesh3D,
    Euclidean2DGeometry, Euclidean3DGeometry, Geometry,
};
use shapefile::{Patch, PolygonRing, Shape, NO_DATA};

use crate::errors::{ShapefileError, SourceError};

/// A shapefile vertex's horizontal position, whatever optional channels it carries.
trait Position {
    /// The stored `x`, an easting.
    fn x(&self) -> f64;
    /// The stored `y`, a northing.
    fn y(&self) -> f64;
    /// The measure, or [`NO_DATA`] for a vertex that carries none.
    fn m(&self) -> f64 {
        NO_DATA
    }
}

/// A shapefile vertex that also carries an elevation.
trait Elevated: Position {
    /// The elevation.
    fn z(&self) -> f64;
}

impl Position for shapefile::Point {
    fn x(&self) -> f64 {
        self.x
    }
    fn y(&self) -> f64 {
        self.y
    }
}

impl Position for shapefile::PointM {
    fn x(&self) -> f64 {
        self.x
    }
    fn y(&self) -> f64 {
        self.y
    }
    fn m(&self) -> f64 {
        self.m
    }
}

impl Position for shapefile::PointZ {
    fn x(&self) -> f64 {
        self.x
    }
    fn y(&self) -> f64 {
        self.y
    }
    fn m(&self) -> f64 {
        self.m
    }
}

impl Elevated for shapefile::PointZ {
    fn z(&self) -> f64 {
        self.z
    }
}

/// Converts the shapes of one shapefile into geometries, in the coordinate frame
/// its `.prj` names.
pub(super) struct ShapeConverter {
    /// Frame for geometries keeping their elevation.
    frame_3d: CoordinateFrame,
    /// Frame for geometries with no elevation: `frame_3d` with its vertical axis
    /// dropped.
    frame_2d: CoordinateFrame,
    /// Whether the frame declares its horizontal axes as `(northing, easting)`, so
    /// that shapefile's `(x, y)` must be swapped into the order the frame expects.
    swap: bool,
    /// Whether to drop elevations, converting every shape into a 2D geometry.
    force_2d: bool,
    /// Whether any converted shape carried a measure.
    discarded_measures: Cell<bool>,
}

impl ShapeConverter {
    /// Build a converter for a shapefile whose `.prj` resolved to `epsg`, or for
    /// one with no resolvable CRS when it is `None`.
    pub(super) fn new(epsg: Option<EpsgCode>, force_2d: bool) -> Self {
        let frame_3d = match epsg {
            Some(code) => CoordinateFrame::Crs(code),
            None => CoordinateFrame::Euclidean,
        };
        let frame_2d = match frame_3d.demote_to_2d() {
            Ok(frame) => frame,
            Err(error) => {
                tracing::warn!(
                    %error,
                    "keeping the shapefile's CRS on its 2D coordinates, which \
                     describes them with an axis they do not have"
                );
                frame_3d.clone()
            }
        };
        let swap = match frame_3d.orientation_sign() {
            Ok(sign) => sign < 0,
            Err(error) => {
                tracing::warn!(
                    %error,
                    "cannot establish the axis order of the shapefile's CRS; \
                     reading its coordinates as stored, which reverses them if the \
                     CRS declares (northing, easting)"
                );
                false
            }
        };
        Self {
            frame_3d,
            frame_2d,
            swap,
            force_2d,
            discarded_measures: Cell::new(false),
        }
    }

    /// Report the measures dropped over this converter's lifetime, if any.
    pub(super) fn report_discarded_measures(&self) {
        if self.discarded_measures.get() {
            tracing::warn!(
                "the shapefile carries measures (M values), which have no geometry \
                 counterpart and were discarded"
            );
        }
    }

    /// The geometry `shape` converts to.
    ///
    /// Errors on a shape whose kind has no counterpart, and on a `Multipatch` when
    /// elevations are being dropped.
    pub(super) fn convert(&self, shape: Shape) -> Result<Geometry, SourceError> {
        Ok(match shape {
            Shape::NullShape => Geometry::None,
            Shape::Point(p) => self.point(&p),
            Shape::PointM(p) => self.point(&p),
            Shape::PointZ(p) => self.point_z(&p),
            Shape::Polyline(l) => self.curve(l.parts()),
            Shape::PolylineM(l) => self.curve(l.parts()),
            Shape::PolylineZ(l) => self.curve_z(l.parts()),
            Shape::Polygon(p) => self.area(p.rings())?,
            Shape::PolygonM(p) => self.area(p.rings())?,
            Shape::PolygonZ(p) => self.area_z(p.rings())?,
            Shape::Multipoint(m) => self.multipoint(m.points()),
            Shape::MultipointM(m) => self.multipoint(m.points()),
            Shape::MultipointZ(m) => self.multipoint_z(m.points()),
            Shape::Multipatch(m) => self.multipatch(m.patches())?,
        })
    }

    /// A stored `[x, y]`, in the order the frame declares.
    fn xy(&self, p: &impl Position) -> [f64; 2] {
        self.note_measure(p);
        if self.swap {
            [p.y(), p.x()]
        } else {
            [p.x(), p.y()]
        }
    }

    /// A stored `[x, y, z]`, its horizontal pair in the order the frame declares.
    fn xyz(&self, p: &impl Elevated) -> [f64; 3] {
        let [x, y] = self.xy(p);
        [x, y, p.z()]
    }

    /// Remember that a measure was seen, if `p` carries one.
    fn note_measure(&self, p: &impl Position) {
        if p.m() > NO_DATA {
            self.discarded_measures.set(true);
        }
    }

    /// A point, without its elevation.
    fn point(&self, p: &impl Position) -> Geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
            self.frame_2d.clone(),
            self.xy(p),
        )))
    }

    /// A point, with its elevation unless elevations are being dropped.
    fn point_z(&self, p: &shapefile::PointZ) -> Geometry {
        if self.force_2d {
            return self.point(p);
        }
        Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
            self.frame_3d.clone(),
            self.xyz(p),
        )))
    }

    /// A polyline's parts: one `LineString` per part, collected when there is more
    /// than one.
    fn curve<P: Position>(&self, parts: &[Vec<P>]) -> Geometry {
        let lines = parts.iter().map(|part| {
            Euclidean2DGeometry::LineString(LineString2D::from_coords(
                self.frame_2d.clone(),
                part.iter().map(|p| self.xy(p)),
            ))
        });
        Geometry::Euclidean2D(one_or_collection_2d(lines))
    }

    /// [`Self::curve`], with elevations unless they are being dropped.
    fn curve_z(&self, parts: &[Vec<shapefile::PointZ>]) -> Geometry {
        if self.force_2d {
            return self.curve(parts);
        }
        let lines = parts.iter().map(|part| {
            Euclidean3DGeometry::LineString(LineString3D::from_coords(
                self.frame_3d.clone(),
                part.iter().map(|p| self.xyz(p)),
            ))
        });
        Geometry::Euclidean3D(one_or_collection_3d(lines))
    }

    /// A polygon's rings, grouped into faces: each outer ring starts a face and the
    /// inner rings around it are its holes. Every ring is reversed into the
    /// winding a geometry expects.
    fn area<P: Position>(&self, rings: &[PolygonRing<P>]) -> Result<Geometry, SourceError> {
        let faces = group_rings(rings)?;
        let polygons = faces.into_iter().map(|(exterior, holes)| {
            Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
                self.frame_2d.clone(),
                exterior.iter().rev().map(|p| self.xy(p)),
                holes
                    .into_iter()
                    .map(|hole| hole.iter().rev().map(|p| self.xy(p)).collect::<Vec<_>>()),
            )))
        });
        Ok(Geometry::Euclidean2D(one_or_collection_2d(polygons)))
    }

    /// [`Self::area`], with elevations unless they are being dropped.
    fn area_z(&self, rings: &[PolygonRing<shapefile::PointZ>]) -> Result<Geometry, SourceError> {
        if self.force_2d {
            return self.area(rings);
        }
        let faces = group_rings(rings)?;
        let polygons = faces.into_iter().map(|(exterior, holes)| {
            Euclidean3DGeometry::Polygon(Box::new(Polygon3D::from_rings(
                self.frame_3d.clone(),
                exterior.iter().rev().map(|p| self.xyz(p)),
                holes
                    .into_iter()
                    .map(|hole| hole.iter().rev().map(|p| self.xyz(p)).collect::<Vec<_>>()),
            )))
        });
        Ok(Geometry::Euclidean3D(one_or_collection_3d(polygons)))
    }

    /// A multipoint's points as a collection, without their elevations.
    fn multipoint(&self, points: &[impl Position]) -> Geometry {
        Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new(
            points.iter().map(|p| {
                Euclidean2DGeometry::Point(Point2D::new(self.frame_2d.clone(), self.xy(p)))
            }),
        )))
    }

    /// [`Self::multipoint`], with elevations unless they are being dropped.
    fn multipoint_z(&self, points: &[shapefile::PointZ]) -> Geometry {
        if self.force_2d {
            return self.multipoint(points);
        }
        Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new(
            points.iter().map(|p| {
                Euclidean3DGeometry::Point(Point3D::new(self.frame_3d.clone(), self.xyz(p)))
            }),
        )))
    }

    /// A multipatch's patches: the ring patches as one polygon mesh, each triangle
    /// strip or fan as its own triangle mesh, collected when there is more than one.
    /// Rings and triangles are reversed into the winding a geometry expects. A hole
    /// patch preceding any outer ring becomes a face of its own.
    ///
    /// Errors when elevations are being dropped, a multipatch describing a surface
    /// in space that a 2D geometry cannot stand in for.
    fn multipatch(&self, patches: &[Patch]) -> Result<Geometry, SourceError> {
        if self.force_2d {
            return Err(ShapefileError::MultipatchNotTwoDimensional.into());
        }

        let mut members: Vec<Euclidean3DGeometry> = Vec::new();
        let mut faces: Vec<Polygon3D> = Vec::new();
        let mut holes: Vec<Vec<[f64; 3]>> = Vec::new();
        let mut exterior: Option<Vec<[f64; 3]>> = None;

        let flush = |exterior: &mut Option<Vec<[f64; 3]>>,
                     holes: &mut Vec<Vec<[f64; 3]>>,
                     faces: &mut Vec<Polygon3D>| {
            if let Some(ring) = exterior.take() {
                faces.push(Polygon3D::from_rings(
                    self.frame_3d.clone(),
                    ring,
                    std::mem::take(holes),
                ));
            }
        };

        for patch in patches {
            match patch {
                Patch::OuterRing(ring) | Patch::FirstRing(ring) => {
                    flush(&mut exterior, &mut holes, &mut faces);
                    exterior = Some(ring.iter().rev().map(|p| self.xyz(p)).collect());
                }
                Patch::InnerRing(ring) | Patch::Ring(ring) => {
                    let ring: Vec<[f64; 3]> = ring.iter().rev().map(|p| self.xyz(p)).collect();
                    match &exterior {
                        Some(_) => holes.push(ring),
                        None => exterior = Some(ring),
                    }
                }
                Patch::TriangleStrip(points) | Patch::TriangleFan(points) => {
                    let vertices: Vec<[f64; 3]> = points.iter().map(|p| self.xyz(p)).collect();
                    let indices = match patch {
                        Patch::TriangleStrip(_) => strip_indices(vertices.len()),
                        _ => fan_indices(vertices.len()),
                    };
                    if indices.is_empty() {
                        continue;
                    }
                    let mesh =
                        TriangularMesh3D::from_parts(self.frame_3d.clone(), vertices, indices)
                            .map_err(|e| {
                                SourceError::shapefile_reader(format!(
                                    "Failed to build a triangle mesh from a multipatch patch: {e}"
                                ))
                            })?;
                    members.push(Euclidean3DGeometry::TriangularMesh(Box::new(mesh)));
                }
            }
        }
        flush(&mut exterior, &mut holes, &mut faces);

        if !faces.is_empty() {
            let mesh =
                PolygonMesh3D::from_polygons(self.frame_3d.clone(), faces.iter()).map_err(|e| {
                    SourceError::shapefile_reader(format!(
                        "Failed to build a polygon mesh from a multipatch: {e}"
                    ))
                })?;
            members.push(Euclidean3DGeometry::PolygonMesh(Box::new(mesh)));
        }

        if members.is_empty() {
            return Err(ShapefileError::MultipatchNoPatches.into());
        }
        Ok(Geometry::Euclidean3D(one_or_collection_3d(
            members.into_iter(),
        )))
    }
}

/// The rings of a polygon, grouped into `(exterior, holes)` faces in file order.
/// A hole follows its outer ring, except that holes written before any outer ring
/// belong to the first one.
///
/// Errors on a polygon with no ring at all, and on one whose rings are all holes.
#[allow(clippy::type_complexity)]
fn group_rings<P: Position>(
    rings: &[PolygonRing<P>],
) -> Result<Vec<(&[P], Vec<&[P]>)>, SourceError> {
    if rings.is_empty() {
        return Err(ShapefileError::PolygonNoRings.into());
    }

    let mut faces: Vec<(&[P], Vec<&[P]>)> = Vec::new();
    let mut leading_holes: Vec<&[P]> = Vec::new();
    for ring in rings {
        match ring {
            PolygonRing::Outer(points) => {
                faces.push((points.as_slice(), std::mem::take(&mut leading_holes)))
            }
            PolygonRing::Inner(points) => match faces.last_mut() {
                Some((_, holes)) => holes.push(points.as_slice()),
                None => leading_holes.push(points.as_slice()),
            },
        }
    }

    if faces.is_empty() {
        return Err(ShapefileError::PolygonNoOuterRings.into());
    }
    Ok(faces)
}

/// The triangle index list of a strip of `n` vertices, every vertex after the first
/// two completing a triangle with its two predecessors. Every other triangle is
/// wound back so the strip keeps one orientation, and every triangle is wound
/// opposite to the strip's own, which shows its front to a clockwise viewer.
fn strip_indices(n: usize) -> Vec<u32> {
    (0..n.saturating_sub(2))
        .flat_map(|i| {
            let (a, b, c) = (i as u32, i as u32 + 1, i as u32 + 2);
            if i % 2 == 0 {
                [a, c, b]
            } else {
                [a, b, c]
            }
        })
        .collect()
}

/// The triangle index list of a fan of `n` vertices, every vertex after the first
/// two completing a triangle with its predecessor and the first vertex, wound
/// opposite to the fan's own, which shows its front to a clockwise viewer.
fn fan_indices(n: usize) -> Vec<u32> {
    (1..n.saturating_sub(1))
        .flat_map(|i| [0, i as u32 + 1, i as u32])
        .collect()
}

/// One geometry when `members` holds exactly one, a collection of them otherwise.
fn one_or_collection_2d(members: impl Iterator<Item = Euclidean2DGeometry>) -> Euclidean2DGeometry {
    let mut members: Vec<_> = members.collect();
    if members.len() == 1 {
        return members.remove(0);
    }
    Euclidean2DGeometry::Collection(Collection2D::new(members))
}

/// One geometry when `members` holds exactly one, a collection of them otherwise.
fn one_or_collection_3d(members: impl Iterator<Item = Euclidean3DGeometry>) -> Euclidean3DGeometry {
    let mut members: Vec<_> = members.collect();
    if members.len() == 1 {
        return members.remove(0);
    }
    Euclidean3DGeometry::Collection(Collection3D::new(members))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn converter() -> ShapeConverter {
        ShapeConverter::new(None, false)
    }

    #[test]
    fn a_null_shape_is_an_absent_geometry() {
        assert_eq!(
            converter().convert(Shape::NullShape).unwrap(),
            Geometry::None
        );
    }

    #[test]
    fn a_measured_point_keeps_its_position_and_drops_its_measure() {
        let converter = converter();
        let geometry = converter
            .convert(Shape::PointM(shapefile::PointM::new(1.0, 2.0, 7.5)))
            .unwrap();
        let Geometry::Euclidean2D(Euclidean2DGeometry::Point(point)) = geometry else {
            panic!("expected a 2D point, got {geometry:?}");
        };
        assert_eq!(point.position(), [1.0, 2.0]);
        assert!(converter.discarded_measures.get());
    }

    #[test]
    fn a_northing_first_crs_swaps_the_horizontal_pair() {
        let converter = ShapeConverter::new(Some(EpsgCode::new(6668)), false);
        let geometry = converter
            .convert(Shape::Point(shapefile::Point::new(139.0, 35.0)))
            .unwrap();
        let Geometry::Euclidean2D(Euclidean2DGeometry::Point(point)) = geometry else {
            panic!("expected a 2D point, got {geometry:?}");
        };
        assert_eq!(point.position(), [35.0, 139.0]);
    }

    #[test]
    fn a_single_part_polyline_is_one_line_string() {
        let line = shapefile::Polyline::new(vec![
            shapefile::Point::new(0.0, 0.0),
            shapefile::Point::new(1.0, 1.0),
        ]);
        let geometry = converter().convert(Shape::Polyline(line)).unwrap();
        assert!(matches!(
            geometry,
            Geometry::Euclidean2D(Euclidean2DGeometry::LineString(_))
        ));
    }

    #[test]
    fn a_multi_part_polyline_collects_its_parts() {
        let line = shapefile::Polyline::with_parts(vec![
            vec![
                shapefile::Point::new(0.0, 0.0),
                shapefile::Point::new(1.0, 1.0),
            ],
            vec![
                shapefile::Point::new(2.0, 2.0),
                shapefile::Point::new(3.0, 3.0),
            ],
        ]);
        let geometry = converter().convert(Shape::Polyline(line)).unwrap();
        let Geometry::Euclidean2D(Euclidean2DGeometry::Collection(collection)) = geometry else {
            panic!("expected a 2D collection, got {geometry:?}");
        };
        assert_eq!(collection.members().len(), 2);
    }

    #[test]
    fn a_polygon_hole_lands_on_the_ring_before_it() {
        let outer = vec![
            shapefile::Point::new(0.0, 0.0),
            shapefile::Point::new(0.0, 4.0),
            shapefile::Point::new(4.0, 4.0),
            shapefile::Point::new(4.0, 0.0),
            shapefile::Point::new(0.0, 0.0),
        ];
        let inner = vec![
            shapefile::Point::new(1.0, 1.0),
            shapefile::Point::new(2.0, 1.0),
            shapefile::Point::new(2.0, 2.0),
            shapefile::Point::new(1.0, 1.0),
        ];
        let polygon = shapefile::Polygon::with_rings(vec![
            PolygonRing::Outer(outer),
            PolygonRing::Inner(inner),
        ]);
        let geometry = converter().convert(Shape::Polygon(polygon)).unwrap();
        let Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(polygon)) = geometry else {
            panic!("expected a 2D polygon, got {geometry:?}");
        };
        assert_eq!(polygon.interiors().count(), 1);
    }

    #[test]
    fn forcing_two_dimensions_drops_the_elevation() {
        let converter = ShapeConverter::new(None, true);
        let geometry = converter
            .convert(Shape::PointZ(shapefile::PointZ::new(
                1.0, 2.0, 3.0, NO_DATA,
            )))
            .unwrap();
        let Geometry::Euclidean2D(Euclidean2DGeometry::Point(point)) = geometry else {
            panic!("expected a 2D point, got {geometry:?}");
        };
        assert_eq!(point.position(), [1.0, 2.0]);
    }

    #[test]
    fn ring_patches_build_one_mesh_with_their_holes() {
        let outer = vec![
            shapefile::PointZ::new(0.0, 0.0, 0.0, NO_DATA),
            shapefile::PointZ::new(0.0, 4.0, 0.0, NO_DATA),
            shapefile::PointZ::new(4.0, 4.0, 0.0, NO_DATA),
            shapefile::PointZ::new(0.0, 0.0, 0.0, NO_DATA),
        ];
        let inner = vec![
            shapefile::PointZ::new(1.0, 1.0, 0.0, NO_DATA),
            shapefile::PointZ::new(2.0, 1.0, 0.0, NO_DATA),
            shapefile::PointZ::new(2.0, 2.0, 0.0, NO_DATA),
            shapefile::PointZ::new(1.0, 1.0, 0.0, NO_DATA),
        ];
        let patch = shapefile::Multipatch::with_parts(vec![
            Patch::OuterRing(outer),
            Patch::InnerRing(inner),
        ]);
        let geometry = converter().convert(Shape::Multipatch(patch)).unwrap();
        let Geometry::Euclidean3D(Euclidean3DGeometry::PolygonMesh(mesh)) = geometry else {
            panic!("expected a 3D polygon mesh, got {geometry:?}");
        };
        assert_eq!(mesh.num_faces(), 1);
    }

    #[test]
    fn a_hole_before_the_first_outer_ring_lands_on_it() {
        let outer = vec![
            shapefile::Point::new(0.0, 0.0),
            shapefile::Point::new(0.0, 4.0),
            shapefile::Point::new(4.0, 4.0),
            shapefile::Point::new(4.0, 0.0),
            shapefile::Point::new(0.0, 0.0),
        ];
        let inner = vec![
            shapefile::Point::new(1.0, 1.0),
            shapefile::Point::new(2.0, 1.0),
            shapefile::Point::new(2.0, 2.0),
            shapefile::Point::new(1.0, 1.0),
        ];
        let polygon = shapefile::Polygon::with_rings(vec![
            PolygonRing::Inner(inner),
            PolygonRing::Outer(outer),
        ]);
        let geometry = converter().convert(Shape::Polygon(polygon)).unwrap();
        let Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(polygon)) = geometry else {
            panic!("expected a 2D polygon, got {geometry:?}");
        };
        assert_eq!(polygon.interiors().count(), 1);
    }

    #[test]
    fn rings_are_reversed_into_the_geometry_winding() {
        let outer = vec![
            shapefile::Point::new(0.0, 0.0),
            shapefile::Point::new(0.0, 4.0),
            shapefile::Point::new(4.0, 4.0),
            shapefile::Point::new(4.0, 0.0),
            shapefile::Point::new(0.0, 0.0),
        ];
        let inner = vec![
            shapefile::Point::new(1.0, 1.0),
            shapefile::Point::new(2.0, 1.0),
            shapefile::Point::new(2.0, 2.0),
            shapefile::Point::new(1.0, 1.0),
        ];
        let polygon = shapefile::Polygon::with_rings(vec![
            PolygonRing::Outer(outer),
            PolygonRing::Inner(inner),
        ]);
        let geometry = converter().convert(Shape::Polygon(polygon)).unwrap();
        let Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(polygon)) = geometry else {
            panic!("expected a 2D polygon, got {geometry:?}");
        };
        assert!(signed_area(polygon.exterior()) > 0.0);
        assert!(signed_area(polygon.interiors().next().unwrap()) < 0.0);
    }

    /// Twice the shoelace area: positive for a counter-clockwise ring.
    fn signed_area(ring: &[[f64; 2]]) -> f64 {
        ring.windows(2)
            .map(|w| w[0][0] * w[1][1] - w[1][0] * w[0][1])
            .sum()
    }

    #[test]
    fn a_strip_winds_every_other_triangle_back() {
        assert_eq!(strip_indices(4), vec![0, 2, 1, 1, 2, 3]);
    }

    #[test]
    fn a_fan_shares_its_first_vertex() {
        assert_eq!(fan_indices(4), vec![0, 2, 1, 0, 3, 2]);
    }

    #[test]
    fn a_multipatch_cannot_be_forced_to_two_dimensions() {
        let converter = ShapeConverter::new(None, true);
        let patch = shapefile::Multipatch::new(Patch::TriangleFan(vec![
            shapefile::PointZ::new(0.0, 0.0, 0.0, NO_DATA),
            shapefile::PointZ::new(1.0, 0.0, 0.0, NO_DATA),
            shapefile::PointZ::new(0.0, 1.0, 0.0, NO_DATA),
        ]));
        assert!(converter.convert(Shape::Multipatch(patch)).is_err());
    }
}
