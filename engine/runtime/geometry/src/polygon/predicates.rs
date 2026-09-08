//! Geometric predicates over face leaves.

use super::{Polygon2D, Polygon3D};
use crate::predicates::{self, Equal};

impl Equal for Polygon2D {
    fn equal(&self, rhs: &Self, tolerance: f64) -> predicates::Result<bool> {
        use crate::predicates::equal::{pair_off, ring_curves_2d};

        predicates::require_same_frame(self.frame(), rhs.frame())?;
        // Exterior against exterior, holes against holes; see `Polygon3D`.
        let (here, there) = (self.elevation(), rhs.elevation());
        if !ring_curves_2d(self.exterior(), here)
            .within(&ring_curves_2d(rhs.exterior(), there), tolerance)
        {
            return Ok(false);
        }
        let ours: Vec<_> = self.interiors().map(|r| ring_curves_2d(r, here)).collect();
        let theirs: Vec<_> = rhs.interiors().map(|r| ring_curves_2d(r, there)).collect();
        pair_off(&ours, &theirs, |a, b| Ok(a.within(b, tolerance)))
    }
}

impl Equal for Polygon3D {
    fn equal(&self, rhs: &Self, tolerance: f64) -> predicates::Result<bool> {
        use crate::predicates::equal::FaceCurves;

        // Reprojection stays the caller's explicit step.
        predicates::require_same_frame(self.frame(), rhs.frame())?;
        FaceCurves::new(self.exterior(), self.interiors())
            .within(&FaceCurves::new(rhs.exterior(), rhs.interiors()), tolerance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::Collection3D;
    use crate::coordinate::{CoordinateFrame, EpsgCode};
    use crate::predicates::PredicateError;
    use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry, GeometryCollection};

    fn tolerance() -> f64 {
        1e-9
    }

    fn ring(lo: f64, hi: f64) -> Vec<[f64; 3]> {
        vec![
            [lo, lo, 0.0],
            [hi, lo, 0.0],
            [hi, hi, 0.0],
            [lo, hi, 0.0],
            [lo, lo, 0.0],
        ]
    }

    fn face(exterior: Vec<[f64; 3]>, interiors: Vec<Vec<[f64; 3]>>) -> Polygon3D {
        Polygon3D::from_rings(CoordinateFrame::Euclidean, exterior, interiors)
    }

    #[test]
    fn a_face_is_not_equal_to_its_ring_inverted_twin() {
        let big = ring(0.0, 10.0);
        let small = ring(3.0, 7.0);
        let donut = face(big.clone(), vec![small.clone()]);
        // Invalid, but it reaches the engine: the exterior lies inside the hole.
        let inverted = face(small.clone(), vec![big.clone()]);

        assert!(!donut.equal(&inverted, tolerance()).unwrap());
        assert!(!inverted.equal(&donut, tolerance()).unwrap());
        // Neither is the plain square that traces only one of the two rings.
        assert!(!donut.equal(&face(big, vec![]), tolerance()).unwrap());
        assert!(!donut.equal(&face(small, vec![]), tolerance()).unwrap());
    }

    #[test]
    fn a_face_keeps_its_identity_under_a_vertex_added_on_an_edge() {
        let donut = face(ring(0.0, 10.0), vec![ring(3.0, 7.0)]);
        let mut split = ring(0.0, 10.0);
        split.insert(1, [3.0, 0.0, 0.0]);
        let resplit = face(split, vec![ring(3.0, 7.0)]);

        assert!(donut.equal(&resplit, tolerance()).unwrap());
        assert!(resplit.equal(&donut, tolerance()).unwrap());
    }

    #[test]
    fn holes_pair_off_in_any_order() {
        let big = ring(0.0, 10.0);
        let one = face(big.clone(), vec![ring(1.0, 2.0), ring(5.0, 6.0)]);
        let other = face(big, vec![ring(5.0, 6.0), ring(1.0, 2.0)]);

        assert!(one.equal(&other, tolerance()).unwrap());
    }

    #[test]
    fn a_face_is_not_equal_to_one_with_a_hole_it_lacks() {
        let big = ring(0.0, 10.0);
        assert!(!face(big.clone(), vec![ring(3.0, 7.0)])
            .equal(&face(big, vec![]), tolerance())
            .unwrap());
    }

    #[test]
    fn faces_in_different_frames_are_refused_rather_than_answered() {
        // Reprojection is the caller's explicit step, so a frame mismatch is a
        // question this cannot answer — not a pair of different shapes.
        let outline = ring(0.0, 10.0);
        let here = face(outline.clone(), vec![]);
        let elsewhere = Polygon3D::from_rings(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            outline,
            Vec::<Vec<[f64; 3]>>::new(),
        );

        assert_eq!(
            here.equal(&elsewhere, tolerance()),
            Err(PredicateError::MixedFrames)
        );
    }

    #[test]
    fn a_2d_and_a_3d_geometry_are_refused_rather_than_answered() {
        let flat = Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                CoordinateFrame::Euclidean,
                vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]],
                Vec::<Vec<[f64; 2]>>::new(),
            ),
        )));
        let solid = Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(face(
            ring(0.0, 1.0),
            vec![],
        ))));

        assert_eq!(
            flat.equal(&solid, tolerance()),
            Err(PredicateError::CrossDimension)
        );
        assert_eq!(
            solid.equal(&flat, tolerance()),
            Err(PredicateError::CrossDimension)
        );
    }

    fn wrap(face: Polygon3D) -> Euclidean3DGeometry {
        Euclidean3DGeometry::Polygon(Box::new(face))
    }

    fn bag(members: Vec<Euclidean3DGeometry>) -> Euclidean3DGeometry {
        Euclidean3DGeometry::Collection(Collection3D::new(members))
    }

    #[test]
    fn a_collection_is_refused_whatever_it_holds() {
        // Not "different": a collection has no point set of its own, so a
        // one-member bag is refused exactly as a two-member one is, and from
        // either side of the pair.
        let bare = wrap(face(ring(0.0, 1.0), vec![]));
        let one = bag(vec![wrap(face(ring(0.0, 1.0), vec![]))]);
        let two = bag(vec![
            wrap(face(ring(0.0, 1.0), vec![])),
            wrap(face(ring(5.0, 6.0), vec![])),
        ]);
        let empty = bag(vec![]);

        let refused = Err(PredicateError::Unsupported {
            geometry: core::any::type_name::<Collection3D>(),
        });
        for collection in [&one, &two, &empty] {
            assert_eq!(bare.equal(collection, tolerance()), refused);
            assert_eq!(collection.equal(&bare, tolerance()), refused);
        }
        assert_eq!(one.equal(&two, tolerance()), refused);
    }

    #[test]
    fn a_heterogeneous_collection_is_refused_too() {
        let bare = Geometry::Euclidean3D(wrap(face(ring(0.0, 1.0), vec![])));
        let wrapped = Geometry::GeometryCollection(GeometryCollection::new([bare.clone()]));

        let refused = Err(PredicateError::Unsupported {
            geometry: core::any::type_name::<GeometryCollection>(),
        });
        assert_eq!(bare.equal(&wrapped, tolerance()), refused);
        assert_eq!(wrapped.equal(&bare, tolerance()), refused);
        // A collection outranks the absent geometry it is weighed against: the
        // answer is unknown, not `false`.
        assert_eq!(Geometry::None.equal(&wrapped, tolerance()), refused);
    }
}
