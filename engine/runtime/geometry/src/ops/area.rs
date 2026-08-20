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

use crate::ops::UnsupportedOperation;
use crate::polygon::Polygon3D;
use crate::validation_next::measure::newell_vector_3d;

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
// TODO(Task 3): consumed once `TriangularMesh2D`/`PolygonMesh2D` gain real
// `Area` impls; unused until then.
#[allow(dead_code)]
pub(crate) fn triangle_area_2d(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1])).abs() / 2.0
}

/// Sum `measure` over every triangle of an indexed 3D mesh, each rebuilt as its
/// own three-vertex ring.
// TODO(Task 3): consumed once `TriangularMesh3D`/`PolygonMesh3D` gain real
// `Area` impls; unused until then.
#[allow(dead_code)]
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
}
