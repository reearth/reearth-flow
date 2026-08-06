//! Applying a caller-supplied 3D affine to a geometry's coordinates and setting
//! its coordinate frame. Policy-free: it applies whatever affine it is handed,
//! so all geospatial decisions stay in the calling action.

use crate::coordinate::CoordinateFrame;

/// A 3D affine transform: a row-major 3x3 rotation followed by a translation.
#[derive(Debug, Clone, PartialEq)]
pub struct Affine3 {
    rotation: [[f64; 3]; 3],
    translation: [f64; 3],
}

impl Affine3 {
    /// Build from a row-major rotation and a translation.
    pub fn new(rotation: [[f64; 3]; 3], translation: [f64; 3]) -> Self {
        Self {
            rotation,
            translation,
        }
    }

    /// The transform that leaves coordinates unchanged.
    pub fn identity() -> Self {
        Self::new(
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            [0.0; 3],
        )
    }

    /// Rotate then translate `p`.
    pub fn apply(&self, p: [f64; 3]) -> [f64; 3] {
        let r = &self.rotation;
        [
            r[0][0] * p[0] + r[0][1] * p[1] + r[0][2] * p[2] + self.translation[0],
            r[1][0] * p[0] + r[1][1] * p[1] + r[1][2] * p[2] + self.translation[1],
            r[2][0] * p[0] + r[2][1] * p[1] + r[2][2] * p[2] + self.translation[2],
        ]
    }

    /// `self ∘ inner`: the transform that applies `inner` first, then `self`.
    pub fn compose(&self, inner: &Affine3) -> Affine3 {
        let mut rotation = [[0.0; 3]; 3];
        for (i, row) in rotation.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                *cell = (0..3)
                    .map(|k| self.rotation[i][k] * inner.rotation[k][j])
                    .sum();
            }
        }
        Affine3 {
            rotation,
            translation: self.apply(inner.translation),
        }
    }
}

/// Apply `affine` to every coordinate in `coords`, in place.
pub(crate) fn apply_affine_3d(coords: &mut [[f64; 3]], affine: &Affine3) {
    for c in coords.iter_mut() {
        *c = affine.apply(*c);
    }
}

/// Apply an affine to a geometry's coordinates and set its coordinate frame.
#[enum_dispatch::enum_dispatch]
pub trait Place {
    fn place(&mut self, affine: &Affine3, frame: &CoordinateFrame) -> crate::error::Result<()>;
}

// The boxed enum variants (`Box<Polygon3D>`, `Box<Solid>`, …) need the trait on
// the `Box` itself: `enum_dispatch` forwards by UFCS, not auto-deref.
impl<T: Place + ?Sized> Place for Box<T> {
    fn place(&mut self, affine: &Affine3, frame: &CoordinateFrame) -> crate::error::Result<()> {
        (**self).place(affine, frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Row-major matrix that maps (x, y, z) -> (x, -z, y): the Y-up to Z-up flip.
    fn y_up_to_z_up() -> [[f64; 3]; 3] {
        [[1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]]
    }

    #[test]
    fn identity_leaves_a_point_unchanged() {
        assert_eq!(Affine3::identity().apply([1.0, 2.0, 3.0]), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn rotation_and_translation_are_applied_in_order() {
        let a = Affine3::new(y_up_to_z_up(), [10.0, 20.0, 30.0]);
        // (1,2,3) -> rotate -> (1,-3,2) -> translate -> (11,17,32)
        assert_eq!(a.apply([1.0, 2.0, 3.0]), [11.0, 17.0, 32.0]);
    }

    #[test]
    fn compose_applies_inner_first() {
        let flip = Affine3::new(y_up_to_z_up(), [0.0; 3]);
        let shift = Affine3::new(
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            [1.0, 0.0, 0.0],
        );
        // shift ∘ flip: flip first, then shift.
        assert_eq!(
            shift.compose(&flip).apply([1.0, 2.0, 3.0]),
            [2.0, -3.0, 2.0]
        );
    }

    #[test]
    fn apply_affine_3d_rewrites_every_coordinate() {
        let mut coords = [[1.0, 2.0, 3.0], [0.0, 0.0, 0.0]];
        apply_affine_3d(&mut coords, &Affine3::new(y_up_to_z_up(), [1.0, 1.0, 1.0]));
        assert_eq!(coords, [[2.0, -2.0, 3.0], [1.0, 1.0, 1.0]]);
    }

    #[test]
    fn placing_a_triangular_mesh_rewrites_vertices_and_sets_the_frame() {
        use crate::coordinate::CoordinateFrame;
        use crate::triangular_mesh::TriangularMesh3D;
        use crate::EpsgCode;

        let mut mesh = TriangularMesh3D::from_soup(
            CoordinateFrame::Euclidean,
            vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]],
        );
        let target = CoordinateFrame::Crs(EpsgCode::new(4978));
        mesh.place(&Affine3::new(y_up_to_z_up(), [0.0; 3]), &target)
            .unwrap();

        assert_eq!(*mesh.frame(), target, "frame is set to the target");
        assert!(
            mesh.vertices().contains(&[1.0, -3.0, 2.0]),
            "vertices were rotated Y-up to Z-up: {:?}",
            mesh.vertices()
        );
    }

    #[test]
    fn placing_2d_geometry_is_an_error() {
        // `Euclidean2DGeometry` derives no `Default`, so build a concrete leaf;
        // the error comes from the embedding, not the leaf's value.
        use crate::coordinate::CoordinateFrame;
        use crate::point::Point2D;
        use crate::{Euclidean2DGeometry, Geometry};

        let mut g = Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
            CoordinateFrame::Euclidean,
            [0.0, 0.0],
        )));
        assert!(g
            .place(&Affine3::identity(), &CoordinateFrame::Euclidean)
            .is_err());
    }
}
