//! Area measurement: how much ground a geometry covers, and how much surface
//! it actually has.
//!
//! Both measures come from one piece of ring math. A planar ring's Newell
//! vector has magnitude twice the ring's true area, and its z component is
//! twice the ring's signed XY-projected area — so
//! [`newell_vector_3d`](crate::validation_next::measure::newell_vector_3d)
//! answers both questions and the two can never drift apart. A face standing
//! vertical projects to a line, and its zero falls out of the z component
//! rather than needing a special case.
//!
//! **Faces are summed, never unioned.** `projected_area` on a closed body adds
//! up each face's own projection, so a unit cube covers `2.0` (its top and its
//! bottom; the four walls project to nothing) and has `6.0` of surface. That is
//! deliberate: the old geometry model's action summed each CityGML polygon's
//! own projection too, so a user measuring a whole building has always got
//! roughly twice its footprint, and measurements do not move under the
//! migration. Footprint semantics would need a 2D union and would silently
//! change every existing workflow's numbers.

use crate::coordinate::CoordinateFrame;
use crate::ops::UnsupportedOperation;
use crate::polygon::Polygon3D;
use crate::validation_next::measure::newell_vector_3d;
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry, GeometryCollection};

/// The area of a geometry, measured two ways.
///
/// The default bodies return [`UnsupportedOperation`], so a leaf that cannot be
/// measured needs only an (empty) `impl`, stamped by
/// [`unsupported!`](crate::unsupported). A geometry that genuinely encloses
/// nothing — a point, a curve, a point cloud — measures `0.0` instead, stamped
/// by [`no_area!`](crate::no_area): that is an answer, not a refusal, and a
/// caller should never have to tell "no area" from "not measured".
#[enum_dispatch::enum_dispatch]
pub trait Area {
    /// The area the geometry covers on the XY plane: each face projected and
    /// the projections summed, never unioned. Vertical faces contribute zero.
    fn projected_area(&self) -> Result<f64, UnsupportedOperation> {
        Err(UnsupportedOperation {
            geometry: core::any::type_name::<Self>(),
            operation: "projected_area",
        })
    }

    /// The true surface area, following the slope of each face.
    fn surface_area(&self) -> Result<f64, UnsupportedOperation> {
        Err(UnsupportedOperation {
            geometry: core::any::type_name::<Self>(),
            operation: "surface_area",
        })
    }
}

// The boxed enum variants (`Box<Polygon2D>`, `Box<Solid>`, …) need the trait on
// the `Box` itself: `enum_dispatch` forwards by UFCS, not auto-deref.
impl<T: Area + ?Sized> Area for Box<T> {
    fn projected_area(&self) -> Result<f64, UnsupportedOperation> {
        (**self).projected_area()
    }

    fn surface_area(&self) -> Result<f64, UnsupportedOperation> {
        (**self).surface_area()
    }
}

/// The true area of one planar 3D ring: half its Newell vector's magnitude. A
/// ring stored open is measured with its closing edge restored; a degenerate
/// ring encloses nothing and measures zero.
pub(crate) fn ring_surface_area(ring: &[[f64; 3]]) -> f64 {
    let n = newell_vector_3d(ring);
    (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt() / 2.0
}

/// The area of one 3D ring projected onto the XY plane: half the magnitude of
/// its Newell vector's z component. A ring standing vertical projects to a line
/// and measures zero.
pub(crate) fn ring_projected_area(ring: &[[f64; 3]]) -> f64 {
    newell_vector_3d(ring)[2].abs() / 2.0
}

/// One face's area: its exterior ring's, less its holes', floored at zero —
/// holes larger than the ring holding them would otherwise measure negative.
/// Mirrors [`Polygon2D::area`](crate::polygon::Polygon2D::area).
pub(crate) fn face_area(exterior: f64, holes: impl Iterator<Item = f64>) -> f64 {
    (exterior - holes.sum::<f64>()).max(0.0)
}

/// A 3D face's XY-projected area, holes subtracted.
pub(crate) fn polygon_3d_projected_area(p: &Polygon3D) -> f64 {
    face_area(
        ring_projected_area(p.exterior()),
        p.interiors().map(ring_projected_area),
    )
}

/// A 3D face's true surface area, holes subtracted.
pub(crate) fn polygon_3d_surface_area(p: &Polygon3D) -> f64 {
    face_area(
        ring_surface_area(p.exterior()),
        p.interiors().map(ring_surface_area),
    )
}

/// The area of one 2D triangle: half the magnitude of its edge cross product.
pub(crate) fn triangle_area_2d(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])).abs() / 2.0
}

/// Sum `measure` over every triangle of an indexed 3D mesh, each rebuilt as its
/// own three-vertex ring.
pub(crate) fn triangle_area_sum_3d(
    vertices: &[[f64; 3]],
    triangles: impl Iterator<Item = [u32; 3]>,
    measure: fn(&[[f64; 3]]) -> f64,
) -> f64 {
    triangles
        .map(|t| {
            let ring = [
                vertices[t[0] as usize],
                vertices[t[1] as usize],
                vertices[t[2] as usize],
            ];
            measure(&ring)
        })
        .sum()
}

/// What the coordinate frames of a geometry's area-carrying parts say about a
/// measured area.
#[derive(Clone, Debug, PartialEq)]
pub enum AreaFrame {
    /// Every part that carries area sits in this one frame, so the result is in
    /// that frame's unit squared.
    One(CoordinateFrame),
    /// Parts sit in more than one frame, so the sum may add square metres to
    /// square degrees.
    Mixed,
    /// Nothing carrying area, so no frame to report.
    Nothing,
}

/// What [`Area`] measured, beyond the number itself.
///
/// A second, cheap traversal rather than a wider trait signature: the geometry
/// crate carries no logger, so a caller that has one — an action — needs these
/// facts as data in order to warn about them.
#[derive(Clone, Debug, PartialEq)]
pub struct AreaReport {
    /// The frame the measurement is expressed in.
    pub frame: AreaFrame,
    /// How many parts [`Area`] could not measure and skipped. Only an
    /// unevaluated [`Csg`](crate::csg::Csg) is unmeasurable, so this is
    /// normally zero.
    pub skipped: usize,
}

impl AreaReport {
    /// Fold in one area-carrying part's frame: the first sets the frame, a
    /// second, different one makes the whole report `Mixed`, and `Mixed` is
    /// absorbing.
    fn note(&mut self, frame: &CoordinateFrame) {
        self.frame = match std::mem::replace(&mut self.frame, AreaFrame::Nothing) {
            AreaFrame::Nothing => AreaFrame::One(frame.clone()),
            AreaFrame::One(seen) if &seen == frame => AreaFrame::One(seen),
            AreaFrame::One(_) | AreaFrame::Mixed => AreaFrame::Mixed,
        };
    }
}

/// Walk `geometry` and report which frames its area-carrying parts sit in and
/// how many parts could not be measured.
///
/// Only parts that can carry area contribute a frame — polygons, the meshes and
/// solids. A point, a curve or a point cloud measures exactly zero, so its
/// frame does not colour the result and including it would warn about the units
/// of a number that is zero either way.
pub fn area_report(geometry: &Geometry) -> AreaReport {
    let mut report = AreaReport {
        frame: AreaFrame::Nothing,
        skipped: 0,
    };
    walk(geometry, &mut report);
    report
}

fn walk(geometry: &Geometry, report: &mut AreaReport) {
    match geometry {
        Geometry::None => {}
        Geometry::Euclidean2D(g) => walk_2d(g, report),
        Geometry::Euclidean3D(g) => walk_3d(g, report),
        Geometry::GeometryCollection(c) => walk_collection(c, report),
    }
}

fn walk_collection(collection: &GeometryCollection, report: &mut AreaReport) {
    for member in collection.members() {
        walk(member, report);
    }
}

fn walk_2d(geometry: &Euclidean2DGeometry, report: &mut AreaReport) {
    match geometry {
        // Carries no area, so contributes no unit.
        Euclidean2DGeometry::Point(_) | Euclidean2DGeometry::LineString(_) => {}
        Euclidean2DGeometry::Polygon(g) => report.note(g.frame()),
        Euclidean2DGeometry::PolygonMesh(g) => report.note(g.frame()),
        Euclidean2DGeometry::TriangularMesh(g) => report.note(g.frame()),
        Euclidean2DGeometry::Collection(c) => {
            for member in c.members() {
                walk_2d(member, report);
            }
        }
    }
}

fn walk_3d(geometry: &Euclidean3DGeometry, report: &mut AreaReport) {
    match geometry {
        Euclidean3DGeometry::Point(_)
        | Euclidean3DGeometry::LineString(_)
        | Euclidean3DGeometry::PointCloud(_) => {}
        Euclidean3DGeometry::Polygon(g) => report.note(g.frame()),
        Euclidean3DGeometry::PolygonMesh(g) => report.note(g.frame()),
        Euclidean3DGeometry::TriangularMesh(g) => report.note(g.frame()),
        Euclidean3DGeometry::Solid(g) => report.note(g.frame()),
        // The one unmeasurable geometry: an unevaluated boolean tree.
        Euclidean3DGeometry::Csg(_) => report.skipped += 1,
        Euclidean3DGeometry::Collection(c) => {
            for member in c.members() {
                walk_3d(member, report);
            }
        }
    }
}

/// Whether an area measured in `frame` is in a linear unit squared — square
/// metres, square feet — rather than square degrees, which means nothing.
///
/// `Err` when the CRS cannot be resolved at all: the caller should say it could
/// not establish the unit rather than treat the area as meaningless.
pub fn frame_area_is_linear(frame: &CoordinateFrame) -> crate::error::Result<bool> {
    match frame {
        // Bare Euclidean space and a tangent plane are both plain lengths.
        CoordinateFrame::Euclidean | CoordinateFrame::Tangent(_) => Ok(true),
        CoordinateFrame::Crs(epsg) => crate::ops::crs_is_linear(*epsg),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use crate::point::Point3D;
    use crate::polygon::{Polygon2D, Polygon3D};

    /// The unit square in the XY plane, stored closed.
    fn unit_square_3d() -> Polygon3D {
        Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        )
    }

    /// The unit square tilted 45 degrees about the x axis: still one unit of
    /// surface, but only 1/sqrt(2) of it lands on the XY plane.
    fn tilted_square_3d() -> Polygon3D {
        let h = std::f64::consts::FRAC_1_SQRT_2;
        Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, h, h],
                [0.0, h, h],
                [0.0, 0.0, 0.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        )
    }

    #[test]
    fn unit_square_measures_one_both_ways() {
        let p = unit_square_3d();
        assert_eq!(p.projected_area().unwrap(), 1.0);
        assert_eq!(p.surface_area().unwrap(), 1.0);
    }

    /// The test that makes the two measures impossible to conflate: a plain
    /// unit square would pass with either method wired to both.
    #[test]
    fn tilted_square_projects_smaller_than_its_surface() {
        let p = tilted_square_3d();
        assert!((p.surface_area().unwrap() - 1.0).abs() < 1e-12);
        assert!(
            (p.projected_area().unwrap() - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-12,
            "projected {}",
            p.projected_area().unwrap()
        );
    }

    #[test]
    fn a_hole_is_subtracted_and_winding_does_not_matter() {
        // 4x4 exterior with a 2x2 hole: 16 - 4 = 12.
        let exterior = vec![
            [0.0, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [4.0, 4.0, 0.0],
            [0.0, 4.0, 0.0],
            [0.0, 0.0, 0.0],
        ];
        let hole = vec![
            [1.0, 1.0, 0.0],
            [3.0, 1.0, 0.0],
            [3.0, 3.0, 0.0],
            [1.0, 3.0, 0.0],
            [1.0, 1.0, 0.0],
        ];
        let mut reversed = hole.clone();
        reversed.reverse();

        for hole in [hole, reversed] {
            let p = Polygon3D::from_rings(CoordinateFrame::Euclidean, exterior.clone(), vec![hole]);
            assert_eq!(p.projected_area().unwrap(), 12.0);
            assert_eq!(p.surface_area().unwrap(), 12.0);
        }
    }

    /// A 2D face has no elevation to slope, so it answers both questions with
    /// its planar area — matching the old world, which ignored `areaType`
    /// entirely for 2D geometry.
    #[test]
    fn a_2d_face_answers_both_questions_identically() {
        let p = Polygon2D::from_rings(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [2.0, 0.0], [2.0, 3.0], [0.0, 3.0], [0.0, 0.0]],
            Vec::<Vec<[f64; 2]>>::new(),
        );
        assert_eq!(p.projected_area().unwrap(), 6.0);
        assert_eq!(p.surface_area().unwrap(), 6.0);
    }

    /// A point encloses nothing, which is an answer rather than a refusal.
    #[test]
    fn a_point_measures_zero_rather_than_refusing() {
        let p = Point3D::new(CoordinateFrame::Euclidean, [1.0, 2.0, 3.0]);
        assert_eq!(p.projected_area().unwrap(), 0.0);
        assert_eq!(p.surface_area().unwrap(), 0.0);
    }

    /// A ring stored open is measured with its closing edge restored, matching
    /// how `Polygon2D::area` treats one.
    #[test]
    fn an_open_ring_is_measured_as_if_closed() {
        let open = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        assert_eq!(open.surface_area().unwrap(), 1.0);
    }

    use crate::collection::Collection3D;
    use crate::csg::Csg;
    use crate::{Euclidean3DGeometry, Geometry, GeometryCollection};

    fn polygon_member(square: Polygon3D) -> Euclidean3DGeometry {
        Euclidean3DGeometry::Polygon(Box::new(square))
    }

    #[test]
    fn a_collection_sums_its_members() {
        let c = Collection3D::new(vec![
            polygon_member(unit_square_3d()),
            polygon_member(unit_square_3d()),
        ]);
        assert_eq!(c.projected_area().unwrap(), 2.0);
        assert_eq!(c.surface_area().unwrap(), 2.0);
    }

    #[test]
    fn an_empty_collection_measures_zero() {
        let c = Collection3D::new(Vec::<Euclidean3DGeometry>::new());
        assert_eq!(c.projected_area().unwrap(), 0.0);
        assert_eq!(c.surface_area().unwrap(), 0.0);
    }

    /// The only unmeasurable geometry in the model: an unevaluated boolean tree
    /// over two trivial solids. `ThreeDimensional` has `From<Solid>`, so the
    /// operands convert with `.into()`.
    fn csg() -> Csg {
        let solid = || {
            crate::solid::Solid::from_exterior(
                CoordinateFrame::Euclidean,
                crate::triangular_mesh::TriangularMesh3DData::from_parts(
                    vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
                    [0u32, 1, 2],
                )
                .unwrap(),
            )
        };
        Csg::Union(Box::new(solid().into()), Box::new(solid().into()))
    }

    /// An unmeasurable member is skipped rather than failing its siblings, the
    /// same way the CSV writer's geometry export omits one part of a feature
    /// without refusing the rest.
    #[test]
    fn a_collection_skips_an_unmeasurable_member() {
        let c = Collection3D::new(vec![
            polygon_member(unit_square_3d()),
            Euclidean3DGeometry::Csg(csg()),
        ]);
        assert_eq!(c.projected_area().unwrap(), 1.0);
        assert_eq!(c.surface_area().unwrap(), 1.0);
    }

    /// The refusal itself, so the skip above is provably skipping something.
    #[test]
    fn a_csg_refuses_rather_than_measuring_zero() {
        assert!(csg().projected_area().is_err());
        assert!(csg().surface_area().is_err());
    }

    /// An absent geometry measures zero rather than refusing, so the action
    /// always has a number to write.
    #[test]
    fn an_absent_geometry_measures_zero() {
        assert_eq!(Geometry::None.projected_area().unwrap(), 0.0);
        assert_eq!(Geometry::None.surface_area().unwrap(), 0.0);
    }

    #[test]
    fn a_geometry_collection_sums_across_dimensions() {
        let flat = Geometry::Euclidean2D(crate::Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                CoordinateFrame::Euclidean,
                vec![[0.0, 0.0], [2.0, 0.0], [2.0, 1.0], [0.0, 1.0], [0.0, 0.0]],
                Vec::<Vec<[f64; 2]>>::new(),
            ),
        )));
        let tilted = Geometry::Euclidean3D(polygon_member(tilted_square_3d()));
        let c = Geometry::GeometryCollection(GeometryCollection::new(vec![flat, tilted]));

        assert!((c.surface_area().unwrap() - 3.0).abs() < 1e-12);
        assert!(
            (c.projected_area().unwrap() - (2.0 + std::f64::consts::FRAC_1_SQRT_2)).abs() < 1e-12
        );
    }

    use crate::polygon_mesh::PolygonMesh3D;
    use crate::triangular_mesh::{TriangularMesh2D, TriangularMesh3D};

    /// Two unit right triangles sharing an edge: together, one unit square.
    #[test]
    fn a_two_triangle_3d_mesh_sums_its_faces() {
        let m = TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            [0u32, 1, 2, 0, 2, 3],
        )
        .unwrap();
        assert_eq!(m.projected_area().unwrap(), 1.0);
        assert_eq!(m.surface_area().unwrap(), 1.0);
    }

    /// A vertical triangle covers no ground but still has surface.
    #[test]
    fn a_vertical_triangle_projects_to_nothing() {
        let m = TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 0.0, 2.0]],
            [0u32, 1, 2],
        )
        .unwrap();
        assert_eq!(m.projected_area().unwrap(), 0.0);
        assert_eq!(m.surface_area().unwrap(), 2.0);
    }

    #[test]
    fn a_2d_triangle_mesh_answers_both_questions_identically() {
        let m = TriangularMesh2D::from_parts(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0], [4.0, 0.0], [0.0, 3.0]],
            [0u32, 1, 2],
        )
        .unwrap();
        assert_eq!(m.projected_area().unwrap(), 6.0);
        assert_eq!(m.surface_area().unwrap(), 6.0);
    }

    /// A face's holes still subtract once the face is one of a mesh's, and two
    /// faces sum. `PolygonMesh3D::from_polygons` is the constructor that
    /// preserves interior rings; `from_parts` takes exterior index lists only.
    #[test]
    fn a_polygon_mesh_sums_its_faces_and_subtracts_their_holes() {
        // A 4x4 face carrying a 2x2 hole: 12.
        let holed = Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            vec![
                [0.0, 0.0, 0.0],
                [4.0, 0.0, 0.0],
                [4.0, 4.0, 0.0],
                [0.0, 4.0, 0.0],
                [0.0, 0.0, 0.0],
            ],
            vec![vec![
                [1.0, 1.0, 0.0],
                [3.0, 1.0, 0.0],
                [3.0, 3.0, 0.0],
                [1.0, 3.0, 0.0],
                [1.0, 1.0, 0.0],
            ]],
        );
        let mesh =
            PolygonMesh3D::from_polygons(CoordinateFrame::Euclidean, &[holed, unit_square_3d()])
                .unwrap();
        // 12 from the holed face, 1 from the unit square.
        assert_eq!(mesh.projected_area().unwrap(), 13.0);
        assert_eq!(mesh.surface_area().unwrap(), 13.0);
    }

    #[test]
    fn an_empty_mesh_measures_zero() {
        let m = TriangularMesh3D::from_parts(
            CoordinateFrame::Euclidean,
            Vec::<[f64; 3]>::new(),
            Vec::<u32>::new(),
        )
        .unwrap();
        assert_eq!(m.projected_area().unwrap(), 0.0);
        assert_eq!(m.surface_area().unwrap(), 0.0);
    }

    use crate::solid::{Shell, Solid};
    use crate::triangular_mesh::TriangularMesh3DData;

    /// A closed triangle shell over the axis-aligned box `[min, min + size]`.
    /// Copied from `footprint_replacer.rs`'s test helper, which builds the same
    /// twelve triangles.
    fn box_shell(min: [f64; 3], size: [f64; 3]) -> TriangularMesh3DData {
        let corners: Vec<[f64; 3]> = (0..8u32)
            .map(|i| {
                [
                    min[0] + if i & 1 != 0 { size[0] } else { 0.0 },
                    min[1] + if i & 2 != 0 { size[1] } else { 0.0 },
                    min[2] + if i & 4 != 0 { size[2] } else { 0.0 },
                ]
            })
            .collect();
        #[rustfmt::skip]
        const TRIS: [u32; 36] = [
            0, 1, 3,  0, 3, 2,
            4, 7, 5,  4, 6, 7,
            0, 4, 5,  0, 5, 1,
            2, 3, 7,  2, 7, 6,
            0, 2, 6,  0, 6, 4,
            1, 5, 7,  1, 7, 3,
        ];
        TriangularMesh3DData::from_parts(corners, TRIS).unwrap()
    }

    /// **Pin.** A unit cube covers 2.0 and has 6.0 of surface: the top and the
    /// bottom each project to 1.0, the four walls project to zero-area lines,
    /// and all six faces contribute their full area to the surface.
    ///
    /// This is deliberately not footprint semantics. The old geometry model's
    /// action summed each CityGML polygon's own XY projection too, so a user
    /// measuring a whole building has always got roughly twice its footprint.
    /// Changing it would need a 2D union and would silently move every existing
    /// workflow's numbers. Do not "correct" this test.
    #[test]
    fn a_unit_cube_covers_two_and_has_six_of_surface() {
        let cube = Solid::from_exterior(
            CoordinateFrame::Euclidean,
            Shell::TriangularMesh(box_shell([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])),
        );
        assert!((cube.projected_area().unwrap() - 2.0).abs() < 1e-12);
        assert!((cube.surface_area().unwrap() - 6.0).abs() < 1e-12);
    }

    /// A void's faces are real surfaces, so hollowing a body *adds* area. This
    /// is deliberately unlike a polygon's holes, which subtract, and the
    /// difference is recorded here so it is not mistaken for a bug.
    #[test]
    fn a_void_adds_area_where_a_polygon_hole_would_subtract() {
        let solid = Solid::from_exterior(
            CoordinateFrame::Euclidean,
            Shell::TriangularMesh(box_shell([0.0, 0.0, 0.0], [2.0, 2.0, 2.0])),
        );
        let hollow = Solid::new(
            CoordinateFrame::Euclidean,
            Shell::TriangularMesh(box_shell([0.0, 0.0, 0.0], [2.0, 2.0, 2.0])),
            vec![Shell::TriangularMesh(box_shell(
                [0.5, 0.5, 0.5],
                [1.0, 1.0, 1.0],
            ))],
        );
        // Exterior alone: 6 faces of 2x2 = 24.
        assert!((solid.surface_area().unwrap() - 24.0).abs() < 1e-12);
        // Plus the void's own 6 unit faces = 30.
        assert!((hollow.surface_area().unwrap() - 30.0).abs() < 1e-12);
        assert!(hollow.surface_area().unwrap() > solid.surface_area().unwrap());
    }

    use crate::coordinate::EpsgCode;

    #[test]
    fn a_single_frame_is_reported_as_one() {
        let g = Geometry::Euclidean3D(polygon_member(unit_square_3d()));
        assert_eq!(
            area_report(&g),
            AreaReport {
                frame: AreaFrame::One(CoordinateFrame::Euclidean),
                skipped: 0,
            }
        );
    }

    #[test]
    fn an_absent_geometry_reports_nothing() {
        assert_eq!(
            area_report(&Geometry::None),
            AreaReport {
                frame: AreaFrame::Nothing,
                skipped: 0,
            }
        );
    }

    /// A point carries no area, so its frame does not colour the result: a sum
    /// of one polygon and one point is in the polygon's frame alone.
    #[test]
    fn a_geometry_that_carries_no_area_contributes_no_frame() {
        let point = Euclidean3DGeometry::Point(crate::point::Point3D::new(
            CoordinateFrame::Crs(EpsgCode::from(4326)),
            [0.0, 0.0, 0.0],
        ));
        let c = Collection3D::new(vec![polygon_member(unit_square_3d()), point]);
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(c));
        assert_eq!(
            area_report(&g).frame,
            AreaFrame::One(CoordinateFrame::Euclidean)
        );
    }

    /// Two area-carrying members in different frames: the sum adds square
    /// metres to square degrees, and the caller has to be told.
    #[test]
    fn members_in_different_frames_report_mixed() {
        let projected = Polygon3D::from_rings(
            CoordinateFrame::Crs(EpsgCode::from(6677)),
            vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
            ],
            Vec::<Vec<[f64; 3]>>::new(),
        );
        let c = Collection3D::new(vec![
            polygon_member(unit_square_3d()),
            polygon_member(projected),
        ]);
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(c));
        assert_eq!(area_report(&g).frame, AreaFrame::Mixed);
    }

    /// The count behind the skip: `Area` returns the polygon's area silently,
    /// and this is how a caller learns something was left out of it.
    #[test]
    fn an_unmeasurable_member_is_counted_as_skipped() {
        let c = Collection3D::new(vec![
            polygon_member(unit_square_3d()),
            Euclidean3DGeometry::Csg(csg()),
        ]);
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(c));
        assert_eq!(
            area_report(&g),
            AreaReport {
                frame: AreaFrame::One(CoordinateFrame::Euclidean),
                skipped: 1,
            }
        );
    }

    /// Euclidean and tangent-plane coordinates are plain lengths, so an area in
    /// them is always in a linear unit squared.
    #[test]
    fn euclidean_coordinates_are_linear() {
        assert!(frame_area_is_linear(&CoordinateFrame::Euclidean).unwrap());
    }

    /// A projected CRS measures in metres; a geographic one measures in
    /// degrees, and an area in square degrees means nothing.
    #[test]
    fn a_projected_crs_is_linear_and_a_geographic_one_is_not() {
        // JGD2011 / Japan Plane Rectangular CS IX: metres.
        assert!(frame_area_is_linear(&CoordinateFrame::Crs(EpsgCode::from(6677))).unwrap());
        // WGS 84: degrees.
        assert!(!frame_area_is_linear(&CoordinateFrame::Crs(EpsgCode::from(4326))).unwrap());
    }

    /// `walk` must recurse through all three container kinds — a
    /// `GeometryCollection` holding a `Geometry::Euclidean3D(Collection3D)`
    /// that itself holds the area-carrying member and the `Csg` — not stop at
    /// the outermost. Every other nesting test in this module bottoms out at
    /// `Euclidean3D(Collection)` directly; this one goes one level deeper.
    #[test]
    fn area_report_walks_through_a_geometry_collection() {
        let inner = Collection3D::new(vec![
            polygon_member(unit_square_3d()),
            Euclidean3DGeometry::Csg(csg()),
        ]);
        let g = Geometry::GeometryCollection(GeometryCollection::new(vec![Geometry::Euclidean3D(
            Euclidean3DGeometry::Collection(inner),
        )]));
        assert_eq!(
            area_report(&g),
            AreaReport {
                frame: AreaFrame::One(CoordinateFrame::Euclidean),
                skipped: 1,
            }
        );
    }

    /// `Mixed` is absorbing across three or more observations, not just a
    /// comparison against the immediately previous frame: returning to an
    /// already-seen frame after `Mixed` has been reached must not undo it, in
    /// either order.
    #[test]
    fn mixed_is_absorbing_across_more_than_two_observations() {
        let projected = || {
            Polygon3D::from_rings(
                CoordinateFrame::Crs(EpsgCode::from(6677)),
                vec![
                    [0.0, 0.0, 0.0],
                    [1.0, 0.0, 0.0],
                    [1.0, 1.0, 0.0],
                    [0.0, 0.0, 0.0],
                ],
                Vec::<Vec<[f64; 3]>>::new(),
            )
        };

        // A, B, A.
        let a_b_a = Collection3D::new(vec![
            polygon_member(unit_square_3d()),
            polygon_member(projected()),
            polygon_member(unit_square_3d()),
        ]);
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(a_b_a));
        assert_eq!(area_report(&g).frame, AreaFrame::Mixed);

        // B, A, B.
        let b_a_b = Collection3D::new(vec![
            polygon_member(projected()),
            polygon_member(unit_square_3d()),
            polygon_member(projected()),
        ]);
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(b_a_b));
        assert_eq!(area_report(&g).frame, AreaFrame::Mixed);
    }

    /// A geometry that is present but carries no area at all — as opposed to
    /// `Geometry::None` — still reports `Nothing`: "no frame to report" is
    /// distinct from "a frame whose area happens to be zero".
    #[test]
    fn a_geometry_with_no_area_carrying_parts_reports_nothing() {
        let point = Euclidean3DGeometry::Point(crate::point::Point3D::new(
            CoordinateFrame::Euclidean,
            [0.0, 0.0, 0.0],
        ));
        let line = Euclidean3DGeometry::LineString(crate::line_string::LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
        ));
        let c = Collection3D::new(vec![point, line]);
        let g = Geometry::Euclidean3D(Euclidean3DGeometry::Collection(c));
        assert_eq!(
            area_report(&g),
            AreaReport {
                frame: AreaFrame::Nothing,
                skipped: 0,
            }
        );
    }
}
