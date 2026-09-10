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

use bytes::Bytes;
use reearth_flow_diagnostics::ErrorCode;
use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};
use reearth_flow_geometry::{
    line_string::{LineString2D, LineString3D},
    point::{Point2D, Point3D},
    polygon::{Polygon2D, Polygon3D},
    Euclidean2DGeometry, Euclidean3DGeometry, Geometry,
};
use reearth_flow_runtime::executor_operation::NodeContext;
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use serde_json::Value;

use super::{
    extract_common_attributes, extract_extra_czml_properties, extract_polygon_holes,
    extract_rectangle_bounds, parse_time_tagged_position, sample_timestamp,
    CzmlReaderCompiledParam, TimeSamplingStrategy,
};
use crate::errors::SourceError;

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
    for (key, radians) in [
        ("cartographicDegrees", false),
        ("cartographicRadians", true),
    ] {
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

/// Close a ring the way CZML means it. CZML never repeats the first vertex because
/// Cesium closes implicitly, while `Polygon::from_rings` stores rings verbatim and
/// never closes them. Translating that implicit closure is the reader's job.
fn closed(ring: &[[f64; 3]]) -> Vec<[f64; 3]> {
    let mut ring = ring.to_vec();
    match (ring.first().copied(), ring.last().copied()) {
        (Some(first), Some(last)) if first != last => ring.push(first),
        _ => {}
    }
    ring
}

/// A point at the first coordinate.
pub(super) fn point_geometry(coords: &Coords, force_2d: bool) -> Result<Geometry, PositionProblem> {
    let frame = resolve_frame(coords.frame, force_2d).map_err(PositionProblem::Unsupported)?;
    let Some(&[x, y, z]) = coords.triples.first() else {
        return Err(PositionProblem::Malformed(
            "a position needs one coordinate".to_string(),
        ));
    };
    Ok(if force_2d {
        Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(frame, [x, y])))
    } else {
        Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(frame, [x, y, z])))
    })
}

/// A line string through every coordinate. Not closed: a polyline, corridor
/// centreline or wall centreline is an open chain.
pub(super) fn line_geometry(coords: &Coords, force_2d: bool) -> Result<Geometry, PositionProblem> {
    let frame = resolve_frame(coords.frame, force_2d).map_err(PositionProblem::Unsupported)?;
    if coords.triples.len() < 2 {
        return Err(PositionProblem::Malformed(format!(
            "a line needs at least 2 coordinates, got {}",
            coords.triples.len()
        )));
    }
    Ok(if force_2d {
        Geometry::Euclidean2D(Euclidean2DGeometry::LineString(LineString2D::from_coords(
            frame,
            coords.triples.iter().map(|&[x, y, _]| [x, y]),
        )))
    } else {
        Geometry::Euclidean3D(Euclidean3DGeometry::LineString(LineString3D::from_coords(
            frame,
            coords.triples.iter().copied(),
        )))
    })
}

/// A polygon: `coords` is the exterior ring, `holes` the interiors. Every ring is
/// closed on the way in. Vertex order is preserved exactly: see
/// `a_ccw_lon_lat_ring_is_canonically_counter_clockwise` for why reversing would be
/// wrong, and note that CZML specifies no winding at all, so none is normalised.
pub(super) fn area_geometry(
    coords: &Coords,
    holes: Vec<Vec<[f64; 3]>>,
    force_2d: bool,
) -> Result<Geometry, PositionProblem> {
    let frame = resolve_frame(coords.frame, force_2d).map_err(PositionProblem::Unsupported)?;
    if coords.triples.len() < 3 {
        return Err(PositionProblem::Malformed(format!(
            "a polygon ring needs at least 3 coordinates, got {}",
            coords.triples.len()
        )));
    }
    let exterior = closed(&coords.triples);
    let interiors: Vec<Vec<[f64; 3]>> = holes
        .iter()
        // A ring of fewer than 3 vertices bounds no area; the model drops empty
        // interiors anyway, and this keeps a stray one from becoming a sliver.
        .filter(|hole| hole.len() >= 3)
        .map(|hole| closed(hole))
        .collect();
    Ok(if force_2d {
        Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
            Polygon2D::from_rings(
                frame,
                exterior.iter().map(|&[x, y, _]| [x, y]),
                interiors
                    .iter()
                    .map(|hole| hole.iter().map(|&[x, y, _]| [x, y]).collect::<Vec<_>>()),
            ),
        )))
    } else {
        Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
            Polygon3D::from_rings(frame, exterior, interiors),
        )))
    })
}

/// Read a whole CZML document.
///
/// Three outcomes per packet, per Action Standard 4.3. Input that is present and
/// malformed fails the read with the packet id in the message. Valid CZML this
/// reader does not support is reported through `ctx.report_drop` and the packet is
/// skipped. A packet with attributes and no geometry becomes a `Geometry::None`
/// feature, which is a change from the old world, where it vanished with its
/// attributes.
pub(super) fn read(
    ctx: &NodeContext,
    content: &Bytes,
    params: &CzmlReaderCompiledParam,
) -> Result<Vec<Feature>, SourceError> {
    let text = String::from_utf8(content.to_vec())
        .map_err(|e| SourceError::CzmlReader(format!("Invalid UTF-8: {e}")))?;
    let packets: Vec<Value> = serde_json::from_str(&text)
        .map_err(|e| SourceError::CzmlReader(format!("Failed to parse CZML: {e}")))?;

    let mut features = Vec::with_capacity(packets.len());
    for packet in &packets {
        let is_document = packet.get("version").is_some_and(Value::is_string);
        if is_document && params.skip_document_packet {
            continue;
        }
        match packet_features(packet, params, is_document) {
            Ok(built) => features.extend(built),
            Err(PositionProblem::Unsupported(code)) => {
                ctx.report_drop(code, None, None);
            }
            Err(PositionProblem::Malformed(reason)) => {
                return Err(SourceError::CzmlReader(format!(
                    "packet `{}`: {reason}",
                    packet_id(packet)
                )));
            }
        }
    }
    Ok(features)
}

/// The packet's `id`, or a placeholder, for error messages.
fn packet_id(packet: &Value) -> &str {
    packet
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("<no id>")
}

/// One packet's features. Mirrors the old world's `packet_to_features`: a
/// time-tagged position takes the timeseries path, everything else builds static
/// geometry.
///
/// Takes no `NodeContext` on purpose: every problem is returned as a
/// `PositionProblem` and `read` alone decides whether it reports or fails. Keeping
/// the reporting in one place is what makes the three-way failure policy auditable.
fn packet_features(
    packet: &Value,
    params: &CzmlReaderCompiledParam,
    is_document: bool,
) -> Result<Vec<Feature>, PositionProblem> {
    let base = extract_common_attributes(packet);

    if let Some(position) = packet.get("position") {
        if let Some(ts) = parse_time_tagged_position(position) {
            return timeseries_features(&ts, &base, packet, params);
        }
    }

    let geometry = packet_geometry(packet, params.force_2d)?;
    let mut feature = Feature::new_with_attributes_and_geometry(base, geometry);
    let attributes = std::sync::Arc::make_mut(&mut feature.attributes);
    extract_extra_czml_properties(packet, attributes);
    if is_document {
        // `CZML_COMMON_PROPERTIES` holds `version` and `clock` out of the shared
        // capture, so a kept document packet would otherwise lose exactly the
        // fields that identify it.
        for key in ["version", "clock"] {
            if let Some(value) = packet.get(key) {
                attributes.insert(
                    Attribute::new(format!("czml.{key}")),
                    AttributeValue::String(serde_json::to_string(value).unwrap_or_default()),
                );
            }
        }
    }
    Ok(vec![feature])
}

/// The packet's geometry, first match wins, in the old world's order. A graphics
/// property present but missing its positions is not malformed: it falls through to
/// the next candidate, and a packet where every candidate falls through gets
/// `Geometry::None`.
fn packet_geometry(packet: &Value, force_2d: bool) -> Result<Geometry, PositionProblem> {
    if let Some(positions) = packet.pointer("/polygon/positions") {
        if let Some(coords) = position_list(positions)? {
            let holes = packet
                .pointer("/polygon/holes")
                .map(|value| hole_rings(value, coords.frame))
                .transpose()?
                .unwrap_or_default();
            return area_geometry(&coords, holes, force_2d);
        }
    }
    for pointer in ["/polyline/positions", "/corridor/positions", "/wall/positions"] {
        if let Some(positions) = packet.pointer(pointer) {
            if let Some(coords) = position_list(positions)? {
                return line_geometry(&coords, force_2d);
            }
        }
    }
    if let Some(coordinates) = packet.pointer("/rectangle/coordinates") {
        if let Some(wsen) = extract_rectangle_bounds(coordinates) {
            return rectangle_geometry(&wsen, force_2d);
        }
    }
    if let (Some(ellipse), Some(value)) = (packet.get("ellipse"), packet.get("position")) {
        // `position`, not `position_list`: an ellipse's centre is a singular CZML
        // Position, and only `position` carries the time-tagged guard. Routing a
        // singular position through `position_list` would let a time-tagged
        // cartesian centre (groups of four) pass the "multiple of three" check
        // whenever the sample count made the total divisible by three, and be
        // silently misread as a run of bogus triples.
        if let Some(centre) = position(value)?.and_then(|c| c.triples.first().copied()) {
            return ellipse_geometry(ellipse, centre, force_2d);
        }
    }
    if let Some(value) = packet.get("position") {
        if let Some(coords) = position(value)? {
            return point_geometry(&coords, force_2d);
        }
    }
    Ok(Geometry::None)
}

/// Interior rings from a `PositionListOfLists`, in the exterior's frame. A hole
/// stated in a different frame than its exterior is not something this reader can
/// place consistently.
fn hole_rings(value: &Value, exterior: CzmlFrame) -> Result<Vec<Vec<[f64; 3]>>, PositionProblem> {
    let mut rings = Vec::new();
    for ring in extract_polygon_holes(value) {
        let raw = Raw {
            frame: exterior,
            radians: false,
            numbers: ring,
        };
        let coords = into_coords(raw)?;
        if coords.frame != exterior {
            return Err(PositionProblem::Unsupported(
                ErrorCode::CzmlUnsupportedPosition,
            ));
        }
        rings.push(coords.triples);
    }
    Ok(rings)
}

/// A rectangle's four corners, closed. `wsen` is west, south, east, north in
/// degrees with an optional height, as `extract_rectangle_bounds` returns it.
fn rectangle_geometry(wsen: &[f64], force_2d: bool) -> Result<Geometry, PositionProblem> {
    let [west, south, east, north] = match wsen {
        [w, s, e, n, ..] => [*w, *s, *e, *n],
        _ => {
            return Err(PositionProblem::Malformed(
                "a rectangle needs west, south, east and north bounds".to_string(),
            ));
        }
    };
    let height = wsen.get(4).copied().unwrap_or(0.0);
    // Stored latitude-first, like every other cartographic coordinate here.
    let coords = Coords {
        frame: CzmlFrame::Cartographic,
        triples: vec![
            [south, west, height],
            [south, east, height],
            [north, east, height],
            [north, west, height],
        ],
    };
    area_geometry(&coords, vec![], force_2d)
}

/// An ellipse as a 32-gon, unchanged from the old world. `rotation` is still
/// ignored and `semiMajorAxis` still lands on the east axis, both of which
/// disagree with the CZML schema; fixing them is a follow-up, not this port.
fn ellipse_geometry(
    ellipse: &Value,
    centre: [f64; 3],
    force_2d: bool,
) -> Result<Geometry, PositionProblem> {
    const SIDES: usize = 32;
    let [lat, lon, height] = centre;
    let semi_major = ellipse
        .get("semiMajorAxis")
        .and_then(Value::as_f64)
        .unwrap_or(100.0);
    let semi_minor = ellipse
        .get("semiMinorAxis")
        .and_then(Value::as_f64)
        .unwrap_or(100.0);
    let metres_per_degree_lat = 111_000.0;
    let metres_per_degree_lon = metres_per_degree_lat * lat.to_radians().cos();
    if metres_per_degree_lon.abs() < f64::EPSILON {
        return Err(PositionProblem::Malformed(
            "an ellipse centred at a pole has no longitude scale".to_string(),
        ));
    }
    let triples = (0..SIDES)
        .map(|i| {
            let angle = (i as f64) * std::f64::consts::TAU / (SIDES as f64);
            [
                lat + (semi_minor / metres_per_degree_lat) * angle.sin(),
                lon + (semi_major / metres_per_degree_lon) * angle.cos(),
                height,
            ]
        })
        .collect();
    let coords = Coords {
        frame: CzmlFrame::Cartographic,
        triples,
    };
    area_geometry(&coords, vec![], force_2d)
}

/// Features from a time-tagged position, one per `TimeSamplingStrategy`.
///
/// The samples in `czml.timeseries` stay in CZML's own longitude, latitude, height
/// order. The writer reads them as raw JSON and its own comment states they must not
/// be reprojected or swapped, because the reader already parsed them out of
/// `cartographicDegrees`. Only the feature's geometry is latitude-first.
fn timeseries_features(
    ts: &super::TimeTaggedPosition,
    base: &reearth_flow_types::Attributes,
    packet: &Value,
    params: &CzmlReaderCompiledParam,
) -> Result<Vec<Feature>, PositionProblem> {
    let sample_point = |sample: &super::TimeSample| -> Result<Geometry, PositionProblem> {
        let coords = Coords {
            frame: CzmlFrame::Cartographic,
            triples: vec![[sample.lat, sample.lon, sample.height]],
        };
        point_geometry(&coords, params.force_2d)
    };

    match params.time_sampling {
        TimeSamplingStrategy::FirstSampleOnly => {
            let Some(sample) = ts.samples.first() else {
                return Ok(vec![]);
            };
            let mut feature =
                Feature::new_with_attributes_and_geometry(base.clone(), sample_point(sample)?);
            let attributes = std::sync::Arc::make_mut(&mut feature.attributes);
            add_interpolation(attributes, ts);
            extract_extra_czml_properties(packet, attributes);
            Ok(vec![feature])
        }
        TimeSamplingStrategy::AllSamples => {
            let mut features = Vec::with_capacity(ts.samples.len());
            for sample in &ts.samples {
                let mut feature =
                    Feature::new_with_attributes_and_geometry(base.clone(), sample_point(sample)?);
                let attributes = std::sync::Arc::make_mut(&mut feature.attributes);
                attributes.insert(
                    Attribute::new("czml.timestamp"),
                    AttributeValue::String(sample_timestamp(sample, ts.epoch.as_deref())),
                );
                attributes.insert(
                    Attribute::new("czml.timeOffset"),
                    AttributeValue::Number(
                        serde_json::Number::from_f64(sample.time_offset)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ),
                );
                add_interpolation(attributes, ts);
                features.push(feature);
            }
            Ok(features)
        }
        TimeSamplingStrategy::PreserveRaw => {
            let Some(first) = ts.samples.first() else {
                return Ok(vec![]);
            };
            let mut feature =
                Feature::new_with_attributes_and_geometry(base.clone(), sample_point(first)?);
            let samples: Vec<Value> = ts
                .samples
                .iter()
                .map(|s| {
                    serde_json::json!({
                        "time": sample_timestamp(s, ts.epoch.as_deref()),
                        "timeOffset": s.time_offset,
                        "lon": s.lon,
                        "lat": s.lat,
                        "height": s.height,
                    })
                })
                .collect();
            let attributes = std::sync::Arc::make_mut(&mut feature.attributes);
            attributes.insert(
                Attribute::new("czml.timeseries"),
                AttributeValue::String(serde_json::to_string(&samples).unwrap_or_default()),
            );
            add_interpolation(attributes, ts);
            extract_extra_czml_properties(packet, attributes);
            Ok(vec![feature])
        }
    }
}

/// The epoch and interpolation attributes, identical to the old world's
/// `add_interpolation_attributes`.
fn add_interpolation(
    attributes: &mut reearth_flow_types::Attributes,
    ts: &super::TimeTaggedPosition,
) {
    if let Some(epoch) = &ts.epoch {
        attributes.insert(
            Attribute::new("czml.epoch"),
            AttributeValue::String(epoch.clone()),
        );
    }
    if let Some(algorithm) = &ts.interpolation_algorithm {
        attributes.insert(
            Attribute::new("czml.interpolationAlgorithm"),
            AttributeValue::String(algorithm.clone()),
        );
    }
    if let Some(degree) = ts.interpolation_degree {
        attributes.insert(
            Attribute::new("czml.interpolationDegree"),
            AttributeValue::Number(
                serde_json::Number::from_f64(degree).unwrap_or_else(|| serde_json::Number::from(0)),
            ),
        );
    }
}

#[cfg(all(test, feature = "new-geometry"))]
mod tests {
    use super::*;
    use reearth_flow_geometry::{Euclidean2DGeometry, Euclidean3DGeometry, Geometry};
    use reearth_flow_runtime::executor_operation::NodeContext;
    use reearth_flow_types::{Attribute, AttributeValue};
    // `SourceError` and `Bytes` are already imported by the implementation's own
    // `use` block above, so `use super::*` brings them into the tests.
    //
    // `Attribute` doubles as a `Feature::get` key: `get` takes
    // `T: AsRef<str> + Display`, and `&Attribute` satisfies both through std's
    // blanket impls, so `feature.get(&Attribute::new("id"))` compiles.

    use crate::file::reader::runner::FileReaderCompiledParam;

    fn params(force_2d: bool, skip_document: bool) -> CzmlReaderCompiledParam {
        // Built directly rather than through `compile`, which needs an expression
        // engine; the reader never reads `common` here. `FileReaderCompiledParam`
        // has no `Default`, but its fields are `pub(crate)` and this is the same
        // crate, so construct it literally rather than adding a `Default` impl to
        // production code.
        CzmlReaderCompiledParam {
            common: FileReaderCompiledParam {
                dataset: None,
                inline: None,
            },
            force_2d,
            skip_document_packet: skip_document,
            time_sampling: TimeSamplingStrategy::PreserveRaw,
        }
    }

    fn read_str(json: &str, params: &CzmlReaderCompiledParam) -> Vec<Feature> {
        read(&NodeContext::default(), &Bytes::from(json.to_string()), params)
            .expect("the document reads")
    }

    #[test]
    fn a_static_polygon_packet_becomes_one_feature() {
        let features = read_str(
            r#"[{"id":"a","polygon":{"positions":{"cartographicDegrees":
               [139.0,35.0,0.0, 140.0,35.0,0.0, 140.0,36.0,0.0]}}}]"#,
            &params(false, true),
        );
        assert_eq!(features.len(), 1);
        assert_eq!(
            features[0].get(&Attribute::new("id")),
            Some(&AttributeValue::String("a".to_string())),
        );
        assert!(matches!(
            &*features[0].geometry,
            Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(_)),
        ));
    }

    #[test]
    fn a_packet_with_no_geometry_keeps_its_attributes() {
        // Today this packet vanishes, taking its attributes with it, which is why
        // `preserveRaw` is not the lossless round trip its docstring claims.
        let features = read_str(
            r#"[{"id":"styles-only","name":"n","point":{"pixelSize":12}}]"#,
            &params(false, true),
        );
        assert_eq!(features.len(), 1);
        assert!(matches!(&*features[0].geometry, Geometry::None));
        assert_eq!(
            features[0].get(&Attribute::new("name")),
            Some(&AttributeValue::String("n".to_string())),
        );
        // The graphics survive for the writer's embedded mode to replay.
        assert!(features[0].get(&Attribute::new("czml.point")).is_some());
    }

    #[test]
    fn a_document_packet_is_skipped_by_default() {
        let features = read_str(
            r#"[{"id":"document","version":"1.0","clock":{"multiplier":60}},
                {"id":"a","position":{"cartographicDegrees":[139.0,35.0,0.0]}}]"#,
            &params(false, true),
        );
        assert_eq!(features.len(), 1);
        assert_eq!(
            features[0].get(&Attribute::new("id")),
            Some(&AttributeValue::String("a".to_string())),
        );
    }

    #[test]
    fn a_kept_document_packet_keeps_version_and_clock() {
        // `CZML_COMMON_PROPERTIES` excludes both from the shared `czml.*` capture,
        // so without an explicit capture the feature would be stripped of exactly
        // what made it a document packet.
        let features = read_str(
            r#"[{"id":"document","version":"1.0","clock":{"multiplier":60}}]"#,
            &params(false, false),
        );
        assert_eq!(features.len(), 1);
        assert!(features[0].get(&Attribute::new("czml.version")).is_some());
        assert!(features[0].get(&Attribute::new("czml.clock")).is_some());
    }

    #[test]
    fn a_timeseries_stays_longitude_first_while_geometry_does_not() {
        // THE REGRESSION TEST FOR THE ASYMMETRY. The writer reads these samples as
        // raw JSON and must not reproject or swap them, so they stay in CZML order
        // even though the feature's geometry is latitude-first.
        let features = read_str(
            r#"[{"id":"v","position":{"epoch":"2024-01-01T00:00:00Z",
               "cartographicDegrees":[0,139.76,35.68,50.0, 60,139.80,35.70,52.0]}}]"#,
            &params(false, true),
        );
        assert_eq!(features.len(), 1);
        let Geometry::Euclidean3D(Euclidean3DGeometry::Point(p)) = &*features[0].geometry else {
            panic!("expected a 3D point");
        };
        // Geometry: latitude first.
        assert_eq!(p.position(), [35.68, 139.76, 50.0]);
        // Timeseries JSON: longitude first, untouched.
        let Some(AttributeValue::String(json)) =
            features[0].get(&Attribute::new("czml.timeseries"))
        else {
            panic!("expected a timeseries attribute");
        };
        let samples: Vec<serde_json::Value> = serde_json::from_str(json).unwrap();
        assert_eq!(samples[0]["lon"].as_f64().unwrap(), 139.76);
        assert_eq!(samples[0]["lat"].as_f64().unwrap(), 35.68);
    }

    #[test]
    fn an_inertial_packet_is_skipped_without_failing_the_file() {
        let features = read_str(
            r#"[{"id":"sat","position":{"referenceFrame":"INERTIAL",
                 "cartesian":[1.0,2.0,3.0]}},
                {"id":"b","position":{"cartographicDegrees":[139.0,35.0,0.0]}}]"#,
            &params(false, true),
        );
        // The inertial packet is reported and dropped; its sibling still reads.
        assert_eq!(features.len(), 1);
        assert_eq!(
            features[0].get(&Attribute::new("id")),
            Some(&AttributeValue::String("b".to_string())),
        );
    }

    #[test]
    fn a_malformed_coordinate_list_fails_the_whole_read() {
        // Action Standard 4.3: a reader that emits part of a file and quietly drops
        // the rest hides corrupt input where the user can still fix it.
        let err = read(
            &NodeContext::default(),
            &Bytes::from(
                r#"[{"id":"bad","polyline":{"positions":{"cartographicDegrees":[1.0,2.0]}}}]"#
                    .to_string(),
            ),
            &params(false, true),
        )
        .unwrap_err();
        // The message must name the packet so the user can find it.
        assert!(format!("{err}").contains("bad"), "message was: {err}");
        // And it must be the malformed arm, not a report-and-skip: a file with a
        // ragged coordinate list produces no features at all.
        assert!(matches!(err, SourceError::CzmlReader(_)));
    }

    #[test]
    fn a_cartesian_packet_reads_as_geocentric() {
        let features = read_str(
            r#"[{"id":"e","position":{"cartesian":[3960000.0,3350000.0,3700000.0]}}]"#,
            &params(false, true),
        );
        let Geometry::Euclidean3D(Euclidean3DGeometry::Point(p)) = &*features[0].geometry else {
            panic!("expected a 3D point");
        };
        assert_eq!(*p.frame(), CoordinateFrame::Crs(WGS84_GEOCENTRIC));
        assert_eq!(p.position(), [3960000.0, 3350000.0, 3700000.0]);
    }

    #[test]
    fn force_2d_skips_a_cartesian_packet_rather_than_mangling_it() {
        let features = read_str(
            r#"[{"id":"e","position":{"cartesian":[1.0,2.0,3.0]}}]"#,
            &params(true, true),
        );
        assert!(features.is_empty());
    }

    fn cartographic(triples: Vec<[f64; 3]>) -> Coords {
        Coords {
            frame: CzmlFrame::Cartographic,
            triples,
        }
    }

    /// A square wound counter-clockwise in longitude/latitude order, open, as CZML
    /// writes it. Longitudes 139 to 140, latitudes 35 to 36.
    fn open_ccw_square() -> Coords {
        cartographic(vec![
            [35.0, 139.0, 0.0],
            [35.0, 140.0, 0.0],
            [36.0, 140.0, 0.0],
            [36.0, 139.0, 0.0],
        ])
    }

    #[test]
    fn a_point_carries_its_frame_and_order() {
        let coords = cartographic(vec![[35.68, 139.76, 10.0]]);
        let Geometry::Euclidean3D(Euclidean3DGeometry::Point(p)) =
            point_geometry(&coords, false).unwrap()
        else {
            panic!("expected a 3D point");
        };
        assert_eq!(*p.frame(), CoordinateFrame::Crs(WGS84_GEOGRAPHIC_3D));
        assert_eq!(p.position(), [35.68, 139.76, 10.0]);
    }

    #[test]
    fn force_2d_drops_the_height_and_demotes_the_frame() {
        let coords = cartographic(vec![[35.68, 139.76, 10.0]]);
        let Geometry::Euclidean2D(Euclidean2DGeometry::Point(p)) =
            point_geometry(&coords, true).unwrap()
        else {
            panic!("expected a 2D point");
        };
        assert_eq!(*p.frame(), CoordinateFrame::Crs(WGS84_GEOGRAPHIC_2D));
        assert_eq!(p.position(), [35.68, 139.76]);
    }

    #[test]
    fn force_2d_on_a_geocentric_point_is_reported() {
        let coords = Coords {
            frame: CzmlFrame::Geocentric,
            triples: vec![[1.0, 2.0, 3.0]],
        };
        assert_eq!(
            point_geometry(&coords, true).unwrap_err(),
            PositionProblem::Unsupported(ErrorCode::CzmlGeocentricForce2d),
        );
    }

    #[test]
    fn a_polygon_ring_is_closed_on_the_way_in() {
        // CZML does not repeat the closing vertex; the model stores rings verbatim
        // and never closes them, so the reader must.
        let Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(polygon)) =
            area_geometry(&open_ccw_square(), vec![], false).unwrap()
        else {
            panic!("expected a 3D polygon");
        };
        let exterior = polygon.exterior();
        assert_eq!(exterior.len(), 5, "four corners plus the closing vertex");
        assert_eq!(exterior[0], exterior[4]);
    }

    #[test]
    fn an_already_closed_ring_is_not_closed_twice() {
        let mut coords = open_ccw_square();
        coords.triples.push(coords.triples[0]);
        let Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(polygon)) =
            area_geometry(&coords, vec![], false).unwrap()
        else {
            panic!("expected a 3D polygon");
        };
        assert_eq!(polygon.exterior().len(), 5);
    }

    #[test]
    fn a_ccw_lon_lat_ring_is_canonically_counter_clockwise() {
        // THE REGRESSION TEST FOR "DO NOT REVERSE". Winding is judged as stored
        // winding times the frame's orientation sign, and EPSG:4979 is -1. Writing
        // longitude/latitude points latitude-first already mirrors them, so a
        // counter-clockwise lon/lat ring stores clockwise and lands canonically
        // counter-clockwise on its own. `shapefile_next` reverses its rings because
        // a shapefile winds outer rings clockwise; CZML does not, so reversing here
        // would double-flip and invert every face.
        let Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(polygon)) =
            area_geometry(&open_ccw_square(), vec![], false).unwrap()
        else {
            panic!("expected a 3D polygon");
        };
        let exterior = polygon.exterior();
        // Stored order must still be the input order: same first vertex, and the
        // second vertex is the input's second, not its last.
        assert_eq!(exterior[0], [35.0, 139.0, 0.0]);
        assert_eq!(exterior[1], [35.0, 140.0, 0.0]);
        // Stored winding, by the shoelace over (x, y) as stored, is negative
        // (clockwise); times the frame's -1 that is canonically counter-clockwise.
        let stored: f64 = exterior
            .windows(2)
            .map(|w| w[0][0] * w[1][1] - w[1][0] * w[0][1])
            .sum();
        assert!(
            stored < 0.0,
            "stored shoelace was {stored}, expected clockwise"
        );
    }

    #[test]
    fn polygon_holes_are_carried_and_closed() {
        let hole = vec![[35.2, 139.2, 0.0], [35.2, 139.8, 0.0], [35.8, 139.8, 0.0]];
        let Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(polygon)) =
            area_geometry(&open_ccw_square(), vec![hole], false).unwrap()
        else {
            panic!("expected a 3D polygon");
        };
        let interiors: Vec<_> = polygon.interiors().collect();
        assert_eq!(interiors.len(), 1);
        assert_eq!(
            interiors[0].len(),
            4,
            "three corners plus the closing vertex"
        );
    }

    #[test]
    fn a_line_keeps_every_vertex_and_is_not_closed() {
        let coords = cartographic(vec![[35.0, 139.0, 0.0], [36.0, 140.0, 1.0]]);
        let Geometry::Euclidean3D(Euclidean3DGeometry::LineString(line)) =
            line_geometry(&coords, false).unwrap()
        else {
            panic!("expected a 3D line");
        };
        assert_eq!(line.coords(), &[[35.0, 139.0, 0.0], [36.0, 140.0, 1.0]]);
    }

    #[test]
    fn a_line_with_one_vertex_fails_the_read() {
        let coords = cartographic(vec![[35.0, 139.0, 0.0]]);
        assert!(matches!(
            line_geometry(&coords, false).unwrap_err(),
            PositionProblem::Malformed(_),
        ));
    }

    #[test]
    fn a_ring_with_two_vertices_fails_the_read() {
        let coords = cartographic(vec![[35.0, 139.0, 0.0], [36.0, 140.0, 0.0]]);
        assert!(matches!(
            area_geometry(&coords, vec![], false).unwrap_err(),
            PositionProblem::Malformed(_),
        ));
    }

    #[test]
    fn a_geocentric_polygon_keeps_x_y_z_order() {
        let coords = Coords {
            frame: CzmlFrame::Geocentric,
            triples: vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        };
        let Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(polygon)) =
            area_geometry(&coords, vec![], false).unwrap()
        else {
            panic!("expected a 3D polygon");
        };
        assert_eq!(*polygon.frame(), CoordinateFrame::Crs(WGS84_GEOCENTRIC));
        assert_eq!(polygon.exterior()[0], [1.0, 0.0, 0.0]);
    }

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
