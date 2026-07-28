//! Reproject geometry types between coordinate reference systems.

use crate::coordinate::EpsgCode;
use crate::error::{Error, Result};

mod ffi;

pub use ffi::ReprojectionCache;
pub(crate) use ffi::{axis_order_sign, crs_demote_to_2d, crs_is_linear, TwoDimensionalCrs};

/// Reproject a geometry's coordinates to a target CRS.
#[enum_dispatch::enum_dispatch]
pub trait Reproject {
    /// Reproject every coordinate to `target` (an EPSG code). The default body
    /// reports the type as unsupported; a leaf opts in by overriding it.
    fn reproject(
        &mut self,
        target: EpsgCode,
        cache: &mut ReprojectionCache,
    ) -> crate::error::Result<()> {
        let _ = (target, cache);
        Err(Error::projection(format!(
            "reproject is not supported by `{}`",
            core::any::type_name::<Self>()
        )))
    }
}

/// Pair a 2D coordinate buffer with the leaf's elevation, the shared body of
/// every 2D leaf's `into_3d`.
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
    fn reproject(&mut self, target: EpsgCode, cache: &mut ReprojectionCache) -> Result<()> {
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

/// Reproject a 2D coordinate buffer in place from `from` to `target` (EPSG),
/// carrying the leaf's single elevation through the transform when present.
///
/// The elevation feeds the vertical component of every coordinate's transform, so
/// the horizontal result is correct everywhere. The output elevation is then read
/// off the **first** coordinate: the vertical result of a transform can vary with
/// position, and a 2.5D leaf has exactly one height to keep. Anything that needs
/// the per-vertex vertical result is a 3D leaf, which [`transform_coords_3d`]
/// handles exactly.
///
/// A leaf with no coordinates offers no position at which to evaluate the vertical
/// transform, so it comes out pure 2D rather than keeping a height that belongs to
/// `from` while the leaf now declares `target`.
pub(crate) fn transform_coords_2d(
    cache: &mut ReprojectionCache,
    from: EpsgCode,
    target: EpsgCode,
    coords: &mut [[f64; 2]],
    z: &mut Option<f64>,
) -> Result<()> {
    let elevation = z.unwrap_or(0.0);
    let mut first_out_z = None;
    for c in coords.iter_mut() {
        let [x, y, new_z] = cache.transform(from, target, [c[0], c[1], elevation])?;
        *c = [x, y];
        if first_out_z.is_none() {
            first_out_z = Some(new_z);
        }
    }
    if z.is_some() {
        *z = first_out_z;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::collection::Collection3D;
    use crate::coordinate::CoordinateFrame;
    use crate::line_string::LineString2D;
    use crate::point::{Point2D, Point3D};
    use crate::point_cloud::PointCloud;
    use crate::Euclidean3DGeometry;

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
        p.reproject(EpsgCode::new(4978), &mut cache).unwrap();
        assert_eq!(
            p,
            Point3D::new(CoordinateFrame::Crs(EpsgCode::new(4978)), expected)
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
        p.reproject(EpsgCode::new(3857), &mut cache).unwrap();
        assert_eq!(
            p,
            Point2D::new(CoordinateFrame::Crs(EpsgCode::new(3857)), [x, y])
        );
    }

    #[test]
    fn scratch_euclid_crs_roundtrip() {
        use crate::line_string::LineString2D;
        use crate::ops::ConvertFrame;
        use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry as G};
        let mut cache = ReprojectionCache::new();
        let pts = [[35.680, 139.760], [35.685, 139.765]];
        let wgs = CoordinateFrame::Crs(EpsgCode::new(4326));
        let euc = CoordinateFrame::Euclidean;
        let bp = Some([1.0, 2.0, 3.0]);

        let mk = |f: &CoordinateFrame| {
            G::Euclidean2D(Euclidean2DGeometry::LineString(
                LineString2D::from_coords_at_elevation(f.clone(), pts, 10.0),
            ))
        };
        let label = |g: &G| match g {
            G::Euclidean2D(Euclidean2DGeometry::LineString(l)) => format!(
                "2.5D kept, elevation {:?}, frame {:?}",
                l.elevation(),
                l.frame()
            ),
            G::Euclidean3D(Euclidean3DGeometry::LineString(_)) => "PROMOTED TO 3D".to_string(),
            _ => "other".into(),
        };

        for (name, src, dst, base) in [
            ("euclid -> crs  (base point)", &euc, &wgs, bp),
            ("crs    -> euclid (base point)", &wgs, &euc, bp),
            ("euclid -> crs  (as-is)", &euc, &wgs, None),
            ("crs    -> euclid (as-is)", &wgs, &euc, None),
            ("euclid -> euclid (base point)", &euc, &euc, bp),
            ("euclid -> euclid (as-is)", &euc, &euc, None),
        ] {
            let mut g = mk(src);
            match g.convert_frame(dst, base, &mut cache) {
                Ok(()) => println!("SCRATCH {name:<30} => {}", label(&g)),
                Err(e) => println!("SCRATCH {name:<30} => ERR {e}"),
            }
        }

        // And a full round trip: crs -> euclid -> crs.
        let mut g = mk(&wgs);
        g.convert_frame(&euc, bp, &mut cache).unwrap();
        g.convert_frame(&wgs, bp, &mut cache).unwrap();
        println!("SCRATCH {:<30} => {}", "crs -> euclid -> crs", label(&g));
    }

    #[test]
    fn linestring2d_reproject_carries_elevation() {
        let mut cache = ReprojectionCache::new();
        let raw = [[35.6, 139.7], [35.7, 139.8]];
        let expected: Vec<[f64; 3]> = raw
            .iter()
            .map(|&[x, y]| {
                cache
                    .transform(EpsgCode::new(4326), EpsgCode::new(3857), [x, y, 10.0])
                    .unwrap()
            })
            .collect();

        let mut ls = LineString2D::from_coords_at_elevation(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            raw,
            10.0,
        );
        ls.reproject(EpsgCode::new(3857), &mut cache).unwrap();
        // The horizontal result comes from transforming each coordinate at the
        // chain's elevation; the one elevation kept is the first coordinate's.
        assert_eq!(
            ls,
            LineString2D::from_coords_at_elevation(
                CoordinateFrame::Crs(EpsgCode::new(3857)),
                expected.iter().map(|&[x, y, _]| [x, y]),
                expected[0][2],
            )
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
        col.reproject(EpsgCode::new(4978), &mut cache).unwrap();
        assert_eq!(
            col,
            Collection3D::new([
                Euclidean3DGeometry::Point(Point3D::new(
                    CoordinateFrame::Crs(EpsgCode::new(4978)),
                    ea
                )),
                Euclidean3DGeometry::Point(Point3D::new(
                    CoordinateFrame::Crs(EpsgCode::new(4978)),
                    eb
                )),
            ])
        );
    }

    #[test]
    fn pure_2d_leaf_stays_pure_2d() {
        let mut cache = ReprojectionCache::new();
        let mut ls = LineString2D::from_coords(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            [[35.6, 139.7], [35.7, 139.8]],
        );
        ls.reproject(EpsgCode::new(3857), &mut cache).unwrap();
        // A leaf with no elevation does not acquire one from the transform.
        assert_eq!(ls.elevation(), None);
    }

    #[test]
    fn coordinateless_leaf_does_not_keep_a_source_frame_elevation() {
        let mut cache = ReprojectionCache::new();
        let mut ls = LineString2D::from_coords_at_elevation(
            CoordinateFrame::Crs(EpsgCode::new(4326)),
            Vec::<[f64; 2]>::new(),
            10.0,
        );
        ls.reproject(EpsgCode::new(3857), &mut cache).unwrap();
        // The frame moved, so an untransformed height would now be stated in a
        // frame it was never expressed in. With no coordinate to evaluate the
        // vertical transform at, the leaf comes out pure 2D instead.
        assert_eq!(ls.frame(), &CoordinateFrame::Crs(EpsgCode::new(3857)));
        assert_eq!(ls.elevation(), None);
    }

    #[test]
    fn reproject_same_crs_is_noop() {
        let mut cache = ReprojectionCache::new();
        let mut p = Point3D::new(
            CoordinateFrame::Crs(EpsgCode::new(4979)),
            [139.7, 35.6, 50.0],
        );
        p.reproject(EpsgCode::new(4979), &mut cache).unwrap();
        assert_eq!(
            p,
            Point3D::new(
                CoordinateFrame::Crs(EpsgCode::new(4979)),
                [139.7, 35.6, 50.0]
            )
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
