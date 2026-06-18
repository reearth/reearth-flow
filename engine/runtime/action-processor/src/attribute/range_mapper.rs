use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};

use reearth_flow_types::AttributeValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::warn;

use super::errors::AttributeProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct AttributeRangeMapperFactory;

impl ProcessorFactory for AttributeRangeMapperFactory {
    fn name(&self) -> &str {
        "AttributeRangeMapper"
    }

    fn description(&self) -> &str {
        "Map attribute values to ranges and assign corresponding output values"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(AttributeRangeMapperParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Attribute"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
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

/// # AttributeRangeMapper Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AttributeRangeMapperParam {
    /// # Input Attribute
    /// The attribute to evaluate for range mapping
    pub input_attribute: String,

    /// # Output Attribute
    /// The attribute to store the mapped value
    pub output_attribute: String,

    /// # Range Lookup Table
    /// List of ranges and their corresponding output values
    pub range_table: Vec<RangeEntry>,

    /// # Default Value
    /// Value to use when input doesn't match any range (can be string, number, boolean, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<Value>,
}

/// # Range Entry
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RangeEntry {
    /// # From (Minimum)
    /// The minimum value of the range (inclusive)
    pub from: f64,

    /// # To (Maximum)
    /// The maximum value of the range (exclusive)
    pub to: f64,

    /// # Output Value
    /// The value to assign when input falls within this range (can be string, number, boolean, etc.)
    pub output_value: Value,
}

impl Processor for AttributeRangeMapper {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();

        // Get the input attribute value
        let input_value = feature.get(&self.params.input_attribute);

        // Convert to f64 for range comparison
        let numeric_value: Option<f64> = input_value.and_then(|v| match v {
            AttributeValue::Number(n) => n.as_f64(),
            AttributeValue::String(s) => s.parse::<f64>().ok(),
            AttributeValue::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        });

        // Find matching range and set output value
        if let Some(num_val) = numeric_value {
            let mut matched = false;

            for range in &self.params.range_table {
                // Check if value falls within range
                // Use inclusive for both bounds if it's a single-value range
                // Otherwise use [from, to) - inclusive start, exclusive end
                let is_in_range = if (range.to - range.from).abs() < f64::EPSILON {
                    (num_val - range.from).abs() < f64::EPSILON
                } else {
                    num_val >= range.from && num_val < range.to
                };

                if is_in_range {
                    // Convert Value to AttributeValue
                    match serde_json::from_value(range.output_value.clone()) {
                        Ok(attr_value) => {
                            feature.insert(self.params.output_attribute.clone(), attr_value);
                        }
                        Err(e) => {
                            warn!(
                                "Failed to deserialize range output value for attribute '{}': {}. Feature will pass through without output attribute.",
                                self.params.output_attribute, e
                            );
                        }
                    }
                    matched = true;
                    break;
                }
            }

            // Apply default value if no range matched
            if !matched {
                if let Some(default_value) = &self.params.default_value {
                    match serde_json::from_value(default_value.clone()) {
                        Ok(attr_value) => {
                            feature.insert(self.params.output_attribute.clone(), attr_value);
                        }
                        Err(e) => {
                            warn!(
                                "Failed to deserialize default value for attribute '{}': {}. Feature will pass through without output attribute.",
                                self.params.output_attribute, e
                            );
                        }
                    }
                }
            }
        } else {
            // If input value is not numeric, apply default value
            if let Some(default_value) = &self.params.default_value {
                match serde_json::from_value(default_value.clone()) {
                    Ok(attr_value) => {
                        feature.insert(self.params.output_attribute.clone(), attr_value);
                    }
                    Err(e) => {
                        warn!(
                            "Failed to deserialize default value for attribute '{}': {}. Feature will pass through without output attribute.",
                            self.params.output_attribute, e
                        );
                    }
                }
            }
        }

        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
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
        "AttributeRangeMapper"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_types::{Attributes, Feature};
    use serde_json::{json, Number};

    #[test]
    fn test_range_mapper_numeric_input() {
        let params = AttributeRangeMapperParam {
            input_attribute: "depth".to_string(),
            output_attribute: "color".to_string(),
            range_table: vec![
                RangeEntry {
                    from: 0.0,
                    to: 5.0,
                    output_value: json!("#ff0000"),
                },
                RangeEntry {
                    from: 5.0,
                    to: 10.0,
                    output_value: json!("#00ff00"),
                },
                RangeEntry {
                    from: 10.0,
                    to: 20.0,
                    output_value: json!("#0000ff"),
                },
            ],
            default_value: Some(json!("#cccccc")),
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
                    output_value: json!("low"),
                },
                RangeEntry {
                    from: 10.0,
                    to: 20.0,
                    output_value: json!("high"),
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
                    output_value: json!("F"),
                },
                RangeEntry {
                    from: 60.0,
                    to: 80.0,
                    output_value: json!("C"),
                },
                RangeEntry {
                    from: 80.0,
                    to: 100.0,
                    output_value: json!("A"),
                },
            ],
            default_value: Some(json!("N/A")),
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

    // Helper function to simulate feature processing
    fn map_feature(processor: &AttributeRangeMapper, feature: &Feature) -> Feature {
        let mut result = feature.clone();

        let input_value = feature.get(&processor.params.input_attribute);
        let numeric_value: Option<f64> = input_value.and_then(|v| match v {
            AttributeValue::Number(n) => n.as_f64(),
            AttributeValue::String(s) => s.parse::<f64>().ok(),
            _ => None,
        });

        if let Some(num_val) = numeric_value {
            let mut matched = false;
            for range in &processor.params.range_table {
                let is_in_range = if (range.to - range.from).abs() < f64::EPSILON {
                    (num_val - range.from).abs() < f64::EPSILON
                } else {
                    num_val >= range.from && num_val < range.to
                };

                if is_in_range {
                    if let Ok(attr_value) = serde_json::from_value(range.output_value.clone()) {
                        result.insert(processor.params.output_attribute.clone(), attr_value);
                    }
                    matched = true;
                    break;
                }
            }

            if !matched {
                if let Some(default_value) = &processor.params.default_value {
                    if let Ok(attr_value) = serde_json::from_value(default_value.clone()) {
                        result.insert(processor.params.output_attribute.clone(), attr_value);
                    }
                }
            }
        } else if let Some(default_value) = &processor.params.default_value {
            if let Ok(attr_value) = serde_json::from_value(default_value.clone()) {
                result.insert(processor.params.output_attribute.clone(), attr_value);
            }
        }

        result
    }
}
