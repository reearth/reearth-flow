use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};

use reearth_flow_types::{AttributeValue, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::AttributeProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct AttributeRangeMapperFactory;

impl ProcessorFactory for AttributeRangeMapperFactory {
    fn name(&self) -> &str {
        "Attribute Range Mapper"
    }

    fn description(&self) -> &str {
        "Classifies a numeric attribute by looking its value up in a table of ranges and writing the matched range's output value to another attribute."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AttributeRangeMapperParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Attribute"]
    }

    fn tags(&self) -> &[&'static str] {
        &["mapping"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: AttributeRangeMapperParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                AttributeProcessorError::RangeMapperFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                AttributeProcessorError::RangeMapperFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(AttributeProcessorError::RangeMapperFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let processor = AttributeRangeMapper { params };
        Ok(Box::new(processor))
    }
}

#[derive(Debug, Clone)]
struct AttributeRangeMapper {
    params: AttributeRangeMapperParam,
}

/// # Attribute Range Mapper Parameters
/// Defines the attribute to classify, the table of ranges to match it against, and where the
/// matched value is written.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AttributeRangeMapperParam {
    /// # Input Attribute
    /// Attribute holding the value to classify. Numbers are used directly, numeric strings are
    /// parsed, and booleans count as 1 and 0. Any other type is treated as unclassifiable and
    /// takes the default value.
    pub input_attribute: String,

    /// # Output Attribute
    /// Attribute the matched value is written to. An existing value is overwritten.
    pub output_attribute: String,

    /// # Range Lookup Table
    /// Ranges to test the input against, in order. The first match wins, so overlapping ranges
    /// resolve to whichever is listed first.
    pub range_table: Vec<RangeEntry>,

    /// # Default Value
    /// Value written when no range matches, and also when the input attribute is absent or is
    /// not a number, numeric string, or boolean. When omitted, those features pass through with
    /// the output attribute left unset rather than being rejected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<MappedValue>,
}

/// # Mapped Value
///
/// A value written to an attribute. Accepts text, a number, or true/false, written as the type
/// given — `"3"` stays text and `3` stays a number.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(untagged)]
pub enum MappedValue {
    /// # Text
    /// Written as text.
    Text(String),
    /// # Number
    /// Written as a number.
    Number(serde_json::Number),
    /// # True or False
    /// Written as a true/false value.
    Boolean(bool),
}

impl MappedValue {
    /// The attribute value this writes. Closed over the three scalar types, so unlike an
    /// open JSON value it cannot fail to convert at runtime.
    fn to_attribute_value(&self) -> AttributeValue {
        match self {
            MappedValue::Text(value) => AttributeValue::String(value.clone()),
            MappedValue::Number(value) => AttributeValue::Number(value.clone()),
            MappedValue::Boolean(value) => AttributeValue::Bool(*value),
        }
    }
}

/// # Range Entry
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RangeEntry {
    /// # From (Minimum)
    /// Lower bound of the range, inclusive.
    pub from: f64,

    /// # To (Maximum)
    /// Upper bound of the range, exclusive — a value equal to it falls into the next range.
    /// Setting it equal to the lower bound makes the entry match that one exact value instead.
    pub to: f64,

    /// # Output Value
    /// Value written to the output attribute when the input falls in this range.
    pub output_value: MappedValue,
}

impl AttributeRangeMapper {
    /// The value to write to the output attribute, or `None` to leave it unset.
    ///
    /// A feature whose input is a number, a numeric string or a boolean is tested against each
    /// range in order and takes the first match. Anything else — including an absent attribute —
    /// takes the default value, which is itself optional.
    fn mapped_value(&self, feature: &Feature) -> Option<AttributeValue> {
        let numeric_value: Option<f64> =
            feature
                .get(&self.params.input_attribute)
                .and_then(|v| match v {
                    AttributeValue::Number(n) => n.as_f64(),
                    AttributeValue::String(s) => s.parse::<f64>().ok(),
                    AttributeValue::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
                    _ => None,
                });

        if let Some(num_val) = numeric_value {
            for range in &self.params.range_table {
                // A range is [from, to) — inclusive start, exclusive end — except when the
                // bounds are equal, which matches that one exact value.
                let is_in_range = if (range.to - range.from).abs() < f64::EPSILON {
                    (num_val - range.from).abs() < f64::EPSILON
                } else {
                    num_val >= range.from && num_val < range.to
                };
                if is_in_range {
                    return Some(range.output_value.to_attribute_value());
                }
            }
        }

        self.params
            .default_value
            .as_ref()
            .map(MappedValue::to_attribute_value)
    }
}

impl Processor for AttributeRangeMapper {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        if let Some(value) = self.mapped_value(&feature) {
            feature.insert(self.params.output_attribute.clone(), value);
        }

        fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
        Ok(())
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "Attribute Range Mapper"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_types::{Attributes, Feature};
    use serde_json::Number;

    #[test]
    fn test_range_mapper_numeric_input() {
        let params = AttributeRangeMapperParam {
            input_attribute: "depth".to_string(),
            output_attribute: "color".to_string(),
            range_table: vec![
                RangeEntry {
                    from: 0.0,
                    to: 5.0,
                    output_value: MappedValue::Text("#ff0000".to_string()),
                },
                RangeEntry {
                    from: 5.0,
                    to: 10.0,
                    output_value: MappedValue::Text("#00ff00".to_string()),
                },
                RangeEntry {
                    from: 10.0,
                    to: 20.0,
                    output_value: MappedValue::Text("#0000ff".to_string()),
                },
            ],
            default_value: Some(MappedValue::Text("#cccccc".to_string())),
        };

        let processor = AttributeRangeMapper { params };

        // Test value in first range
        let mut feature = Feature::new_with_attributes(Attributes::new());
        feature.insert(
            "depth",
            AttributeValue::Number(Number::from_f64(3.5).unwrap()),
        );
        assert_eq!(
            map_feature(&processor, &feature).get("color"),
            Some(&AttributeValue::String("#ff0000".to_string()))
        );

        // Test value in second range
        let mut feature = Feature::new_with_attributes(Attributes::new());
        feature.insert(
            "depth",
            AttributeValue::Number(Number::from_f64(7.0).unwrap()),
        );
        assert_eq!(
            map_feature(&processor, &feature).get("color"),
            Some(&AttributeValue::String("#00ff00".to_string()))
        );

        // Test value in third range
        let mut feature = Feature::new_with_attributes(Attributes::new());
        feature.insert(
            "depth",
            AttributeValue::Number(Number::from_f64(15.0).unwrap()),
        );
        assert_eq!(
            map_feature(&processor, &feature).get("color"),
            Some(&AttributeValue::String("#0000ff".to_string()))
        );

        // Test value outside all ranges (should use default)
        let mut feature = Feature::new_with_attributes(Attributes::new());
        feature.insert(
            "depth",
            AttributeValue::Number(Number::from_f64(25.0).unwrap()),
        );
        assert_eq!(
            map_feature(&processor, &feature).get("color"),
            Some(&AttributeValue::String("#cccccc".to_string()))
        );
    }

    #[test]
    fn test_range_mapper_boundary_values() {
        let params = AttributeRangeMapperParam {
            input_attribute: "value".to_string(),
            output_attribute: "result".to_string(),
            range_table: vec![
                RangeEntry {
                    from: 0.0,
                    to: 10.0,
                    output_value: MappedValue::Text("low".to_string()),
                },
                RangeEntry {
                    from: 10.0,
                    to: 20.0,
                    output_value: MappedValue::Text("high".to_string()),
                },
            ],
            default_value: None,
        };

        let processor = AttributeRangeMapper { params };

        // Test lower boundary (inclusive)
        let mut feature = Feature::new_with_attributes(Attributes::new());
        feature.insert(
            "value",
            AttributeValue::Number(Number::from_f64(0.0).unwrap()),
        );
        assert_eq!(
            map_feature(&processor, &feature).get("result"),
            Some(&AttributeValue::String("low".to_string()))
        );

        // Test upper boundary (exclusive for lower range, inclusive for upper)
        let mut feature = Feature::new_with_attributes(Attributes::new());
        feature.insert(
            "value",
            AttributeValue::Number(Number::from_f64(10.0).unwrap()),
        );
        assert_eq!(
            map_feature(&processor, &feature).get("result"),
            Some(&AttributeValue::String("high".to_string()))
        );
    }

    #[test]
    fn test_range_mapper_string_number_conversion() {
        let params = AttributeRangeMapperParam {
            input_attribute: "score".to_string(),
            output_attribute: "grade".to_string(),
            range_table: vec![
                RangeEntry {
                    from: 0.0,
                    to: 60.0,
                    output_value: MappedValue::Text("F".to_string()),
                },
                RangeEntry {
                    from: 60.0,
                    to: 80.0,
                    output_value: MappedValue::Text("C".to_string()),
                },
                RangeEntry {
                    from: 80.0,
                    to: 100.0,
                    output_value: MappedValue::Text("A".to_string()),
                },
            ],
            default_value: Some(MappedValue::Text("N/A".to_string())),
        };

        let processor = AttributeRangeMapper { params };

        // Test with string input that can be parsed to number
        let mut feature = Feature::new_with_attributes(Attributes::new());
        feature.insert("score", AttributeValue::String("75".to_string()));
        assert_eq!(
            map_feature(&processor, &feature).get("grade"),
            Some(&AttributeValue::String("C".to_string()))
        );

        // Test with non-numeric string (should use default)
        let mut feature = Feature::new_with_attributes(Attributes::new());
        feature.insert("score", AttributeValue::String("invalid".to_string()));
        assert_eq!(
            map_feature(&processor, &feature).get("grade"),
            Some(&AttributeValue::String("N/A".to_string()))
        );
    }

    /// The description has always claimed booleans count as 1 and 0, but the old test helper
    /// reimplemented the coercion and omitted that arm, so it was never exercised.
    #[test]
    fn test_range_mapper_boolean_input() {
        let processor = AttributeRangeMapper {
            params: AttributeRangeMapperParam {
                input_attribute: "flag".to_string(),
                output_attribute: "label".to_string(),
                range_table: vec![
                    RangeEntry {
                        from: 0.0,
                        to: 1.0,
                        output_value: MappedValue::Text("false".to_string()),
                    },
                    RangeEntry {
                        from: 1.0,
                        to: 2.0,
                        output_value: MappedValue::Text("true".to_string()),
                    },
                ],
                default_value: None,
            },
        };

        let mut feature = Feature::new_with_attributes(Attributes::new());
        feature.insert("flag", AttributeValue::Bool(true));
        assert_eq!(
            map_feature(&processor, &feature).get("label"),
            Some(&AttributeValue::String("true".to_string()))
        );

        let mut feature = Feature::new_with_attributes(Attributes::new());
        feature.insert("flag", AttributeValue::Bool(false));
        assert_eq!(
            map_feature(&processor, &feature).get("label"),
            Some(&AttributeValue::String("false".to_string()))
        );
    }

    /// A number written as a number, not coerced to text.
    #[test]
    fn test_range_mapper_writes_a_number_as_a_number() {
        let processor = AttributeRangeMapper {
            params: AttributeRangeMapperParam {
                input_attribute: "score".to_string(),
                output_attribute: "band".to_string(),
                range_table: vec![RangeEntry {
                    from: 0.0,
                    to: 10.0,
                    output_value: MappedValue::Number(Number::from(1)),
                }],
                default_value: Some(MappedValue::Boolean(false)),
            },
        };

        let mut feature = Feature::new_with_attributes(Attributes::new());
        feature.insert("score", AttributeValue::Number(Number::from(5)));
        assert_eq!(
            map_feature(&processor, &feature).get("band"),
            Some(&AttributeValue::Number(Number::from(1)))
        );

        // Nothing matches, so the default applies — and keeps its own type.
        let mut feature = Feature::new_with_attributes(Attributes::new());
        feature.insert("score", AttributeValue::Number(Number::from(99)));
        assert_eq!(
            map_feature(&processor, &feature).get("band"),
            Some(&AttributeValue::Bool(false))
        );
    }

    /// Runs the processor's own classification over `feature`, so the tests exercise the
    /// same code path `process` does rather than a copy of it.
    fn map_feature(processor: &AttributeRangeMapper, feature: &Feature) -> Feature {
        let mut result = feature.clone();
        if let Some(value) = processor.mapped_value(feature) {
            result.insert(processor.params.output_attribute.clone(), value);
        }
        result
    }
}
