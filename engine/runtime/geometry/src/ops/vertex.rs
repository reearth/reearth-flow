//! Vertex counting: how many coordinates a geometry stores.
//!
//! The count is of *stored* coordinates, not of distinct positions: a ring
//! written with its first vertex repeated at the end counts that repeat, and a
//! mesh counts its shared vertex pool once rather than once per face corner.
//! Callers ask this to inspect how the geometry was written — whether a
//! triangle really carries four coordinates, say — so normalizing the answer
//! would destroy the very thing being measured.

/// The number of coordinates a geometry stores.
///
/// Total over the hierarchy: a container sums its members. The default body
/// returns `0`, so a leaf that stores no coordinates of its own needs only an
/// (empty) `impl`, stamped by [`unsupported!`](crate::unsupported). Counting
/// never fails; a geometry with nothing to count answers `0`.
///
/// Coordinates are counted **verbatim**, as stored:
///
/// * A ring keeps its closing vertex if it has one, so a closed triangle counts
///   `4` and the same triangle stored open counts `3`.
/// * A face sums its exterior and every interior ring.
/// * A mesh counts its vertex pool, so a vertex shared by several faces counts
///   once.
/// * A [`Solid`](crate::solid::Solid) sums the pools of its exterior and
///   interior shells.
#[enum_dispatch::enum_dispatch]
pub trait CountVertices {
    /// The number of coordinates this geometry stores, recursing into
    /// collections.
    fn count_vertices(&self) -> usize {
        0
    }
}

// The boxed enum variants (`Box<Polygon2D>`, `Box<Solid>`, …) need the trait on
// the `Box` itself: `enum_dispatch` forwards by UFCS, not auto-deref.
impl<T: CountVertices + ?Sized> CountVertices for Box<T> {
    fn count_vertices(&self) -> usize {
        (**self).count_vertices()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use crate::line_string::LineString3D;
    use crate::point::Point3D;
    use crate::polygon::Polygon3D;
    use crate::{Euclidean3DGeometry, Geometry, GeometryCollection};

    fn triangle(ring: Vec<[f64; 3]>) -> Polygon3D {
        Polygon3D::from_rings(
            CoordinateFrame::Euclidean,
            ring,
            Vec::<Vec<[f64; 3]>>::new(),
        )
    }

    /// The distinction the CityGML quality checks rely on: a triangle written
    /// with its closing vertex counts four, the same triangle written open
    /// counts three. Normalizing either way would hide the defect being looked
    /// for.
    #[test]
    fn a_ring_is_counted_as_stored_closing_vertex_included() {
        let closed = triangle(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ]);
        let open = triangle(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        assert_eq!(closed.count_vertices(), 4);
        assert_eq!(open.count_vertices(), 3);
    }

    #[test]
    fn a_face_sums_its_exterior_and_every_hole() {
        let with_hole = Polygon3D::from_rings(
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
                [1.0, 3.0, 0.0],
                [3.0, 3.0, 0.0],
                [3.0, 1.0, 0.0],
                [1.0, 1.0, 0.0],
            ]],
        );
        assert_eq!(with_hole.count_vertices(), 10);
    }

    #[test]
    fn a_point_is_one_and_a_curve_is_its_coordinates() {
        let point = Point3D::new(CoordinateFrame::Euclidean, [1.0, 2.0, 3.0]);
        let curve = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [2.0, 2.0, 2.0]],
        );
        assert_eq!(point.count_vertices(), 1);
        assert_eq!(curve.count_vertices(), 3);
    }

    /// A collection sums its members, and an absent geometry contributes
    /// nothing rather than refusing to be counted.
    #[test]
    fn a_collection_sums_its_members_and_none_counts_zero() {
        let collection = GeometryCollection::new(vec![
            Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(triangle(vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0],
            ])))),
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
                CoordinateFrame::Euclidean,
                [0.0, 0.0, 0.0],
            ))),
            Geometry::None,
        ]);
        assert_eq!(Geometry::GeometryCollection(collection).count_vertices(), 5);
    }
}
