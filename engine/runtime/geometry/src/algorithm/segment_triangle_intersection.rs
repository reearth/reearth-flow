use crate::types::{coordinate::Coordinate3D, line::Line3D};

/// returns the intersection point of a line segment and a triangle if they intersect, otherwise returns None.
/// If the intersection geometry is a line segment (i.e., the segment lies on an edge of the triangle), it returns None.
pub fn segment_triangle_intersection(
    line: &Line3D<f64>,
    triangle: &[Coordinate3D<f64>; 3],
    epsilon: f64,
) -> Option<Coordinate3D<f64>> {
    let p0 = line.start;
    let p1 = line.end;
    // Möller–Trumbore ray-triangle intersection algorithm
    let v0 = triangle[0];
    let v1 = triangle[1];
    let v2 = triangle[2];

    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    let ray_direction = p1 - p0;

    let h = ray_direction.cross(&edge2);
    let a = edge1.dot(&h);

    let unit_ray = ray_direction.normalize();
    let normal = edge1.cross(&edge2).normalize();

    // If Ray is parallel to triangle, then no intersection as we consider only proper intersections.
    if unit_ray.dot(&normal).abs() < epsilon {
        return None;
    }

    let f = 1.0 / a;
    let s = p0 - v0;
    let u = f * s.dot(&h);

    if !(0.0..=1.0).contains(&u) {
        return None;
    }

    let q = s.cross(&edge1);
    let v = f * ray_direction.dot(&q);

    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    // Compute t to find intersection point
    let t_param = f * edge2.dot(&q);

    // Check if intersection is within the line segment
    // Use strict inequality to exclude edges
    if t_param > epsilon && t_param < 1.0 - epsilon {
        let intersection_point = p0 + ray_direction * t_param;
        Some(intersection_point)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_segment_triangle_intersection_direct_hit1() {
        let triangle = [
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(2.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 2.0, 0.0),
        ];

        // Segment that passes through the triangle
        let p0 = Coordinate3D::new__(1.0, 0.5, -1.0);
        let p1 = Coordinate3D::new__(1.0, 0.5, 1.0);
        let line = Line3D::<f64>::new_(p0, p1);

        assert_eq!(
            segment_triangle_intersection(&line, &triangle, 1e-10).unwrap(),
            Coordinate3D::new__(1.0, 0.5, 0.0)
        );
    }

    #[test]
    fn test_segment_triangle_intersection_direct_hit2() {
        let t = [
            Coordinate3D::new__(-2.0, 0.0, 0.0),
            Coordinate3D::new__(2.0, 1.0, 0.0), 
            Coordinate3D::new__(2.0, -1.0, 0.0),
        ];

        // Segment that passes through the triangle
        let p0 = Coordinate3D::new__(-2.0, 0.0, -1.0);
        let p1 = Coordinate3D::new__(0.0, 0.0, 1.0);
        let line = Line3D::<f64>::new_(p0, p1);

        assert_eq!(
            segment_triangle_intersection(&line, &t, 1e-10).unwrap(),
            Coordinate3D::new__(-1.0, 0.0, 0.0)
        );
    }

    #[test]
    fn test_segment_triangle_intersection_miss() {
        let triangle = [
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(0.0, 1.0, 0.0),
        ];

        // Segment that misses the triangle
        let p0 = Coordinate3D::new__(2.0, 0.0, 0.0);
        let p1 = Coordinate3D::new__(2.0, 1.0, 0.0);
        let line = Line3D::<f64>::new_(p0, p1);

        assert!(segment_triangle_intersection(&line, &triangle, 1e-10).is_none());
    }

    #[test]
    fn test_segment_triangle_intersection_parallel() {
        // Triangle in XY plane
        let triangle = [
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(0.0, 1.0, 0.0),
        ];

        // Segment parallel to the triangle plane
        let p0 = Coordinate3D::new__(0.0, 0.0, 1.0);
        let p1 = Coordinate3D::new__(1.0, 1.0, 1.0);
        let line = Line3D::<f64>::new_(p0, p1);

        assert!(segment_triangle_intersection(&line, &triangle, 1e-10).is_none());
    }

    #[test]
    fn test_segment_triangle_intersection_edge_case() {
        let triangle = [
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(0.5, 1.0, 0.0),
        ];

        // Segment that just grazes the vertex (1.0, 0.0, 0.0)
        let p0 = Coordinate3D::new__(1.0, 0.0, -1.0);
        let p1 = Coordinate3D::new__(1.0, 0.0, 1.0);
        let line = Line3D::<f64>::new_(p0, p1);

        // This should return true as it passes through a vertex
        // Vertices are considered part of the triangle for robustness
        assert_eq!(
            segment_triangle_intersection(&line, &triangle, 1e-10).unwrap(),
            Coordinate3D::new__(1.0, 0.0, 0.0)
        );
    }
}
