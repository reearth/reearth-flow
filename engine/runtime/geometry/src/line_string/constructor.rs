//! LineString constructors.
//!
//! A `LineString` is a flat coordinate chain: every reader (CityGML `gml:Curve`,
//! shapefile polylines, GeoJSON, WKT, GeoPackage WKB) hands one over as a plain
//! sequence of points — no shared pool, no indices, no rings. So construction is
//! just wrapping that buffer; the 2D form optionally carries the one elevation the
//! whole chain lies at (2.5D). Lines are stored as given (not closed) and carry no
//! appearance.

use crate::coordinate::CoordinateFrame;

use super::{LineString2D, LineString3D};

impl LineString2D {
    /// Build a 2D polyline from `[x, y]` coordinates. The result is pure 2D (no
    /// elevation); to place the chain at a height use
    /// [`LineString2D::from_coords_at_elevation`].
    pub fn from_coords(frame: CoordinateFrame, coords: impl IntoIterator<Item = [f64; 2]>) -> Self {
        Self {
            frame,
            coords: coords.into_iter().collect(),
            z: None,
        }
    }

    /// Build a 2.5D polyline: an `[x, y]` chain lying wholly at `elevation`. A
    /// chain whose vertices sit at differing heights is not representable here —
    /// that is a [`LineString3D`].
    pub fn from_coords_at_elevation(
        frame: CoordinateFrame,
        coords: impl IntoIterator<Item = [f64; 2]>,
        elevation: f64,
    ) -> Self {
        Self {
            frame,
            coords: coords.into_iter().collect(),
            z: Some(elevation),
        }
    }

    /// Build from an already-built coordinate buffer and the chain's optional
    /// elevation.
    pub fn from_raw_parts(frame: CoordinateFrame, coords: Box<[[f64; 2]]>, z: Option<f64>) -> Self {
        Self { frame, coords, z }
    }
}

impl LineString3D {
    /// Build a 3D polyline from `[x, y, z]` coordinates.
    pub fn from_coords(frame: CoordinateFrame, coords: impl IntoIterator<Item = [f64; 3]>) -> Self {
        Self {
            frame,
            coords: coords.into_iter().collect(),
        }
    }

    /// Build from an already-built coordinate buffer.
    pub fn from_raw_parts(frame: CoordinateFrame, coords: Box<[[f64; 3]]>) -> Self {
        Self { frame, coords }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_coords_2d_is_open_and_pure() {
        let l = LineString2D::from_coords(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]],
        );
        // Stored as given: no closing vertex appended.
        assert_eq!(l.coords.len(), 3);
        assert_eq!(l.coords[0], [0.0, 0.0]);
        assert!(l.z.is_none());
        assert_eq!(l.frame, CoordinateFrame::Euclidean);
    }

    #[test]
    fn from_coords_at_elevation_keeps_one_height() {
        let l = LineString2D::from_coords_at_elevation(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0], [1.0, 0.0]],
            10.0,
        );
        assert_eq!(l.coords, vec![[0.0, 0.0], [1.0, 0.0]].into_boxed_slice());
        // One elevation for the chain, not one per vertex.
        assert_eq!(l.elevation(), Some(10.0));
    }

    #[test]
    fn from_raw_parts_2d_carries_elevation() {
        let coords: Box<[[f64; 2]]> = vec![[0.0, 0.0], [1.0, 0.0]].into_boxed_slice();
        let l = LineString2D::from_raw_parts(CoordinateFrame::Euclidean, coords, Some(3.0));
        assert_eq!(l.elevation(), Some(3.0));
    }

    #[test]
    fn from_coords_3d() {
        let l = LineString3D::from_coords(
            CoordinateFrame::Euclidean,
            [[0.0, 0.0, 0.0], [1.0, 2.0, 3.0]],
        );
        assert_eq!(l.coords.len(), 2);
        assert_eq!(l.coords[1], [1.0, 2.0, 3.0]);
    }
}
