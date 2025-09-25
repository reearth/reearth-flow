use num_traits::Float;

use crate::types::{
    coordinate::{Coordinate2D, Coordinate3D},
    coordnum::CoordNum,
    line::Line3D,
};

pub fn segment_intersects_triangle<T: Float + CoordNum>(
    line: &Line3D<T>,
    triangle: &[Coordinate3D<T>; 3],
    epsilon: T,
) -> bool {
    let p0 = line.start;
    let p1 = line.end;
    // Möller–Trumbore ray-triangle intersection algorithm
    let v0 = triangle[0];
    let v1 = triangle[1];
    let v2 = triangle[2];
    let triangle = [v0, v1, v2];

    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    let ray_direction = p1 - p0;

    let h = ray_direction.cross(&edge2);
    let a = edge1.dot(&h);

    // Ray is parallel to triangle - check for coplanar case
    if a.abs() < epsilon {
        // Check if segment is coplanar with triangle
        let normal = edge1.cross(&edge2);
        let d = -normal.dot(&v0);
        let dist0 = normal.dot(&p0) + d;
        let dist1 = normal.dot(&p1) + d;

        if dist0.abs() < epsilon && dist1.abs() < epsilon {
            // Segment is coplanar, check 2D intersection
            return segment_intersects_triangle_2d(p0, p1, triangle);
        }
        return false;
    }

    let f = T::one() / a;
    let s = p0 - v0;
    let u = f * s.dot(&h);

    if !(T::zero()..=T::one()).contains(&u) {
        return false;
    }

    let q = s.cross(&edge1);
    let v = f * ray_direction.dot(&q);

    if v < T::zero() || u + v > T::one() {
        return false;
    }

    // Compute t to find intersection point
    let t_param = f * edge2.dot(&q);

    // Check if intersection is within the line segment
    // Use strict inequality to exclude edges
    t_param > epsilon && t_param < T::one() - epsilon
}

fn segment_intersects_triangle_2d<T: Float + CoordNum>(
    p0: Coordinate3D<T>,
    p1: Coordinate3D<T>,
    triangle: [Coordinate3D<T>; 3],
) -> bool {
    // Project to 2D plane and check intersection
    // Find dominant axis to project out
    let v0 = triangle[0];
    let v1 = triangle[1];
    let v2 = triangle[2];

    let normal = (v1 - v0).cross(&(v2 - v0));

    // Choose projection plane (remove largest component)
    let (i0, i1) = if normal.x.abs() >= normal.y.abs() && normal.x.abs() >= normal.z.abs() {
        (1, 2) // Project to YZ plane
    } else if normal.y.abs() >= normal.x.abs() && normal.y.abs() >= normal.z.abs() {
        (0, 2) // Project to XZ plane
    } else {
        (0, 1) // Project to XY plane
    };

    // Check if segment intersects any triangle edge in 2D
    for i in 0..3 {
        let j = (i + 1) % 3;
        if segments_intersect_2d(
            Coordinate2D::new_(p0[i0], p0[i1]),
            Coordinate2D::new_(p1[i0], p1[i1]),
            Coordinate2D::new_(triangle[i][i0], triangle[i][i1]),
            Coordinate2D::new_(triangle[j][i0], triangle[j][i1]),
        ) {
            return true;
        }
    }

    // Check if either endpoint is inside the triangle
    point_in_triangle_2d(
        Coordinate2D::new_(p0[i0], p0[i1]),
        Coordinate2D::new_(triangle[0][i0], triangle[0][i1]),
        Coordinate2D::new_(triangle[1][i0], triangle[1][i1]),
        Coordinate2D::new_(triangle[2][i0], triangle[2][i1]),
    ) || point_in_triangle_2d(
        Coordinate2D::new_(p1[i0], p1[i1]),
        Coordinate2D::new_(triangle[0][i0], triangle[0][i1]),
        Coordinate2D::new_(triangle[1][i0], triangle[1][i1]),
        Coordinate2D::new_(triangle[2][i0], triangle[2][i1]),
    )
}

fn point_in_triangle_2d<T: Float + CoordNum>(
    p: Coordinate2D<T>,
    v0: Coordinate2D<T>,
    v1: Coordinate2D<T>,
    v2: Coordinate2D<T>,
) -> bool {
    let sign = |p1: Coordinate2D<T>, p2: Coordinate2D<T>, p3: Coordinate2D<T>| -> T {
        (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
    };

    let d1 = sign(p, v0, v1);
    let d2 = sign(p, v1, v2);
    let d3 = sign(p, v2, v0);

    let zero = T::zero();

    let has_neg = (d1 < zero) || (d2 < zero) || (d3 < zero);
    let has_pos = (d1 > zero) || (d2 > zero) || (d3 > zero);

    !(has_neg && has_pos)
}

fn segments_intersect_2d<T: Float + CoordNum>(
    p0: Coordinate2D<T>,
    p1: Coordinate2D<T>,
    q0: Coordinate2D<T>,
    q1: Coordinate2D<T>,
) -> bool {
    let epsilon = T::from(1e-10).unwrap();
    let d1 = (q1.x - q0.x) * (p0.y - q0.y) - (q1.y - q0.y) * (p0.x - q0.x);
    let d2 = (q1.x - q0.x) * (p1.y - q0.y) - (q1.y - q0.y) * (p1.x - q0.x);
    let d3 = (p1.x - p0.x) * (q0.y - p0.y) - (p1.y - p0.y) * (q0.x - p0.x);
    let d4 = (p1.x - p0.x) * (q1.y - p0.y) - (p1.y - p0.y) * (q1.x - p0.x);

    if d1 * d2 < T::zero() && d3 * d4 < T::zero() {
        return true;
    }

    // Check for collinear segments
    if d1.abs() < epsilon && d2.abs() < epsilon {
        // Check if segments overlap
        let min_px = p0.x.min(p1.x);
        let max_px = p0.x.max(p1.x);
        let min_py = p0.y.min(p1.y);
        let max_py = p0.y.max(p1.y);
        let min_qx = q0.x.min(q1.x);
        let max_qx = q0.x.max(q1.x);
        let min_qy = q0.y.min(q1.y);
        let max_qy = q0.y.max(q1.y);

        return max_px >= min_qx && max_qx >= min_px && max_py >= min_qy && max_qy >= min_py;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_segment_intersects_triangle_direct_hit() {
        let triangle = [
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(2.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 2.0, 0.0),
        ];

        // Segment that passes through the triangle
        let p0 = Coordinate3D::new__(1.0, 0.5, -1.0);
        let p1 = Coordinate3D::new__(1.0, 0.5, 1.0);
        let line = Line3D::<f64>::new_(p0, p1);

        assert!(segment_intersects_triangle(&line, &triangle, 1e-10));
    }

    #[test]
    fn test_segment_intersects_triangle_miss() {
        let triangle = [
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 0.0, 0.0),
            Coordinate3D::new__(0.0, 1.0, 0.0),
        ];

        // Segment that misses the triangle
        let p0 = Coordinate3D::new__(2.0, 0.0, 0.0);
        let p1 = Coordinate3D::new__(2.0, 1.0, 0.0);
        let line = Line3D::<f64>::new_(p0, p1);

        assert!(!segment_intersects_triangle(&line, &triangle, 1e-10));
    }

    #[test]
    fn test_segment_intersects_triangle_parallel() {
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

        assert!(!segment_intersects_triangle(&line, &triangle, 1e-10));
    }

    #[test]
    fn test_segment_intersects_triangle_edge_case() {
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
        assert!(segment_intersects_triangle(&line, &triangle, 1e-10));
    }
}
