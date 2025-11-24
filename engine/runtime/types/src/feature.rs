use std::{
    collections::HashMap,
    fmt::Display,
    hash::{Hash, Hasher},
    sync::Arc,
};

use indexmap::IndexMap;
use nutype::nutype;
use reearth_flow_common::{str, xml::XmlXpathValue};
use reearth_flow_eval_expr::{engine::Engine, scope::Scope};
use serde::{Deserialize, Serialize};
use serde_json::Number;
use sqlx::{any::AnyTypeInfoKind, Column, Row, ValueRef};

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
    pub attributes: IndexMap<Attribute, AttributeValue>,
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

impl From<IndexMap<String, AttributeValue>> for Feature {
    fn from(v: IndexMap<String, AttributeValue>) -> Self {
        let attributes = v
            .iter()
            .map(|(k, v)| (Attribute::new(k.to_string()), v.clone()))
            .collect::<IndexMap<_, _>>();
        Self {
            id: uuid::Uuid::new_v4(),
            attributes,
            metadata: Default::default(),
            geometry: Default::default(),
        }
    }
}

impl Hash for Feature {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl From<IndexMap<Attribute, AttributeValue>> for Feature {
    fn from(v: IndexMap<Attribute, AttributeValue>) -> Self {
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
            attributes: IndexMap::new(),
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
            .collect::<IndexMap<_, _>>();
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
        schema.types.insert(feature_type, v.into());
        schema
    }
}

impl From<&Feature> for nusamai_citygml::schema::TypeDef {
    fn from(v: &Feature) -> Self {
        let mut attributes = nusamai_citygml::schema::Map::default();
        for (k, v) in v
            .attributes
            .iter()
            .filter(|(_, v)| v.convertible_nusamai_type_ref())
        {
            if let AttributeValue::Number(value) = v {
                attributes.insert(
                    k.to_string(),
                    AttributeValue::String(value.to_string()).into(),
                );
                continue;
            }
            attributes.insert(k.to_string(), v.clone().into());
        }
        nusamai_citygml::schema::TypeDef::Feature(nusamai_citygml::schema::FeatureTypeDef {
            attributes,
            additional_attributes: true,
        })
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
            .collect::<IndexMap<_, _>>();
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

impl TryFrom<sqlx::any::AnyRow> for Feature {
    type Error = crate::error::Error;

    fn try_from(value: sqlx::any::AnyRow) -> Result<Self, Self::Error> {
        let attributes = value
            .columns
            .iter()
            .map(|column| {
                let raw = value.try_get_raw(column.name()).map_err(|e| {
                    crate::error::Error::Conversion(format!("Failed to get column: {e}"))
                })?;
                let type_info = raw.type_info();
                let result = match type_info.kind() {
                    AnyTypeInfoKind::Text => {
                        let value: String = value.try_get(column.ordinal()).map_err(|e| {
                            crate::error::Error::Conversion(format!("Failed to get text: {e}"))
                        })?;
                        (
                            Attribute::new(column.name.to_string()),
                            AttributeValue::String(value),
                        )
                    }
                    AnyTypeInfoKind::SmallInt
                    | AnyTypeInfoKind::BigInt
                    | AnyTypeInfoKind::Integer => {
                        let value: i64 = value.try_get(column.ordinal()).map_err(|e| {
                            crate::error::Error::Conversion(format!("Failed to get integer: {e}"))
                        })?;
                        (
                            Attribute::new(column.name.to_string()),
                            AttributeValue::Number(Number::from(value)),
                        )
                    }
                    AnyTypeInfoKind::Double | AnyTypeInfoKind::Real => {
                        let value: f64 = value.try_get(column.ordinal()).map_err(|e| {
                            crate::error::Error::Conversion(format!("Failed to get float: {e}"))
                        })?;
                        (
                            Attribute::new(column.name.to_string()),
                            AttributeValue::Number(
                                Number::from_f64(value).unwrap_or(Number::from(0)),
                            ),
                        )
                    }
                    AnyTypeInfoKind::Bool => {
                        let value: bool = value.try_get(column.ordinal()).map_err(|e| {
                            crate::error::Error::Conversion(format!("Failed to get bool: {e}"))
                        })?;
                        (
                            Attribute::new(column.name.to_string()),
                            AttributeValue::Bool(value),
                        )
                    }
                    AnyTypeInfoKind::Null => (
                        Attribute::new(column.name.to_string()),
                        AttributeValue::Null,
                    ),
                    _ => (
                        Attribute::new(column.name.to_string()),
                        AttributeValue::Null,
                    ),
                };
                Ok::<(Attribute, AttributeValue), Self::Error>(result)
            })
            .collect::<Result<IndexMap<_, _>, _>>()?;
        Ok(Self::from(attributes))
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
            attributes: IndexMap::new(),
            metadata: Metadata::new(),
            geometry: Geometry::new(),
        }
    }

    pub fn new_with_id_and_attributes(
        id: uuid::Uuid,
        attributes: IndexMap<Attribute, AttributeValue>,
    ) -> Self {
        Self {
            id,
            attributes,
            metadata: Default::default(),
            geometry: Default::default(),
        }
    }

    pub fn new_with_attributes(attributes: IndexMap<Attribute, AttributeValue>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            attributes,
            metadata: Default::default(),
            geometry: Default::default(),
        }
    }

    pub fn new_with_attributes_and_geometry(
        attributes: IndexMap<Attribute, AttributeValue>,
        geometry: Geometry,
        metadata: Metadata,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            attributes,
            geometry,
            metadata,
        }
    }

    pub fn refresh_id(&mut self) {
        self.id = uuid::Uuid::new_v4();
    }

    pub fn with_attributes(&self, attributes: IndexMap<Attribute, AttributeValue>) -> Self {
        Self {
            id: self.id,
            attributes,
            geometry: self.geometry.clone(),
            metadata: self.metadata.clone(),
        }
    }

    pub fn into_with_attributes(self, attributes: IndexMap<Attribute, AttributeValue>) -> Self {
        Self {
            id: self.id,
            attributes,
            geometry: self.geometry,
            metadata: self.metadata,
        }
    }

    pub fn contains_key<T: AsRef<str> + std::fmt::Display>(&self, key: T) -> bool {
        self.attributes
            .contains_key(&Attribute::new(key.to_string()))
    }

    pub fn get<T: AsRef<str> + std::fmt::Display>(&self, key: T) -> Option<&AttributeValue> {
        self.attributes.get(&Attribute::new(key.to_string()))
    }

    pub fn get_by_keys<T: AsRef<str> + std::fmt::Display>(
        &self,
        keys: &[T],
    ) -> Option<AttributeValue> {
        let mut result = Vec::new();
        for key in keys {
            if let Some(v) = self.get(Attribute::new(key.to_string())) {
                result.push(v.clone());
            }
        }
        if result.is_empty() {
            None
        } else {
            Some(AttributeValue::Array(result))
        }
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

    pub fn remove<T: AsRef<str> + std::fmt::Display>(&mut self, key: T) -> Option<AttributeValue> {
        self.attributes
            .swap_remove(&Attribute::new(key.to_string()))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Attribute, &AttributeValue)> {
        self.attributes.iter()
    }

    pub fn new_scope(
        &self,
        engine: Arc<Engine>,
        with: &Option<HashMap<String, serde_json::Value>>,
    ) -> Scope {
        let scope = engine.new_scope();
        let value: serde_json::Value = serde_json::Value::Object(
            self.attributes
                .clone()
                .into_iter()
                .map(|(k, v)| (k.into_inner().to_string(), v.into()))
                .collect::<serde_json::Map<_, _>>(),
        );
        scope.set("__value", value);
        scope.set(
            "__feature_type",
            serde_json::Value::String(self.feature_type().unwrap_or_default()),
        );
        scope.set(
            "__feature_id",
            if let Some(id) = self.feature_id() {
                serde_json::Value::String(id)
            } else {
                serde_json::Value::Null
            },
        );
        scope.set(
            "__lod",
            serde_json::Value::String(self.lod().unwrap_or_default()),
        );
        if let Some(with) = with {
            for (k, v) in with {
                scope.set(k, v.clone());
            }
        }
        scope
    }

    pub fn as_map(&self) -> HashMap<String, AttributeValue> {
        self.attributes
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    pub fn fetch_attribute_value(
        &self,
        engine: Arc<Engine>,
        with: &Option<HashMap<String, serde_json::Value>>,
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
            let scope = self.new_scope(engine.clone(), with);
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

    pub fn lod(&self) -> Option<String> {
        self.metadata
            .lod
            .and_then(|lod| lod.highest_lod().map(|lod| lod.to_string()))
    }

    pub fn update_feature_type(&mut self, feature_type: String) {
        self.metadata.feature_type = Some(feature_type);
    }

    pub fn update_feature_id(&mut self, feature_id: String) {
        self.metadata.feature_id = Some(feature_id);
    }
}
