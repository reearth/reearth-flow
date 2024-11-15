use std::{collections::HashMap, fmt::Display, sync::Arc};

use nutype::nutype;
use reearth_flow_common::{str, xml::XmlXpathValue};
use reearth_flow_eval_expr::{engine::Engine, scope::Scope};
use serde::{Deserialize, Serialize};

pub use crate::attribute::AttributeValue;
use crate::{all_attribute_keys, attribute::Attribute, geometry::Geometry, metadata::Metadata};

#[nutype(
    sanitize(trim),
    derive(
        Debug,
        Display,
        Clone,
        Eq,
        PartialEq,
        PartialOrd,
        Ord,
        AsRef,
        Serialize,
        Deserialize,
        Hash,
        JsonSchema
    )
)]
pub struct MetadataKey(String);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Feature {
    pub id: uuid::Uuid,
    pub attributes: HashMap<Attribute, AttributeValue>,
    pub metadata: Metadata,
    pub geometry: Geometry,
}

impl Default for Feature {
    fn default() -> Self {
        Self::new()
    }
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
            metadata: Default::default(),
            geometry: Default::default(),
        }
    }
}

impl From<HashMap<Attribute, AttributeValue>> for Feature {
    fn from(v: HashMap<Attribute, AttributeValue>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            attributes: v,
            metadata: Default::default(),
            geometry: Default::default(),
        }
    }
}

impl From<Geometry> for Feature {
    fn from(v: Geometry) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            geometry: v,
            metadata: Default::default(),
            attributes: HashMap::new(),
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
            .map(|(k, v)| (Attribute::new(k.to_string()), v.clone()))
            .collect::<HashMap<_, _>>();
        Self {
            id: uuid::Uuid::new_v4(),
            attributes,
            metadata: Default::default(),
            geometry: Default::default(),
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

impl From<&Feature> for nusamai_citygml::schema::Schema {
    fn from(v: &Feature) -> Self {
        let mut schema = nusamai_citygml::schema::Schema::default();
        let Some(feature_type) = v.feature_type() else {
            return schema;
        };
        let mut attributes = nusamai_citygml::schema::Map::default();
        for (k, v) in v
            .attributes
            .iter()
            .filter(|(_, v)| v.convertible_nusamai_type_ref())
        {
            attributes.insert(k.to_string(), v.clone().into());
        }
        schema.types.insert(
            feature_type,
            nusamai_citygml::schema::TypeDef::Feature(nusamai_citygml::schema::FeatureTypeDef {
                attributes,
                additional_attributes: true,
            }),
        );
        schema
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
        let Some(serde_json::Value::Object(attributes)) = v
            .get("attributes")
            .cloned()
            .or_else(|| Some(serde_json::Value::Object(serde_json::Map::new())))
        else {
            return Self::new();
        };
        let attributes = attributes
            .iter()
            .map(|(k, v)| {
                (
                    Attribute::new(k.to_string()),
                    AttributeValue::from(v.clone()),
                )
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

        let metadata: Option<Metadata> = v
            .get("metadata")
            .cloned()
            .map(|v| serde_json::from_value(v).unwrap_or_default());
        Self {
            id,
            attributes,
            geometry: geometry.unwrap_or_default(),
            metadata: metadata.unwrap_or_default(),
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
        map.insert(
            "metadata".to_string(),
            serde_json::to_value(v.metadata).unwrap_or_default(),
        );
        serde_json::Value::Object(map)
    }
}

impl Feature {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            attributes: HashMap::new(),
            metadata: Metadata::new(),
            geometry: Geometry::new(),
        }
    }

    pub fn new_with_id_and_attributes(
        id: uuid::Uuid,
        attributes: HashMap<Attribute, AttributeValue>,
    ) -> Self {
        Self {
            id,
            attributes,
            metadata: Default::default(),
            geometry: Default::default(),
        }
    }

    pub fn new_with_attributes(attributes: HashMap<Attribute, AttributeValue>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            attributes,
            metadata: Default::default(),
            geometry: Default::default(),
        }
    }

    pub fn new_with_attributes_and_geometry(
        attributes: HashMap<Attribute, AttributeValue>,
        geometry: Geometry,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            attributes,
            geometry,
            metadata: Default::default(),
        }
    }

    pub fn refresh_id(&mut self) {
        self.id = uuid::Uuid::new_v4();
    }

    pub fn with_attributes(&self, attributes: HashMap<Attribute, AttributeValue>) -> Self {
        Self {
            id: self.id,
            attributes,
            geometry: self.geometry.clone(),
            metadata: self.metadata.clone(),
        }
    }

    pub fn into_with_attributes(self, attributes: HashMap<Attribute, AttributeValue>) -> Self {
        Self {
            id: self.id,
            attributes,
            geometry: self.geometry,
            metadata: self.metadata,
        }
    }

    pub fn contains_key<T: AsRef<str> + std::fmt::Display>(&self, key: &T) -> bool {
        self.attributes
            .contains_key(&Attribute::new(key.to_string()))
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

    pub fn extend(&mut self, attributes: HashMap<Attribute, AttributeValue>) {
        self.attributes.extend(attributes);
    }

    pub fn extend_attributes(&mut self, attributes: HashMap<String, AttributeValue>) {
        self.attributes
            .extend(attributes.into_iter().map(|(k, v)| (Attribute::new(k), v)));
    }

    pub fn remove<T: AsRef<str> + std::fmt::Display>(&mut self, key: &T) -> Option<AttributeValue> {
        self.attributes.remove(&Attribute::new(key.to_string()))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Attribute, &AttributeValue)> {
        self.attributes.iter()
    }

    pub fn new_scope(&self, engine: Arc<Engine>) -> Scope {
        let scope = engine.new_scope();
        let value: serde_json::Value = serde_json::Value::Object(
            self.attributes
                .clone()
                .into_iter()
                .map(|(k, v)| (k.into_inner().to_string(), v.into()))
                .collect::<serde_json::Map<_, _>>(),
        );
        scope.set("__value", value);
        scope
    }

    pub fn to_map(&self) -> HashMap<String, AttributeValue> {
        self.attributes
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    pub fn fetch_attribute_value(
        &self,
        engine: Arc<Engine>,
        attribute: &Option<Vec<Attribute>>,
        attribute_ast: &Option<rhai::AST>,
    ) -> String {
        if let Some(attribute_values) = attribute {
            let values = attribute_values
                .iter()
                .flat_map(|key| self.get(key))
                .cloned()
                .collect::<Vec<_>>();
            values
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join("-")
        } else if let Some(attribute_ast) = attribute_ast {
            let scope = self.new_scope(engine.clone());
            let value = scope.eval_ast::<String>(attribute_ast);

            value.unwrap_or_else(|_| "".to_string())
        } else {
            "".to_string()
        }
    }

    pub fn all_attribute_keys(&self) -> Vec<String> {
        let mut keys = Vec::new();
        for (key, value) in &self.attributes {
            keys.push(key.clone().to_string());
            if let AttributeValue::Map(map) = value {
                keys.extend(all_attribute_keys(map));
            }
        }
        keys
    }

    pub fn feature_id(&self) -> Option<String> {
        self.metadata.feature_id.clone()
    }

    pub fn feature_type(&self) -> Option<String> {
        self.metadata.feature_type.clone()
    }
}
