use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::{Hash, Hasher};

use bytes::Bytes;
use reearth_flow_common::uri::Uri;
use rhai::serde::from_dynamic;
use serde::{Deserialize, Serialize};
use serde_json::Number;

use reearth_flow_common::str::base64_encode;
use reearth_flow_common::xml::XmlXpathValue;

use crate::error;
use crate::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActionValue {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<ActionValue>),
    Bytes(Bytes),
    Map(HashMap<String, ActionValue>),
}

impl PartialEq for ActionValue {
    fn eq(&self, rhs: &Self) -> bool {
        match (&self, &rhs) {
            (&ActionValue::Null, &ActionValue::Null) => true,
            (&ActionValue::Bool(v0), &ActionValue::Bool(v1)) if v0 == v1 => true,
            (&ActionValue::Number(v0), &ActionValue::Number(v1)) if v0 == v1 => true,
            (&ActionValue::String(v0), &ActionValue::String(v1)) if v0 == v1 => true,
            (&ActionValue::Array(v0), &ActionValue::Array(v1)) if v0 == v1 => true,
            (&ActionValue::Bytes(v0), &ActionValue::Bytes(v1)) if v0 == v1 => true,
            (&ActionValue::Map(v0), &ActionValue::Map(v1)) if v0 == v1 => true,
            _ => false,
        }
    }
}

impl Ord for ActionValue {
    fn cmp(&self, rhs: &Self) -> Ordering {
        match (&self, &rhs) {
            (&ActionValue::Null, &ActionValue::Null) => Ordering::Equal,
            (&ActionValue::Bool(v0), &ActionValue::Bool(v1)) => v0.cmp(v1),
            (&ActionValue::Number(v0), &ActionValue::Number(v1)) => {
                compare_numbers(v0, v1).unwrap()
            }
            (&ActionValue::String(v0), &ActionValue::String(v1)) => v0.cmp(v1),
            (&ActionValue::Array(v0), &ActionValue::Array(v1)) => v0.cmp(v1),
            (&ActionValue::Bytes(v0), &ActionValue::Bytes(v1)) => v0.cmp(v1),
            (v0, v1) => v0.discriminant().cmp(&v1.discriminant()),
        }
    }
}

impl ActionValue {
    fn discriminant(&self) -> usize {
        match *self {
            ActionValue::Null => 0,
            ActionValue::Bool(..) => 1,
            ActionValue::Number(..) => 2,
            ActionValue::String(..) => 3,
            ActionValue::Array(..) => 4,
            ActionValue::Bytes(..) => 5,
            ActionValue::Map(..) => 6,
        }
    }

    pub fn extend(self, value: Self) -> Result<Self> {
        match (self, value) {
            (ActionValue::Map(mut a), ActionValue::Map(b)) => {
                for (k, v) in b {
                    a.insert(k, v);
                }
                Ok(ActionValue::Map(a))
            }
            (ActionValue::Array(mut a), ActionValue::Array(b)) => {
                a.extend(b);
                Ok(ActionValue::Array(a))
            }
            _ => Err(error::Error::internal_runtime("Cannot extend")),
        }
    }
}

impl Eq for ActionValue {}
impl PartialOrd for ActionValue {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        Some(self.cmp(rhs))
    }
}

impl Default for ActionValue {
    fn default() -> Self {
        Self::String("".to_owned())
    }
}

impl Display for ActionValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionValue::Null => write!(f, "null"),
            ActionValue::Bool(v) => write!(f, "{}", v),
            ActionValue::Number(v) => write!(f, "{}", v),
            ActionValue::String(v) => write!(f, "{}", v),
            ActionValue::Array(v) => write!(f, "{:?}", v),
            ActionValue::Bytes(v) => write!(f, "{:?}", v),
            ActionValue::Map(v) => write!(f, "{:?}", v),
        }
    }
}

impl From<serde_json::Value> for ActionValue {
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => ActionValue::Null,
            serde_json::Value::Bool(v) => ActionValue::Bool(v),
            serde_json::Value::Number(v) => ActionValue::Number(v),
            serde_json::Value::String(v) => ActionValue::String(v),
            serde_json::Value::Array(v) => {
                ActionValue::Array(v.into_iter().map(ActionValue::from).collect::<Vec<_>>())
            }
            serde_json::Value::Object(v) => ActionValue::Map(
                v.into_iter()
                    .map(|(k, v)| (k, ActionValue::from(v)))
                    .collect::<HashMap<_, _>>(),
            ),
        }
    }
}

impl From<ActionValue> for serde_json::Value {
    fn from(value: ActionValue) -> Self {
        match value {
            ActionValue::Null => serde_json::Value::Null,
            ActionValue::Bool(v) => serde_json::Value::Bool(v),
            ActionValue::Number(v) => serde_json::Value::Number(v),
            ActionValue::String(v) => serde_json::Value::String(v),
            ActionValue::Array(v) => serde_json::Value::Array(
                v.into_iter()
                    .map(serde_json::Value::from)
                    .collect::<Vec<_>>(),
            ),
            ActionValue::Bytes(v) => serde_json::Value::String(base64_encode(v.as_ref())),
            ActionValue::Map(v) => serde_json::Value::Object(
                v.into_iter()
                    .map(|(k, v)| (k, serde_json::Value::from(v)))
                    .collect::<serde_json::Map<_, _>>(),
            ),
        }
    }
}

impl From<nusamai_citygml::Value> for ActionValue {
    fn from(value: nusamai_citygml::Value) -> Self {
        match value {
            nusamai_citygml::Value::String(v) => ActionValue::String(v),
            nusamai_citygml::Value::Code(v) => ActionValue::String(v.value().to_owned()),
            nusamai_citygml::Value::Integer(v) => ActionValue::Number(Number::from(v)),
            nusamai_citygml::Value::NonNegativeInteger(v) => ActionValue::Number(Number::from(v)),
            nusamai_citygml::Value::Double(v) => ActionValue::Number(Number::from_f64(v).unwrap()),
            nusamai_citygml::Value::Measure(v) => {
                ActionValue::Number(Number::from_f64(v.value()).unwrap())
            }
            nusamai_citygml::Value::Boolean(v) => ActionValue::Bool(v),
            nusamai_citygml::Value::Uri(v) => ActionValue::String(v.value().to_string()),
            nusamai_citygml::Value::Date(v) => ActionValue::String(v.to_string()),
            nusamai_citygml::Value::Point(v) => ActionValue::Map(
                vec![
                    ("type".to_string(), ActionValue::String("Point".to_string())),
                    (
                        "coordinates".to_string(),
                        ActionValue::Array(
                            v.coordinates()
                                .iter()
                                .map(|v| ActionValue::Number(Number::from_f64(*v).unwrap()))
                                .collect(),
                        ),
                    ),
                ]
                .into_iter()
                .collect(),
            ),
            nusamai_citygml::Value::Array(v) => {
                ActionValue::Array(v.into_iter().map(ActionValue::from).collect())
            }
            nusamai_citygml::Value::Object(v) => {
                let m = v
                    .attributes
                    .iter()
                    .map(|(k, v)| (k.into(), ActionValue::from(v.clone())))
                    .collect();
                ActionValue::Map(m)
            }
        }
    }
}

impl From<XmlXpathValue> for ActionValue {
    fn from(value: XmlXpathValue) -> Self {
        std::convert::Into::<ActionValue>::into(
            value.to_string().parse::<serde_json::Value>().unwrap(),
        )
    }
}

impl TryFrom<ActionValue> for rhai::Dynamic {
    type Error = error::Error;

    fn try_from(value: ActionValue) -> std::result::Result<Self, Self::Error> {
        let value: serde_json::Value = value.into();
        let value: rhai::Dynamic =
            serde_json::from_value(value).map_err(error::Error::internal_runtime)?;
        Ok(value)
    }
}

impl TryFrom<rhai::Dynamic> for ActionValue {
    type Error = error::Error;

    fn try_from(value: rhai::Dynamic) -> std::result::Result<Self, Self::Error> {
        let value: serde_json::Value =
            from_dynamic(&value).map_err(error::Error::internal_runtime)?;
        let value: Self = value.into();
        Ok(normalize_action_value(value))
    }
}

fn normalize_action_value(value: ActionValue) -> ActionValue {
    match &value {
        ActionValue::Map(v) => match v.len() {
            len if len > 1 => {
                let mut value = HashMap::new();
                for (k, v) in v.iter() {
                    value.insert(k.clone(), normalize_action_value(v.clone()));
                }
                ActionValue::Map(value)
            }
            1 => {
                let (k, v) = v.iter().next().unwrap();
                match k.as_str() {
                    "String" => v.clone(),
                    "Number" => v.clone(),
                    _ => value,
                }
            }
            _ => value,
        },
        ActionValue::Array(v) => {
            let result = v
                .iter()
                .map(|value| normalize_action_value(value.clone()))
                .collect::<Vec<_>>();
            ActionValue::Array(result)
        }
        _ => value,
    }
}

impl TryFrom<Uri> for ActionValue {
    type Error = error::Error;

    fn try_from(value: Uri) -> std::result::Result<Self, Self::Error> {
        let value: serde_json::Value =
            serde_json::to_value(value).map_err(error::Error::internal_runtime)?;
        Ok(value.into())
    }
}

impl Hash for ActionValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ActionValue::Null => "Null".hash(state),
            ActionValue::Bool(b) => {
                "Bool".hash(state);
                b.hash(state);
            }
            ActionValue::Number(n) => {
                "Number".hash(state);
                n.hash(state);
            }
            ActionValue::String(s) => {
                "String".hash(state);
                s.hash(state);
            }
            ActionValue::Array(arr) => {
                "Array".hash(state);
                arr.hash(state);
            }
            ActionValue::Bytes(b) => {
                "Bytes".hash(state);
                b.hash(state);
            }
            ActionValue::Map(map) => {
                "Map".hash(state);
                for (k, v) in map {
                    k.hash(state);
                    v.hash(state);
                }
            }
        }
    }
}

fn compare_numbers(n1: &Number, n2: &Number) -> Option<Ordering> {
    if let Some(i1) = n1.as_i64() {
        if let Some(i2) = n2.as_i64() {
            return i1.partial_cmp(&i2);
        }
    }
    if let Some(f1) = n1.as_f64() {
        if let Some(f2) = n2.as_f64() {
            return f1.partial_cmp(&f2);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_from_rhai_dynamic() {
        let dynamic_value = rhai::Dynamic::from(42);
        let action_value: std::result::Result<ActionValue, _> = dynamic_value.try_into();
        assert_eq!(action_value.unwrap(), ActionValue::Number(Number::from(42)));

        let dynamic_value = rhai::Dynamic::from("Hello");
        let action_value: std::result::Result<ActionValue, _> = dynamic_value.try_into();
        assert_eq!(
            action_value.unwrap(),
            ActionValue::String("Hello".to_string())
        );
    }

    #[test]
    fn test_partial_ord() {
        let number1 = ActionValue::Number(Number::from(42));
        let number2 = ActionValue::Number(Number::from(42));
        assert_eq!(number1.partial_cmp(&number2), Some(Ordering::Equal));

        let string1 = ActionValue::String("Hello".to_string());
        let string2 = ActionValue::String("World".to_string());
        assert_eq!(string1.partial_cmp(&string2), Some(Ordering::Less));
    }

    #[test]
    fn test_eq() {
        let number1 = ActionValue::Number(Number::from(42));
        let number2 = ActionValue::Number(Number::from(42));
        assert_eq!(number1, number2);

        let string1 = ActionValue::String("Hello".to_string());
        let string2 = ActionValue::String("Hello".to_string());
        assert_eq!(string1, string2);

        let map1 = ActionValue::Map(
            vec![("key".to_string(), ActionValue::String("value".to_string()))]
                .into_iter()
                .collect(),
        );
        let map2 = ActionValue::Map(
            vec![("key".to_string(), ActionValue::String("value".to_string()))]
                .into_iter()
                .collect(),
        );
        assert_eq!(map1, map2);
    }

    #[test]
    fn test_compare_numbers() {
        let number1 = Number::from(42);
        let number2 = Number::from(42);
        assert_eq!(compare_numbers(&number1, &number2), Some(Ordering::Equal));

        let number1 = Number::from(42);
        let number2 = Number::from(43);
        assert_eq!(compare_numbers(&number1, &number2), Some(Ordering::Less));

        let number1 = Number::from(43);
        let number2 = Number::from(42);
        assert_eq!(compare_numbers(&number1, &number2), Some(Ordering::Greater));
    }
}
