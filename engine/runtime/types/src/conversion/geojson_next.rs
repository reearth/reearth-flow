//! GeoJSON -> `Feature` conversion for the new-geometry world. Builds
//! `reearth_flow_geometry::Geometry` (per-leaf `CoordinateFrame`) rather than the
//! old `GeometryValue` wrapper.

use std::sync::Arc;

use reearth_flow_geometry::types::conversion::{is_2d_geojson_value, is_3d_geojson_value};
use reearth_flow_geometry::{
    collection::{Collection2D, Collection3D},
    coordinate::{CoordinateFrame, EpsgCode},
    line_string::{LineString2D, LineString3D},
    point::{Point2D, Point3D},
    polygon::{Polygon2D, Polygon3D},
    Euclidean2DGeometry, Euclidean3DGeometry, Geometry, GeometryCollection,
};

use crate::{
    error::{Error, Result},
    Attribute, Attributes, Feature,
};

// WGS84 geographic CRS codes, defined here so this new-geometry module carries no
// dependency on nusamai-projection (which is slated for removal after the migration).
const EPSG_WGS84_GEOGRAPHIC_2D: u16 = 4326;
const EPSG_WGS84_GEOGRAPHIC_3D: u16 = 4979;

impl TryFrom<geojson::Feature> for Feature {
    type Error = Error;

    fn try_from(geom: geojson::Feature) -> Result<Self> {
        let attributes = geom
            .properties
            .as_ref()
            .map(geojson_object_to_attributes)
            .unwrap_or_default();
        let geometry = match geom.geometry {
            Some(g) => geojson_value_to_geometry(g.value)?,
            None => Geometry::None,
        };
        Ok(Feature {
            id: geom.id.map_or_else(uuid::Uuid::new_v4, geojson_id_to_uuid),
            attributes: Arc::new(attributes),
            geometry: Arc::new(geometry),
        })
    }
}

/// A GeoJSON string id that is a valid UUID is preserved; anything else gets a fresh UUID.
fn geojson_id_to_uuid(id: geojson::feature::Id) -> uuid::Uuid {
    match id {
        geojson::feature::Id::String(v) => {
            uuid::Uuid::parse_str(&v).unwrap_or_else(|_| uuid::Uuid::new_v4())
        }
        geojson::feature::Id::Number(_) => uuid::Uuid::new_v4(),
    }
}

fn geojson_object_to_attributes(obj: &geojson::JsonObject) -> Attributes {
    obj.iter()
        .map(|(k, v)| (Attribute::new(k), v.clone().into()))
        .collect()
}

/// WGS84 geographic frame for 2D coordinates, stored (lat, lon) per EPSG:4326.
fn wgs84_2d() -> CoordinateFrame {
    CoordinateFrame::Crs(EpsgCode::new(EPSG_WGS84_GEOGRAPHIC_2D))
}

/// WGS84 geographic frame for 3D coordinates, stored (lat, lon, height) per EPSG:4979.
fn wgs84_3d() -> CoordinateFrame {
    CoordinateFrame::Crs(EpsgCode::new(EPSG_WGS84_GEOGRAPHIC_3D))
}

fn geojson_value_to_geometry(value: geojson::Value) -> Result<Geometry> {
    match value {
        // A heterogeneous collection: each member converts on its own, so members
        // may differ in dimension / coordinate frame.
        geojson::Value::GeometryCollection(geometries) => {
            let members = geometries
                .into_iter()
                .map(|g| geojson_value_to_geometry(g.value))
                .collect::<Result<Vec<_>>>()?;
            Ok(Geometry::GeometryCollection(GeometryCollection::new(
                members,
            )))
        }
        // A single geometry's coordinates must be uniformly 2D or 3D; mixed or
        // degenerate coordinates are rejected rather than indexed into blindly.
        value if is_2d_geojson_value(&value) => Ok(Geometry::Euclidean2D(value_to_2d(value)?)),
        value if is_3d_geojson_value(&value) => Ok(Geometry::Euclidean3D(value_to_3d(value)?)),
        _ => Err(mixed_dimensions()),
    }
}

fn value_to_2d(value: geojson::Value) -> Result<Euclidean2DGeometry> {
    match value {
        geojson::Value::Point(p) => Ok(Euclidean2DGeometry::Point(point_2d(&p))),
        geojson::Value::MultiPoint(ps) => Ok(collection_2d(
            ps.iter().map(|p| Euclidean2DGeometry::Point(point_2d(p))),
        )),
        geojson::Value::LineString(coords) => {
            Ok(Euclidean2DGeometry::LineString(line_string_2d(&coords)))
        }
        geojson::Value::MultiLineString(lines) => {
            Ok(collection_2d(lines.iter().map(|l| {
                Euclidean2DGeometry::LineString(line_string_2d(l))
            })))
        }
        geojson::Value::Polygon(rings) => {
            Ok(Euclidean2DGeometry::Polygon(Box::new(polygon_2d(&rings))))
        }
        geojson::Value::MultiPolygon(polys) => {
            Ok(collection_2d(polys.iter().map(|rings| {
                Euclidean2DGeometry::Polygon(Box::new(polygon_2d(rings)))
            })))
        }
        _ => Err(mixed_dimensions()),
    }
}

fn value_to_3d(value: geojson::Value) -> Result<Euclidean3DGeometry> {
    match value {
        geojson::Value::Point(p) => Ok(Euclidean3DGeometry::Point(point_3d(&p))),
        geojson::Value::MultiPoint(ps) => Ok(collection_3d(
            ps.iter().map(|p| Euclidean3DGeometry::Point(point_3d(p))),
        )),
        geojson::Value::LineString(coords) => {
            Ok(Euclidean3DGeometry::LineString(line_string_3d(&coords)))
        }
        geojson::Value::MultiLineString(lines) => {
            Ok(collection_3d(lines.iter().map(|l| {
                Euclidean3DGeometry::LineString(line_string_3d(l))
            })))
        }
        geojson::Value::Polygon(rings) => {
            Ok(Euclidean3DGeometry::Polygon(Box::new(polygon_3d(&rings))))
        }
        geojson::Value::MultiPolygon(polys) => {
            Ok(collection_3d(polys.iter().map(|rings| {
                Euclidean3DGeometry::Polygon(Box::new(polygon_3d(rings)))
            })))
        }
        _ => Err(mixed_dimensions()),
    }
}

fn mixed_dimensions() -> Error {
    Error::unsupported_feature(
        "GeoJSON geometry has mixed or unsupported coordinate dimensions \
         (every coordinate must be uniformly 2D or 3D)",
    )
}

fn collection_2d(members: impl IntoIterator<Item = Euclidean2DGeometry>) -> Euclidean2DGeometry {
    Euclidean2DGeometry::Collection(Collection2D::new(members))
}

fn collection_3d(members: impl IntoIterator<Item = Euclidean3DGeometry>) -> Euclidean3DGeometry {
    Euclidean3DGeometry::Collection(Collection3D::new(members))
}

// GeoJSON coordinates are (lon, lat[, height]) per RFC 7946, but the WGS84 frames
// they are tagged with declare (lat, lon[, height]) axis order, so the horizontal
// pair is swapped on read. The swap also flips ring winding, which the frame's
// orientation sign accounts for, keeping the canonical orientation intact.

fn point_2d(p: &[f64]) -> Point2D {
    Point2D::new(wgs84_2d(), [p[1], p[0]])
}

fn point_3d(p: &[f64]) -> Point3D {
    Point3D::new(wgs84_3d(), [p[1], p[0], p[2]])
}

fn line_string_2d(coords: &[Vec<f64>]) -> LineString2D {
    LineString2D::from_coords(wgs84_2d(), coords.iter().map(|c| [c[1], c[0]]))
}

fn line_string_3d(coords: &[Vec<f64>]) -> LineString3D {
    LineString3D::from_coords(wgs84_3d(), coords.iter().map(|c| [c[1], c[0], c[2]]))
}

fn polygon_2d(rings: &[Vec<Vec<f64>>]) -> Polygon2D {
    let mut rings = rings
        .iter()
        .map(|r| r.iter().map(|c| [c[1], c[0]]).collect::<Vec<_>>());
    let exterior = rings.next().unwrap_or_default();
    Polygon2D::from_rings(wgs84_2d(), exterior, rings)
}

fn polygon_3d(rings: &[Vec<Vec<f64>>]) -> Polygon3D {
    let mut rings = rings
        .iter()
        .map(|r| r.iter().map(|c| [c[1], c[0], c[2]]).collect::<Vec<_>>());
    let exterior = rings.next().unwrap_or_default();
    Polygon3D::from_rings(wgs84_3d(), exterior, rings)
}

#[cfg(test)]
mod tests {
    use reearth_flow_geometry::{
        collection::{Collection2D, Collection3D},
        coordinate::{CoordinateFrame, EpsgCode},
        line_string::{LineString2D, LineString3D},
        point::{Point2D, Point3D},
        polygon::{Polygon2D, Polygon3D},
        Euclidean2DGeometry, Euclidean3DGeometry, Geometry, GeometryCollection,
    };

    use super::*;
    use crate::{Attribute, AttributeValue};

    fn crs(code: u16) -> CoordinateFrame {
        CoordinateFrame::Crs(EpsgCode::new(code))
    }

    fn geojson_feature(value: geojson::Value) -> geojson::Feature {
        geojson::Feature {
            bbox: None,
            geometry: Some(geojson::Geometry::new(value)),
            id: None,
            properties: None,
            foreign_members: None,
        }
    }

    // A 2D GeoJSON Point (lon, lat) becomes a Euclidean2D Point stored (lat, lon).
    #[test]
    fn point_2d_converts_to_euclidean_2d_wgs84() {
        let gj = geojson_feature(geojson::Value::Point(vec![139.7, 35.6]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
                crs(4326),
                [35.6, 139.7],
            )))
        );
    }

    // A 3D GeoJSON Point (lon, lat, h) becomes a Euclidean3D Point stored (lat, lon, h).
    #[test]
    fn point_3d_converts_to_euclidean_3d_wgs84() {
        let gj = geojson_feature(geojson::Value::Point(vec![139.7, 35.6, 12.5]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
                crs(4979),
                [35.6, 139.7, 12.5],
            )))
        );
    }

    #[test]
    fn line_string_2d_converts() {
        let gj = geojson_feature(geojson::Value::LineString(vec![
            vec![0.0, 0.0],
            vec![1.0, 2.0],
        ]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::Euclidean2D(Euclidean2DGeometry::LineString(LineString2D::from_coords(
                crs(4326),
                [[0.0, 0.0], [2.0, 1.0]],
            )))
        );
    }

    #[test]
    fn line_string_3d_converts() {
        let gj = geojson_feature(geojson::Value::LineString(vec![
            vec![0.0, 0.0, 1.0],
            vec![1.0, 2.0, 3.0],
        ]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::Euclidean3D(Euclidean3DGeometry::LineString(LineString3D::from_coords(
                crs(4979),
                [[0.0, 0.0, 1.0], [2.0, 1.0, 3.0]],
            )))
        );
    }

    // A 2D Polygon keeps its exterior ring and interior holes.
    #[test]
    fn polygon_2d_with_hole_converts() {
        let exterior = vec![
            vec![0.0, 0.0],
            vec![4.0, 0.0],
            vec![4.0, 4.0],
            vec![0.0, 0.0],
        ];
        let hole = vec![
            vec![1.0, 1.0],
            vec![2.0, 1.0],
            vec![1.0, 2.0],
            vec![1.0, 1.0],
        ];
        let gj = geojson_feature(geojson::Value::Polygon(vec![exterior, hole]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(Box::new(
                Polygon2D::from_rings(
                    crs(4326),
                    [[0.0, 0.0], [0.0, 4.0], [4.0, 4.0], [0.0, 0.0]],
                    [[[1.0, 1.0], [1.0, 2.0], [2.0, 1.0], [1.0, 1.0]]],
                )
            )))
        );
    }

    /// Shoelace signed area of a closed ring in its stored coordinate order.
    fn signed_area(ring: &[[f64; 2]]) -> f64 {
        ring.windows(2)
            .map(|w| w[0][0] * w[1][1] - w[1][0] * w[0][1])
            .sum::<f64>()
            / 2.0
    }

    // A GeoJSON exterior wound CCW in (lon, lat) is stored CW in the (lat, lon) frame:
    // the axis swap flips the raw winding, and the frame's orientation sign (-1 for
    // EPSG:4326) flips it back, so the canonical orientation stays CCW.
    #[test]
    fn ccw_geojson_exterior_is_stored_clockwise() {
        // (0,0) -> (4,0) -> (4,4) -> (0,0): CCW in (lon, lat), positive area.
        let exterior = vec![
            vec![0.0, 0.0],
            vec![4.0, 0.0],
            vec![4.0, 4.0],
            vec![0.0, 0.0],
        ];
        let gj = geojson_feature(geojson::Value::Polygon(vec![exterior]));

        let feature: Feature = gj.try_into().unwrap();

        let Geometry::Euclidean2D(Euclidean2DGeometry::Polygon(polygon)) = &*feature.geometry
        else {
            panic!("expected a 2D polygon");
        };
        assert!(signed_area(polygon.exterior()) < 0.0);
    }

    // A 3D Polygon exterior ring keeps z.
    #[test]
    fn polygon_3d_converts() {
        let exterior = vec![
            vec![0.0, 0.0, 1.0],
            vec![4.0, 0.0, 1.0],
            vec![4.0, 4.0, 1.0],
            vec![0.0, 0.0, 1.0],
        ];
        let gj = geojson_feature(geojson::Value::Polygon(vec![exterior]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::Euclidean3D(Euclidean3DGeometry::Polygon(Box::new(
                Polygon3D::from_rings(
                    crs(4979),
                    [
                        [0.0, 0.0, 1.0],
                        [0.0, 4.0, 1.0],
                        [4.0, 4.0, 1.0],
                        [0.0, 0.0, 1.0]
                    ],
                    std::iter::empty::<Vec<[f64; 3]>>(),
                )
            )))
        );
    }

    // MultiPoint (2D) becomes a Collection of Points (the new geometry has no Multi* leaf).
    #[test]
    fn multi_point_2d_converts_to_collection() {
        let gj = geojson_feature(geojson::Value::MultiPoint(vec![
            vec![0.0, 0.0],
            vec![1.0, 1.0],
        ]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new([
                Euclidean2DGeometry::Point(Point2D::new(crs(4326), [0.0, 0.0])),
                Euclidean2DGeometry::Point(Point2D::new(crs(4326), [1.0, 1.0])),
            ])))
        );
    }

    // MultiPoint (3D) becomes a 3D Collection of Points.
    #[test]
    fn multi_point_3d_converts_to_collection() {
        let gj = geojson_feature(geojson::Value::MultiPoint(vec![
            vec![0.0, 0.0, 5.0],
            vec![1.0, 1.0, 6.0],
        ]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::Euclidean3D(Euclidean3DGeometry::Collection(Collection3D::new([
                Euclidean3DGeometry::Point(Point3D::new(crs(4979), [0.0, 0.0, 5.0])),
                Euclidean3DGeometry::Point(Point3D::new(crs(4979), [1.0, 1.0, 6.0])),
            ])))
        );
    }

    // MultiLineString (2D) becomes a Collection of LineStrings.
    #[test]
    fn multi_line_string_2d_converts_to_collection() {
        let gj = geojson_feature(geojson::Value::MultiLineString(vec![
            vec![vec![0.0, 0.0], vec![1.0, 1.0]],
            vec![vec![2.0, 2.0], vec![3.0, 3.0]],
        ]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new([
                Euclidean2DGeometry::LineString(LineString2D::from_coords(
                    crs(4326),
                    [[0.0, 0.0], [1.0, 1.0]],
                )),
                Euclidean2DGeometry::LineString(LineString2D::from_coords(
                    crs(4326),
                    [[2.0, 2.0], [3.0, 3.0]],
                )),
            ])))
        );
    }

    // MultiPolygon (2D) becomes a Collection of Polygons.
    #[test]
    fn multi_polygon_2d_converts_to_collection() {
        let poly = vec![vec![
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![0.0, 1.0],
            vec![0.0, 0.0],
        ]];
        let gj = geojson_feature(geojson::Value::MultiPolygon(vec![poly]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::Euclidean2D(Euclidean2DGeometry::Collection(Collection2D::new([
                Euclidean2DGeometry::Polygon(Box::new(Polygon2D::from_rings(
                    crs(4326),
                    [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [0.0, 0.0]],
                    std::iter::empty::<Vec<[f64; 2]>>(),
                ))),
            ])))
        );
    }

    // A GeometryCollection converts to the new GeometryCollection; its members may
    // differ in dimension (each carries its own coordinate frame).
    #[test]
    fn geometry_collection_converts_with_mixed_dimension_members() {
        let gj = geojson_feature(geojson::Value::GeometryCollection(vec![
            geojson::Geometry::new(geojson::Value::Point(vec![0.0, 0.0])),
            geojson::Geometry::new(geojson::Value::Point(vec![1.0, 1.0, 2.0])),
        ]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::GeometryCollection(GeometryCollection::new([
                Geometry::Euclidean2D(Euclidean2DGeometry::Point(Point2D::new(
                    crs(4326),
                    [0.0, 0.0],
                ))),
                Geometry::Euclidean3D(Euclidean3DGeometry::Point(Point3D::new(
                    crs(4979),
                    [1.0, 1.0, 2.0],
                ))),
            ]))
        );
    }

    // A nested GeometryCollection recurses.
    #[test]
    fn nested_geometry_collection_converts() {
        let inner = geojson::Value::GeometryCollection(vec![geojson::Geometry::new(
            geojson::Value::Point(vec![3.0, 4.0]),
        )]); // (lon, lat) -> stored (lat, lon) = [4.0, 3.0]
        let gj = geojson_feature(geojson::Value::GeometryCollection(vec![
            geojson::Geometry::new(inner),
        ]));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            *feature.geometry,
            Geometry::GeometryCollection(GeometryCollection::new([Geometry::GeometryCollection(
                GeometryCollection::new([Geometry::Euclidean2D(Euclidean2DGeometry::Point(
                    Point2D::new(crs(4326), [4.0, 3.0])
                ),)])
            ),]))
        );
    }

    // Mixed-dimension coordinates are rejected, not panicked on.
    #[test]
    fn mixed_dimension_multi_point_is_unsupported() {
        let gj = geojson_feature(geojson::Value::MultiPoint(vec![
            vec![0.0, 0.0],      // 2D
            vec![1.0, 1.0, 1.0], // 3D
        ]));

        let result: Result<Feature> = gj.try_into();

        assert!(result.is_err());
    }

    // A degenerate coordinate (fewer than 2 elements) is rejected, not panicked on.
    #[test]
    fn degenerate_coordinate_is_unsupported() {
        let gj = geojson_feature(geojson::Value::Point(vec![0.0]));

        let result: Result<Feature> = gj.try_into();

        assert!(result.is_err());
    }

    // Feature properties are carried over as attributes.
    #[test]
    fn properties_become_attributes() {
        let mut props = geojson::JsonObject::new();
        props.insert(
            "name".to_string(),
            serde_json::Value::String("bldg-1".to_string()),
        );
        let mut gj = geojson_feature(geojson::Value::Point(vec![0.0, 0.0]));
        gj.properties = Some(props);

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(
            feature.attributes.get(&Attribute::new("name")),
            Some(&AttributeValue::String("bldg-1".to_string()))
        );
    }

    // A string feature id that is a valid UUID is preserved.
    #[test]
    fn string_id_parses_as_uuid() {
        let id = "550e8400-e29b-41d4-a716-446655440000";
        let mut gj = geojson_feature(geojson::Value::Point(vec![0.0, 0.0]));
        gj.id = Some(geojson::feature::Id::String(id.to_string()));

        let feature: Feature = gj.try_into().unwrap();

        assert_eq!(feature.id, uuid::Uuid::parse_str(id).unwrap());
    }
}
