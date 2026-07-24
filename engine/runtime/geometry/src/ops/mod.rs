//! Operation traits for the geometry types.
//!
//! Each operation is a separate trait carrying a default method that returns
//! [`UnsupportedOperation`]. A leaf opts in by overriding the method (in its
//! `{type}/ops.rs`), and opts out with an empty `impl` block, stamped by the
//! [`unsupported!`](crate::unsupported) macro. The traits are
//! `#[enum_dispatch]`, so a call on `Geometry` / `Euclidean{2,3}DGeometry`
//! chains through to the concrete leaf. `GeometryCollection` and the per-frame
//! `Collection`s recurse by hand over their children.

pub mod reproject;
pub mod split;
pub mod triangulation;

pub use split::Split;
pub(crate) use reproject::{axis_order_sign, crs_is_linear};
pub use reproject::{Reproject, ReprojectionCache};

use crate::coordinate::{CoordinateFrame, EpsgCode};
use crate::error::Error;

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

    /// Whether two boxes overlap. Contact (shared face, edge, or corner) counts
    /// as overlap — the test is inclusive on every axis, matching the closed-set
    /// semantics the predicate kernels expect from a quick-reject.
    ///
    /// Same-embedding boxes compare directly; a 2D box mixed with a 3D one is
    /// placed in the `z = 0` plane first (as in [`union`](Aabb::union)), so the
    /// mixed case never spuriously rejects on the `z` axis.
    pub fn intersects(&self, other: &Aabb) -> bool {
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
            return (0..2).all(|i| amin[i] <= bmax[i] && bmin[i] <= amax[i]);
        }
        let (amin, amax) = self.as_3d();
        let (bmin, bmax) = other.as_3d();
        (0..3).all(|i| amin[i] <= bmax[i] && bmin[i] <= amax[i])
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

/// Shift every coordinate by a vector.
///
/// A general primitive carrying no frame semantics: translating a CRS geometry
/// keeps it in the same CRS and a Euclidean one Euclidean. Frame changes are the
/// job of [`Reproject`] and [`ConvertFrame`].
#[enum_dispatch::enum_dispatch]
pub trait Translate {
    /// Add `delta` to every coordinate. For a 2D-embedded geometry, `delta`'s
    /// `z` applies to the optional per-vertex elevation. The default body reports
    /// the type as unsupported; a leaf opts in by overriding it.
    fn translate(&mut self, delta: [f64; 3]) -> crate::error::Result<()> {
        let _ = delta;
        Err(Error::projection(format!(
            "translate is not supported by `{}`",
            core::any::type_name::<Self>()
        )))
    }
}

// The boxed enum variants (`Box<Polygon2D>`, `Box<Solid>`, …) need the trait on
// the `Box` itself: `enum_dispatch` forwards by UFCS, not auto-deref.
impl<T: Translate + ?Sized> Translate for Box<T> {
    fn translate(&mut self, delta: [f64; 3]) -> crate::error::Result<()> {
        (**self).translate(delta)
    }
}

/// Add `delta`'s `(x, y)` to a 2D coordinate buffer, and its `z` to a parallel
/// elevation buffer when present.
pub(crate) fn translate_2d(coords: &mut [[f64; 2]], z: Option<&mut [f64]>, delta: [f64; 3]) {
    for c in coords.iter_mut() {
        c[0] += delta[0];
        c[1] += delta[1];
    }
    if let Some(z) = z {
        for elevation in z.iter_mut() {
            *elevation += delta[2];
        }
    }
}

/// Add `delta` to a 3D coordinate buffer.
pub(crate) fn translate_3d(coords: &mut [[f64; 3]], delta: [f64; 3]) {
    for c in coords.iter_mut() {
        c[0] += delta[0];
        c[1] += delta[1];
        c[2] += delta[2];
    }
}

/// Convert a geometry's coordinate frame to `target`.
///
/// A CRS-to-CRS conversion reprojects (delegating to [`Reproject`]); supplying a
/// `base_point` for it is an error, since the offset cannot apply. A conversion
/// that crosses the Euclidean/CRS boundary translates
/// by `base_point` (an offset in the CRS-side frame, defaulting to the origin)
/// and retags: a Euclidean coordinate maps to `base_point + coordinate` in the
/// CRS, a CRS coordinate to `coordinate - base_point` in Euclidean space. The
/// bridge is a positional reinterpretation: coordinate values and ring winding
/// are left unchanged, so a ring's orientation follows the axis order of the
/// frame it is retagged into. A `Tangent` frame on either side is rejected.
#[enum_dispatch::enum_dispatch]
pub trait ConvertFrame {
    /// Convert every coordinate to `target`. The default body reports the type
    /// as unsupported; a leaf opts in by overriding it.
    fn convert_frame(
        &mut self,
        target: &crate::coordinate::CoordinateFrame,
        base_point: Option<[f64; 3]>,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<()> {
        let _ = (target, base_point, cache);
        Err(Error::projection(format!(
            "convert_frame is not supported by `{}`",
            core::any::type_name::<Self>()
        )))
    }
}

impl<T: ConvertFrame + ?Sized> ConvertFrame for Box<T> {
    fn convert_frame(
        &mut self,
        target: &crate::coordinate::CoordinateFrame,
        base_point: Option<[f64; 3]>,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<()> {
        (**self).convert_frame(target, base_point, cache)
    }
}

/// The concrete step a leaf takes to reach a target frame from its current one.
pub(crate) enum FrameStep {
    /// Coordinates are already in the target frame.
    Noop,
    /// Reproject across CRSs to this EPSG code.
    Reproject(EpsgCode),
    /// Translate every coordinate by this offset, then adopt this frame.
    Translate([f64; 3], CoordinateFrame),
}

/// Decide how a leaf currently in `src` reaches `target`, given the base point.
/// Errors when either frame is a `Tangent` plane, or when a base point is
/// supplied for a CRS-to-CRS step.
pub(crate) fn plan_frame_step(
    src: &CoordinateFrame,
    target: &CoordinateFrame,
    base_point: Option<[f64; 3]>,
) -> crate::error::Result<FrameStep> {
    let base = base_point.unwrap_or([0.0; 3]);
    match (src, target) {
        (CoordinateFrame::Crs(from), CoordinateFrame::Crs(to)) => {
            if base_point.is_some() {
                return Err(Error::projection(
                    "a base point does not apply to a CRS-to-CRS reprojection",
                ));
            }
            Ok(if from == to {
                FrameStep::Noop
            } else {
                FrameStep::Reproject(*to)
            })
        }
        (CoordinateFrame::Euclidean, CoordinateFrame::Crs(_)) => {
            Ok(FrameStep::Translate(base, target.clone()))
        }
        (CoordinateFrame::Crs(_), CoordinateFrame::Euclidean) => Ok(FrameStep::Translate(
            [-base[0], -base[1], -base[2]],
            target.clone(),
        )),
        (CoordinateFrame::Euclidean, CoordinateFrame::Euclidean) => Ok(if base == [0.0; 3] {
            FrameStep::Noop
        } else {
            FrameStep::Translate(base, target.clone())
        }),
        (CoordinateFrame::Tangent(_), _) | (_, CoordinateFrame::Tangent(_)) => Err(
            Error::projection("cannot convert to or from a Tangent-plane frame"),
        ),
    }
}

#[cfg(test)]
mod translate_tests {
    use super::Translate;
    use crate::coordinate::{CoordinateFrame, EpsgCode};
    use crate::line_string::LineString3D;
    use crate::point::Point3D;

    #[test]
    fn translate_preserves_frame() {
        let mut p = Point3D::new(CoordinateFrame::Crs(EpsgCode::new(4979)), [1.0, 2.0, 3.0]);
        p.translate([10.0, 20.0, 30.0]).unwrap();
        assert_eq!(p.position(), [11.0, 22.0, 33.0]);
        // A translation is frame-preserving: still the same CRS.
        assert_eq!(p.frame(), &CoordinateFrame::Crs(EpsgCode::new(4979)));
    }

    #[test]
    fn translate_shifts_every_coordinate() {
        let mut ls = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]],
        );
        ls.translate([2.0, 3.0, 100.0]).unwrap();
        assert_eq!(ls.coords(), &[[2.0, 3.0, 100.0], [3.0, 4.0, 101.0]]);
    }
}

#[cfg(test)]
mod convert_frame_tests {
    use super::{ConvertFrame, ReprojectionCache};
    use crate::coordinate::{BaseFrame, CoordinateFrame, EpsgCode, TangentPlane};
    use crate::point::Point3D;

    #[test]
    fn euclidean_to_crs_adds_base_point() {
        let mut cache = ReprojectionCache::new();
        let mut p = Point3D::new(CoordinateFrame::Euclidean, [1.0, 2.0, 3.0]);
        p.convert_frame(
            &CoordinateFrame::Crs(EpsgCode::new(6697)),
            Some([10.0, 20.0, 30.0]),
            &mut cache,
        )
        .unwrap();
        assert_eq!(p.position(), [11.0, 22.0, 33.0]);
        assert_eq!(p.frame(), &CoordinateFrame::Crs(EpsgCode::new(6697)));
    }

    #[test]
    fn crs_to_euclidean_subtracts_base_point() {
        let mut cache = ReprojectionCache::new();
        let mut p = Point3D::new(
            CoordinateFrame::Crs(EpsgCode::new(6697)),
            [11.0, 22.0, 33.0],
        );
        p.convert_frame(
            &CoordinateFrame::Euclidean,
            Some([10.0, 20.0, 30.0]),
            &mut cache,
        )
        .unwrap();
        assert_eq!(p.position(), [1.0, 2.0, 3.0]);
        assert_eq!(p.frame(), &CoordinateFrame::Euclidean);
    }

    #[test]
    fn as_is_bridge_only_retags() {
        let mut cache = ReprojectionCache::new();
        let mut p = Point3D::new(CoordinateFrame::Euclidean, [1.0, 2.0, 3.0]);
        p.convert_frame(&CoordinateFrame::Crs(EpsgCode::new(4979)), None, &mut cache)
            .unwrap();
        assert_eq!(p.position(), [1.0, 2.0, 3.0]);
        assert_eq!(p.frame(), &CoordinateFrame::Crs(EpsgCode::new(4979)));
    }

    #[test]
    fn same_crs_is_noop() {
        let mut cache = ReprojectionCache::new();
        let mut p = Point3D::new(CoordinateFrame::Crs(EpsgCode::new(4979)), [1.0, 2.0, 3.0]);
        p.convert_frame(&CoordinateFrame::Crs(EpsgCode::new(4979)), None, &mut cache)
            .unwrap();
        assert_eq!(p.position(), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn crs_to_crs_reprojects_without_a_base_point() {
        let mut cache = ReprojectionCache::new();
        // 4979 (geographic 3D) -> 4978 (ECEF) is a grid-free datum-identity transform.
        let mut p = Point3D::new(
            CoordinateFrame::Crs(EpsgCode::new(4979)),
            [35.0, 139.0, 0.0],
        );
        p.convert_frame(&CoordinateFrame::Crs(EpsgCode::new(4978)), None, &mut cache)
            .unwrap();
        assert_eq!(p.frame(), &CoordinateFrame::Crs(EpsgCode::new(4978)));
        // ECEF magnitude is ~ Earth radius.
        let [x, y, z] = p.position();
        let r = (x * x + y * y + z * z).sqrt();
        assert!(
            r > 6_000_000.0 && r < 6_500_000.0,
            "unexpected ECEF radius {r}"
        );
    }

    #[test]
    fn crs_to_crs_with_a_base_point_is_rejected() {
        let mut cache = ReprojectionCache::new();
        let mut p = Point3D::new(
            CoordinateFrame::Crs(EpsgCode::new(4979)),
            [35.0, 139.0, 0.0],
        );
        assert!(p
            .convert_frame(
                &CoordinateFrame::Crs(EpsgCode::new(4978)),
                Some([999.0, 999.0, 999.0]),
                &mut cache,
            )
            .is_err());
    }

    #[test]
    fn tangent_frame_is_rejected() {
        let mut cache = ReprojectionCache::new();
        let tangent = CoordinateFrame::Tangent(Box::new(TangentPlane {
            base: BaseFrame::Euclidean,
            origin: [0.0, 0.0, 0.0],
            u: [1.0, 0.0, 0.0],
            v: [0.0, 1.0, 0.0],
        }));
        let mut p = Point3D::new(tangent, [1.0, 2.0, 3.0]);
        assert!(p
            .convert_frame(&CoordinateFrame::Crs(EpsgCode::new(4979)), None, &mut cache)
            .is_err());
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
    fn intersects_overlapping_and_disjoint_2d() {
        let a = Aabb::D2 {
            min: [0.0, 0.0],
            max: [2.0, 2.0],
        };
        let overlap = Aabb::D2 {
            min: [1.0, 1.0],
            max: [3.0, 3.0],
        };
        let disjoint = Aabb::D2 {
            min: [3.0, 3.0],
            max: [4.0, 4.0],
        };
        // Touching along an edge counts (inclusive).
        let touching = Aabb::D2 {
            min: [2.0, 0.0],
            max: [4.0, 2.0],
        };
        assert!(a.intersects(&overlap));
        assert!(a.intersects(&touching));
        assert!(!a.intersects(&disjoint));
        // Symmetric.
        assert!(overlap.intersects(&a));
    }

    #[test]
    fn intersects_mixed_dim_uses_z_zero_plane() {
        let flat = Aabb::D2 {
            min: [0.0, 0.0],
            max: [2.0, 2.0],
        };
        // A 3D box straddling z = 0 overlaps the z = 0 plane.
        let through_plane = Aabb::D3 {
            min: [1.0, 1.0, -1.0],
            max: [3.0, 3.0, 1.0],
        };
        // A 3D box entirely above z = 0 does not.
        let above_plane = Aabb::D3 {
            min: [1.0, 1.0, 1.0],
            max: [3.0, 3.0, 2.0],
        };
        assert!(flat.intersects(&through_plane));
        assert!(!flat.intersects(&above_plane));
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
