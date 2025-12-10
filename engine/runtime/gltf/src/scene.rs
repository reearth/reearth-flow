use nalgebra::{Matrix4, UnitQuaternion, Vector3};
use reearth_flow_geometry::types::coordinate::Coordinate;

/// A 3D transformation that can be applied to coordinates.
/// Wraps a nalgebra 4x4 homogeneous transformation matrix.
#[derive(Debug, Clone)]
pub struct Transform {
    matrix: Matrix4<f64>,
}

impl Transform {
    /// Creates an identity transform (no transformation)
    pub fn identity() -> Self {
        Self {
            matrix: Matrix4::identity(),
        }
    }

    /// Creates a transform from a glTF node
    pub fn from_node(node: &gltf::Node) -> Self {
        match node.transform() {
            gltf::scene::Transform::Matrix { matrix } => Self::from_matrix_f32(&matrix),
            gltf::scene::Transform::Decomposed {
                translation,
                rotation,
                scale,
            } => Self::from_trs(translation, rotation, scale),
        }
    }

    /// Creates a transform from a 4x4 matrix (column-major f32)
    fn from_matrix_f32(matrix: &[[f32; 4]; 4]) -> Self {
        let matrix = Matrix4::new(
            matrix[0][0] as f64,
            matrix[1][0] as f64,
            matrix[2][0] as f64,
            matrix[3][0] as f64,
            matrix[0][1] as f64,
            matrix[1][1] as f64,
            matrix[2][1] as f64,
            matrix[3][1] as f64,
            matrix[0][2] as f64,
            matrix[1][2] as f64,
            matrix[2][2] as f64,
            matrix[3][2] as f64,
            matrix[0][3] as f64,
            matrix[1][3] as f64,
            matrix[2][3] as f64,
            matrix[3][3] as f64,
        );
        Self { matrix }
    }

    /// Creates a transform from Translation-Rotation-Scale decomposition
    fn from_trs(translation: [f32; 3], rotation: [f32; 4], scale: [f32; 3]) -> Self {
        // glTF quaternion is [x, y, z, w]
        let quat = UnitQuaternion::from_quaternion(nalgebra::Quaternion::new(
            rotation[3] as f64, // w
            rotation[0] as f64, // x
            rotation[1] as f64, // y
            rotation[2] as f64, // z
        ));

        let translation = Vector3::new(
            translation[0] as f64,
            translation[1] as f64,
            translation[2] as f64,
        );

        let scale_vec = Vector3::new(scale[0] as f64, scale[1] as f64, scale[2] as f64);

        // Build TRS matrix: M = T * R * S
        let mut matrix = Matrix4::identity();
        matrix
            .fixed_view_mut::<3, 3>(0, 0)
            .copy_from(quat.to_rotation_matrix().matrix());
        matrix.fixed_view_mut::<3, 1>(0, 3).copy_from(&translation);

        // Apply scale to the rotation part
        for i in 0..3 {
            matrix[(0, i)] *= scale_vec[i];
            matrix[(1, i)] *= scale_vec[i];
            matrix[(2, i)] *= scale_vec[i];
        }

        Self { matrix }
    }

    /// Checks if this is an identity transform (within floating point tolerance)
    pub fn is_identity(&self) -> bool {
        const EPSILON: f64 = 1e-10;
        self.matrix
            .relative_eq(&Matrix4::identity(), EPSILON, EPSILON)
    }

    /// Applies this transform to a coordinate
    pub fn apply(&self, coord: &Coordinate) -> Coordinate {
        if self.is_identity() {
            return *coord;
        }

        let point = nalgebra::Point3::new(coord.x, coord.y, coord.z);
        let transformed = self.matrix.transform_point(&point);

        Coordinate {
            x: transformed.x,
            y: transformed.y,
            z: transformed.z,
        }
    }

    /// Composes this transform with a parent transform
    /// Returns: parent * self (parent applied after self)
    pub fn compose(&self, parent: &Transform) -> Transform {
        Self {
            matrix: parent.matrix * self.matrix,
        }
    }
}

/// Traverses the scene graph and calls the callback for each node.
/// Callback receives: (node, accumulated_world_transform)
pub fn traverse_scene<'a, F, E>(scene: &gltf::Scene<'a>, mut callback: F) -> Result<(), E>
where
    F: FnMut(&gltf::Node<'a>, &Transform) -> Result<(), E>,
{
    for root_node in scene.nodes() {
        let root_transform = Transform::from_node(&root_node);
        traverse_node(&root_node, &root_transform, &mut callback)?;
    }
    Ok(())
}

fn traverse_node<'a, F, E>(
    node: &gltf::Node<'a>,
    world_transform: &Transform,
    callback: &mut F,
) -> Result<(), E>
where
    F: FnMut(&gltf::Node<'a>, &Transform) -> Result<(), E>,
{
    // Call callback for this node
    callback(node, world_transform)?;

    // Recursively traverse children with accumulated transforms
    for child in node.children() {
        let child_local = Transform::from_node(&child);
        let child_world = child_local.compose(world_transform);
        traverse_node(&child, &child_world, callback)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_transform() {
        let transform = Transform::identity();
        assert!(transform.is_identity());

        let coord = Coordinate {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let result = transform.apply(&coord);

        assert_eq!(result.x, coord.x);
        assert_eq!(result.y, coord.y);
        assert_eq!(result.z, coord.z);
    }

    #[test]
    fn test_translation_transform() {
        let transform =
            Transform::from_trs([10.0, 20.0, 30.0], [0.0, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0]);

        let coord = Coordinate {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let result = transform.apply(&coord);

        assert!((result.x - 11.0).abs() < 1e-10);
        assert!((result.y - 22.0).abs() < 1e-10);
        assert!((result.z - 33.0).abs() < 1e-10);
    }

    #[test]
    fn test_scale_transform() {
        let transform = Transform::from_trs([0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0], [2.0, 3.0, 4.0]);

        let coord = Coordinate {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let result = transform.apply(&coord);

        assert!((result.x - 2.0).abs() < 1e-10);
        assert!((result.y - 6.0).abs() < 1e-10);
        assert!((result.z - 12.0).abs() < 1e-10);
    }

    #[test]
    fn test_compose_transforms() {
        let translate =
            Transform::from_trs([10.0, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0]);
        let scale = Transform::from_trs([0.0, 0.0, 0.0], [0.0, 0.0, 0.0, 1.0], [2.0, 2.0, 2.0]);

        // compose() semantics: parent * self (parent applied after self)
        // So translate.compose(&scale) applies translate first, then scale
        let composed = translate.compose(&scale);

        let coord = Coordinate {
            x: 1.0,
            y: 2.0,
            z: 3.0,
        };
        let result = composed.apply(&coord);

        // Translate first, then scale: (1+10)*2, (2+0)*2, (3+0)*2
        assert!((result.x - 22.0).abs() < 1e-10, "x: {} != 22.0", result.x);
        assert!((result.y - 4.0).abs() < 1e-10, "y: {} != 4.0", result.y);
        assert!((result.z - 6.0).abs() < 1e-10, "z: {} != 6.0", result.z);
    }
}
