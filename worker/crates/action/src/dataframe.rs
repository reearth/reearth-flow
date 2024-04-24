use std::{collections::HashMap, fmt::Display};

use nutype::nutype;

use reearth_flow_common::{str, xml::XmlXpathValue};
use serde::{Deserialize, Serialize};

pub use crate::value::AttributeValue;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Dataframe {
    pub features: Vec<Feature>,
}

impl Dataframe {
    pub fn new(features: Vec<Feature>) -> Self {
        Self { features }
    }
}

impl From<Vec<AttributeValue>> for Dataframe {
    fn from(v: Vec<AttributeValue>) -> Self {
        let features = v.into_iter().map(Feature::from).collect::<Vec<_>>();
        Self { features }
    }
}

impl From<Vec<HashMap<Attribute, AttributeValue>>> for Dataframe {
    fn from(v: Vec<HashMap<Attribute, AttributeValue>>) -> Self {
        let features = v.into_iter().map(Feature::from).collect::<Vec<_>>();
        Self { features }
    }
}

impl From<Dataframe> for Vec<AttributeValue> {
    fn from(v: Dataframe) -> Self {
        v.features
            .into_iter()
            .map(Feature::into)
            .collect::<Vec<_>>()
    }
}

impl From<AttributeValue> for Dataframe {
    fn from(v: AttributeValue) -> Self {
        let features = match v {
            AttributeValue::Array(v) => v,
            _ => Vec::new(),
        };
        let features = features.into_iter().map(Feature::from).collect::<Vec<_>>();
        Self { features }
    }
}

impl From<Vec<HashMap<String, AttributeValue>>> for Dataframe {
    fn from(v: Vec<HashMap<String, AttributeValue>>) -> Self {
        let features = v.into_iter().map(Feature::from).collect::<Vec<_>>();
        Self { features }
    }
}

impl From<Dataframe> for Vec<HashMap<String, AttributeValue>> {
    fn from(v: Dataframe) -> Self {
        v.features
            .into_iter()
            .map(HashMap::from)
            .collect::<Vec<_>>()
    }
}

impl From<serde_json::Value> for Dataframe {
    fn from(v: serde_json::Value) -> Self {
        let serde_json::Value::Array(v) = v else {
            return Self::default();
        };
        let features = v.into_iter().map(Feature::from).collect::<Vec<_>>();
        Self { features }
    }
}

impl From<Dataframe> for serde_json::Value {
    fn from(v: Dataframe) -> Self {
        serde_json::Value::Array(
            v.features
                .into_iter()
                .map(serde_json::Value::from)
                .collect(),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
pub struct Feature {
    pub attributes: HashMap<Attribute, AttributeValue>,
}

impl Display for Feature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

impl From<HashMap<String, AttributeValue>> for Feature {
    fn from(v: HashMap<String, AttributeValue>) -> Self {
        let attributes = v
            .iter()
            .map(|(k, v)| (Attribute::new(k.to_string()), v.clone()))
            .collect::<HashMap<_, _>>();
        Self { attributes }
    }
}

impl From<HashMap<Attribute, AttributeValue>> for Feature {
    fn from(v: HashMap<Attribute, AttributeValue>) -> Self {
        Self { attributes: v }
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
            .map(|(k, v)| (Attribute::new(k.clone()), v.clone()))
            .collect::<HashMap<_, _>>();
        Self { attributes }
    }
}

impl From<Feature> for AttributeValue {
    fn from(v: Feature) -> Self {
        AttributeValue::Map(
            v.attributes
                .into_iter()
                .map(|(k, v)| (k.into_inner(), v))
                .collect::<HashMap<_, _>>(),
        )
    }
}

impl From<Feature> for HashMap<String, AttributeValue> {
    fn from(v: Feature) -> Self {
        v.attributes
            .iter()
            .map(|(k, v)| (k.clone().into_inner(), v.clone()))
            .collect::<HashMap<_, _>>()
    }
}

impl From<serde_json::Value> for Feature {
    fn from(v: serde_json::Value) -> Self {
        let serde_json::Value::Object(v) = v else {
            return Self::new();
        };
        let attributes = v
            .into_iter()
            .map(|(k, v)| (Attribute::new(k), AttributeValue::from(v)))
            .collect::<HashMap<_, _>>();
        Self { attributes }
    }
}

impl From<Feature> for serde_json::Value {
    fn from(v: Feature) -> Self {
        let mut map = serde_json::Map::new();
        for (k, v) in v.attributes {
            map.insert(k.into_inner().to_string(), v.into());
        }
        serde_json::Value::Object(map)
    }
}

impl Feature {
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
        }
    }

    pub fn new_with_attributes(attributes: HashMap<Attribute, AttributeValue>) -> Self {
        Self { attributes }
    }

    pub fn with_attributes(&self, attributes: HashMap<Attribute, AttributeValue>) -> Self {
        Self { attributes }
    }

    pub fn into_with_attributes(self, attributes: HashMap<Attribute, AttributeValue>) -> Self {
        Self { attributes }
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

#[nutype(
    sanitize(trim),
    derive(
        Debug,
        Clone,
        Eq,
        PartialEq,
        PartialOrd,
        Ord,
        AsRef,
        Serialize,
        Deserialize,
        Hash
    )
)]
pub struct Attribute(String);

impl Attribute {
    pub fn inner(&self) -> String {
        self.clone().into_inner()
    }
}
