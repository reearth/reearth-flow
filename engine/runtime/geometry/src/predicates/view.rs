//! Zero-copy coordinate views feeding the [`kernel`](super::kernel).
//!
//! The predicates operate over lightweight borrows of the new leaves' flat
//! buffers rather than over the leaf types directly, so a `Polygon` ring, a
//! `PolygonMesh` face, and a `TriangularMesh` triangle can all reach the same
//! kernel code without an intermediate copy. A ring view is simply a
//! `&[[f64; N]]` slice — the leaves' own storage — and a triangle is `[[f64; N]; 3]`.
//!
//! Phase 1 lands the polygon ring views and the triangle primitives; the mesh
//! face/triangle views build on these as their predicates land.

use super::kernel::{self, Orientation};
use crate::polygon::{Polygon2D, Polygon3D};

/// The rings of a 2D polygon — exterior first, then each interior (hole) — as
/// borrowed slices over the polygon's own buffer.
pub fn polygon2d_rings(polygon: &Polygon2D) -> impl Iterator<Item = &[[f64; 2]]> {
    core::iter::once(polygon.exterior()).chain(polygon.interiors())
}

/// The rings of a 3D polygon — exterior first, then each interior — as borrowed
/// slices over the polygon's own buffer.
pub fn polygon3d_rings(polygon: &Polygon3D) -> impl Iterator<Item = &[[f64; 3]]> {
    core::iter::once(polygon.exterior()).chain(polygon.interiors())
}

/// Resolve an indexed triangle into its three 2D corners from a vertex pool.
#[inline]
pub fn resolve_triangle_2d(vertices: &[[f64; 2]], tri: [u32; 3]) -> [[f64; 2]; 3] {
    [
        vertices[tri[0] as usize],
        vertices[tri[1] as usize],
        vertices[tri[2] as usize],
    ]
}

/// Resolve an indexed triangle into its three 3D corners from a vertex pool.
#[inline]
pub fn resolve_triangle_3d(vertices: &[[f64; 3]], tri: [u32; 3]) -> [[f64; 3]; 3] {
    [
        vertices[tri[0] as usize],
        vertices[tri[1] as usize],
        vertices[tri[2] as usize],
    ]
}

/// Whether a coordinate lies inside or on a 2D triangle, by robust orientation.
///
/// A point is inside/on when it is on the same side of (or collinear with) all
/// three directed edges; this is orientation-independent, so a CW or CCW triangle
/// gives the same answer.
pub fn point_in_triangle_2d(p: [f64; 2], tri: [[f64; 2]; 3]) -> bool {
    let d0 = kernel::orient2d(tri[0], tri[1], p);
    let d1 = kernel::orient2d(tri[1], tri[2], p);
    let d2 = kernel::orient2d(tri[2], tri[0], p);
    let has_cw = d0 == Orientation::Clockwise
        || d1 == Orientation::Clockwise
        || d2 == Orientation::Clockwise;
    let has_ccw = d0 == Orientation::CounterClockwise
        || d1 == Orientation::CounterClockwise
        || d2 == Orientation::CounterClockwise;
    // Inside/on iff it never lies on both sides across the three edges.
    !(has_cw && has_ccw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coordinate::CoordinateFrame;
    use crate::predicates::kernel::{coord_pos_relative_to_ring, CoordPos};

    #[test]
    fn polygon_rings_yield_exterior_then_holes() {
        let e = CoordinateFrame::Euclidean;
        let square = [[0.0, 0.0], [4.0, 0.0], [4.0, 4.0], [0.0, 4.0], [0.0, 0.0]];
        let hole = vec![[1.0, 1.0], [3.0, 1.0], [3.0, 3.0], [1.0, 3.0], [1.0, 1.0]];
        let poly = Polygon2D::from_rings(e, square, vec![hole]);

        let rings: Vec<&[[f64; 2]]> = polygon2d_rings(&poly).collect();
        assert_eq!(rings.len(), 2);
        assert_eq!(rings[0], poly.exterior());
        assert_eq!(
            coord_pos_relative_to_ring([2.0, 2.0], rings[0]),
            CoordPos::Inside
        );
        // Second ring is the hole.
        assert_eq!(
            coord_pos_relative_to_ring([2.0, 2.0], rings[1]),
            CoordPos::Inside
        );
    }

    #[test]
    fn point_in_triangle_inside_edge_outside() {
        let tri = [[0.0, 0.0], [4.0, 0.0], [0.0, 4.0]];
        assert!(point_in_triangle_2d([1.0, 1.0], tri));
        assert!(point_in_triangle_2d([2.0, 0.0], tri)); // on an edge
        assert!(point_in_triangle_2d([0.0, 0.0], tri)); // on a vertex
        assert!(!point_in_triangle_2d([3.0, 3.0], tri));
        assert!(!point_in_triangle_2d([-1.0, 1.0], tri));
    }

    #[test]
    fn resolve_triangle_indexes_the_pool() {
        let verts = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        assert_eq!(
            resolve_triangle_3d(&verts, [0, 1, 2]),
            [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
        );
    }
}
