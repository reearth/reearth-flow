use std::collections::HashMap;

use itertools::Itertools;

use crate::{
    error::{Error, Result},
    Attribute, AttributeValue, Feature, GeometryValue,
};

impl TryFrom<Feature> for Vec<geojson::Feature> {
    type Error = Error;

    fn try_from(geom: Feature) -> Result<Self> {
        let properites = extract_properties(&geom.attributes);
        let Some(geometry) = geom.geometry else {
            return Err(Error::unsupported_feature("no geometry found"));
        };
        let geojson_features = match geometry.value {
            GeometryValue::CityGmlGeometry(gml_geometry) => gml_geometry
                .features
                .into_iter()
                .flat_map(|f| {
                    let geometries: Vec<geojson::Value> = f.into();
                    geometries
                        .into_iter()
                        .map(|g| geojson::Feature {
                            bbox: None,
                            geometry: Some(g.into()),
                            id: None,
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
                    id: None,
                    properties: Some(properites),
                    foreign_members: None,
                }]
            }
            GeometryValue::FlowGeometry3D(flow_geometry) => {
                vec![geojson::Feature {
                    bbox: None,
                    geometry: Some(flow_geometry.into()),
                    id: None,
                    properties: Some(properites),
                    foreign_members: None,
                }]
            }
            GeometryValue::None => {
                return Err(Error::unsupported_feature("no geometry found"));
            }
        };
        Ok(geojson_features)
    }
}

fn extract_properties(map: &HashMap<Attribute, AttributeValue>) -> geojson::JsonObject {
    let mut properties = geojson::JsonObject::new();
    for (k, v) in map.iter() {
        properties.insert(k.to_string(), v.clone().into());
    }
    properties
}
