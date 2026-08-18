//! Conversion of `shapefile` shapes into `reearth_flow_geometry::Geometry`.
//!
//! Every ring and triangle is reversed on the way in: a shapefile winds outer
//! rings clockwise and multipatch fronts clockwise, a geometry the other way
//! round. Measures (M values) are discarded and reported once per read.

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

/// A shapefile vertex's horizontal position.
trait Position {
    /// The stored `x` (easting).
    fn x(&self) -> f64;
    /// The stored `y` (northing).
    fn y(&self) -> f64;
    /// The measure, or [`NO_DATA`].
    fn m(&self) -> f64 {
        NO_DATA
    }
}

/// A shapefile vertex carrying an elevation.
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

/// Converts the shapes of one shapefile into geometries in the frame `epsg`
/// names.
pub(super) struct ShapeConverter {
    /// Frame for 3D geometries.
    frame_3d: CoordinateFrame,
    /// Frame for 2D geometries: `frame_3d` without its vertical axis.
    frame_2d: CoordinateFrame,
    /// Whether the frame declares `(northing, easting)`, so `(x, y)` is swapped.
    swap: bool,
    /// Whether to drop elevations.
    force_2d: bool,
    /// Whether any converted shape carried a measure.
    discarded_measures: Cell<bool>,
}

impl ShapeConverter {
    /// A converter for a shapefile in `epsg`, or in no CRS when `None`.
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

    /// Warn once if any measure was discarded.
    pub(super) fn report_discarded_measures(&self) {
        if self.discarded_measures.get() {
            tracing::warn!(
                "the shapefile carries measures (M values), which have no geometry \
                 counterpart and were discarded"
            );
        }
    }

    /// The geometry `shape` converts to. Errors on a `Multipatch` when elevations
    /// are being dropped, and on a polygon with no outer ring.
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

    /// `[x, y]` in the frame's axis order.
    fn xy(&self, p: &impl Position) -> [f64; 2] {
        self.note_measure(p);
        if self.swap {
            [p.y(), p.x()]
        } else {
            [p.x(), p.y()]
        }
    }

    /// `[x, y, z]` in the frame's axis order.
    fn xyz(&self, p: &impl Elevated) -> [f64; 3] {
        let [x, y] = self.xy(p);
        [x, y, p.z()]
    }

    /// Note whether `p` carries a measure.
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

    /// A polyline: one line string per part, collected when there are several.
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

    /// A polygon: one face per outer ring with the holes following it, each ring
    /// reversed.
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

    /// A multipoint as a collection of points, without elevations.
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

    /// A multipatch: its ring patches as one polygon mesh, its strips and fans as
    /// one triangular mesh, every face reversed. Errors when elevations are being
    /// dropped.
    fn multipatch(&self, patches: &[Patch]) -> Result<Geometry, SourceError> {
        if self.force_2d {
            return Err(ShapefileError::MultipatchNotTwoDimensional.into());
        }

        let mut faces: Vec<Polygon3D> = Vec::new();
        let mut triangle_vertices: Vec<[f64; 3]> = Vec::new();
        let mut triangle_indices: Vec<u32> = Vec::new();
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
                    let indices = match patch {
                        Patch::TriangleStrip(_) => strip_indices(points.len()),
                        _ => fan_indices(points.len()),
                    };
                    let offset = triangle_vertices.len() as u32;
                    triangle_indices.extend(indices.into_iter().map(|i| i + offset));
                    triangle_vertices.extend(points.iter().map(|p| self.xyz(p)));
                }
            }
        }
        flush(&mut exterior, &mut holes, &mut faces);

        let mut members: Vec<Euclidean3DGeometry> = Vec::new();
        if !faces.is_empty() {
            let mesh =
                PolygonMesh3D::from_polygons(self.frame_3d.clone(), faces.iter()).map_err(|e| {
                    SourceError::shapefile_reader(format!(
                        "Failed to build a polygon mesh from a multipatch: {e}"
                    ))
                })?;
            members.push(Euclidean3DGeometry::PolygonMesh(Box::new(mesh)));
        }
        if !triangle_indices.is_empty() {
            let mesh = TriangularMesh3D::from_parts(
                self.frame_3d.clone(),
                triangle_vertices,
                triangle_indices,
            )
            .map_err(|e| {
                SourceError::shapefile_reader(format!(
                    "Failed to build a triangular mesh from a multipatch: {e}"
                ))
            })?;
            members.push(Euclidean3DGeometry::TriangularMesh(Box::new(mesh)));
        }

        if members.is_empty() {
            return Err(ShapefileError::MultipatchNoPatches.into());
        }
        Ok(Geometry::Euclidean3D(one_or_collection_3d(
            members.into_iter(),
        )))
    }
}

/// The rings grouped into `(exterior, holes)` faces in file order; holes before
/// the first outer ring belong to it. Errors when there is no outer ring.
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

/// The triangle indices of a strip of `n` vertices, each triangle wound opposite
/// to the strip's own.
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

/// The triangle indices of a fan of `n` vertices, each triangle wound opposite to
/// the fan's own.
fn fan_indices(n: usize) -> Vec<u32> {
    (1..n.saturating_sub(1))
        .flat_map(|i| [0, i as u32 + 1, i as u32])
        .collect()
}

/// The one member alone, or a collection of several.
fn one_or_collection_2d(members: impl Iterator<Item = Euclidean2DGeometry>) -> Euclidean2DGeometry {
    let mut members: Vec<_> = members.collect();
    if members.len() == 1 {
        return members.remove(0);
    }
    Euclidean2DGeometry::Collection(Collection2D::new(members))
}

/// The one member alone, or a collection of several.
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
    fn a_polyline_is_one_line_string_or_a_collection_of_its_parts() {
        let part = |x: f64| vec![shapefile::Point::new(x, 0.0), shapefile::Point::new(x, 1.0)];
        let one = shapefile::Polyline::new(part(0.0));
        assert!(matches!(
            converter().convert(Shape::Polyline(one)).unwrap(),
            Geometry::Euclidean2D(Euclidean2DGeometry::LineString(_))
        ));
        let two = shapefile::Polyline::with_parts(vec![part(0.0), part(2.0)]);
        let geometry = converter().convert(Shape::Polyline(two)).unwrap();
        let Geometry::Euclidean2D(Euclidean2DGeometry::Collection(collection)) = geometry else {
            panic!("expected a 2D collection, got {geometry:?}");
        };
        assert_eq!(collection.members().len(), 2);
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
    fn strips_and_fans_build_one_triangular_mesh() {
        let quad = |x: f64| {
            vec![
                shapefile::PointZ::new(x, 0.0, 0.0, NO_DATA),
                shapefile::PointZ::new(x, 1.0, 0.0, NO_DATA),
                shapefile::PointZ::new(x + 1.0, 0.0, 0.0, NO_DATA),
                shapefile::PointZ::new(x + 1.0, 1.0, 0.0, NO_DATA),
            ]
        };
        let patch = shapefile::Multipatch::with_parts(vec![
            Patch::TriangleStrip(quad(0.0)),
            Patch::TriangleFan(quad(5.0)),
        ]);
        let geometry = converter().convert(Shape::Multipatch(patch)).unwrap();
        let Geometry::Euclidean3D(Euclidean3DGeometry::TriangularMesh(mesh)) = geometry else {
            panic!("expected a 3D triangular mesh, got {geometry:?}");
        };
        assert_eq!(mesh.num_triangles(), 4);
        assert_eq!(mesh.vertices().len(), 8);
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
}
