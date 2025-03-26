use nalgebra::Vector3;

use crate::types::point::Point3D;

use super::geo_distance_converter::coordinate_diff_to_meter;

/// Compute the normal vector of a plane defined by three points in 3D space.
pub fn compute_normal_3d(
    a: Point3D<f64>,
    b: Point3D<f64>,
    c: Point3D<f64>,
    normalize: bool,
) -> Option<Point3D<f64>> {
    let a_vec = Vector3::new(a.x(), a.y(), a.z());
    let b_vec = Vector3::new(b.x(), b.y(), b.z());
    let c_vec = Vector3::new(c.x(), c.y(), c.z());

    let ab = b_vec - a_vec;
    let ac = c_vec - a_vec;

    let normal = ab.cross(&ac);

    // Check if the normal is a zero vector (points are collinear)
    if normal.norm() == 0.0 {
        return None;
    }

    let result = if normalize {
        let normalized = normal.normalize();
        Point3D::new(normalized.x, normalized.y, normalized.z)
    } else {
        Point3D::new(normal.x, normal.y, normal.z)
    };

    Some(result)
}

/// Check if a set of points are on the same plane defined by a normal vector.
fn are_points_on_same_normal_plane(
    normal: Point3D<f64>,
    points: &[Point3D<f64>],
    epsilon: f64,
) -> bool {
    if points.len() <= 1 {
        return true;
    }

    let normal_vec = Vector3::new(normal.x(), normal.y(), normal.z());

    let first_point = Vector3::new(points[0].x(), points[0].y(), points[0].z());
    let reference_dot = normal_vec.dot(&first_point);

    for point in &points[1..] {
        let point_vec = Vector3::new(point.x(), point.y(), point.z());
        let dot_product = normal_vec.dot(&point_vec);

        if (dot_product - reference_dot).abs() > epsilon {
            return false;
        }
    }

    true
}

/// Compute the normal vector of a plane defined by three points in 3D space.
pub fn compute_normal_3d_from_coords(
    points: Vec<Point3D<f64>>,
    origin: Point3D<f64>,
    normalize: bool,
    epsilon: f64,
) -> Option<Point3D<f64>> {
    if points.len() < 3 {
        return None;
    }

    let points_geometry = points
        .iter()
        .map(|p| {
            let (x, y) = coordinate_diff_to_meter(
                p.x() - origin.x(),
                p.y() - origin.y(),
                (p.y() + origin.y()) / 2.0,
            );

            Point3D::new(x, y, p.z())
        })
        .collect::<Vec<Point3D<f64>>>();

    let a = points_geometry[0];
    let b = points_geometry[1];
    let c = points_geometry[2];

    let normal = compute_normal_3d(a, b, c, normalize)?;

    if are_points_on_same_normal_plane(normal, &points_geometry, epsilon) {
        Some(normal)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_are_points_on_same_normal_plane_xy_plane() {
        let normal = Point3D::new(0.0, 0.0, 1.0);
        let points = vec![
            Point3D::new(0.0, 0.0, 5.0),
            Point3D::new(1.0, 0.0, 5.0),
            Point3D::new(0.0, 1.0, 5.0),
            Point3D::new(1.0, 1.0, 5.0),
        ];
        assert!(are_points_on_same_normal_plane(normal, &points, 1e-10));

        let normal = Point3D::new(0.0, 0.0, -1.0);
        assert!(are_points_on_same_normal_plane(normal, &points, 1e-10));

        let normal = Point3D::new(0.0, 1.0, 0.0);
        assert!(!are_points_on_same_normal_plane(normal, &points, 1e-10));
    }

    #[test]
    fn test_are_points_on_same_normal_plane_different_planes() {
        let normal = Point3D::new(0.0, 0.0, 1.0);
        let points = vec![
            Point3D::new(0.0, 0.0, 5.0),
            Point3D::new(1.0, 0.0, 5.0),
            Point3D::new(0.0, 1.0, 6.0),
            Point3D::new(1.0, 1.0, 5.0),
        ];

        assert!(!are_points_on_same_normal_plane(normal, &points, 1e-10));
    }

    #[test]
    fn test_are_points_on_same_normal_plane_arbitrary_normal() {
        let normal = Point3D::new(1.0, 1.0, 1.0);

        let points = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, -1.0, 0.0),
            Point3D::new(1.0, 0.0, -1.0),
            Point3D::new(0.0, 1.0, -1.0),
        ];

        assert!(are_points_on_same_normal_plane(normal, &points, 1e-10));
    }

    #[test]
    fn test_compute_normal_3d_basic() {
        // Points on the XY plane should have a normal in the Z direction
        let a = Point3D::new(0.0, 0.0, 0.0);
        let b = Point3D::new(1.0, 0.0, 0.0);
        let c = Point3D::new(0.0, 1.0, 0.0);

        let normal = compute_normal_3d(a, b, c, true).unwrap();
        assert_relative_eq!(normal.x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(normal.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(normal.z(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_compute_normal_3d_without_normalization() {
        // Test without normalization
        let a = Point3D::new(0.0, 0.0, 0.0);
        let b = Point3D::new(2.0, 0.0, 0.0);
        let c = Point3D::new(0.0, 2.0, 0.0);

        let normal = compute_normal_3d(a, b, c, false).unwrap();
        assert_relative_eq!(normal.x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(normal.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(normal.z(), 4.0, epsilon = 1e-10);
    }

    #[test]
    fn test_compute_normal_3d_with_normalization() {
        // Same test but with normalization
        let a = Point3D::new(0.0, 0.0, 0.0);
        let b = Point3D::new(2.0, 0.0, 0.0);
        let c = Point3D::new(0.0, 2.0, 0.0);

        let normal = compute_normal_3d(a, b, c, true).unwrap();
        assert_relative_eq!(normal.x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(normal.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(normal.z(), 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_compute_normal_3d_arbitrary_plane() {
        // Points on an arbitrary plane
        let a = Point3D::new(1.0, 0.0, 0.0);
        let b = Point3D::new(0.0, 1.0, 0.0);
        let c = Point3D::new(0.0, 0.0, 1.0);

        let normal = compute_normal_3d(a, b, c, true).unwrap();
        let expected_norm = (3.0_f64).sqrt().recip(); // 1/√3
        assert_relative_eq!(normal.x(), expected_norm, epsilon = 1e-10);
        assert_relative_eq!(normal.y(), expected_norm, epsilon = 1e-10);
        assert_relative_eq!(normal.z(), expected_norm, epsilon = 1e-10);
    }

    #[test]
    fn test_compute_normal_3d_collinear_points() {
        // Collinear points should return None
        let a = Point3D::new(0.0, 0.0, 0.0);
        let b = Point3D::new(1.0, 1.0, 1.0);
        let c = Point3D::new(2.0, 2.0, 2.0);

        let normal = compute_normal_3d(a, b, c, true);
        assert!(normal.is_none());
    }
}
