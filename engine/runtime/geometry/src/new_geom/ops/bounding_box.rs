//! `BoundingBox`: a coord-free read operation.

use super::UnsupportedOperation;

/// Axis-aligned bounding box in the geometry's own coordinate frame.
#[derive(Clone, Debug, PartialEq)]
pub struct Aabb {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

impl Aabb {
    pub fn point(x: f64, y: f64, z: f64) -> Self {
        Aabb {
            min: [x, y, z],
            max: [x, y, z],
        }
    }

    /// Grow `acc` to include `p`, seeding it if empty.
    pub fn extend(acc: &mut Option<Aabb>, p: [f64; 3]) {
        match acc {
            None => *acc = Some(Aabb { min: p, max: p }),
            Some(a) => {
                for ((mn, mx), &c) in a.min.iter_mut().zip(a.max.iter_mut()).zip(p.iter()) {
                    *mn = mn.min(c);
                    *mx = mx.max(c);
                }
            }
        }
    }
}

/// Coordinate-free operation: every leaf computes its box from its own
/// coordinates. The default returns `UnsupportedOperation` so a leaf that does
/// not support it needs only an (empty) impl block; see [`crate::new_geom`].
#[enum_dispatch::enum_dispatch]
pub trait BoundingBox {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        Err(UnsupportedOperation {
            geometry: core::any::type_name::<Self>(),
            operation: "bounding_box",
        })
    }
}
