//! Reproject geometry types between coordinate reference systems.

use crate::coordinate::EpsgCode;
use crate::error::{Error, Result};
use crate::Geometry;

mod ffi;
pub(crate) mod grids;

pub use ffi::ReprojectionCache;
pub(crate) use ffi::{axis_order_sign, crs_demote_to_2d, crs_is_linear, TwoDimensionalCrs};

/// Reproject a geometry's coordinates to a target CRS.
///
/// Consumes its input: the implementor deconstructs `&mut self` into the
/// returned [`Geometry`]. A 2D leaf lying at a single elevation comes out 3D.
#[enum_dispatch::enum_dispatch]
pub trait Reproject {
    /// Reproject every coordinate to `target` (an EPSG code), consuming `self`
    /// into the result. The default body reports the type as unsupported.
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<Geometry> {
        let _ = (target, cache);
        Err(Error::projection(format!(
            "reproject is not supported by `{}`",
            core::any::type_name::<Self>()
        )))
    }
}

/// Pair a 2D coordinate buffer with the leaf's elevation.
pub(crate) fn lift_coords<'a>(
    coords: impl IntoIterator<Item = &'a [f64; 2]>,
    z: Option<f64>,
) -> Vec<[f64; 3]> {
    let z = z.unwrap_or(0.0);
    coords.into_iter().map(|&[x, y]| [x, y, z]).collect()
}

// The boxed enum variants (`Box<Polygon2D>`, `Box<Solid>`, …) need the trait on
// the `Box` itself: `enum_dispatch` forwards by UFCS, not auto-deref.
impl<T: Reproject + ?Sized> Reproject for Box<T> {
    fn reproject(&mut self, target: EpsgCode, cache: &mut ReprojectionCache) -> Result<Geometry> {
        (**self).reproject(target, cache)
    }
}

/// Reproject a 3D coordinate buffer in place from `from` to `target` (EPSG).
pub fn transform_coords_3d(
    cache: &mut ReprojectionCache,
    from: EpsgCode,
    target: EpsgCode,
    coords: &mut [[f64; 3]],
) -> Result<()> {
    for c in coords.iter_mut() {
        *c = cache.transform(from, target, *c)?;
    }
    Ok(())
}

/// Reproject a 2D coordinate buffer in place from `from` to `target` (EPSG).
pub(crate) fn transform_coords_2d(
    cache: &mut ReprojectionCache,
    from: EpsgCode,
    target: EpsgCode,
    coords: &mut [[f64; 2]],
) -> Result<()> {
    for c in coords.iter_mut() {
        let [x, y, _] = cache.transform(from, target, [c[0], c[1], 0.0])?;
        *c = [x, y];
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::collection::{Collection2D, Collection3D};
    use crate::coordinate::CoordinateFrame;
    use crate::line_string::{LineString2D, LineString3D};
    use crate::point::{Point2D, Point3D};
    use crate::point_cloud::PointCloud;
    use crate::{Euclidean2DGeometry, Euclidean3DGeometry};

    #[test]
    fn transform_round_trip_3d() {
        let mut cache = ReprojectionCache::new();
        let p = [35.681, 139.767, 100.0];
        let ecef = cache
            .transform(EpsgCode::new(4979), EpsgCode::new(4978), p)
            .unwrap();
        let back = cache
            .transform(EpsgCode::new(4978), EpsgCode::new(4979), ecef)
            .unwrap();
        assert_relative_eq!(back[0], p[0], epsilon = 1e-7);
        assert_relative_eq!(back[1], p[1], epsilon = 1e-7);
        assert_relative_eq!(back[2], p[2], epsilon = 1e-3);
    }

    #[test]
    fn transform_uses_each_crs_own_axis_order() {
        let mut cache = ReprojectionCache::new();
        // EPSG:4326 is officially (lat, lon); EPSG:3857 is (x, y) easting/northing.
        let out = cache
            .transform(
                EpsgCode::new(4326),
                EpsgCode::new(3857),
                [35.681, 139.767, 0.0],
            )
            .unwrap();
        assert_relative_eq!(out[0], 1.5558e7, epsilon = 1e4);
        assert_relative_eq!(out[1], 4.2575e6, epsilon = 1e4);
    }

    #[test]
    fn transform_is_true_3d_z_changes() {
        let mut cache = ReprojectionCache::new();
        let out = cache
            .transform(EpsgCode::new(4979), EpsgCode::new(4978), [0.0, 0.0, 0.0])
            .unwrap();
        assert_relative_eq!(out[0], 6_378_137.0, epsilon = 1.0);
        assert!(out[0].is_finite() && out[1].abs() < 1.0 && out[2].abs() < 1.0);
    }

    #[test]
    fn point3d_reproject_updates_position_and_frame() {
        let mut cache = ReprojectionCache::new();
        let start = [35.681, 139.767, 100.0];
        let expected = cache
            .transform(EpsgCode::new(4979), EpsgCode::new(4978), start)
            .unwrap();

        let mut p = Point3D::new(CoordinateFrame::Crs(EpsgCode::new(4979)), start);
        let out = p.reproject(EpsgCode::new(4978), &mut cache).unwrap();
        assert_eq!(
            out,
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
                CoordinateFrame::Crs(EpsgCode::new(4978)),
                expected
            )))
        );
    }

    #[test]
    fn point2d_reproject_drops_z() {
        let mut cache = ReprojectionCache::new();
        let [x, y, _] = cache
            .transform(
                EpsgCode::new(4326),
                EpsgCode::new(3857),
                [35.681, 139.767, 0.0],
            )
            .unwrap();

        let mut p = Point2D::new(CoordinateFrame::Crs(EpsgCode::new(4326)), [35.681, 139.767]);
        let out = p.reproject(EpsgCode::new(3857), &mut cache).unwrap();
        assert_eq!(
            out,
            Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
                CoordinateFrame::Crs(EpsgCode::new(3857)),
                [x, y]
            )))
        );
    }

    #[test]
    fn a_2_5d_leaf_reprojects_to_3d_keeping_every_vertical_result() {
        let mut cache = ReprojectionCache::new();
        let raw = [[35.6, 139.7], [35.9, 140.0]];
        let expected: Vec<[f64; 3]> = raw
            .iter()
            .map(|&[x, y]| {
                cache
                    .transform(EpsgCode::new(4979), EpsgCode::new(4978), [x, y, 10.0])
                    .unwrap()
            })
            .collect();
        assert_ne!(expected[0][2], expected[1][2]);

        let mut ls = LineString2D::from_coords_at_elevation(
            CoordinateFrame::Crs(EpsgCode::new(4979)),
            raw,
            10.0,
        );
        let out = ls.reproject(EpsgCode::new(4978), &mut cache).unwrap();
        assert_eq!(
            out,
            Geometry::Euclidean3D(Euclidean3DGeometry::LineString(LineString3D::from_coords(
                CoordinateFrame::Crs(EpsgCode::new(4978)),
                expected,
            )))
        );
        assert!(ls.coords().is_empty());
    }

    #[test]
    fn the_same_promotion_happens_through_geometry() {
        let mut cache = ReprojectionCache::new();
        let raw = [[35.6, 139.7], [35.9, 140.0]];
        let mk = || {
            LineString2D::from_coords_at_elevation(
                CoordinateFrame::Crs(EpsgCode::new(4979)),
                raw,
                10.0,
            )
        };
        let direct = mk().reproject(EpsgCode::new(4978), &mut cache).unwrap();
        let via_geometry = Geometry::Euclidean2D(Euclidean2DGeometry::LineString(mk()))
            .reproject(EpsgCode::new(4978), &mut cache)
            .unwrap();
        assert_eq!(direct, via_geometry);
    }

    #[test]
    fn same_crs_reprojection_of_a_2_5d_leaf_keeps_it_2_5d() {
        let mut cache = ReprojectionCache::new();
        let before = LineString2D::from_coords_at_elevation(
            CoordinateFrame::Crs(EpsgCode::new(4979)),
            [[35.6, 139.7], [35.7, 139.8]],
            10.0,
        );
        let out = before
            .clone()
            .reproject(EpsgCode::new(4979), &mut cache)
            .unwrap();
        assert_eq!(
            out,
            Geometry::Euclidean2D(Euclidean2DGeometry::LineString(before))
        );
    }

    #[test]
    fn collection_reproject_dispatches_to_each_member() {
        let mut cache = ReprojectionCache::new();
        let a = [35.6, 139.7, 1.0];
        let b = [35.9, 140.0, 2.0];
        let ea = cache
            .transform(EpsgCode::new(4979), EpsgCode::new(4978), a)
            .unwrap();
        let eb = cache
            .transform(EpsgCode::new(4979), EpsgCode::new(4978), b)
            .unwrap();

        let mut col = Collection3D::new([
            Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::Crs(EpsgCode::new(4979)), a)),
            Euclidean3DGeometry::Point(Point3D::new(CoordinateFrame::Crs(EpsgCode::new(4979)), b)),
        ]);
        let out = col.reproject(EpsgCode::new(4978), &mut cache).unwrap();
        assert_eq!(
            out,
            Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new([
                Euclidean3DGeometry::Point(Point3D::new(
                    CoordinateFrame::Crs(EpsgCode::new(4978)),
                    ea
                )),
                Euclidean3DGeometry::Point(Point3D::new(
                    CoordinateFrame::Crs(EpsgCode::new(4978)),
                    eb
                )),
            ])))
        );
    }

    #[test]
    fn a_2_5d_collection_gives_up_its_embedding_as_a_unit() {
        let mut cache = ReprojectionCache::new();
        let mut col = Collection2D::new([
            Euclidean2DGeometry::LineString(LineString2D::from_coords_at_elevation(
                CoordinateFrame::Crs(EpsgCode::new(4979)),
                [[35.6, 139.7]],
                10.0,
            )),
            Euclidean2DGeometry::Point(Point2D::new(
                CoordinateFrame::Crs(EpsgCode::new(4979)),
                [35.9, 140.0],
            )),
        ]);
        let out = col.reproject(EpsgCode::new(4978), &mut cache).unwrap();
        let Geometry::Euclidean3D(Euclidean3DGeometry::Collection(c)) = &out else {
            panic!("expected a promoted 3D collection, got {out:?}");
        };
        assert_eq!(c.members().len(), 2);
        assert!(matches!(c.members()[1], Euclidean3DGeometry::Point(_),));
    }

    #[test]
    fn pure_2d_leaf_stays_pure_2d() {
        let mut cache = ReprojectionCache::new();
        let mut ls = LineString2D::from_coords(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            [[35.6, 139.7], [35.7, 139.8]],
        );
        let out = ls.reproject(EpsgCode::new(3857), &mut cache).unwrap();
        let Geometry::Euclidean2D(Euclidean2DGeometry::LineString(ls)) = &out else {
            panic!("expected a 2D line string, got {out:?}");
        };
        assert_eq!(ls.elevation(), None);
    }

    #[test]
    fn a_coordinateless_2_5d_leaf_promotes_rather_than_keeping_a_stale_height() {
        let mut cache = ReprojectionCache::new();
        let mut ls = LineString2D::from_coords_at_elevation(
            CoordinateFrame::Crs(EpsgCode::new(4979)),
            Vec::<[f64; 2]>::new(),
            10.0,
        );
        let out = ls.reproject(EpsgCode::new(4978), &mut cache).unwrap();
        let Geometry::Euclidean3D(Euclidean3DGeometry::LineString(ls)) = &out else {
            panic!("expected a promoted 3D line string, got {out:?}");
        };
        assert_eq!(ls.frame(), &CoordinateFrame::Crs(EpsgCode::new(4978)));
        assert!(ls.coords().is_empty());
    }

    #[test]
    fn reproject_same_crs_is_noop() {
        let mut cache = ReprojectionCache::new();
        let mut p = Point3D::new(
            CoordinateFrame::Crs(EpsgCode::new(4979)),
            [139.7, 35.6, 50.0],
        );
        let out = p.reproject(EpsgCode::new(4979), &mut cache).unwrap();
        assert_eq!(
            out,
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
                CoordinateFrame::Crs(EpsgCode::new(4979)),
                [139.7, 35.6, 50.0]
            )))
        );
    }

    #[test]
    fn non_crs_frame_is_error() {
        let mut cache = ReprojectionCache::new();
        let mut p = Point3D::new(CoordinateFrame::Euclidean, [1.0, 2.0, 3.0]);
        assert!(matches!(
            p.reproject(EpsgCode::new(4326), &mut cache),
            Err(Error::Projection(_))
        ));
    }

    #[test]
    fn unsupported_leaf_is_error() {
        let mut cache = ReprojectionCache::new();
        let pc = PointCloud::from_positions(
            CoordinateFrame::Crs(EpsgCode::new(4979)),
            [[139.7, 35.6, 1.0]],
        );
        let mut geom = Euclidean3DGeometry::PointCloud(Box::new(pc));
        assert!(matches!(
            geom.reproject(EpsgCode::new(4978), &mut cache),
            Err(Error::Projection(_))
        ));
    }
}
