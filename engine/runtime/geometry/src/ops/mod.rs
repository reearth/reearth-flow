//! Operation traits for the geometry types.
//!
//! Each operation is a separate trait carrying a default method that returns
//! [`UnsupportedOperation`]. A leaf opts in by overriding the method (in its
//! `{type}/ops.rs`), and opts out with an empty `impl` block, stamped by the
//! [`unsupported!`](crate::unsupported) macro. The traits are
//! `#[enum_dispatch]`, so a call on `Geometry` / `Euclidean{2,3}DGeometry`
//! chains through to the concrete leaf. `GeometryCollection` and the per-frame
//! `Collection`s recurse by hand over their children.

pub mod triangulation;
pub mod reproject;

/// Returned by an operation a given geometry type does not support. Carries the
/// concrete type name (via [`type_name`](core::any::type_name)) and the
/// operation name for diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnsupportedOperation {
    /// The concrete geometry type the operation was called on.
    pub geometry: &'static str,
    /// The operation that is not supported.
    pub operation: &'static str,
}

impl core::fmt::Display for UnsupportedOperation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "`{}` is not supported by `{}`",
            self.operation, self.geometry
        )
    }
}

impl std::error::Error for UnsupportedOperation {}

/// An axis-aligned bounding box, in the geometry's own embedding and coordinate
/// frame.
///
/// The variant mirrors the geometry's embedding: a 2D-embedded geometry yields
/// [`Aabb::D2`], a 3D-embedded one yields [`Aabb::D3`]. The optional 2.5D
/// elevation a 2D leaf may carry is not folded into the box. The box stays
/// planar, matching the 2D embedding.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Aabb {
    /// Box of a 2D-embedded geometry.
    D2 {
        /// Per-axis minimum `(x, y)`.
        min: [f64; 2],
        /// Per-axis maximum `(x, y)`.
        max: [f64; 2],
    },
    /// Box of a 3D-embedded geometry.
    D3 {
        /// Per-axis minimum `(x, y, z)`.
        min: [f64; 3],
        /// Per-axis maximum `(x, y, z)`.
        max: [f64; 3],
    },
}

impl Aabb {
    /// A degenerate box at a single 2D point.
    pub fn point_2d(p: [f64; 2]) -> Self {
        Aabb::D2 { min: p, max: p }
    }

    /// A degenerate box at a single 3D point.
    pub fn point_3d(p: [f64; 3]) -> Self {
        Aabb::D3 { min: p, max: p }
    }

    /// The box of a set of 2D points, or `None` if the iterator is empty.
    ///
    /// `f64::min` / `f64::max` ignore `NaN`, so non-finite coordinates never
    /// widen the box past finite neighbours (a fully-`NaN` axis stays `NaN`).
    pub fn from_points_2d(points: impl IntoIterator<Item = [f64; 2]>) -> Option<Aabb> {
        let mut it = points.into_iter();
        let first = it.next()?;
        let mut min = first;
        let mut max = first;
        for p in it {
            for i in 0..2 {
                min[i] = min[i].min(p[i]);
                max[i] = max[i].max(p[i]);
            }
        }
        Some(Aabb::D2 { min, max })
    }

    /// The box of a set of 3D points, or `None` if the iterator is empty.
    pub fn from_points_3d(points: impl IntoIterator<Item = [f64; 3]>) -> Option<Aabb> {
        let mut it = points.into_iter();
        let first = it.next()?;
        let mut min = first;
        let mut max = first;
        for p in it {
            for i in 0..3 {
                min[i] = min[i].min(p[i]);
                max[i] = max[i].max(p[i]);
            }
        }
        Some(Aabb::D3 { min, max })
    }

    /// `(min, max)` as 3D, placing a 2D box in the `z = 0` plane.
    fn as_3d(self) -> ([f64; 3], [f64; 3]) {
        match self {
            Aabb::D2 { min, max } => ([min[0], min[1], 0.0], [max[0], max[1], 0.0]),
            Aabb::D3 { min, max } => (min, max),
        }
    }

    /// The smallest box containing both. Two 2D boxes stay 2D; a 2D box mixed
    /// with a 3D one is promoted into the `z = 0` plane and the result is 3D.
    pub fn union(self, other: Aabb) -> Aabb {
        if let (
            Aabb::D2 {
                min: amin,
                max: amax,
            },
            Aabb::D2 {
                min: bmin,
                max: bmax,
            },
        ) = (self, other)
        {
            return Aabb::D2 {
                min: [amin[0].min(bmin[0]), amin[1].min(bmin[1])],
                max: [amax[0].max(bmax[0]), amax[1].max(bmax[1])],
            };
        }
        let (amin, amax) = self.as_3d();
        let (bmin, bmax) = other.as_3d();
        Aabb::D3 {
            min: [
                amin[0].min(bmin[0]),
                amin[1].min(bmin[1]),
                amin[2].min(bmin[2]),
            ],
            max: [
                amax[0].max(bmax[0]),
                amax[1].max(bmax[1]),
                amax[2].max(bmax[2]),
            ],
        }
    }
}

/// Reduce child boxes into one, ignoring children that produced no box (`Err`).
/// Returns `None` when no child produced a box. Used by the hand-written
/// recursive impls (`GeometryCollection`, `Collection2D`, `Collection3D`).
pub(crate) fn union_results(
    boxes: impl IntoIterator<Item = Result<Aabb, UnsupportedOperation>>,
) -> Option<Aabb> {
    boxes.into_iter().flatten().reduce(Aabb::union)
}

/// Axis-aligned bounding box in the geometry's own embedding and frame.
///
/// Coordinate-free: every leaf computes its box from its own stored
/// coordinates. The default body returns [`UnsupportedOperation`], so a leaf
/// that does not support it needs only an (empty) `impl` block — though every
/// leaf currently supports it.
#[enum_dispatch::enum_dispatch]
pub trait BoundingBox {
    /// The axis-aligned bounding box, or [`UnsupportedOperation`] when the type
    /// has no box (also returned for empty geometries, which have no extent).
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        Err(UnsupportedOperation {
            geometry: core::any::type_name::<Self>(),
            operation: "bounding_box",
        })
    }
}

// The boxed enum variants (`Box<Polygon2D>`, `Box<Solid>`, …) need the trait on
// the `Box` itself: `enum_dispatch` forwards by UFCS, not auto-deref. This
// blanket impl delegates through the box to the contained leaf.
impl<T: BoundingBox + ?Sized> BoundingBox for Box<T> {
    fn bounding_box(&self) -> Result<Aabb, UnsupportedOperation> {
        (**self).bounding_box()
    }
}

/// Tessellation: re-represent a polygonal geometry as a triangle mesh.
/// Defined for `Polygon` and `PolygonMesh`; every other
/// leaf opts out via [`unsupported!`](crate::unsupported).
///
/// The result is a [`Geometry`](crate::Geometry) wrapping a `TriangularMesh` in
/// the input's embedding (2D in, 2D out; 3D in, 3D out) and frame. A degenerate
/// face simply contributes no triangles, so a fully-degenerate
/// input yields a mesh with vertices but no faces rather than an error.
///
/// Tessellation takes `&mut self` and **consumes** the geometry's buffers (vertex
/// pool, appearance, UV are moved into the new mesh), so on success `self` is left
/// moved-from and must be discarded or overwritten. On error `self` is untouched.
#[enum_dispatch::enum_dispatch]
pub trait Triangulate {
    fn triangulate(
        &mut self,
        cache: &mut crate::ops::triangulation::Cache,
    ) -> Result<crate::Geometry, UnsupportedOperation> {
        let _ = cache;
        Err(UnsupportedOperation {
            geometry: core::any::type_name::<Self>(),
            operation: "triangulate",
        })
    }
}

impl<T: Triangulate + ?Sized> Triangulate for Box<T> {
    fn triangulate(
        &mut self,
        cache: &mut crate::ops::triangulation::Cache,
    ) -> Result<crate::Geometry, UnsupportedOperation> {
        (**self).triangulate(cache)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_points_returns_none_when_empty() {
        assert!(Aabb::from_points_2d(Vec::<[f64; 2]>::new()).is_none());
        assert!(Aabb::from_points_3d(Vec::<[f64; 3]>::new()).is_none());
    }

    #[test]
    fn union_of_two_2d_boxes_stays_2d() {
        let a = Aabb::D2 {
            min: [0.0, 0.0],
            max: [2.0, 2.0],
        };
        let b = Aabb::D2 {
            min: [-1.0, 1.0],
            max: [1.0, 5.0],
        };
        assert_eq!(
            a.union(b),
            Aabb::D2 {
                min: [-1.0, 0.0],
                max: [2.0, 5.0]
            }
        );
    }

    #[test]
    fn union_of_2d_and_3d_promotes_to_z_zero_plane() {
        let a = Aabb::D2 {
            min: [0.0, 0.0],
            max: [2.0, 2.0],
        };
        let b = Aabb::D3 {
            min: [-1.0, -1.0, 5.0],
            max: [1.0, 1.0, 9.0],
        };
        // The 2D box sits in z = 0, so the merged z-range is [0, 9].
        assert_eq!(
            a.union(b),
            Aabb::D3 {
                min: [-1.0, -1.0, 0.0],
                max: [2.0, 2.0, 9.0]
            }
        );
    }

    #[test]
    fn union_results_skips_errors_and_reduces() {
        let err = || {
            Err(UnsupportedOperation {
                geometry: "x",
                operation: "bounding_box",
            })
        };
        let boxes = [
            err(),
            Ok(Aabb::point_2d([1.0, 1.0])),
            err(),
            Ok(Aabb::point_2d([3.0, -1.0])),
        ];
        assert_eq!(
            union_results(boxes),
            Some(Aabb::D2 {
                min: [1.0, -1.0],
                max: [3.0, 1.0]
            })
        );
    }

    #[test]
    fn union_results_is_none_when_all_error() {
        let boxes: [Result<Aabb, UnsupportedOperation>; 1] = [Err(UnsupportedOperation {
            geometry: "x",
            operation: "bounding_box",
        })];
        assert!(union_results(boxes).is_none());
    }
}
