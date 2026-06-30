//! Reproject geometry types between coordinate reference systems.

use crate::coordinate::EpsgCode;
use crate::error::{Error, Result};

mod ffi;

pub use ffi::ReprojectionCache;

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

// The boxed enum variants (`Box<Polygon2D>`, `Box<Solid>`, …) need the trait on
// the `Box` itself: `enum_dispatch` forwards by UFCS, not auto-deref.
impl<T: Reproject + ?Sized> Reproject for Box<T> {
    fn reproject(&mut self, target: EpsgCode, cache: &mut ReprojectionCache) -> Result<()> {
        (**self).reproject(target, cache)
    }
}

/// Reproject a 3D coordinate buffer in place from `from` to `target` (EPSG).
pub(crate) fn transform_coords_3d(
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
/// transforming the parallel elevation buffer too when present.
pub(crate) fn transform_coords_2d(
    cache: &mut ReprojectionCache,
    from: EpsgCode,
    target: EpsgCode,
    coords: &mut [[f64; 2]],
    z: Option<&mut [f64]>,
) -> Result<()> {
    if let Some(elevations) = z {
        if elevations.len() != coords.len() {
            return Err(Error::projection(format!(
                "elevation buffer length {} does not match coordinate count {}",
                elevations.len(),
                coords.len()
            )));
        }
        for (c, elevation) in coords.iter_mut().zip(elevations.iter_mut()) {
            let [x, y, new_z] = cache.transform(from, target, [c[0], c[1], *elevation])?;
            *c = [x, y];
            *elevation = new_z;
        }
    } else {
        for c in coords.iter_mut() {
            let [x, y, _] = cache.transform(from, target, [c[0], c[1], 0.0])?;
            *c = [x, y];
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::collection::Collection3D;
    use crate::coordinate::Coordinate;
    use crate::line_string::LineString2D;
    use crate::point::{Point2D, Point3D};
    use crate::point_cloud::PointCloud;
    use crate::Euclidean3DGeometry;

    #[test]
    fn transform_round_trip_3d() {
        let mut cache = ReprojectionCache::new();
        let p = [139.767, 35.681, 100.0];
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
    fn transform_axis_order_is_lon_lat() {
        let mut cache = ReprojectionCache::new();
        let out = cache
            .transform(
                EpsgCode::new(4326),
                EpsgCode::new(3857),
                [139.767, 35.681, 0.0],
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
        let start = [139.767, 35.681, 100.0];
        let expected = cache
            .transform(EpsgCode::new(4979), EpsgCode::new(4978), start)
            .unwrap();

        let mut p = Point3D::new(Coordinate::Crs(EpsgCode::new(4979)), start);
        p.reproject(EpsgCode::new(4978), &mut cache).unwrap();
        assert_eq!(
            p,
            Point3D::new(Coordinate::Crs(EpsgCode::new(4978)), expected)
        );
    }

    #[test]
    fn point2d_reproject_drops_z() {
        let mut cache = ReprojectionCache::new();
        let [x, y, _] = cache
            .transform(
                EpsgCode::new(4326),
                EpsgCode::new(3857),
                [139.767, 35.681, 0.0],
            )
            .unwrap();

        let mut p = Point2D::new(Coordinate::Crs(EpsgCode::new(4326)), [139.767, 35.681]);
        p.reproject(EpsgCode::new(3857), &mut cache).unwrap();
        assert_eq!(
            p,
            Point2D::new(Coordinate::Crs(EpsgCode::new(3857)), [x, y])
        );
    }

    #[test]
    fn linestring2d_reproject_carries_elevation() {
        let mut cache = ReprojectionCache::new();
        let raw = [[139.7, 35.6, 10.0], [139.8, 35.7, 20.0]];
        let expected: Vec<[f64; 3]> = raw
            .iter()
            .map(|&[x, y, z]| {
                cache
                    .transform(EpsgCode::new(4326), EpsgCode::new(3857), [x, y, z])
                    .unwrap()
            })
            .collect();

        let mut ls =
            LineString2D::from_coords_with_elevation(Coordinate::Crs(EpsgCode::new(4326)), raw);
        ls.reproject(EpsgCode::new(3857), &mut cache).unwrap();
        assert_eq!(
            ls,
            LineString2D::from_coords_with_elevation(
                Coordinate::Crs(EpsgCode::new(3857)),
                expected
            )
        );
    }

    #[test]
    fn collection_reproject_dispatches_to_each_member() {
        let mut cache = ReprojectionCache::new();
        let a = [139.7, 35.6, 1.0];
        let b = [140.0, 35.9, 2.0];
        let ea = cache
            .transform(EpsgCode::new(4979), EpsgCode::new(4978), a)
            .unwrap();
        let eb = cache
            .transform(EpsgCode::new(4979), EpsgCode::new(4978), b)
            .unwrap();

        let mut col = Collection3D::new([
            Euclidean3DGeometry::Point(Point3D::new(Coordinate::Crs(EpsgCode::new(4979)), a)),
            Euclidean3DGeometry::Point(Point3D::new(Coordinate::Crs(EpsgCode::new(4979)), b)),
        ]);
        col.reproject(EpsgCode::new(4978), &mut cache).unwrap();
        assert_eq!(
            col,
            Collection3D::new([
                Euclidean3DGeometry::Point(Point3D::new(Coordinate::Crs(EpsgCode::new(4978)), ea)),
                Euclidean3DGeometry::Point(Point3D::new(Coordinate::Crs(EpsgCode::new(4978)), eb)),
            ])
        );
    }

    #[test]
    fn mismatched_elevation_buffer_is_error() {
        let mut cache = ReprojectionCache::new();
        let mut coords = [[139.7, 35.6], [139.8, 35.7]];
        let mut z = [10.0]; // one short of `coords`
        assert!(matches!(
            transform_coords_2d(
                &mut cache,
                EpsgCode::new(4326),
                EpsgCode::new(3857),
                &mut coords,
                Some(&mut z)
            ),
            Err(Error::Projection(_))
        ));
    }

    #[test]
    fn reproject_same_crs_is_noop() {
        let mut cache = ReprojectionCache::new();
        let mut p = Point3D::new(Coordinate::Crs(EpsgCode::new(4979)), [139.7, 35.6, 50.0]);
        p.reproject(EpsgCode::new(4979), &mut cache).unwrap();
        assert_eq!(
            p,
            Point3D::new(Coordinate::Crs(EpsgCode::new(4979)), [139.7, 35.6, 50.0])
        );
    }

    #[test]
    fn non_crs_frame_is_error() {
        let mut cache = ReprojectionCache::new();
        let mut p = Point3D::new(Coordinate::Euclidean, [1.0, 2.0, 3.0]);
        assert!(matches!(
            p.reproject(EpsgCode::new(4326), &mut cache),
            Err(Error::Projection(_))
        ));
    }

    #[test]
    fn unsupported_leaf_is_error() {
        let mut cache = ReprojectionCache::new();
        let pc =
            PointCloud::from_positions(Coordinate::Crs(EpsgCode::new(4979)), [[139.7, 35.6, 1.0]]);
        let mut geom = Euclidean3DGeometry::PointCloud(Box::new(pc));
        assert!(matches!(
            geom.reproject(EpsgCode::new(4978), &mut cache),
            Err(Error::Projection(_))
        ));
    }
}
