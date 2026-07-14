// Several imports here are only used by the `Feature`-geometry conversions that
// are gated off under `new-geometry`.
#![cfg_attr(feature = "new-geometry", allow(unused_imports))]

use std::sync::Arc;

use indexmap::IndexMap;
use itertools::Itertools;
use nusamai_projection::crs::{EPSG_WGS84_GEOGRAPHIC_2D, EPSG_WGS84_GEOGRAPHIC_3D};
use reearth_flow_geometry::types::conversion::is_2d_geojson_value;
use reearth_flow_geometry::types::point::Point3D;

use crate::{
    error::{Error, Result},
    Attribute, AttributeValue, Feature, GeometryValue, GmlGeometry,
};

// TODO(new-geometry): port to new geometry; gated off until then.
#[cfg(not(feature = "new-geometry"))]
impl TryFrom<Feature> for Vec<geojson::Feature> {
    type Error = Error;

    fn try_from(geom: Feature) -> Result<Self> {
        let properites = from_attribute_value_map_to_geojson_object(&geom.attributes);
        // Clone geometry value since it's Arc-wrapped
        let geometry_value = geom.geometry.value.clone();
        let geojson_features = match geometry_value {
            GeometryValue::CityGmlGeometry(gml_geometry) => gml_geometry
                .gml_geometries
                .into_iter()
                .flat_map(|f| {
                    let geometries: Vec<geojson::Value> = f.into();
                    geometries
                        .into_iter()
                        .map(|g| geojson::Feature {
                            bbox: None,
                            geometry: Some(g.into()),
                            id: Some(geojson::feature::Id::String(
                                uuid::Uuid::new_v4().to_string(),
                            )),
                            properties: Some(properites.clone()),
                            foreign_members: None,
                        })
                        .collect_vec()
                })
                .collect(),
            GeometryValue::FlowGeometry2D(flow_geometry) => {
                vec![geojson::Feature {
                    bbox: None,
                    geometry: Some(flow_geometry.into()),
                    id: Some(geojson::feature::Id::String(geom.id.to_string())),
                    properties: Some(properites),
                    foreign_members: None,
                }]
            }
            GeometryValue::FlowGeometry3D(flow_geometry) => {
                vec![geojson::Feature {
                    bbox: None,
                    geometry: Some(flow_geometry.into()),
                    id: Some(geojson::feature::Id::String(geom.id.to_string())),
                    properties: Some(properites),
                    foreign_members: None,
                }]
            }
            GeometryValue::None => {
                vec![geojson::Feature {
                    bbox: None,
                    geometry: None,
                    id: Some(geojson::feature::Id::String(geom.id.to_string())),
                    properties: Some(properites),
                    foreign_members: None,
                }]
            }
        };
        Ok(geojson_features)
    }
}

#[cfg(not(feature = "new-geometry"))]
fn from_attribute_value_map_to_geojson_object(
    map: &IndexMap<Attribute, AttributeValue>,
) -> geojson::JsonObject {
    let mut properties = geojson::JsonObject::new();
    for (k, v) in map.iter() {
        properties.insert(k.to_string(), v.clone().into());
    }
    properties
}

impl TryFrom<geojson::Value> for GeometryValue {
    type Error = Error;

    fn try_from(value: geojson::Value) -> Result<Self> {
        if is_2d_geojson_value(&value) {
            Ok(GeometryValue::FlowGeometry2D(
                value.try_into().map_err(Error::unsupported_feature)?,
            ))
        } else {
            Ok(GeometryValue::FlowGeometry3D(
                value.try_into().map_err(Error::unsupported_feature)?,
            ))
        }
    }
}

impl From<GmlGeometry> for Vec<geojson::Value> {
    fn from(feature: GmlGeometry) -> Self {
        let mut values = feature
            .polygons
            .into_iter()
            .map(|poly| poly.into())
            .collect::<Vec<_>>();
        values.extend(feature.line_strings.into_iter().map(|line| line.into()));
        values.extend(feature.points.into_iter().map(|point| {
            let point: Point3D<f64> = point.into();
            point.into()
        }));
        values
    }
}

// TODO(new-geometry): port to new geometry; gated off until then.
#[cfg(not(feature = "new-geometry"))]
impl TryFrom<geojson::Feature> for Feature {
    type Error = Error;

    fn try_from(geom: geojson::Feature) -> Result<Self> {
        let attributes = if let Some(attributes) = geom.properties {
            from_geojson_object_to_attribute_value_map(&attributes)
        } else {
            IndexMap::new()
        };
        let geometry = if let Some(geometry) = geom.geometry {
            geometry.value.try_into()?
        } else {
            GeometryValue::None
        };
        let epsg = if let GeometryValue::FlowGeometry3D(_) = geometry {
            EPSG_WGS84_GEOGRAPHIC_3D
        } else {
            EPSG_WGS84_GEOGRAPHIC_2D
        };

        Ok(Feature {
            id: geom.id.map_or_else(uuid::Uuid::new_v4, |id| {
                if let geojson::feature::Id::String(v) = id {
                    uuid::Uuid::parse_str(&v).unwrap_or_else(|_| uuid::Uuid::new_v4())
                } else {
                    uuid::Uuid::new_v4()
                }
            }),
            attributes: Arc::new(attributes),
            geometry: Arc::new(crate::Geometry {
                epsg: Some(epsg),
                value: geometry,
            }),
        })
    }
}

#[cfg(not(feature = "new-geometry"))]
fn from_geojson_object_to_attribute_value_map(
    obj: &geojson::JsonObject,
) -> IndexMap<Attribute, AttributeValue> {
    obj.iter()
        .map(|(k, v)| (Attribute::new(k), v.clone().into()))
        .collect()
}

#[cfg(all(test, not(feature = "new-geometry")))]
mod tests {
    use reearth_flow_geometry::types::coordinate::Coordinate3D;
    use reearth_flow_geometry::types::line_string::LineString3D;
    use reearth_flow_geometry::types::polygon::Polygon3D;

    use super::*;
    use crate::geometry::{CityGmlGeometry, Geometry, GeometryType};
    use crate::Attributes;

    fn sample_polygon() -> Polygon3D<f64> {
        Polygon3D::new(
            LineString3D::new(vec![
                Coordinate3D::new__(0.0, 0.0, 0.0),
                Coordinate3D::new__(1.0, 0.0, 0.0),
                Coordinate3D::new__(0.0, 1.0, 0.0),
                Coordinate3D::new__(0.0, 0.0, 0.0),
            ]),
            vec![],
        )
    }

    fn sample_line_string() -> LineString3D<f64> {
        LineString3D::new(vec![
            Coordinate3D::new__(0.0, 0.0, 0.0),
            Coordinate3D::new__(1.0, 1.0, 1.0),
        ])
    }

    // Case 1: a Point-only GmlGeometry converts to a single Point value, preserving z.
    #[test]
    fn gml_geometry_with_only_points_converts_to_point_values() {
        let gml_geometry = GmlGeometry {
            points: vec![Coordinate3D::new__(137.32, 34.68, 12.5)],
            len: 1,
            ..GmlGeometry::new(GeometryType::Point, Some(0))
        };

        let values: Vec<geojson::Value> = gml_geometry.into();

        assert_eq!(values.len(), 1);
        assert_eq!(values[0], geojson::Value::Point(vec![137.32, 34.68, 12.5]));
    }

    // Case 2: polygon + line-string + point all convert, in the order polygons -> lines -> points.
    #[test]
    fn gml_geometry_mixed_converts_all_in_order() {
        let gml_geometry = GmlGeometry {
            polygons: vec![sample_polygon()],
            line_strings: vec![sample_line_string()],
            points: vec![Coordinate3D::new__(2.0, 3.0, 4.0)],
            len: 1,
            ..GmlGeometry::new(GeometryType::Surface, Some(0))
        };

        let values: Vec<geojson::Value> = gml_geometry.into();

        assert_eq!(values.len(), 3);
        assert!(matches!(values[0], geojson::Value::Polygon(_)));
        assert!(matches!(values[1], geojson::Value::LineString(_)));
        assert_eq!(values[2], geojson::Value::Point(vec![2.0, 3.0, 4.0]));
    }

    // A feature whose CityGML geometry contains only points converts to a
    // GeoJSON feature with a Point geometry.
    #[test]
    fn feature_with_only_points_yields_at_least_one_geojson_feature() {
        let gml_geometry = GmlGeometry {
            points: vec![Coordinate3D::new__(137.32, 34.68, 12.5)],
            len: 1,
            ..GmlGeometry::new(GeometryType::Point, Some(0))
        };
        let citygml_geometry = CityGmlGeometry::new(vec![gml_geometry], Vec::new(), Vec::new());
        let feature = Feature::new_with_attributes_and_geometry(
            Attributes::new(),
            Geometry::with_value(GeometryValue::CityGmlGeometry(citygml_geometry)),
        );

        let geojson_features: Vec<geojson::Feature> = feature.try_into().unwrap();

        assert_eq!(geojson_features.len(), 1);
        assert!(matches!(
            geojson_features[0].geometry.as_ref().map(|g| &g.value),
            Some(geojson::Value::Point(_))
        ));
    }
}
