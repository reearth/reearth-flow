use std::{collections::HashMap, fmt::Display};

use reearth_flow_common::{str, xml::XmlXpathValue};
use serde::{Deserialize, Serialize};

pub use crate::attribute::AttributeValue;
use crate::{geometry::Geometry, Attribute};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Feature {
    #[serde(rename = "_id")]
    pub id: uuid::Uuid,
    pub attributes: HashMap<Attribute, AttributeValue>,
    pub geometry: Option<Geometry>,
}

impl Display for Feature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl PartialEq for Feature {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Feature {}

impl From<HashMap<String, AttributeValue>> for Feature {
    fn from(v: HashMap<String, AttributeValue>) -> Self {
        let attributes = v
            .iter()
            .map(|(k, v)| (Attribute::new(k.to_string()), v.clone()))
            .collect::<HashMap<_, _>>();
        Self {
            id: uuid::Uuid::new_v4(),
            attributes,
            geometry: None,
        }
    }
}

impl From<HashMap<Attribute, AttributeValue>> for Feature {
    fn from(v: HashMap<Attribute, AttributeValue>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            attributes: v,
            geometry: None,
        }
    }
}

impl From<Geometry> for Feature {
    fn from(v: Geometry) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            attributes: HashMap::new(),
            geometry: Some(v),
        }
    }
}

impl From<XmlXpathValue> for Feature {
    fn from(value: XmlXpathValue) -> Self {
        std::convert::Into::<Feature>::into(value.to_string().parse::<serde_json::Value>().unwrap())
    }
}

impl From<AttributeValue> for Feature {
    fn from(v: AttributeValue) -> Self {
        let attributes = match v {
            AttributeValue::Map(v) => v,
            _ => HashMap::new(),
        };
        let attributes = attributes
            .iter()
            .flat_map(|(k, v)| {
                if k == "_id" {
                    None
                } else {
                    Some((Attribute::new(k.to_string()), v.clone()))
                }
            })
            .collect::<HashMap<_, _>>();
        let id = if let Some(AttributeValue::String(id)) = attributes.get(&Attribute::new("_id")) {
            uuid::Uuid::parse_str(id).unwrap_or_else(|_| uuid::Uuid::new_v4())
        } else {
            uuid::Uuid::new_v4()
        };
        Self {
            id,
            attributes,
            geometry: None,
        }
    }
}

impl From<Feature> for AttributeValue {
    fn from(v: Feature) -> Self {
        let mut attributes = v
            .attributes
            .into_iter()
            .map(|(k, v)| (k.into_inner(), v))
            .collect::<HashMap<_, _>>();
        attributes.insert("_id".to_string(), AttributeValue::String(v.id.to_string()));
        AttributeValue::Map(attributes)
    }
}

impl From<Feature> for HashMap<String, AttributeValue> {
    fn from(v: Feature) -> Self {
        let mut attributes = v
            .attributes
            .iter()
            .map(|(k, v)| (k.clone().into_inner(), v.clone()))
            .collect::<HashMap<_, _>>();
        attributes.insert("_id".to_string(), AttributeValue::String(v.id.to_string()));
        attributes
    }
}

impl From<serde_json::Value> for Feature {
    fn from(v: serde_json::Value) -> Self {
        let serde_json::Value::Object(v) = v else {
            return Self::new();
        };
        let Some(serde_json::Value::Object(attributes)) = v
            .get("attributes")
            .cloned()
            .or_else(|| Some(serde_json::Value::Object(serde_json::Map::new())))
        else {
            return Self::new();
        };
        let attributes = attributes
            .iter()
            .flat_map(|(k, v)| {
                if k == "_id" {
                    None
                } else {
                    Some((
                        Attribute::new(k.to_string()),
                        AttributeValue::from(v.clone()),
                    ))
                }
            })
            .collect::<HashMap<_, _>>();
        let id = if let Some(serde_json::Value::String(id)) = v.get(&"id".to_string()) {
            uuid::Uuid::parse_str(id).unwrap_or_else(|_| uuid::Uuid::new_v4())
        } else {
            uuid::Uuid::new_v4()
        };
        let geometry: Option<Geometry> = v
            .get("geometry")
            .cloned()
            .map(|v| serde_json::from_value(v).unwrap_or_default());
        Self {
            id,
            attributes,
            geometry,
        }
    }
}

impl From<Feature> for serde_json::Value {
    fn from(v: Feature) -> Self {
        let mut map = serde_json::Map::new();
        map.insert(
            "id".to_string(),
            serde_json::Value::String(v.id.to_string()),
        );
        map.insert(
            "attributes".to_string(),
            serde_json::Value::Object(
                v.attributes
                    .into_iter()
                    .map(|(k, v)| (k.into_inner().to_string(), v.into()))
                    .collect::<serde_json::Map<_, _>>(),
            ),
        );
        map.insert(
            "geometry".to_string(),
            serde_json::to_value(v.geometry).unwrap_or_default(),
        );
        serde_json::Value::Object(map)
    }
}

impl Feature {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            attributes: HashMap::new(),
            geometry: None,
        }
    }

    pub fn new_with_id_and_attributes(
        id: uuid::Uuid,
        attributes: HashMap<Attribute, AttributeValue>,
    ) -> Self {
        Self {
            id,
            attributes,
            geometry: None,
        }
    }

    pub fn new_with_attributes(attributes: HashMap<Attribute, AttributeValue>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            attributes,
            geometry: None,
        }
    }

    pub fn with_attributes(&self, attributes: HashMap<Attribute, AttributeValue>) -> Self {
        Self {
            id: self.id,
            attributes,
            geometry: None,
        }
    }

    pub fn into_with_attributes(self, attributes: HashMap<Attribute, AttributeValue>) -> Self {
        Self {
            id: self.id,
            attributes,
            geometry: None,
        }
    }

    pub fn get<T: AsRef<str> + std::fmt::Display>(&self, key: &T) -> Option<&AttributeValue> {
        self.attributes.get(&Attribute::new(key.to_string()))
    }

    pub fn insert<T: AsRef<str> + std::fmt::Display>(
        &mut self,
        key: T,
        value: AttributeValue,
    ) -> Option<AttributeValue> {
        self.attributes
            .insert(Attribute::new(key.to_string()), value)
    }

    pub fn remove<T: AsRef<str> + std::fmt::Display>(&mut self, key: &T) -> Option<AttributeValue> {
        self.attributes.remove(&Attribute::new(key.to_string()))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Attribute, &AttributeValue)> {
        self.attributes.iter()
    }
}
