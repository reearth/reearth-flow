//! LineString constructors.
//!
//! A `LineString` is a flat coordinate chain: every reader (CityGML `gml:Curve`,
//! shapefile polylines, GeoJSON, WKT, GeoPackage WKB) hands one over as a plain
//! sequence of points — no shared pool, no indices, no rings. So construction is
//! just wrapping that buffer; the 2D form optionally carries per-vertex elevation
//! (2.5D), split out of `[x, y, z]` input. Lines are stored as given (not closed)
//! and carry no appearance.

use crate::coordinate::Coordinate;
use crate::error::Error;

use super::{LineString2D, LineString3D};

impl LineString2D {
    /// Build a 2D polyline from `[x, y]` coordinates. The result is pure 2D (no
    /// elevation); for per-vertex elevation use
    /// [`LineString2D::from_coords_with_elevation`].
    pub fn from_coords(coordinate: Coordinate, coords: impl IntoIterator<Item = [f64; 2]>) -> Self {
        Self {
            coordinate,
            coords: coords.into_iter().collect(),
            z: None,
        }
    }

    /// Build a 2.5D polyline from `[x, y, z]` coordinates: the `(x, y)` populate
    /// `coords` and the `z` the parallel elevation buffer.
    pub fn from_coords_with_elevation(
        coordinate: Coordinate,
        coords: impl IntoIterator<Item = [f64; 3]>,
    ) -> Self {
        let coords = coords.into_iter();
        let cap = coords.size_hint().0;
        let mut xy = Vec::with_capacity(cap);
        let mut z = Vec::with_capacity(cap);
        for [x, y, elevation] in coords {
            xy.push([x, y]);
            z.push(elevation);
        }
        Self {
            coordinate,
            coords: xy.into_boxed_slice(),
            z: Some(z.into_boxed_slice()),
        }
    }

    /// Build from an already-built coordinate buffer and optional parallel
    /// elevation. Errors if `z` is present and not the same length as `coords`.
    pub fn from_raw_parts(
        coordinate: Coordinate,
        coords: Box<[[f64; 2]]>,
        z: Option<Box<[f64]>>,
    ) -> Result<Self, Error> {
        if let Some(z) = z.as_ref() {
            if z.len() != coords.len() {
                return Err(Error::invalid_geometry(format!(
                    "elevation buffer length {} does not match coordinate count {}",
                    z.len(),
                    coords.len()
                )));
            }
        }
        Ok(Self {
            coordinate,
            coords,
            z,
        })
    }
}

impl LineString3D {
    /// Build a 3D polyline from `[x, y, z]` coordinates.
    pub fn from_coords(coordinate: Coordinate, coords: impl IntoIterator<Item = [f64; 3]>) -> Self {
        Self {
            coordinate,
            coords: coords.into_iter().collect(),
        }
    }

    /// Build from an already-built coordinate buffer.
    pub fn from_raw_parts(coordinate: Coordinate, coords: Box<[[f64; 3]]>) -> Self {
        Self { coordinate, coords }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_coords_2d_is_open_and_pure() {
        let l =
            LineString2D::from_coords(Coordinate::Euclidean, [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]);
        // Stored as given: no closing vertex appended.
        assert_eq!(l.coords.len(), 3);
        assert_eq!(l.coords[0], [0.0, 0.0]);
        assert!(l.z.is_none());
        assert_eq!(l.coordinate, Coordinate::Euclidean);
    }

    #[test]
    fn from_coords_with_elevation_splits_z() {
        let l = LineString2D::from_coords_with_elevation(
            Coordinate::Euclidean,
            [[0.0, 0.0, 10.0], [1.0, 0.0, 11.0]],
        );
        assert_eq!(l.coords, vec![[0.0, 0.0], [1.0, 0.0]].into_boxed_slice());
        assert_eq!(l.z.as_deref(), Some(&[10.0, 11.0][..]));
    }

    #[test]
    fn from_raw_parts_2d_rejects_unparallel_z() {
        let coords: Box<[[f64; 2]]> = vec![[0.0, 0.0], [1.0, 0.0]].into_boxed_slice();
        let err = LineString2D::from_raw_parts(
            Coordinate::Euclidean,
            coords,
            Some(vec![0.0].into_boxed_slice()),
        )
        .unwrap_err();
        assert!(matches!(err, Error::InvalidGeometry(_)));
    }

    #[test]
    fn from_raw_parts_2d_accepts_parallel_z() {
        let coords: Box<[[f64; 2]]> = vec![[0.0, 0.0], [1.0, 0.0]].into_boxed_slice();
        let l = LineString2D::from_raw_parts(
            Coordinate::Euclidean,
            coords,
            Some(vec![3.0, 4.0].into_boxed_slice()),
        )
        .unwrap();
        assert_eq!(l.z.as_deref(), Some(&[3.0, 4.0][..]));
    }

    #[test]
    fn from_coords_3d() {
        let l =
            LineString3D::from_coords(Coordinate::Euclidean, [[0.0, 0.0, 0.0], [1.0, 2.0, 3.0]]);
        assert_eq!(l.coords.len(), 2);
        assert_eq!(l.coords[1], [1.0, 2.0, 3.0]);
    }
}
