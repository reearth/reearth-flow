//! Rhai `Dynamic` to `AttributeValue` conversion.

use rhai::serde::from_dynamic;
use std::collections::HashMap;

use crate::error::Error;
use crate::AttributeValue;

pub fn attribute_value_from_rhai(value: rhai::Dynamic) -> Result<AttributeValue, Error> {
    // Skip UNIT (null) values - they should not create attributes
    if value.is_unit() {
        return Err(Error::internal_runtime(
            "UNIT value cannot be converted to AttributeValue",
        ));
    }
    let value: serde_json::Value = from_dynamic(&value).map_err(Error::internal_runtime)?;
    let value: AttributeValue = value.into();
    Ok(normalize_action_value(value))
}

fn normalize_action_value(value: AttributeValue) -> AttributeValue {
    match &value {
        AttributeValue::Map(v) => match v.len() {
            len if len > 1 => {
                let mut value = HashMap::new();
                for (k, v) in v.iter() {
                    value.insert(k.clone(), normalize_action_value(v.clone()));
                }
                AttributeValue::Map(value)
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
        AttributeValue::Array(v) => {
            let result = v
                .iter()
                .map(|value| normalize_action_value(value.clone()))
                .collect::<Vec<_>>();
            AttributeValue::Array(result)
        }
        _ => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Number;

    #[test]
    fn test_from_rhai_dynamic() {
        let dynamic_value = rhai::Dynamic::from(42_i64);
        let action_value = attribute_value_from_rhai(dynamic_value);
        assert_eq!(
            action_value.unwrap(),
            AttributeValue::Number(Number::from(42))
        );

        let dynamic_value = rhai::Dynamic::from("Hello");
        let action_value = attribute_value_from_rhai(dynamic_value);
        assert_eq!(
            action_value.unwrap(),
            AttributeValue::String("Hello".to_string())
        );
    }
}
