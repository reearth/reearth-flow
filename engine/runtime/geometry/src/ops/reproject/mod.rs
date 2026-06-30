use nusamai_projection::crs::EpsgCode;

use crate::collection::{Collection2D, Collection3D};
use crate::error::{Error, Result};
use crate::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry, GeometryCollection};

mod ffi;

pub use ffi::ReprojectionCache;

pub trait Reproject {
    fn reproject(&mut self, target: EpsgCode, cache: &mut ReprojectionCache) -> Result<()>;
}

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

impl Reproject for Geometry {
    fn reproject(&mut self, target: EpsgCode, cache: &mut ReprojectionCache) -> Result<()> {
        match self {
            Geometry::None => Ok(()),
            Geometry::Euclidean2D(g) => g.reproject(target, cache),
            Geometry::Euclidean3D(g) => g.reproject(target, cache),
            Geometry::GeometryCollection(c) => c.reproject(target, cache),
        }
    }
}

impl Reproject for GeometryCollection {
    fn reproject(&mut self, target: EpsgCode, cache: &mut ReprojectionCache) -> Result<()> {
        for member in self.members_mut() {
            member.reproject(target, cache)?;
        }
        Ok(())
    }
}

impl Reproject for Euclidean2DGeometry {
    fn reproject(&mut self, target: EpsgCode, cache: &mut ReprojectionCache) -> Result<()> {
        match self {
            Euclidean2DGeometry::Point(p) => p.reproject(target, cache),
            Euclidean2DGeometry::LineString(l) => l.reproject(target, cache),
            Euclidean2DGeometry::Polygon(p) => p.reproject(target, cache),
            Euclidean2DGeometry::PolygonMesh(m) => m.reproject(target, cache),
            Euclidean2DGeometry::TriangularMesh(m) => m.reproject(target, cache),
            Euclidean2DGeometry::Collection(c) => c.reproject(target, cache),
        }
    }
}

impl Reproject for Collection2D {
    fn reproject(&mut self, target: EpsgCode, cache: &mut ReprojectionCache) -> Result<()> {
        for member in self.members_mut() {
            member.reproject(target, cache)?;
        }
        Ok(())
    }
}

impl Reproject for Euclidean3DGeometry {
    fn reproject(&mut self, target: EpsgCode, cache: &mut ReprojectionCache) -> Result<()> {
        match self {
            Euclidean3DGeometry::Point(p) => p.reproject(target, cache),
            Euclidean3DGeometry::LineString(l) => l.reproject(target, cache),
            Euclidean3DGeometry::Polygon(p) => p.reproject(target, cache),
            Euclidean3DGeometry::PolygonMesh(m) => m.reproject(target, cache),
            Euclidean3DGeometry::TriangularMesh(m) => m.reproject(target, cache),
            Euclidean3DGeometry::Solid(s) => s.reproject(target, cache),
            Euclidean3DGeometry::Collection(c) => c.reproject(target, cache),
            Euclidean3DGeometry::PointCloud(_) => Err(Error::projection(
                "reproject not yet supported for PointCloud",
            )),
            Euclidean3DGeometry::Csg(_) => {
                Err(Error::projection("reproject not yet supported for Csg"))
            }
        }
    }
}

impl Reproject for Collection3D {
    fn reproject(&mut self, target: EpsgCode, cache: &mut ReprojectionCache) -> Result<()> {
        for member in self.members_mut() {
            member.reproject(target, cache)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::coordinate::Coordinate;
    use crate::line_string::LineString2D;
    use crate::point::{Point2D, Point3D};
    use crate::point_cloud::PointCloud;

    #[test]
    fn transform_round_trip_3d() {
        let mut cache = ReprojectionCache::new();
        let p = [139.767, 35.681, 100.0];
        let ecef = cache.transform(4979, 4978, p).unwrap();
        let back = cache.transform(4978, 4979, ecef).unwrap();
        assert_relative_eq!(back[0], p[0], epsilon = 1e-7);
        assert_relative_eq!(back[1], p[1], epsilon = 1e-7);
        assert_relative_eq!(back[2], p[2], epsilon = 1e-3);
    }

    #[test]
    fn transform_axis_order_is_lon_lat() {
        let mut cache = ReprojectionCache::new();
        let out = cache.transform(4326, 3857, [139.767, 35.681, 0.0]).unwrap();
        assert_relative_eq!(out[0], 1.5558e7, epsilon = 1e4);
        assert_relative_eq!(out[1], 4.2575e6, epsilon = 1e4);
    }

    #[test]
    fn transform_is_true_3d_z_changes() {
        let mut cache = ReprojectionCache::new();
        let out = cache.transform(4979, 4978, [0.0, 0.0, 0.0]).unwrap();
        assert_relative_eq!(out[0], 6_378_137.0, epsilon = 1.0);
        assert!(out[0].is_finite() && out[1].abs() < 1.0 && out[2].abs() < 1.0);
    }

    #[test]
    fn point3d_reproject_updates_position_and_frame() {
        let mut cache = ReprojectionCache::new();
        let start = [139.767, 35.681, 100.0];
        let expected = cache.transform(4979, 4978, start).unwrap();

        let mut p = Point3D::new(Coordinate::Crs(4979), start);
        p.reproject(4978, &mut cache).unwrap();
        assert_eq!(p, Point3D::new(Coordinate::Crs(4978), expected));
    }

    #[test]
    fn point2d_reproject_drops_z() {
        let mut cache = ReprojectionCache::new();
        let [x, y, _] = cache.transform(4326, 3857, [139.767, 35.681, 0.0]).unwrap();

        let mut p = Point2D::new(Coordinate::Crs(4326), [139.767, 35.681]);
        p.reproject(3857, &mut cache).unwrap();
        assert_eq!(p, Point2D::new(Coordinate::Crs(3857), [x, y]));
    }

    #[test]
    fn linestring2d_reproject_carries_elevation() {
        let mut cache = ReprojectionCache::new();
        let raw = [[139.7, 35.6, 10.0], [139.8, 35.7, 20.0]];
        let expected: Vec<[f64; 3]> = raw
            .iter()
            .map(|&[x, y, z]| cache.transform(4326, 3857, [x, y, z]).unwrap())
            .collect();

        let mut ls = LineString2D::from_coords_with_elevation(Coordinate::Crs(4326), raw);
        ls.reproject(3857, &mut cache).unwrap();
        assert_eq!(
            ls,
            LineString2D::from_coords_with_elevation(Coordinate::Crs(3857), expected)
        );
    }

    #[test]
    fn collection_reproject_dispatches_to_each_member() {
        let mut cache = ReprojectionCache::new();
        let a = [139.7, 35.6, 1.0];
        let b = [140.0, 35.9, 2.0];
        let ea = cache.transform(4979, 4978, a).unwrap();
        let eb = cache.transform(4979, 4978, b).unwrap();

        let mut col = Collection3D::new([
            Euclidean3DGeometry::Point(Point3D::new(Coordinate::Crs(4979), a)),
            Euclidean3DGeometry::Point(Point3D::new(Coordinate::Crs(4979), b)),
        ]);
        col.reproject(4978, &mut cache).unwrap();
        assert_eq!(
            col,
            Collection3D::new([
                Euclidean3DGeometry::Point(Point3D::new(Coordinate::Crs(4978), ea)),
                Euclidean3DGeometry::Point(Point3D::new(Coordinate::Crs(4978), eb)),
            ])
        );
    }

    #[test]
    fn mismatched_elevation_buffer_is_error() {
        let mut cache = ReprojectionCache::new();
        let mut coords = [[139.7, 35.6], [139.8, 35.7]];
        let mut z = [10.0]; // one short of `coords`
        assert!(matches!(
            transform_coords_2d(&mut cache, 4326, 3857, &mut coords, Some(&mut z)),
            Err(Error::Projection(_))
        ));
    }

    #[test]
    fn reproject_same_crs_is_noop() {
        let mut cache = ReprojectionCache::new();
        let mut p = Point3D::new(Coordinate::Crs(4979), [139.7, 35.6, 50.0]);
        p.reproject(4979, &mut cache).unwrap();
        assert_eq!(p, Point3D::new(Coordinate::Crs(4979), [139.7, 35.6, 50.0]));
    }

    #[test]
    fn non_crs_frame_is_error() {
        let mut cache = ReprojectionCache::new();
        let mut p = Point3D::new(Coordinate::Euclidean, [1.0, 2.0, 3.0]);
        assert!(matches!(
            p.reproject(4326, &mut cache),
            Err(Error::Projection(_))
        ));
    }

    #[test]
    fn unsupported_leaf_is_error() {
        let mut cache = ReprojectionCache::new();
        let pc = PointCloud::from_positions(Coordinate::Crs(4979), [[139.7, 35.6, 1.0]]);
        let mut geom = Euclidean3DGeometry::PointCloud(Box::new(pc));
        assert!(matches!(
            geom.reproject(4978, &mut cache),
            Err(Error::Projection(_))
        ));
    }
}
