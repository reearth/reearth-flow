use nalgebra::Vector3;

use crate::types::{coordinate::Coordinate3D, point::Point3D};

use super::geo_distance_converter::coordinate_diff_to_meter;

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

pub fn compute_normal_3d_from_coords(
    a: Coordinate3D<f64>,
    b: Coordinate3D<f64>,
    c: Coordinate3D<f64>,
    origin: Coordinate3D<f64>,
    normalize: bool,
) -> Option<Point3D<f64>> {
    let (a_x, a_y) =
        coordinate_diff_to_meter(a.x - origin.x, a.y - origin.y, (a.y + origin.y) / 2.0);
    let (b_x, b_y) =
        coordinate_diff_to_meter(b.x - origin.x, b.y - origin.y, (b.y + origin.y) / 2.0);
    let (c_x, c_y) =
        coordinate_diff_to_meter(c.x - origin.x, c.y - origin.y, (c.y + origin.y) / 2.0);

    let a = Point3D::new(a_x, a_y, a.z);
    let b = Point3D::new(b_x, b_y, b.z);
    let c = Point3D::new(c_x, c_y, c.z);

    compute_normal_3d(a, b, c, normalize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

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
