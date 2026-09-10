//! CZML reading against the new geometry model.
//!
//! CZML states cartographic positions as `[longitude, latitude, height]` in WGS84.
//! EPSG:4979's own authority axis order is `(latitude, longitude, height)`, so every
//! cartographic coordinate is stored latitude-first here; `force2D` demotes to
//! EPSG:4326, which is latitude-first too. A `cartesian` position is ECEF metres,
//! which is EPSG:4978, stored X, Y, Z verbatim.
//!
//! Nothing is reprojected here. The CZML Writer's `to_wgs84` reprojects whatever
//! frame it is handed, so tagging the frame correctly is the whole job.
//!
//! Not read: interval-scoped property arrays, `reference`/`references` positions,
//! time-tagged `cartesian`, `ellipse.rotation`, `wall` and `corridor` surfaces, and
//! the `box`, `cylinder`, `ellipsoid`, `polylineVolume`, `path`, `model` and
//! `tileset` graphics. Each is reported, never silently dropped.

use reearth_flow_diagnostics::ErrorCode;
use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
use serde_json::Value;

/// WGS84 geographic 3D. Authority axis order `(latitude, longitude, height)`.
pub(super) const WGS84_GEOGRAPHIC_3D: EpsgCode = EpsgCode::new(4979);
/// WGS84 geographic 2D, what `demote_to_2d` turns 4979 into. Latitude-first.
pub(super) const WGS84_GEOGRAPHIC_2D: EpsgCode = EpsgCode::new(4326);
/// WGS84 geocentric (ECEF), metres. Axis order X, Y, Z, no swap.
pub(super) const WGS84_GEOCENTRIC: EpsgCode = EpsgCode::new(4978);

/// Which coordinate system a CZML value's numbers are in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CzmlFrame {
    /// `cartographicDegrees` or `cartographicRadians`: WGS84 geographic.
    Cartographic,
    /// `cartesian` in the FIXED reference frame: ECEF metres.
    Geocentric,
}

/// Coordinates lifted off a CZML value, already in the storage order their frame
/// declares: `[latitude, longitude, height]` for `Cartographic`, `[x, y, z]` for
/// `Geocentric`.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Coords {
    pub(super) frame: CzmlFrame,
    pub(super) triples: Vec<[f64; 3]>,
}

/// Why coordinates could not be lifted off a value.
#[derive(Debug, Clone, PartialEq)]
pub(super) enum PositionProblem {
    /// Valid CZML this reader does not support. The caller reports it and skips
    /// the packet.
    Unsupported(ErrorCode),
    /// Present and malformed. The caller fails the read, per Action Standard 4.3.
    Malformed(String),
}

/// The frame geometry gets tagged with, or why it cannot be built.
pub(super) fn resolve_frame(
    frame: CzmlFrame,
    force_2d: bool,
) -> Result<CoordinateFrame, ErrorCode> {
    match (frame, force_2d) {
        (CzmlFrame::Cartographic, false) => Ok(CoordinateFrame::Crs(WGS84_GEOGRAPHIC_3D)),
        (CzmlFrame::Cartographic, true) => Ok(CoordinateFrame::Crs(WGS84_GEOGRAPHIC_2D)),
        (CzmlFrame::Geocentric, false) => Ok(CoordinateFrame::Crs(WGS84_GEOCENTRIC)),
        // EPSG:4978 has no 2D counterpart for `demote_to_2d` to produce, and an
        // ECEF Z is not a height, so dropping it is meaningless rather than lossy.
        (CzmlFrame::Geocentric, true) => Err(ErrorCode::CzmlGeocentricForce2d),
    }
}

/// A single CZML `position`. Errors on a time-tagged `cartesian`, which needs a
/// sample representation the writer cannot read back; see the design spec's
/// "timeseries asymmetry".
pub(super) fn position(value: &Value) -> Result<Option<Coords>, PositionProblem> {
    let Some(raw) = raw_numbers(value)? else {
        return Ok(None);
    };
    // Groups of four are time-tagged. `parse_time_tagged_position` in `czml.rs`
    // handles the cartographic case before we ever get here, so reaching this with
    // a multiple of four means a cartesian one.
    if raw.numbers.len() > 3 && raw.numbers.len() % 4 == 0 {
        return Err(PositionProblem::Unsupported(
            ErrorCode::CzmlUnsupportedPosition,
        ));
    }
    Ok(Some(into_coords(raw)?))
}

/// A CZML `PositionList`, as carried by `polygon.positions`, `polyline.positions`
/// and friends.
pub(super) fn position_list(value: &Value) -> Result<Option<Coords>, PositionProblem> {
    let Some(raw) = raw_numbers(value)? else {
        return Ok(None);
    };
    Ok(Some(into_coords(raw)?))
}

/// A flat number list plus the frame its key implies.
struct Raw {
    frame: CzmlFrame,
    /// True when the numbers are `cartographicRadians`, so the horizontal pair
    /// still needs converting.
    radians: bool,
    numbers: Vec<f64>,
}

/// Pull the flat number list off a CZML position or position list, rejecting the
/// shapes this reader does not support.
fn raw_numbers(value: &Value) -> Result<Option<Raw>, PositionProblem> {
    // A property written as an array is an interval-scoped collection: every CZML
    // property is `"type": ["array", "object"]` with `"items": {"$ref": "#"}`.
    if value.is_array() {
        return Err(PositionProblem::Unsupported(
            ErrorCode::CzmlUnsupportedPosition,
        ));
    }
    let Some(obj) = value.as_object() else {
        return Ok(None);
    };
    if obj.contains_key("reference") || obj.contains_key("references") {
        return Err(PositionProblem::Unsupported(
            ErrorCode::CzmlUnsupportedPosition,
        ));
    }
    if let Some(cartesian) = obj.get("cartesian") {
        // FIXED is the schema's default, so an absent `referenceFrame` is FIXED.
        match obj.get("referenceFrame").and_then(Value::as_str) {
            None | Some("FIXED") => {}
            Some(_) => {
                return Err(PositionProblem::Unsupported(ErrorCode::CzmlInertialFrame));
            }
        }
        return Ok(Some(Raw {
            frame: CzmlFrame::Geocentric,
            radians: false,
            numbers: numbers(cartesian, "cartesian")?,
        }));
    }
    for (key, radians) in [("cartographicDegrees", false), ("cartographicRadians", true)] {
        if let Some(list) = obj.get(key) {
            return Ok(Some(Raw {
                frame: CzmlFrame::Cartographic,
                radians,
                numbers: numbers(list, key)?,
            }));
        }
    }
    Ok(None)
}

/// Every entry of `value` as an `f64`, or `Malformed` naming the offending key.
fn numbers(value: &Value, key: &str) -> Result<Vec<f64>, PositionProblem> {
    let Some(array) = value.as_array() else {
        return Err(PositionProblem::Malformed(format!(
            "`{key}` is not an array"
        )));
    };
    array
        .iter()
        .map(Value::as_f64)
        .collect::<Option<Vec<f64>>>()
        .ok_or_else(|| PositionProblem::Malformed(format!("`{key}` holds a non-numeric entry")))
}

/// Chunk a flat number list into triples, converting radians and swapping to the
/// frame's storage order.
fn into_coords(raw: Raw) -> Result<Coords, PositionProblem> {
    if raw.numbers.is_empty() || raw.numbers.len() % 3 != 0 {
        return Err(PositionProblem::Malformed(format!(
            "a position list needs a whole number of [x, y, z] triples, got {} values",
            raw.numbers.len()
        )));
    }
    let triples = raw
        .numbers
        .chunks_exact(3)
        .map(|chunk| match raw.frame {
            // Stored X, Y, Z: EPSG:4978 declares that order, so nothing moves.
            CzmlFrame::Geocentric => [chunk[0], chunk[1], chunk[2]],
            // CZML gives [longitude, latitude, height]; EPSG:4979 and EPSG:4326
            // both declare latitude first, so the horizontal pair swaps. Height is
            // metres in both, and is never converted.
            CzmlFrame::Cartographic => {
                let (lon, lat) = if raw.radians {
                    (chunk[0].to_degrees(), chunk[1].to_degrees())
                } else {
                    (chunk[0], chunk[1])
                };
                [lat, lon, chunk[2]]
            }
        })
        .collect();
    Ok(Coords {
        frame: raw.frame,
        triples,
    })
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;

    #[test]
    fn cartographic_degrees_store_latitude_first() {
        // Tokyo. Latitude 35.68 and longitude 139.76 are in disjoint ranges, so a
        // fixture written in the wrong order could not pass this.
        let value = serde_json::json!({ "cartographicDegrees": [139.76, 35.68, 10.0] });
        let coords = position(&value).unwrap().expect("a static position");
        assert_eq!(coords.frame, CzmlFrame::Cartographic);
        assert_eq!(coords.triples, vec![[35.68, 139.76, 10.0]]);
    }

    #[test]
    fn cartographic_radians_convert_only_the_horizontal_pair() {
        let value = serde_json::json!({
            "cartographicRadians": [std::f64::consts::FRAC_PI_2, 0.0, 10.0],
        });
        let coords = position(&value).unwrap().expect("a static position");
        let [lat, lon, height] = coords.triples[0];
        assert!((lat - 0.0).abs() < 1e-9, "lat was {lat}");
        assert!((lon - 90.0).abs() < 1e-9, "lon was {lon}");
        // Height is metres, never an angle: it must not be touched.
        assert_eq!(height, 10.0);
    }

    #[test]
    fn cartesian_is_geocentric_and_stored_verbatim() {
        let value = serde_json::json!({ "cartesian": [1.0, 2.0, 3.0] });
        let coords = position(&value).unwrap().expect("a static position");
        assert_eq!(coords.frame, CzmlFrame::Geocentric);
        // No swap: EPSG:4978's axis order is X, Y, Z.
        assert_eq!(coords.triples, vec![[1.0, 2.0, 3.0]]);
    }

    #[test]
    fn an_explicit_fixed_reference_frame_is_still_geocentric() {
        let value = serde_json::json!({
            "cartesian": [1.0, 2.0, 3.0],
            "referenceFrame": "FIXED",
        });
        let coords = position(&value).unwrap().expect("a static position");
        assert_eq!(coords.frame, CzmlFrame::Geocentric);
    }

    #[test]
    fn an_inertial_frame_is_reported_not_placed() {
        let value = serde_json::json!({
            "cartesian": [1.0, 2.0, 3.0],
            "referenceFrame": "INERTIAL",
        });
        assert_eq!(
            position(&value).unwrap_err(),
            PositionProblem::Unsupported(ErrorCode::CzmlInertialFrame),
        );
    }

    #[test]
    fn an_interval_array_is_reported_not_guessed() {
        // Every CZML property may be an array of interval-scoped objects.
        let value = serde_json::json!([
            { "interval": "2024-01-01T00:00:00Z/2024-01-02T00:00:00Z",
              "cartographicDegrees": [139.76, 35.68, 10.0] },
        ]);
        assert_eq!(
            position(&value).unwrap_err(),
            PositionProblem::Unsupported(ErrorCode::CzmlUnsupportedPosition),
        );
    }

    #[test]
    fn a_reference_position_is_reported() {
        let value = serde_json::json!({ "reference": "other-entity#position" });
        assert_eq!(
            position(&value).unwrap_err(),
            PositionProblem::Unsupported(ErrorCode::CzmlUnsupportedPosition),
        );
    }

    #[test]
    fn a_time_tagged_cartesian_is_reported_not_misread() {
        // Groups of four: time, X, Y, Z. Reading these as triples would silently
        // scramble every coordinate.
        let value = serde_json::json!({
            "cartesian": [0.0, 1.0, 2.0, 3.0, 60.0, 4.0, 5.0, 6.0],
        });
        assert_eq!(
            position(&value).unwrap_err(),
            PositionProblem::Unsupported(ErrorCode::CzmlUnsupportedPosition),
        );
    }

    #[test]
    fn a_ragged_position_list_fails_the_read() {
        let value = serde_json::json!({ "cartographicDegrees": [1.0, 2.0, 3.0, 4.0, 5.0] });
        assert!(matches!(
            position_list(&value).unwrap_err(),
            PositionProblem::Malformed(_),
        ));
    }

    #[test]
    fn a_non_numeric_coordinate_fails_the_read() {
        let value = serde_json::json!({ "cartographicDegrees": [1.0, "two", 3.0] });
        assert!(matches!(
            position(&value).unwrap_err(),
            PositionProblem::Malformed(_),
        ));
    }

    #[test]
    fn a_value_with_no_recognised_key_is_absent_not_an_error() {
        let value = serde_json::json!({ "show": true });
        assert!(position(&value).unwrap().is_none());
    }

    #[test]
    fn a_position_list_keeps_every_triple() {
        let value = serde_json::json!({
            "cartographicDegrees": [139.0, 35.0, 0.0, 140.0, 36.0, 1.0],
        });
        let coords = position_list(&value).unwrap().expect("a list");
        assert_eq!(coords.triples, vec![[35.0, 139.0, 0.0], [36.0, 140.0, 1.0]]);
    }

    #[test]
    fn cartographic_frames_are_latitude_first_in_both_dimensions() {
        assert_eq!(
            resolve_frame(CzmlFrame::Cartographic, false).unwrap(),
            CoordinateFrame::Crs(WGS84_GEOGRAPHIC_3D),
        );
        assert_eq!(
            resolve_frame(CzmlFrame::Cartographic, true).unwrap(),
            CoordinateFrame::Crs(WGS84_GEOGRAPHIC_2D),
        );
        // Both declare (latitude, longitude): orientation sign -1. This is the
        // fact the whole storage order depends on, so pin it here.
        assert_eq!(
            CoordinateFrame::Crs(WGS84_GEOGRAPHIC_3D).orientation_sign(),
            Ok(-1),
        );
        assert_eq!(
            CoordinateFrame::Crs(WGS84_GEOGRAPHIC_2D).orientation_sign(),
            Ok(-1),
        );
    }

    #[test]
    fn geocentric_has_no_two_dimensional_form() {
        assert_eq!(
            resolve_frame(CzmlFrame::Geocentric, false).unwrap(),
            CoordinateFrame::Crs(WGS84_GEOCENTRIC),
        );
        // EPSG:4978 is X, Y, Z: no swap, and no 2D counterpart to demote to.
        assert_eq!(
            CoordinateFrame::Crs(WGS84_GEOCENTRIC).orientation_sign(),
            Ok(1),
        );
        assert_eq!(
            resolve_frame(CzmlFrame::Geocentric, true),
            Err(ErrorCode::CzmlGeocentricForce2d),
        );
    }
}
