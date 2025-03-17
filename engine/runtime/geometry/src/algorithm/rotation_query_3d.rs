use nalgebra::Vector3;

use crate::types::point::Point3D;

/// A query to rotate a point in 3D space.
#[derive(Debug, Clone)]
pub struct RotationQuery3D {
    pub degrees: f64,
    pub direction: Point3D<f64>,
    pub origin: Option<Point3D<f64>>,
}

impl RotationQuery3D {
    /// Creates a new `RotationQuery3D` which rotates a vector from `from` to `to`.
    /// This returns `None` if any of the vectors is a zero vector.
    /// Note that if two vectors are same or opposite, the rotation angle will be 0.
    pub fn from_vectors(
        from: Point3D<f64>,
        to: Point3D<f64>,
        origin: Option<Point3D<f64>>,
    ) -> Option<Self> {
        // let a = Vector3::new(from.x(), from.y(), from.z());
        // let b = Vector3::new(to.x(), to.y(), to.z());
        let (a, b) = if let Some(origin) = origin {
            (
                Vector3::new(
                    from.x() - origin.x(),
                    from.y() - origin.y(),
                    from.z() - origin.z(),
                ),
                Vector3::new(
                    to.x() - origin.x(),
                    to.y() - origin.y(),
                    to.z() - origin.z(),
                ),
            )
        } else {
            (
                Vector3::new(from.x(), from.y(), from.z()),
                Vector3::new(to.x(), to.y(), to.z()),
            )
        };

        let a = normalize_vector(a)?;
        let b = normalize_vector(b)?;

        let c = a.dot(&b);

        let v = a.cross(&b);
        let normalized_v = if let Some(v) = normalize_vector(v) {
            v
        } else {
            return Some(Self {
                degrees: 0.0,
                direction: Point3D::new(1.0, 0.0, 0.0),
                origin,
            });
        };

        Some(Self {
            degrees: c.acos().to_degrees(),
            direction: Point3D::new(normalized_v.x, normalized_v.y, normalized_v.z),
            origin,
        })
    }
}

fn normalize_vector(v: Vector3<f64>) -> Option<Vector3<f64>> {
    let norm = v.norm();
    if norm == 0.0 {
        None
    } else {
        Some(v / norm)
    }
}

#[cfg(test)]
mod tests {
    use crate::algorithm::rotate_3d::Rotate3D;

    use super::*;
    use approx::assert_relative_eq;

    // helper function to rotate a point around an direction
    fn rotate(rotation_query: &RotationQuery3D, point: Point3D<f64>) -> Point3D<f64> {
        point.rotate_3d(
            rotation_query.degrees,
            rotation_query.origin,
            rotation_query.direction,
        )
    }

    #[test]
    fn test_rotation_matrix_collinear() {
        let from = Point3D::new(0.0, 0.0, 1.0);
        let to = Point3D::new(0.0, 0.0, -1.0);
        assert_eq!(
            RotationQuery3D::from_vectors(from, to, None)
                .unwrap()
                .degrees,
            0.0
        );
    }

    #[test]
    fn test_rotation_matrix_perpendicular() {
        let from = Point3D::new(0.0, 0.0, 1.0);
        let to = Point3D::new(1.0, 0.0, 0.0);

        let rotation_query = RotationQuery3D::from_vectors(from, to, None).unwrap();

        let point = Point3D::new(0.0, 0.0, 1.0);
        let rotated = rotate(&rotation_query, point);

        assert_relative_eq!(rotated.x(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.z(), 0.0, epsilon = 1e-10);

        let point = Point3D::new(1.0, 0.0, 0.0);
        let rotated = rotate(&rotation_query, point);

        assert_relative_eq!(rotated.x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.z(), -1.0, epsilon = 1e-10);

        let point = Point3D::new(0.0, 1.0, 0.0);
        let rotated = rotate(&rotation_query, point);

        assert_relative_eq!(rotated.x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.y(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.z(), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_rotation_matrix_arbitrary() {
        let from = Point3D::new(0.0, 0.0, 1.0);
        let to = Point3D::new(1.0, 1.0, 1.0);

        let rotation_query = RotationQuery3D::from_vectors(from, to, None).unwrap();

        let point = Point3D::new(0.0, 0.0, 1.0);
        let rotated = rotate(&rotation_query, point);

        let expected_norm = (3.0_f64).sqrt().recip(); // 1/√3
        assert_relative_eq!(rotated.x(), expected_norm, epsilon = 1e-10);
        assert_relative_eq!(rotated.y(), expected_norm, epsilon = 1e-10);
        assert_relative_eq!(rotated.z(), expected_norm, epsilon = 1e-10);
    }

    #[test]
    fn test_rotation_origin() {
        let from = Point3D::new(1.0, 1.0, 2.0);
        let to = Point3D::new(2.0, 2.0, 2.0);
        let origin = Point3D::new(1.0, 1.0, 1.0);

        let rotation_query = RotationQuery3D::from_vectors(from, to, Some(origin)).unwrap();

        let point = Point3D::new(1.0, 1.0, 2.0);
        let rotated = rotate(&rotation_query, point);

        let expected_norm = (3.0_f64).sqrt().recip() + 1.0; // 1/√3 + 1
        assert_relative_eq!(rotated.x(), expected_norm, epsilon = 1e-10);
        assert_relative_eq!(rotated.y(), expected_norm, epsilon = 1e-10);
        assert_relative_eq!(rotated.z(), expected_norm, epsilon = 1e-10);
    }
}
