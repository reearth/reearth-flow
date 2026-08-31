//! Building CZML packets (polygons and polylines) from extracted faces and
//! vertex chains.
//!
//! `Packet.polygon` is the vendored crate's one typed graphics field
//! (`Option<CzmlPolygon>`); `Packet.polyline` (like `.position` and `.point`)
//! is raw JSON (`Option<HashMap<String, Value>>`), so a polyline is built by
//! hand as `{"positions": {"cartographicDegrees": [lon, lat, h, ...]}}`.
//!
//! Every ring or vertex chain is reprojected with [`to_wgs84`] before being
//! flattened. `to_wgs84` returns `[lat, lon, height]` (EPSG:4979's own
//! authority axis order — see `coords`'s module docs); CZML's
//! `cartographicDegrees` wants `[lon, lat, height]`, so every flattening site
//! below swaps the first two components.

use std::collections::HashMap;

use nusamai_czml::{
    CzmlBoolean, CzmlPolygon, Packet, PositionList, PositionListOfLists,
    PositionListOfListsProperties, PositionListProperties, StringProperties, StringValueType,
};
use reearth_flow_geometry::coordinate::CoordinateFrame;
use reearth_flow_geometry::ops::ReprojectionCache;
use serde_json::Value;

use super::coords::{to_wgs84, FrameError};
use super::extract::Face;

/// Reproject `vertices` to WGS84 and flatten to CZML's `cartographicDegrees`
/// order (`[lon, lat, height, ...]`), swapping `to_wgs84`'s `[lat, lon,
/// height]` output.
fn cartographic_degrees(
    cache: &mut ReprojectionCache,
    frame: &CoordinateFrame,
    vertices: &[[f64; 3]],
) -> Result<Vec<f64>, FrameError> {
    let mut coords = vertices.to_vec();
    to_wgs84(cache, frame, &mut coords)?;
    Ok(coords.iter().flat_map(|c| [c[1], c[0], c[2]]).collect())
}

/// A `description` value that references the properties packet's own
/// description, matching `feature_to_packets`'s existing parenting shape.
fn description_reference(parent_id: &str) -> StringValueType {
    StringValueType::Object(StringProperties {
        reference: Some(format!("{parent_id}#description")),
        ..Default::default()
    })
}

/// Build a polygon packet from one [`Face`]: exterior ring first, remaining
/// rings become holes. Reprojects before flattening; propagates a
/// [`FrameError`] rather than emitting a partial packet.
pub(crate) fn face_packet(
    cache: &mut ReprojectionCache,
    face: &Face,
    parent_id: &str,
) -> Result<Packet, FrameError> {
    let mut rings = Vec::with_capacity(face.rings.len());
    for ring in &face.rings {
        rings.push(cartographic_degrees(cache, &face.frame, ring)?);
    }
    let mut rings = rings.into_iter();
    let exterior = rings.next().unwrap_or_default();
    let holes: Vec<Vec<f64>> = rings.collect();

    let mut polygon = CzmlPolygon {
        positions: Some(PositionList::Object(PositionListProperties {
            cartographic_degrees: Some(exterior),
            ..Default::default()
        })),
        // In Cesium, if perPositionHeight is false the polygon height is
        // fixed, flattening any real elevation — matches the old CityGML path.
        per_position_height: CzmlBoolean::Boolean(true),
        ..Default::default()
    };
    if !holes.is_empty() {
        polygon.holes = Some(PositionListOfLists::Object(PositionListOfListsProperties {
            cartographic_degrees: Some(holes),
            ..Default::default()
        }));
    }

    Ok(Packet {
        polygon: Some(polygon),
        description: Some(description_reference(parent_id)),
        parent: Some(parent_id.to_string()),
        ..Default::default()
    })
}

/// Build a polyline packet from a vertex chain (e.g. a `LineString`).
/// Reprojects before flattening; propagates a [`FrameError`] rather than
/// emitting a partial packet.
pub(crate) fn polyline_packet(
    cache: &mut ReprojectionCache,
    vertices: &[[f64; 3]],
    frame: &CoordinateFrame,
    parent_id: &str,
) -> Result<Packet, FrameError> {
    let degrees = cartographic_degrees(cache, frame, vertices)?;

    let mut positions = serde_json::Map::new();
    positions.insert("cartographicDegrees".to_string(), Value::from(degrees));

    let mut polyline: HashMap<String, Value> = HashMap::new();
    polyline.insert("positions".to_string(), Value::Object(positions));

    Ok(Packet {
        polyline: Some(polyline),
        description: Some(description_reference(parent_id)),
        parent: Some(parent_id.to_string()),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_geometry::coordinate::{CoordinateFrame, EpsgCode};

    /// Test-only helpers for reaching into [`CzmlPolygon`]'s real (vendored)
    /// field shape from an assertion.
    mod fixture {
        use nusamai_czml::{CzmlPolygon, PositionList, PositionListOfLists};

        pub(super) fn first_lon(polygon: &CzmlPolygon) -> f64 {
            let PositionList::Object(props) = polygon.positions.as_ref().expect("positions") else {
                panic!("expected PositionList::Object");
            };
            props
                .cartographic_degrees
                .as_ref()
                .expect("cartographicDegrees")[0]
        }

        /// As [`first_lon`], but for the first hole ring's leading component —
        /// so the swap is pinned on `holes` directly, not inherited from the
        /// exterior assertion.
        pub(super) fn first_hole_lon(polygon: &CzmlPolygon) -> f64 {
            let PositionListOfLists::Object(props) = polygon.holes.as_ref().expect("holes") else {
                panic!("expected PositionListOfLists::Object");
            };
            props
                .cartographic_degrees
                .as_ref()
                .expect("cartographicDegrees")[0][0]
        }
    }

    fn wgs84() -> CoordinateFrame {
        CoordinateFrame::Crs(EpsgCode::new(4979))
    }

    #[test]
    fn a_face_becomes_a_polygon_packet() {
        let face = Face {
            rings: vec![vec![
                [139.0, 35.0, 0.0],
                [139.1, 35.0, 0.0],
                [139.1, 35.1, 0.0],
                [139.0, 35.0, 0.0],
            ]],
            frame: wgs84(),
        };
        let mut cache = ReprojectionCache::default();
        let packet = face_packet(&mut cache, &face, "parent").expect("wgs84 face");
        assert!(packet.polygon.is_some(), "a face must produce a polygon");
        assert_eq!(packet.parent.as_deref(), Some("parent"));
    }

    #[test]
    fn a_face_with_a_hole_keeps_the_hole() {
        let face = Face {
            rings: vec![
                vec![
                    [139.0, 35.0, 0.0],
                    [139.4, 35.0, 0.0],
                    [139.4, 35.4, 0.0],
                    [139.0, 35.0, 0.0],
                ],
                vec![
                    [139.1, 35.1, 0.0],
                    [139.2, 35.1, 0.0],
                    [139.2, 35.2, 0.0],
                    [139.1, 35.1, 0.0],
                ],
            ],
            frame: wgs84(),
        };
        let mut cache = ReprojectionCache::default();
        let packet = face_packet(&mut cache, &face, "parent").expect("wgs84 face");
        let polygon = packet.polygon.expect("polygon");
        assert!(polygon.holes.is_some(), "the hole ring must survive");
    }

    #[test]
    fn a_polyline_packet_keeps_every_vertex() {
        // Fixes the defect where a LineString wrote only its first point.
        let vertices = [[139.0, 35.0, 0.0], [139.1, 35.1, 0.0], [139.2, 35.2, 0.0]];
        let mut cache = ReprojectionCache::default();
        let packet =
            polyline_packet(&mut cache, &vertices, &wgs84(), "parent").expect("wgs84 line");
        let polyline = packet.polyline.expect("polyline");
        let positions = polyline.get("positions").expect("positions");
        let degrees = positions
            .get("cartographicDegrees")
            .and_then(|v| v.as_array())
            .expect("cartographicDegrees array");
        assert_eq!(degrees.len(), 9, "three vertices, three components each");
    }

    #[test]
    fn a_polylines_longitude_and_latitude_are_written_in_czml_order() {
        // Same disjoint-range guard as the polygon exterior/hole cases, but
        // for the polyline path, which shares `cartographic_degrees` but is
        // built as raw JSON rather than through `CzmlPolygon`. Pins exact
        // per-vertex values and order, not just the vertex count.
        let vertices = [
            [35.68, 139.76, 1.0],
            [35.69, 139.77, 2.0],
            [35.70, 139.78, 3.0],
        ];
        let mut cache = ReprojectionCache::default();
        let packet =
            polyline_packet(&mut cache, &vertices, &wgs84(), "parent").expect("wgs84 line");
        let polyline = packet.polyline.expect("polyline");
        let positions = polyline.get("positions").expect("positions");
        let degrees: Vec<f64> = positions
            .get("cartographicDegrees")
            .and_then(|v| v.as_array())
            .expect("cartographicDegrees array")
            .iter()
            .map(|v| v.as_f64().expect("numeric component"))
            .collect();
        assert_eq!(
            degrees,
            vec![139.76, 35.68, 1.0, 139.77, 35.69, 2.0, 139.78, 35.70, 3.0],
            "cartographicDegrees must be [lon, lat, height] per vertex, in order"
        );
    }

    #[test]
    fn an_unplaceable_frame_produces_no_packet() {
        let face = Face {
            rings: vec![vec![
                [1.0, 2.0, 0.0],
                [3.0, 4.0, 0.0],
                [5.0, 6.0, 0.0],
                [1.0, 2.0, 0.0],
            ]],
            frame: CoordinateFrame::default(),
        };
        let mut cache = ReprojectionCache::default();
        assert!(matches!(
            face_packet(&mut cache, &face, "parent"),
            Err(FrameError::Unplaceable)
        ));
    }

    #[test]
    fn longitude_and_latitude_are_written_in_czml_order() {
        // `to_wgs84` returns [lat, lon, height] (EPSG:4979 authority order);
        // `cartographicDegrees` wants [lon, lat, height]. Tokyo's lat and lon
        // ranges are disjoint, so a swap cannot pass this.
        let face = Face {
            rings: vec![vec![
                [35.68, 139.76, 0.0],
                [35.69, 139.77, 0.0],
                [35.70, 139.78, 0.0],
                [35.68, 139.76, 0.0],
            ]],
            frame: wgs84(),
        };
        let mut cache = ReprojectionCache::default();
        let packet = face_packet(&mut cache, &face, "parent").expect("wgs84 face");
        let polygon = packet.polygon.expect("polygon");
        let first_lon = fixture::first_lon(&polygon);
        assert!(
            (139.0..140.0).contains(&first_lon),
            "cartographicDegrees must lead with longitude; got {first_lon}"
        );
    }

    #[test]
    fn a_holes_longitude_and_latitude_are_written_in_czml_order() {
        // Exterior and hole sit in disjoint lon/lat ranges from each other,
        // as well as internally disjoint lat vs. lon (Tokyo-like), so this
        // pins the swap on `holes` directly rather than inheriting a pass
        // from the exterior assertion, and a swap (or a wrong-ring read)
        // cannot pass either.
        let face = Face {
            rings: vec![
                // exterior: lat 10..11, lon 150..151.
                vec![
                    [10.0, 150.0, 0.0],
                    [10.0, 151.0, 0.0],
                    [11.0, 151.0, 0.0],
                    [10.0, 150.0, 0.0],
                ],
                // hole: lat 35..36, lon 139..140.
                vec![
                    [35.68, 139.76, 0.0],
                    [35.69, 139.77, 0.0],
                    [35.70, 139.78, 0.0],
                    [35.68, 139.76, 0.0],
                ],
            ],
            frame: wgs84(),
        };
        let mut cache = ReprojectionCache::default();
        let packet = face_packet(&mut cache, &face, "parent").expect("wgs84 face");
        let polygon = packet.polygon.expect("polygon");
        let first_hole_lon = fixture::first_hole_lon(&polygon);
        assert!(
            (139.0..140.0).contains(&first_hole_lon),
            "hole cartographicDegrees must lead with longitude; got {first_hole_lon}"
        );
    }

    #[test]
    fn projected_metres_are_reprojected_before_writing() {
        // The Gulf of Guinea test: raw 6677 metres must not reach the output.
        let face = Face {
            rings: vec![vec![
                [0.0, 0.0, 0.0],
                [100.0, 0.0, 0.0],
                [100.0, 100.0, 0.0],
                [0.0, 0.0, 0.0],
            ]],
            frame: CoordinateFrame::Crs(EpsgCode::new(6677)),
        };
        let mut cache = ReprojectionCache::default();
        let packet = face_packet(&mut cache, &face, "parent").expect("6677 face");
        let polygon = packet.polygon.expect("polygon");
        let first_lon = fixture::first_lon(&polygon);
        assert!((138.0..142.0).contains(&first_lon), "lon was {first_lon}");
    }
}
