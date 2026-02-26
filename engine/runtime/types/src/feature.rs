use std::{
    collections::HashMap,
    fmt::Display,
    hash::{Hash, Hasher},
    sync::Arc,
};

use indexmap::IndexMap;
use nutype::nutype;
use reearth_flow_common::{
    str,
    xml::{xpath_value_to_json, XmlXpathValue},
};
use reearth_flow_eval_expr::{engine::Engine, scope::Scope};
use serde::{Deserialize, Serialize};
use serde_json::Number;
use sqlx::{any::AnyTypeInfoKind, Column, Row, ValueRef};

pub use crate::attribute::AttributeValue;
use crate::{
    all_attribute_keys,
    attribute::Attribute,
    geometry::Geometry,
    metadata::{CITYGML_FEATURE_TYPE_KEY, CITYGML_GML_ID_KEY, CITYGML_LOD_MASK_KEY},
};

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

/// Type alias for feature attributes to reduce verbosity
pub type Attributes = IndexMap<Attribute, AttributeValue>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Feature {
    pub id: uuid::Uuid,
    pub attributes: Arc<Attributes>,
    pub geometry: Arc<Geometry>,
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
            .into_iter()
            .map(|(k, v)| (Attribute::new(k), v))
            .collect::<Attributes>();
        Self {
            id: uuid::Uuid::new_v4(),
            attributes: Arc::new(attributes),
            geometry: Arc::new(Geometry::default()),
        }
    }
}

impl Hash for Feature {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl From<Attributes> for Feature {
    fn from(v: Attributes) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            attributes: Arc::new(v),
            geometry: Arc::new(Geometry::default()),
        }
    }
}

impl From<Geometry> for Feature {
    fn from(v: Geometry) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            geometry: Arc::new(v),
            attributes: Arc::new(Attributes::new()),
        }
    }
}

impl From<XmlXpathValue> for Feature {
    fn from(value: XmlXpathValue) -> Self {
        std::convert::Into::<Feature>::into(xpath_value_to_json(&value))
    }
}

impl From<AttributeValue> for Feature {
    fn from(v: AttributeValue) -> Self {
        let attributes = match v {
            AttributeValue::Map(v) => v,
            _ => HashMap::new(),
        };
        let attributes = attributes
            .into_iter()
            .map(|(k, v)| (Attribute::new(k), v))
            .collect::<Attributes>();
        Self {
            id: uuid::Uuid::new_v4(),
            attributes: Arc::new(attributes),
            geometry: Arc::new(Geometry::default()),
        }
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
            // Use Number as-is without conversion (TypeRef::Integer/Double is auto-determined)
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
            return Self::new_with_attributes(Attributes::new());
        };
        let Some(serde_json::Value::Object(attributes)) = v
            .get("attributes")
            .cloned()
            .or_else(|| Some(serde_json::Value::Object(serde_json::Map::new())))
        else {
            return Self::new_with_attributes(Attributes::new());
        };
        let attributes = attributes
            .iter()
            .map(|(k, v)| {
                (
                    Attribute::new(k.to_string()),
                    AttributeValue::from(v.clone()),
                )
            })
            .collect::<Attributes>();
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
            attributes: Arc::new(attributes),
            geometry: Arc::new(geometry.unwrap_or_default()),
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
            .collect::<Result<Attributes, _>>()?;
        Ok(Self::from(attributes))
    }
}

impl Feature {
    pub fn new_with_id_and_attributes(id: uuid::Uuid, attributes: Attributes) -> Self {
        Self {
            id,
            attributes: Arc::new(attributes),
            geometry: Arc::new(Geometry::default()),
        }
    }

    pub fn new_with_attributes(attributes: Attributes) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            attributes: Arc::new(attributes),
            geometry: Arc::new(Geometry::default()),
        }
    }

    pub fn new_with_attributes_and_geometry(attributes: Attributes, geometry: Geometry) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            attributes: Arc::new(attributes),
            geometry: Arc::new(geometry),
        }
    }

    pub fn refresh_id(&mut self) {
        self.id = uuid::Uuid::new_v4();
    }

    /// Replace attributes, keeping other fields. Wraps in new Arc.
    pub fn with_attributes(&self, attributes: Attributes) -> Self {
        Self {
            id: self.id,
            attributes: Arc::new(attributes),
            geometry: Arc::clone(&self.geometry),
        }
    }

    /// Replace attributes by consuming self. More efficient - reuses Arc for geometry.
    pub fn into_with_attributes(self, attributes: Attributes) -> Self {
        Self {
            id: self.id,
            attributes: Arc::new(attributes),
            geometry: self.geometry,
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
            if let Some(v) = self.get(key) {
                result.push(v.clone());
            }
        }
        if result.is_empty() {
            None
        } else {
            Some(AttributeValue::Array(result))
        }
    }

    /// Insert an attribute. Uses copy-on-write if shared.
    pub fn insert<T: AsRef<str> + std::fmt::Display>(
        &mut self,
        key: T,
        value: AttributeValue,
    ) -> Option<AttributeValue> {
        Arc::make_mut(&mut self.attributes).insert(Attribute::new(key.to_string()), value)
    }

    /// Extend attributes. Uses copy-on-write if shared.
    /// Accepts any IntoIterator (HashMap, IndexMap, Vec, etc.) to preserve caller's ordering.
    pub fn extend<I: IntoIterator<Item = (Attribute, AttributeValue)>>(&mut self, attributes: I) {
        Arc::make_mut(&mut self.attributes).extend(attributes);
    }

    /// Extend attributes from string keys. Uses copy-on-write if shared.
    /// Accepts any IntoIterator (HashMap, IndexMap, Vec, etc.) to preserve caller's ordering.
    pub fn extend_attributes<I: IntoIterator<Item = (String, AttributeValue)>>(
        &mut self,
        attributes: I,
    ) {
        Arc::make_mut(&mut self.attributes)
            .extend(attributes.into_iter().map(|(k, v)| (Attribute::new(k), v)));
    }

    /// Remove an attribute. Uses copy-on-write if shared.
    pub fn remove<T: AsRef<str> + std::fmt::Display>(&mut self, key: T) -> Option<AttributeValue> {
        Arc::make_mut(&mut self.attributes).swap_remove(&Attribute::new(key.to_string()))
    }

    /// Get mutable access to attributes. Uses copy-on-write if shared.
    pub fn attributes_mut(&mut self) -> &mut Attributes {
        Arc::make_mut(&mut self.attributes)
    }

    /// Get mutable access to geometry. Uses copy-on-write if shared.
    pub fn geometry_mut(&mut self) -> &mut Geometry {
        Arc::make_mut(&mut self.geometry)
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
                .iter()
                .map(|(k, v)| (k.clone().into_inner(), v.clone().into()))
                .collect::<serde_json::Map<_, _>>(),
        );
        scope.set("__value", value.clone());
        scope.set_dynamic("value", &value);
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
        for (key, value) in self.attributes.iter() {
            keys.push(key.clone().to_string());
            if let AttributeValue::Map(map) = value {
                keys.extend(all_attribute_keys(map));
            }
        }
        keys
    }

    pub fn feature_id(&self) -> Option<String> {
        self.get(CITYGML_GML_ID_KEY).and_then(|v| v.as_string())
    }

    pub fn feature_type(&self) -> Option<String> {
        self.get(CITYGML_FEATURE_TYPE_KEY)
            .and_then(|v| v.as_string())
    }

    pub fn lod(&self) -> Option<String> {
        self.get(CITYGML_LOD_MASK_KEY)
            .and_then(|v| v.as_i64())
            .and_then(|mask| {
                let mask = mask as u8;
                if mask == 0 {
                    None
                } else {
                    Some((7 - mask.leading_zeros() as u8).to_string())
                }
            })
    }

    pub fn update_feature_type(&mut self, feature_type: String) {
        self.insert(
            CITYGML_FEATURE_TYPE_KEY,
            AttributeValue::String(feature_type),
        );
    }

    pub fn update_feature_id(&mut self, feature_id: String) {
        self.insert(CITYGML_GML_ID_KEY, AttributeValue::String(feature_id));
    }
}
