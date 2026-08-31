//! Placing CZML coordinates on the globe.
//!
//! CZML's `cartographicDegrees` is longitude, latitude and height on the WGS84
//! globe, so every coordinate the writer emits has to get there first. This
//! mirrors `cesium3dtiles/next/mesh.rs`, which solves the same problem for the
//! other Cesium-family writer; keeping the two the same is deliberate.
//!
//! `to_wgs84` reprojects via [`transform_coords_3d`], which (like every other
//! call site in this crate) applies no axis normalization: each CRS keeps its
//! own EPSG-authority axis order. EPSG:4979's authority order is
//! **(latitude, longitude, height)**, not (longitude, latitude, height) — the
//! same order `cesium3dtiles/next/quadtree.rs`'s `GeoBox::of` documents for
//! the same EPSG code. So `to_wgs84`'s output triples are `[lat, lon, height]`;
//! a caller building CZML's `cartographicDegrees` (which wants
//! `[lon, lat, height]`) must swap the first two components itself.

use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
use reearth_flow_geometry::ops::reproject::transform_coords_3d;
use reearth_flow_geometry::ops::ReprojectionCache;

/// WGS84 geographic (EPSG:4979). Its own authority axis order is
/// (latitude, longitude, ellipsoidal height) — see the module docs above.
pub(crate) const WGS84_GEOGRAPHIC: EpsgCode = EpsgCode::new(4979);

/// Why a feature's coordinates could not be placed.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum FrameError {
    /// The frame is not a concrete CRS, so there is no way to know where on the
    /// globe these numbers are. Deliberately not defaulted to 4326: our inputs
    /// are routinely projected metres, and calling metres "degrees" writes the
    /// feature into the Gulf of Guinea.
    Unplaceable,
    /// The CRS is known but the transform failed.
    Transform(String),
}

/// The EPSG a feature's coordinates are reprojected from, or `None` when the
/// frame carries no concrete CRS.
pub(crate) fn source_crs(frame: &CoordinateFrame) -> Option<EpsgCode> {
    match frame {
        CoordinateFrame::Crs(epsg) => Some(*epsg),
        _ => None,
    }
}

/// Reproject `coords` in place to WGS84 geographic.
///
/// On failure `coords` is left untouched, so a caller cannot emit a half
/// reprojected feature.
pub(crate) fn to_wgs84(
    cache: &mut ReprojectionCache,
    frame: &CoordinateFrame,
    coords: &mut Vec<[f64; 3]>,
) -> Result<(), FrameError> {
    let source = source_crs(frame).ok_or(FrameError::Unplaceable)?;
    if source == WGS84_GEOGRAPHIC {
        return Ok(());
    }
    let mut work = coords.clone();
    transform_coords_3d(cache, source, WGS84_GEOGRAPHIC, &mut work)
        .map_err(|e| FrameError::Transform(e.to_string()))?;
    *coords = work;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};

    #[test]
    fn a_concrete_crs_resolves() {
        let frame = CoordinateFrame::Crs(EpsgCode::new(6677));
        assert_eq!(source_crs(&frame), Some(EpsgCode::new(6677)));
    }

    #[test]
    fn a_euclidean_frame_does_not_resolve() {
        assert_eq!(source_crs(&CoordinateFrame::default()), None);
    }

    #[test]
    fn an_unplaceable_frame_is_reported_not_guessed() {
        // The failure must be Unplaceable, NOT a silent fallback to 4326.
        let mut cache = ReprojectionCache::default();
        let mut coords = vec![[1.0, 2.0, 3.0]];
        let err = to_wgs84(&mut cache, &CoordinateFrame::default(), &mut coords).unwrap_err();
        assert!(matches!(err, FrameError::Unplaceable));
        // And the coordinates must be untouched, so a caller cannot half-write them.
        assert_eq!(coords, vec![[1.0, 2.0, 3.0]]);
    }

    #[test]
    fn projected_metres_become_plausible_degrees() {
        // A point in Japan Plane Rectangular zone IX (EPSG:6677), near its origin.
        let mut cache = ReprojectionCache::default();
        let mut coords = vec![[0.0, 0.0, 0.0]];
        to_wgs84(
            &mut cache,
            &CoordinateFrame::Crs(EpsgCode::new(6677)),
            &mut coords,
        )
        .expect("6677 reprojects");
        // `to_wgs84` reprojects to EPSG:4979 without normalizing its axis
        // order (matching `cesium3dtiles/next/mesh.rs` and
        // `transform_coords_3d` elsewhere in this crate, which use each CRS's
        // own authority-defined axis order throughout); EPSG:4979's official
        // order is (lat, lon, height), the same convention documented on
        // `GeoBox::of` in `cesium3dtiles/next/quadtree.rs`.
        let [lat, lon, _h] = coords[0];
        // Zone IX's origin is 36N 139d50mE. Assert we land in Japan, not the
        // Gulf of Guinea, which is what emitting the raw metres would produce.
        assert!((138.0..142.0).contains(&lon), "lon was {lon}");
        assert!((34.0..38.0).contains(&lat), "lat was {lat}");
    }

    #[test]
    fn wgs84_input_passes_through_untouched_in_lat_lon_order() {
        // Input is already EPSG:4979, so it is stated in 4979's own authority
        // order: latitude first, then longitude, then height (see the module
        // docs). Tokyo's lat (35.68) and lon (139.76) ranges are disjoint, so
        // a fixture written lon-first — or an implementation that swapped —
        // could not pass this.
        let mut cache = ReprojectionCache::default();
        let mut coords = vec![[35.68, 139.76, 10.0]];
        to_wgs84(
            &mut cache,
            &CoordinateFrame::Crs(WGS84_GEOGRAPHIC),
            &mut coords,
        )
        .expect("4979 reprojects");
        // The same-CRS short circuit returns without transforming at all, so
        // this is exact equality, not an approximate comparison: any drift in
        // any component means the short circuit was skipped and a real
        // transform ran.
        assert_eq!(
            coords,
            vec![[35.68, 139.76, 10.0]],
            "a 4979 input must come back byte-identical, still [lat, lon, height]"
        );
    }
}
