use indexmap::IndexMap;
use itertools::Itertools;
use nusamai_projection::crs::{EPSG_WGS84_GEOGRAPHIC_2D, EPSG_WGS84_GEOGRAPHIC_3D};
use reearth_flow_geometry::types::conversion::is_2d_geojson_value;

use crate::{
    error::{Error, Result},
    Attribute, AttributeValue, Feature, GeometryValue, GmlGeometry,
};

impl TryFrom<Feature> for Vec<geojson::Feature> {
    type Error = Error;

    fn try_from(geom: Feature) -> Result<Self> {
        let properites = from_attribute_value_map_to_geojson_object(&geom.attributes);
        let geojson_features = match geom.geometry.value {
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
        values
    }
}

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
            attributes,
            geometry: crate::Geometry {
                epsg: Some(epsg),
                value: geometry,
            },
            metadata: Default::default(),
        })
    }
}

fn from_geojson_object_to_attribute_value_map(
    obj: &geojson::JsonObject,
) -> IndexMap<Attribute, AttributeValue> {
    obj.iter()
        .map(|(k, v)| (Attribute::new(k), v.clone().into()))
        .collect()
}
