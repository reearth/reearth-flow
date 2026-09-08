//! Interior-ring (hole) operations.
//!
//! A face's holes are its interior rings. [`CountHoles`] reports how many rings a
//! geometry carries; [`ExtractHoles`] takes them apart, handing back each ring as
//! an area of its own alongside the outer boundary it was cut from.
//!
//! Both share one notion of a hole, so a geometry counted as having `n` holes
//! extracts to `n` [`ExtractedPart::Hole`] parts. In particular a
//! [`Solid`](crate::solid::Solid)'s void shells are not holes in either — a void
//! is a hollow volume rather than the boundary of a face — while the holes in the
//! faces of those shells are.

use crate::coordinate::CoordinateFrame;
use crate::ops::UnsupportedOperation;
use crate::polygon::{Polygon2D, Polygon3D};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};

/// Count the interior (hole) rings a geometry's faces carry.
///
/// Total over the hierarchy: a type that cannot carry a hole inherits the `0`
/// default, so counting never fails and a caller cannot tell "has no holes" from
/// "cannot have holes". The unit counted is the ring, not the face: a face with
/// two holes contributes two, matching the inner boundaries a donut reports.
///
/// A [`Solid`](crate::solid::Solid)'s void shells are not holes in this sense —
/// they are hollow volumes, not boundaries of a face — so they are not counted;
/// the holes in the faces of those shells are.
#[enum_dispatch::enum_dispatch]
pub trait CountHoles {
    /// The number of interior rings across this geometry, recursing into
    /// collections.
    fn count_holes(&self) -> usize {
        0
    }
}

impl<T: CountHoles + ?Sized> CountHoles for Box<T> {
    fn count_holes(&self) -> usize {
        (**self).count_holes()
    }
}

/// Which boundary of a face an extracted part came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExtractedPart {
    /// A face's outer boundary, with its holes removed. A face that had none is
    /// still reported here, unchanged.
    Outershell,
    /// One interior ring, as an area of its own.
    Hole,
    /// A container member that is not area geometry, handed back untouched
    /// instead of failing the whole container. Never produced by a leaf.
    Rejected,
}

/// Take every face of a geometry apart into its boundaries: the outer boundary
/// with the holes removed, and each interior ring as an area of its own.
///
/// `emit` is invoked once per part, in `Outershell`-then-holes order per face,
/// and faces stream rather than being collected. Every emitted area is a bare
/// [`Polygon`](crate::polygon) in the source's frame and embedding:
///
/// * Rings are copied **verbatim** — neither re-wound nor closed. An interior
///   ring keeps the winding it had inside its face, since whether that winding is
///   correct is itself something a caller may want to inspect.
/// * A 2D face's elevation carries onto every part it produces.
/// * Appearance is dropped: a part has fewer corners than the face it came from,
///   so the face's per-corner UV no longer applies.
///
/// A leaf that is not area geometry (a point, a curve, a point cloud, an
/// unevaluated CSG tree) returns [`UnsupportedOperation`] via the default body
/// and emits nothing. A container deaggregates instead: it recurses into its
/// members, and a member that is not area geometry is emitted as
/// [`ExtractedPart::Rejected`] rather than failing the container, so one curve
/// among the surfaces does not discard the surfaces.
#[enum_dispatch::enum_dispatch]
pub trait ExtractHoles {
    /// Emit this geometry's boundaries. The default body reports the type as
    /// unsupported, emitting nothing.
    fn extract_holes(
        &self,
        emit: &mut dyn FnMut(Geometry, ExtractedPart),
    ) -> Result<(), UnsupportedOperation> {
        let _ = emit;
        Err(UnsupportedOperation {
            geometry: core::any::type_name::<Self>(),
            operation: "extract_holes",
        })
    }
}

// The boxed enum variants (`Box<Polygon2D>`, `Box<Solid>`, …) need the trait on
// the `Box` itself: `enum_dispatch` forwards by UFCS, not auto-deref.
impl<T: ExtractHoles + ?Sized> ExtractHoles for Box<T> {
    fn extract_holes(
        &self,
        emit: &mut dyn FnMut(Geometry, ExtractedPart),
    ) -> Result<(), UnsupportedOperation> {
        (**self).extract_holes(emit)
    }
}

/// One ring as a bare, hole-less 2D area at `elevation`.
pub(crate) fn area_2d(
    frame: &CoordinateFrame,
    ring: impl IntoIterator<Item = [f64; 2]>,
    elevation: Option<f64>,
) -> Geometry {
    let no_holes = Vec::<Vec<[f64; 2]>>::new();
    let polygon = match elevation {
        None => Polygon2D::from_rings(frame.clone(), ring, no_holes),
        Some(elevation) => {
            Polygon2D::from_rings_at_elevation(frame.clone(), ring, no_holes, elevation)
        }
    };
    Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(polygon)))
}

/// One ring as a bare, hole-less 3D area.
pub(crate) fn area_3d(
    frame: &CoordinateFrame,
    ring: impl IntoIterator<Item = [f64; 3]>,
) -> Geometry {
    let polygon = Polygon3D::from_rings(frame.clone(), ring, Vec::<Vec<[f64; 3]>>::new());
    Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(polygon)))
}

/// Emit one 2D face's boundaries, returning whether it yielded anything. A face
/// with no exterior ring carries no area and yields nothing.
pub(crate) fn emit_face_2d(
    face: &Polygon2D,
    emit: &mut dyn FnMut(Geometry, ExtractedPart),
) -> bool {
    let exterior = face.exterior();
    if exterior.is_empty() {
        return false;
    }
    let frame = face.frame();
    let elevation = face.elevation();
    emit(
        area_2d(frame, exterior.iter().copied(), elevation),
        ExtractedPart::Outershell,
    );
    for hole in face.interiors() {
        emit(
            area_2d(frame, hole.iter().copied(), elevation),
            ExtractedPart::Hole,
        );
    }
    true
}

/// Emit one 3D face's boundaries, returning whether it yielded anything.
pub(crate) fn emit_face_3d(
    face: &Polygon3D,
    emit: &mut dyn FnMut(Geometry, ExtractedPart),
) -> bool {
    let exterior = face.exterior();
    if exterior.is_empty() {
        return false;
    }
    let frame = face.frame();
    emit(
        area_3d(frame, exterior.iter().copied()),
        ExtractedPart::Outershell,
    );
    for hole in face.interiors() {
        emit(area_3d(frame, hole.iter().copied()), ExtractedPart::Hole);
    }
    true
}

/// Emit one triangle-mesh shell's triangles as outer shells; a triangle carries
/// no interior ring, so none is a hole. The ring is closed, as when splitting a
/// triangle mesh into faces.
pub(crate) fn emit_triangles_3d(
    frame: &CoordinateFrame,
    vertices: &[[f64; 3]],
    triangles: impl Iterator<Item = [u32; 3]>,
    emit: &mut dyn FnMut(Geometry, ExtractedPart),
) {
    for [i, j, k] in triangles {
        let ring = [
            vertices[i as usize],
            vertices[j as usize],
            vertices[k as usize],
            vertices[i as usize],
        ];
        emit(area_3d(frame, ring), ExtractedPart::Outershell);
    }
}

/// Fixtures shared by the two operations, so counting and extraction see the
/// same geometry and cannot drift apart in what they call a hole.
#[cfg(test)]
mod fixtures {
    use super::*;
    use crate::csg::Csg;
    use crate::line_string::LineString3D;
    use crate::point::Point3D;
    use crate::point_cloud::PointCloud;
    use crate::polygon_mesh::PolygonMesh3DData;
    use crate::solid::Solid;
    use crate::triangular_mesh::TriangularMesh3DData;

    /// A closed 4x4 square, as an exterior ring.
    pub(super) const SQUARE: [[f64; 3]; 5] = [
        [0.0, 0.0, 0.0],
        [4.0, 0.0, 0.0],
        [4.0, 4.0, 0.0],
        [0.0, 4.0, 0.0],
        [0.0, 0.0, 0.0],
    ];

    /// [`SQUARE`] without its z, for the 2D faces.
    pub(super) const SQUARE_2D: [[f64; 2]; 5] =
        [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];

    /// A closed square hole ring of side 1, with its lower-left corner at `(x, y)`.
    /// Wound the opposite way from [`SQUARE`], as a hole should be.
    pub(super) fn hole_ring(x: f64, y: f64) -> Vec<[f64; 3]> {
        vec![
            [x, y, 0.0],
            [x, y + 1.0, 0.0],
            [x + 1.0, y + 1.0, 0.0],
            [x + 1.0, y, 0.0],
            [x, y, 0.0],
        ]
    }

    /// [`hole_ring`] without its z.
    pub(super) fn hole_ring_2d(x: f64, y: f64) -> Vec<[f64; 2]> {
        vec![
            [x, y],
            [x, y + 1.0],
            [x + 1.0, y + 1.0],
            [x + 1.0, y],
            [x, y],
        ]
    }

    /// A square face carrying `n` holes.
    pub(super) fn face_with_holes(n: usize) -> Polygon3D {
        let holes: Vec<_> = (0..n)
            .map(|i| hole_ring(1.0 + i as f64 * 1.5, 1.0))
            .collect();
        Polygon3D::from_rings(CoordinateFrame::Euclidean, SQUARE, holes)
    }

    /// A square 2D face at `elevation`, carrying `n` holes.
    pub(super) fn face_2d_with_holes(n: usize, elevation: f64) -> Polygon2D {
        let holes: Vec<_> = (0..n)
            .map(|i| hole_ring_2d(1.0 + i as f64 * 1.5, 1.0))
            .collect();
        Polygon2D::from_rings_at_elevation(CoordinateFrame::Euclidean, SQUARE_2D, holes, elevation)
    }

    /// A one-quad shell mesh whose single face carries `n` holes.
    pub(super) fn shell_with_holes(n: usize) -> PolygonMesh3DData {
        PolygonMesh3DData::from_polygons([&face_with_holes(n)])
    }

    /// A shell bounded by triangles: a unit square tiled by two of them. A
    /// triangle carries no interior ring, so this shell can hold no hole.
    pub(super) fn triangle_shell() -> TriangularMesh3DData {
        TriangularMesh3DData::from_parts(
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            [0u32, 1, 2, 1, 3, 2],
        )
        .unwrap()
    }

    /// The 3D leaves that bound no area: a point, a curve, a point cloud, and an
    /// unevaluated CSG tree. The tree's operands are holed solids, whose holes
    /// must not surface through a tree that has no faces of its own.
    pub(super) fn non_area_3d_leaves() -> Vec<Euclidean3DGeometry> {
        let cloud = PointCloud::from_positions(CoordinateFrame::Euclidean, [[0.0, 0.0, 0.0]]);
        let csg = Csg::union(
            Solid::from_exterior(CoordinateFrame::Euclidean, shell_with_holes(2)),
            Solid::from_exterior(CoordinateFrame::Euclidean, shell_with_holes(1)),
        );
        vec![
            Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::Euclidean, [1.0, 2.0, 3.0])),
            Euclidean3DGeometry::LineString(LineString3D::from_coords(
                CoordinateFrame::Euclidean,
                [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]],
            )),
            Euclidean3DGeometry::PointCloud(Box::new(cloud)),
            Euclidean3DGeometry::Csg(csg),
        ]
    }

    /// A 3D leaf as a whole geometry.
    pub(super) fn geometry_3d(g: Euclidean3DGeometry) -> Geometry {
        Geometry::Euclidean3D(g)
    }
}

#[cfg(test)]
mod count_holes_tests {
    use super::fixtures::*;
    use super::*;
    use crate::collection::Collection3D;
    use crate::point::Point3D;
    use crate::polygon_mesh::{PolygonMesh2D, PolygonMesh3D};
    use crate::solid::{Shell, Solid};
    use crate::triangular_mesh::TriangularMesh3D;
    use crate::GeometryCollection;

    #[test]
    fn a_polygon_counts_its_interior_rings() {
        assert_eq!(face_with_holes(0).count_holes(), 0);
        assert_eq!(face_with_holes(2).count_holes(), 2);
    }

    #[test]
    fn a_2d_polygon_counts_its_interior_rings() {
        assert_eq!(face_2d_with_holes(1, 10.0).count_holes(), 1);
    }

    #[test]
    fn a_mesh_counts_the_holes_of_every_face() {
        // Two faces, one with two holes and one with none: the shared
        // `interior_offsets` holds one entry per interior ring, so the mesh-wide
        // count needs no per-face walk and cannot count a ring twice.
        let mesh = PolygonMesh3D::from_polygons(
            CoordinateFrame::Euclidean,
            [&face_with_holes(2), &face_with_holes(0)],
        )
        .unwrap();
        assert_eq!(mesh.num_faces(), 2);
        assert_eq!(mesh.count_holes(), 2);
    }

    #[test]
    fn a_mesh_without_holes_counts_zero() {
        let mesh = PolygonMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [3.0, 0.0], [3.0, 2.0]],
            vec![vec![0u32, 1, 2]],
        )
        .unwrap();
        assert_eq!(mesh.count_holes(), 0);
    }

    #[test]
    fn a_2d_mesh_counts_the_holes_of_every_face() {
        // One quad face (corners 0..4) with one hole ring (corners 4..8).
        let mesh = PolygonMesh2D::from_raw_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0],
                [4.0, 0.0],
                [4.0, 4.0],
                [0.0, 4.0],
                [1.0, 1.0],
                [2.0, 1.0],
                [2.0, 2.0],
                [1.0, 2.0],
            ],
            vec![0, 1, 2, 3, 4, 5, 6, 7],
            Vec::new(),
            vec![4],
        )
        .unwrap();
        assert_eq!(mesh.count_holes(), 1);
    }

    #[test]
    fn a_solid_counts_the_holes_in_the_faces_of_every_shell() {
        // Exterior face has two holes, the void shell's face has one: three in
        // all. The void shell itself is not a hole and adds nothing.
        let solid = Solid::new(
            CoordinateFrame::Euclidean,
            shell_with_holes(2),
            vec![Shell::from(shell_with_holes(1))],
        );
        assert_eq!(solid.interiors().len(), 1);
        assert_eq!(solid.count_holes(), 3);
    }

    #[test]
    fn a_solid_bounded_by_triangles_counts_zero() {
        let solid = Solid::from_exterior(CoordinateFrame::Euclidean, triangle_shell());
        assert_eq!(solid.count_holes(), 0);
    }

    #[test]
    fn a_type_that_cannot_carry_a_hole_counts_zero() {
        // A triangle mesh does bound area, unlike the rest, but a triangle has no
        // interior ring to count.
        let tris = TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            [0u32, 1, 2],
        )
        .unwrap();
        let mut cannot = non_area_3d_leaves();
        cannot.push(Euclidean3DGeometry::TriangularMesh(Box::new(tris)));

        for g in cannot {
            assert_eq!(geometry_3d(g).count_holes(), 0);
        }
    }

    #[test]
    fn an_absent_geometry_counts_zero() {
        assert_eq!(Geometry::None.count_holes(), 0);
    }

    #[test]
    fn a_collection_sums_its_members() {
        let c = Collection3D::new([
            Euclidean3DGeometry::Polygon(Box::new(face_with_holes(2))),
            Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::Euclidean, [0.0, 0.0, 0.0])),
        ]);
        assert_eq!(
            geometry_3d(Euclidean3DGeometry::Collection(c)).count_holes(),
            2
        );
    }

    #[test]
    fn counting_reaches_through_nested_collections() {
        let inner = Geometry::GeometryCollection(GeometryCollection::new([geometry_3d(
            Euclidean3DGeometry::Polygon(Box::new(face_with_holes(1))),
        )]));
        let outer = Geometry::GeometryCollection(GeometryCollection::new([
            inner,
            geometry_3d(Euclidean3DGeometry::Polygon(Box::new(face_with_holes(3)))),
            Geometry::None,
        ]));
        assert_eq!(outer.count_holes(), 4);
    }
}

#[cfg(test)]
mod extract_holes_tests {
    use super::fixtures::*;
    use super::*;
    use crate::collection::Collection3D;
    use crate::point::Point3D;
    use crate::polygon_mesh::{PolygonMesh2D, PolygonMesh3D};
    use crate::solid::{Shell, Solid};
    use crate::test_support::{bare, theme};
    use crate::triangular_mesh::{TriangularMesh2D, TriangularMesh3D};
    use crate::GeometryCollection;

    /// The parts a geometry extracts to, in emission order.
    fn parts(
        geometry: &impl ExtractHoles,
    ) -> Result<Vec<(Geometry, ExtractedPart)>, UnsupportedOperation> {
        let mut out = Vec::new();
        geometry.extract_holes(&mut |geometry, part| out.push((geometry, part)))?;
        Ok(out)
    }

    fn roles(parts: &[(Geometry, ExtractedPart)]) -> Vec<ExtractedPart> {
        parts.iter().map(|(_, part)| *part).collect()
    }

    fn count(parts: &[(Geometry, ExtractedPart)], role: ExtractedPart) -> usize {
        parts.iter().filter(|(_, part)| *part == role).count()
    }

    fn face_3d(geometry: &Geometry) -> &Polygon3D {
        match geometry {
            Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(p)) => p,
            other => panic!("expected a 3D polygon, got {other:?}"),
        }
    }

    fn face_2d(geometry: &Geometry) -> &Polygon2D {
        match geometry {
            Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(p)) => p,
            other => panic!("expected a 2D polygon, got {other:?}"),
        }
    }

    #[test]
    fn a_face_yields_its_outer_boundary_then_each_hole() {
        let out = parts(&face_with_holes(2)).unwrap();
        assert_eq!(
            roles(&out),
            [
                ExtractedPart::Outershell,
                ExtractedPart::Hole,
                ExtractedPart::Hole
            ]
        );
        // The outer shell keeps the exterior verbatim and carries no hole itself.
        assert_eq!(face_3d(&out[0].0).exterior(), SQUARE);
        assert_eq!(face_3d(&out[0].0).interiors().count(), 0);
        // Each hole becomes an area of its own, also hole-less.
        assert_eq!(face_3d(&out[1].0).exterior(), hole_ring(1.0, 1.0));
        assert_eq!(face_3d(&out[1].0).interiors().count(), 0);
        assert_eq!(face_3d(&out[2].0).exterior(), hole_ring(2.5, 1.0));
    }

    #[test]
    fn a_face_without_holes_yields_only_its_outer_boundary() {
        let out = parts(&face_with_holes(0)).unwrap();
        assert_eq!(roles(&out), [ExtractedPart::Outershell]);
        assert_eq!(face_3d(&out[0].0).exterior(), SQUARE);
    }

    // A hole's winding is what tells a validator whether the hole is oriented
    // correctly, so extraction must not re-wind it to match an outer boundary.
    #[test]
    fn a_hole_keeps_the_winding_it_had_inside_its_face() {
        let ring = hole_ring(1.0, 1.0);
        let out = parts(&face_with_holes(1)).unwrap();
        assert_eq!(face_3d(&out[1].0).exterior(), ring.as_slice());
        // Not the reversed ring: the two differ, so the assertion above is real.
        let mut reversed = ring;
        reversed.reverse();
        assert_ne!(face_3d(&out[1].0).exterior(), reversed.as_slice());
    }

    #[test]
    fn a_2d_face_carries_its_elevation_onto_every_part() {
        let out = parts(&face_2d_with_holes(1, 7.5)).unwrap();
        assert_eq!(
            roles(&out),
            [ExtractedPart::Outershell, ExtractedPart::Hole]
        );
        for (geometry, _) in &out {
            assert_eq!(face_2d(geometry).elevation(), Some(7.5));
        }
    }

    // A part has fewer corners than the face it came from, so the face's
    // per-corner UV no longer applies and the appearance cannot come along.
    #[test]
    fn parts_come_out_bare() {
        let mut face = face_with_holes(1);
        face.set_appearance(theme("rgb"), bare(), None).unwrap();
        assert!(face.appearance().is_some());

        for (geometry, _) in parts(&face).unwrap() {
            assert!(face_3d(&geometry).appearance().is_none());
        }
    }

    #[test]
    fn a_face_with_no_exterior_ring_is_rejected() {
        let empty = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            Vec::<[f64; 3]>::new(),
            Vec::<Vec<[f64; 3]>>::new(),
        );
        assert!(parts(&empty).is_err());
    }

    #[test]
    fn a_mesh_deaggregates_into_its_faces() {
        let mesh = PolygonMesh3D::from_polygons(
            CoordinateFrame::Euclidean,
            [&face_with_holes(2), &face_with_holes(0)],
        )
        .unwrap();
        let out = parts(&mesh).unwrap();
        // One outer shell per face, and the holes of the face that has them.
        assert_eq!(count(&out, ExtractedPart::Outershell), 2);
        assert_eq!(count(&out, ExtractedPart::Hole), 2);
        assert_eq!(count(&out, ExtractedPart::Rejected), 0);
    }

    #[test]
    fn a_2d_mesh_takes_apart_a_holed_face() {
        // One quad face (corners 0..4) with one hole ring (corners 4..7).
        let mesh = PolygonMesh2D::from_raw_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0],
                [4.0, 0.0],
                [4.0, 4.0],
                [0.0, 4.0],
                [1.0, 1.0],
                [1.0, 2.0],
                [2.0, 2.0],
            ],
            vec![0, 1, 2, 3, 4, 5, 6],
            Vec::new(),
            vec![4],
        )
        .unwrap();
        let out = parts(&mesh).unwrap();
        assert_eq!(
            roles(&out),
            [ExtractedPart::Outershell, ExtractedPart::Hole]
        );
        assert_eq!(face_2d(&out[1].0).exterior().len(), 3);
    }

    #[test]
    fn a_2d_mesh_carries_its_elevation() {
        let mesh = PolygonMesh2D::from_parts_at_elevation(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [2.0, 0.0], [2.0, 2.0]],
            vec![vec![0u32, 1, 2]],
            3.0,
        )
        .unwrap();
        let out = parts(&mesh).unwrap();
        assert_eq!(roles(&out), [ExtractedPart::Outershell]);
        assert_eq!(face_2d(&out[0].0).elevation(), Some(3.0));
    }

    #[test]
    fn an_empty_mesh_yields_nothing() {
        let mesh =
            PolygonMesh3D::from_parts(CoordinateFrame::Euclidean, vec![], Vec::<Vec<u32>>::new())
                .unwrap();
        assert!(parts(&mesh).unwrap().is_empty());
    }

    #[test]
    fn a_solid_takes_apart_the_faces_of_every_shell() {
        // Exterior face has two holes, the void shell's face has one.
        let solid = Solid::new(
            CoordinateFrame::Euclidean,
            shell_with_holes(2),
            vec![Shell::from(shell_with_holes(1))],
        );
        let out = parts(&solid).unwrap();
        // One outer shell per boundary face — the void shell itself is a hollow
        // volume, not a hole, so it is never emitted as one.
        assert_eq!(count(&out, ExtractedPart::Outershell), 2);
        assert_eq!(count(&out, ExtractedPart::Hole), 3);
    }

    #[test]
    fn a_solid_bounded_by_triangles_yields_no_holes() {
        let solid = Solid::from_exterior(CoordinateFrame::Euclidean, triangle_shell());
        let out = parts(&solid).unwrap();
        assert_eq!(count(&out, ExtractedPart::Outershell), 2);
        assert_eq!(count(&out, ExtractedPart::Hole), 0);
    }

    #[test]
    fn a_triangle_mesh_deaggregates_into_closed_triangles() {
        let mesh = TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            [0u32, 1, 2, 1, 3, 2],
        )
        .unwrap();
        let out = parts(&mesh).unwrap();
        assert_eq!(
            roles(&out),
            [ExtractedPart::Outershell, ExtractedPart::Outershell]
        );
        let first = face_3d(&out[0].0).exterior();
        assert_eq!(
            first,
            [
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0]
            ]
        );
    }

    #[test]
    fn a_2d_triangle_mesh_carries_its_elevation() {
        let mesh = TriangularMesh2D::from_parts_at_elevation(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [1.0, 0.0], [0.0, 1.0]],
            [0u32, 1, 2],
            2.5,
        )
        .unwrap();
        let out = parts(&mesh).unwrap();
        assert_eq!(roles(&out), [ExtractedPart::Outershell]);
        assert_eq!(face_2d(&out[0].0).elevation(), Some(2.5));
    }

    #[test]
    fn a_type_that_bounds_no_area_is_rejected() {
        for g in non_area_3d_leaves() {
            assert!(parts(&geometry_3d(g)).is_err());
        }
    }

    #[test]
    fn an_absent_geometry_is_rejected() {
        assert!(parts(&Geometry::None).is_err());
    }

    // Deaggregating a container must not let one curve discard the surfaces
    // beside it, so a member that bounds no area comes back as `Rejected`.
    #[test]
    fn a_collection_hands_back_its_non_area_members() {
        let point = Point3D::new(CoordinateFrame::Euclidean, [0.0, 0.0, 0.0]);
        let c = Collection3D::new([
            Euclidean3DGeometry::Polygon(Box::new(face_with_holes(2))),
            Euclidean3DGeometry::Point(point.clone()),
        ]);
        let out = parts(&geometry_3d(Euclidean3DGeometry::Collection(c))).unwrap();
        assert_eq!(
            roles(&out),
            [
                ExtractedPart::Outershell,
                ExtractedPart::Hole,
                ExtractedPart::Hole,
                ExtractedPart::Rejected
            ]
        );
        assert_eq!(
            out[3].0,
            geometry_3d(Euclidean3DGeometry::Point(point)),
            "the rejected member comes back untouched"
        );
    }

    #[test]
    fn extraction_reaches_through_nested_collections() {
        let inner = Geometry::GeometryCollection(GeometryCollection::new([geometry_3d(
            Euclidean3DGeometry::Polygon(Box::new(face_with_holes(1))),
        )]));
        let outer = Geometry::GeometryCollection(GeometryCollection::new([
            inner,
            geometry_3d(Euclidean3DGeometry::Polygon(Box::new(face_with_holes(3)))),
            Geometry::None,
        ]));
        let out = parts(&outer).unwrap();
        assert_eq!(count(&out, ExtractedPart::Outershell), 2);
        assert_eq!(count(&out, ExtractedPart::Hole), 4);
        // The absent member bounds no area, so it comes back rejected.
        assert_eq!(count(&out, ExtractedPart::Rejected), 1);
    }

    #[test]
    fn an_empty_collection_yields_nothing() {
        let c = Collection3D::new(Vec::new());
        let out = parts(&geometry_3d(Euclidean3DGeometry::Collection(c))).unwrap();
        assert!(out.is_empty());
    }

    // The two operations share one notion of a hole, so the workflows that filter
    // on a hole count and then extract cannot disagree about what they will get.
    #[test]
    fn the_hole_count_matches_the_holes_extracted() {
        let mesh = PolygonMesh3D::from_polygons(
            CoordinateFrame::Euclidean,
            [&face_with_holes(2), &face_with_holes(1)],
        )
        .unwrap();
        let solid = Solid::new(
            CoordinateFrame::Euclidean,
            shell_with_holes(2),
            vec![Shell::from(shell_with_holes(1))],
        );
        let collection = Geometry::GeometryCollection(GeometryCollection::new([
            geometry_3d(Euclidean3DGeometry::Polygon(Box::new(face_with_holes(3)))),
            geometry_3d(Euclidean3DGeometry::PolygonMesh(Box::new(mesh.clone()))),
        ]));

        for geometry in [
            geometry_3d(Euclidean3DGeometry::Polygon(Box::new(face_with_holes(2)))),
            geometry_3d(Euclidean3DGeometry::PolygonMesh(Box::new(mesh))),
            geometry_3d(Euclidean3DGeometry::Solid(Box::new(solid))),
            collection,
        ] {
            let extracted = count(&parts(&geometry).unwrap(), ExtractedPart::Hole);
            assert_eq!(geometry.count_holes(), extracted);
        }
    }
}
