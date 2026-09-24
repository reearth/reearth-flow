//! Vertex counting: how many vertices a geometry has.
//!
//! A ring's closing vertex repeats its first one, so it is not counted: a
//! triangle counts three whether or not it is stored closed. Whether a ring is
//! closed is a validation question, not a counting one.

/// The number of vertices a geometry has.
///
/// Total over the hierarchy: a container sums its members. The default body
/// returns `0`, so a leaf that stores no coordinates of its own needs only an
/// (empty) `impl`, stamped by [`unsupported!`](crate::unsupported). Counting
/// never fails; a geometry with nothing to count answers `0`.
///
/// * A ring drops its closing vertex if it has one, so a triangle counts `3`
///   stored closed or open.
/// * A face sums its exterior and every interior ring.
/// * A mesh counts its vertex pool, so a vertex shared by several faces counts
///   once.
/// * A [`Solid`](crate::solid::Solid) sums the pools of its exterior and
///   interior shells.
#[enum_dispatch::enum_dispatch]
pub trait CountVertices {
    /// The number of vertices this geometry has, recursing into
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

    #[test]
    fn a_closing_vertex_is_not_counted() {
        let closed = triangle(vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ]);
        let open = triangle(vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        assert_eq!(closed.count_vertices(), 3);
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
        assert_eq!(with_hole.count_vertices(), 8);
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
        assert_eq!(Geometry::GeometryCollection(collection).count_vertices(), 4);
    }
}
