use nalgebra::Vector3;

use crate::types::{coordinate::Coordinate3D, point::Point3D};

use super::geo_distance_converter::coordinate_diff_to_meter;

/// A query to rotate a point in 3D space.
#[derive(Debug, Clone)]
pub struct RotationQuery3D {
    pub angle_degrees: f64,
    pub direction: Point3D<f64>,
    rotation: nalgebra::Rotation3<f64>,
}

impl RotationQuery3D {
    /// Creates a new `RotationQuery3D` which rotates a vector from `from` to `to`.
    /// This returns `None` if any of the vectors is a zero vector.
    /// Note that if two vectors are same or opposite, the rotation angle will be 0.
    pub fn from_vectors(from: Point3D<f64>, to: Point3D<f64>) -> Option<Self> {
        let a = Vector3::new(from.x(), from.y(), from.z());
        let b = Vector3::new(to.x(), to.y(), to.z());

        let a = normalize_vector(a)?;
        let b = normalize_vector(b)?;

        let c = a.dot(&b);

        let v = a.cross(&b);
        let normalized_v = if let Some(v) = normalize_vector(v) {
            v
        } else {
            return Some(Self {
                angle_degrees: 0.0,
                direction: Point3D::new(1.0, 0.0, 0.0),
                rotation: nalgebra::Rotation3::identity(),
            });
        };

        let angle_degrees = c.acos().to_degrees();

        // Origin of rotation
        let angle_degrees = angle_degrees.to_radians();
        // Rotational axis vector
        let direction = nalgebra::Vector3::new(normalized_v.x, normalized_v.y, normalized_v.z);

        // Create a rotation matrix around the rotation axis.
        let rotation = nalgebra::Rotation3::from_axis_angle(
            &nalgebra::Unit::new_normalize(direction),
            angle_degrees,
        );

        Some(Self {
            angle_degrees,
            direction: Point3D::new(normalized_v.x, normalized_v.y, normalized_v.z),
            rotation,
        })
    }

    pub fn from_coordinates(
        from_coords: Coordinate3D<f64>,
        to_coords: Coordinate3D<f64>,
        origin: Option<Coordinate3D<f64>>,
    ) -> Option<Self> {
        let diff_from = from_coords - origin.unwrap_or(Coordinate3D::new__(0.0, 0.0, 0.0));
        let diff_to = to_coords - origin.unwrap_or(Coordinate3D::new__(0.0, 0.0, 0.0));

        let (from_x, from_y) = coordinate_diff_to_meter(diff_from.x, diff_from.y, from_coords.y);
        let (to_x, to_y) = coordinate_diff_to_meter(diff_to.x, diff_to.y, to_coords.y);

        let from = Point3D::new(from_x, from_y, diff_from.z);
        let to = Point3D::new(to_x, to_y, diff_to.z);

        Self::from_vectors(from, to)
    }

    fn rotate(&self, point: Point3D<f64>, origin: Option<Point3D<f64>>) -> Point3D<f64> {
        let origin = origin
            .map(|p| nalgebra::Vector3::new(p.x(), p.y(), p.z()))
            .unwrap_or(nalgebra::Vector3::new(0.0, 0.0, 0.0));

        let point = nalgebra::Point3::new(point.x(), point.y(), point.z());
        let translated_point = point - origin;
        let rotated_point = self.rotation * translated_point + origin;

        Point3D::new(rotated_point.x, rotated_point.y, rotated_point.z)
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
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_rotation_matrix_collinear() {
        let from = Point3D::new(0.0, 0.0, 1.0);
        let to = Point3D::new(0.0, 0.0, -1.0);
        assert_eq!(
            RotationQuery3D::from_vectors(from, to)
                .unwrap()
                .angle_degrees,
            0.0
        );
    }

    #[test]
    fn test_rotation_matrix_perpendicular() {
        let from = Point3D::new(0.0, 0.0, 1.0);
        let to = Point3D::new(1.0, 0.0, 0.0);

        let rotation_query = RotationQuery3D::from_vectors(from, to).unwrap();

        let point = Point3D::new(0.0, 0.0, 1.0);
        let rotated = rotation_query.rotate(point, None);

        assert_relative_eq!(rotated.x(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.z(), 0.0, epsilon = 1e-10);

        let point = Point3D::new(1.0, 0.0, 0.0);
        let rotated = rotation_query.rotate(point, None);

        assert_relative_eq!(rotated.x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.y(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.z(), -1.0, epsilon = 1e-10);

        let point = Point3D::new(0.0, 1.0, 0.0);
        let rotated = rotation_query.rotate(point, None);

        assert_relative_eq!(rotated.x(), 0.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.y(), 1.0, epsilon = 1e-10);
        assert_relative_eq!(rotated.z(), 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_rotation_matrix_arbitrary() {
        let from = Point3D::new(0.0, 0.0, 1.0);
        let to = Point3D::new(1.0, 1.0, 1.0);

        let rotation_query = RotationQuery3D::from_vectors(from, to).unwrap();

        let point = Point3D::new(0.0, 0.0, 1.0);
        let rotated = rotation_query.rotate(point, None);

        let expected_norm = (3.0_f64).sqrt().recip(); // 1/√3
        assert_relative_eq!(rotated.x(), expected_norm, epsilon = 1e-10);
        assert_relative_eq!(rotated.y(), expected_norm, epsilon = 1e-10);
        assert_relative_eq!(rotated.z(), expected_norm, epsilon = 1e-10);
    }

    // #[test]
    // fn test_rotation_origin() {
    //     let from = Point3D::new(1.0, 1.0, 2.0);
    //     let to = Point3D::new(2.0, 2.0, 2.0);
    //     let origin = Coordinate3D::new_(1.0, 1.0, 1.0);

    //     let rotation_query = RotationQuery3D::from_vectors(from, to, Some(origin)).unwrap();

    //     let point = Point3D::new(1.0, 1.0, 2.0);
    //     let rotated = rotate(&rotation_query, point);

    //     let expected_norm = (3.0_f64).sqrt().recip() + 1.0; // 1/√3 + 1
    //     assert_relative_eq!(rotated.x(), expected_norm, epsilon = 1e-10);
    //     assert_relative_eq!(rotated.y(), expected_norm, epsilon = 1e-10);
    //     assert_relative_eq!(rotated.z(), expected_norm, epsilon = 1e-10);
    // }
}
