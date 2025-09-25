use num_traits::Float;

use crate::algorithm::segment_triangle_intersection::segment_intersects_triangle;
use crate::types::coordnum::CoordNum;
use crate::types::{coordinate::Coordinate3D, line::Line3D};

pub fn triangles_intersect<T: Float + CoordNum>(
    t: &[Coordinate3D<T>; 3],
    s: &[Coordinate3D<T>; 3],
) -> bool {
    let epsilon = T::from(1e-10).unwrap();

    // Check if any edge of triangle t intersects triangle s
    for i in 0..3 {
        let j = (i + 1) % 3;
        let line = Line3D::new_(t[i], t[j]);
        if segment_intersects_triangle(&line, s, epsilon) {
            return true;
        }
    }

    // Check if any edge of triangle s intersects triangle t
    for i in 0..3 {
        let j = (i + 1) % 3;
        let line = Line3D::new_(s[i], s[j]);
        if segment_intersects_triangle(&line, t, epsilon) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_triangles_intersect_coplanar_separate() {
        // Two triangles in the same plane but not intersecting
        let t1 = [
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(0.0, 1.0, 0.0),
        ];

        let t2 = [
            Coordinate3D::new__(2.0, 0.0, 0.0),
            Coordinate3D::new__(3.0, 0.0, 0.0),
            Coordinate3D::new__(2.0, 1.0, 0.0),
        ];

        assert!(!triangles_intersect(&t1, &t2));
    }

    #[test]
    fn test_triangles_intersect_coplanar_overlapping() {
        // Two triangles in the same plane with overlapping edges
        let t1 = [
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(2.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 2.0, 0.0),
        ];

        let t2 = [
            Coordinate3D::new__(1.0, -1.0, 0.0),
            Coordinate3D::new__(3.0, -1.0, 0.0),
            Coordinate3D::new__(2.0, 1.0, 0.0),
        ];

        assert!(triangles_intersect(&t1, &t2));
    }

    #[test]
    fn test_triangles_intersect_perpendicular() {
        // Triangle in XY plane
        let t1 = [
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(2.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 2.0, 0.0),
        ];

        // Triangle in XZ plane intersecting the first
        let t2 = [
            Coordinate3D::new__(1.0, 1.0, -1.0),
            Coordinate3D::new__(1.0, 1.0, 1.0),
            Coordinate3D::new__(1.0, -1.0, 0.0),
        ];

        assert!(triangles_intersect(&t1, &t2));
    }

    #[test]
    fn test_triangles_intersect_parallel_planes() {
        // Triangle in z=0 plane
        let t1 = [
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(0.0, 1.0, 0.0),
        ];

        // Triangle in z=1 plane (parallel, no intersection)
        let t2 = [
            Coordinate3D::new__(0.0, 0.0, 1.0),
            Coordinate3D::new__(1.0, 0.0, 1.0),
            Coordinate3D::new__(0.0, 1.0, 1.0),
        ];

        assert!(!triangles_intersect(&t1, &t2));
    }

    #[test]
    fn test_triangles_intersect_edge_piercing() {
        // Horizontal triangle
        let t1 = [
            Coordinate3D::new__(-1.0, -1.0, 0.0),
            Coordinate3D::new__(1.0, -1.0, 0.0),
            Coordinate3D::new__(0.0, 1.0, 0.0),
        ];

        // Vertical triangle piercing through the horizontal one
        let t2 = [
            Coordinate3D::new__(0.0, 0.0, -1.0),
            Coordinate3D::new__(0.0, 0.0, 1.0),
            Coordinate3D::new__(0.0, -2.0, 0.0),
        ];

        assert!(triangles_intersect(&t1, &t2));
    }

    #[test]
    fn test_triangles_intersect_touching_vertex() {
        // First triangle
        let t1 = [
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(0.0, 1.0, 0.0),
        ];

        // Second triangle touching at a vertex (should not count as intersection)
        let t2 = [
            Coordinate3D::new__(1.0, 1.0, 0.0),
            Coordinate3D::new__(2.0, 1.0, 0.0),
            Coordinate3D::new__(1.0, 2.0, 0.0),
        ];

        assert!(!triangles_intersect(&t1, &t2));
    }

    #[test]
    fn test_triangles_intersect_t_configuration() {
        // First triangle forming the top of a T
        let t1 = [
            Coordinate3D::new__(-1.0, 1.0, 0.0),
            Coordinate3D::new__(1.0, 1.0, 0.0),
            Coordinate3D::new__(0.0, 2.0, 0.0),
        ];

        // Second triangle forming the stem of a T
        let t2 = [
            Coordinate3D::new__(-0.5, 0.0, 0.0),
            Coordinate3D::new__(0.5, 0.0, 0.0),
            Coordinate3D::new__(0.0, 1.5, 0.0),
        ];

        assert!(triangles_intersect(&t1, &t2));
    }
}
