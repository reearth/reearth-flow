use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub use crate::attribute::AttributeValue;
use crate::{feature::Feature, geometry::Geometry, Attribute};

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

impl From<Vec<Geometry>> for Dataframe {
    fn from(v: Vec<Geometry>) -> Self {
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
