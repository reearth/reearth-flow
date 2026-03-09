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

use super::errors::AttributeProcessorError;

const FAILED_PORT: &str = "failed";

#[derive(Debug, Clone, Default)]
pub(super) struct DateTimeConverterFactory;

impl ProcessorFactory for DateTimeConverterFactory {
    fn name(&self) -> &str {
        "DateTimeConverter"
    }

    fn description(&self) -> &str {
        "Convert datetime values between different formats"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(DateTimeConverterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Attribute"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), Port::new(FAILED_PORT)]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: DateTimeConverterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                AttributeProcessorError::DateTimeConverterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                AttributeProcessorError::DateTimeConverterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(AttributeProcessorError::DateTimeConverterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let processor = DateTimeConverter { params };
        Ok(Box::new(processor))
    }
}

#[derive(Debug, Clone)]
struct DateTimeConverter {
    params: DateTimeConverterParam,
}

/// # DateTimeConverter Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DateTimeConverterParam {
    /// Attribute containing the datetime value to convert
    pub attribute: String,
    /// Format of the input value (default: auto)
    #[serde(default)]
    pub input_format: DateTimeInputFormat,
    /// Desired output format
    pub output_format: DateTimeOutputFormat,
    /// Write result to a different attribute (leave input untouched)
    /// Defaults to the same as `attribute`
    #[serde(default)]
    pub output_attribute: Option<String>,
}

/// Input format options for DateTimeConverter
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum DateTimeInputFormat {
    /// Auto-detect from known formats
    #[default]
    Auto,
    /// RFC3339 / ISO 8601 format
    Rfc3339,
    /// Unix timestamp in seconds
    UnixS,
    /// Unix timestamp in milliseconds
    UnixMs,
    /// Date only format (YYYY-MM-DD)
    Date,
    /// Custom format using chrono format specifiers
    Custom(String),
}

/// Output format options for DateTimeConverter
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DateTimeOutputFormat {
    /// RFC3339 / ISO 8601 format
    Rfc3339,
    /// Unix timestamp in seconds
    UnixS,
    /// Unix timestamp in milliseconds
    UnixMs,
    /// Date only format (YYYY-MM-DD)
    Date,
    /// Custom format using chrono format specifiers
    Custom(String),
}

impl Processor for DateTimeConverter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        let output_attr = self
            .params
            .output_attribute
            .as_ref()
            .unwrap_or(&self.params.attribute)
            .clone();

        // Get the input value
        let input_value = match feature.get(&self.params.attribute) {
            Some(v) => v,
            None => {
                // Attribute not found, send to failed port
                fw.send(ctx.new_with_feature_and_port(feature, Port::new(FAILED_PORT)));
                return Ok(());
            }
        };

        // Parse the datetime
        let datetime = match parse_datetime(input_value, &self.params.input_format) {
            Ok(dt) => dt,
            Err(_) => {
                // Parse failed, send to failed port
                fw.send(ctx.new_with_feature_and_port(feature, Port::new(FAILED_PORT)));
                return Ok(());
            }
        };

        // Format the output
        let output_value = format_datetime(&datetime, &self.params.output_format);

        // Write to output attribute
        feature.insert(output_attr, AttributeValue::String(output_value));

        // Send to output port
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
        "DateTimeConverter"
    }
}

fn parse_datetime(
    value: &AttributeValue,
    format: &DateTimeInputFormat,
) -> Result<chrono::DateTime<chrono::Utc>, String> {
    // Extract string and/or numeric values based on input type
    match value {
        AttributeValue::String(s) => {
            let i64_val = s.parse::<i64>().ok();
            parse_from_string_and_number(Some(s), i64_val, format)
        }
        AttributeValue::Number(n) => {
            let s = n.to_string();
            parse_from_string_and_number(Some(&s), n.as_i64(), format)
        }
        _ => Err("Unsupported value type for datetime conversion".to_string()),
    }
}

fn parse_from_string_and_number(
    str_val: Option<&str>,
    i64_val: Option<i64>,
    format: &DateTimeInputFormat,
) -> Result<chrono::DateTime<chrono::Utc>, String> {
    match format {
        DateTimeInputFormat::Auto => {
            // Try string formats first
            if let Some(s) = str_val {
                if let Ok(dt) = reearth_flow_common::datetime::try_from(s) {
                    return Ok(dt);
                }
            }
            // Try numeric formats (Unix timestamps)
            if let Some(n) = i64_val {
                // Try seconds first (smaller values), then milliseconds
                if n < 1_000_000_000_000 {
                    if let Ok(dt) = reearth_flow_common::datetime::try_from_unix_s(n) {
                        return Ok(dt);
                    }
                } else if let Ok(dt) = reearth_flow_common::datetime::try_from_unix_ms(n) {
                    return Ok(dt);
                }
            }
            Err("Could not auto-detect datetime format".to_string())
        }
        DateTimeInputFormat::Rfc3339 => {
            let s = str_val.ok_or("Expected string value for RFC3339")?;
            reearth_flow_common::datetime::try_from(s)
                .map_err(|e| format!("Failed to parse RFC3339: {e}"))
        }
        DateTimeInputFormat::UnixS => {
            let n = i64_val.ok_or("Expected numeric value for Unix timestamp")?;
            reearth_flow_common::datetime::try_from_unix_s(n)
                .map_err(|e| format!("Failed to parse Unix timestamp (seconds): {e}"))
        }
        DateTimeInputFormat::UnixMs => {
            let n = i64_val.ok_or("Expected numeric value for Unix timestamp")?;
            reearth_flow_common::datetime::try_from_unix_ms(n)
                .map_err(|e| format!("Failed to parse Unix timestamp (milliseconds): {e}"))
        }
        DateTimeInputFormat::Date => {
            let s = str_val.ok_or("Expected string value for Date format")?;
            // Parse YYYY-MM-DD format
            let naive_date = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map_err(|e| format!("Failed to parse Date: {e}"))?;
            let naive_dt = naive_date
                .and_hms_opt(0, 0, 0)
                .ok_or("Invalid time for date")?;
            Ok(chrono::DateTime::from_naive_utc_and_offset(
                naive_dt,
                chrono::Utc,
            ))
        }
        DateTimeInputFormat::Custom(fmt) => {
            let s = str_val.ok_or("Expected string value for custom format")?;
            let naive_dt = chrono::NaiveDateTime::parse_from_str(s, fmt)
                .map_err(|e| format!("Failed to parse custom format: {e}"))?;
            Ok(chrono::DateTime::from_naive_utc_and_offset(
                naive_dt,
                chrono::Utc,
            ))
        }
    }
}

fn format_datetime(dt: &chrono::DateTime<chrono::Utc>, format: &DateTimeOutputFormat) -> String {
    match format {
        DateTimeOutputFormat::Rfc3339 => reearth_flow_common::datetime::to_rfc3339(dt),
        DateTimeOutputFormat::UnixS => reearth_flow_common::datetime::to_unix_s(dt).to_string(),
        DateTimeOutputFormat::UnixMs => reearth_flow_common::datetime::to_unix_ms(dt).to_string(),
        DateTimeOutputFormat::Date => reearth_flow_common::datetime::to_date_string(dt),
        DateTimeOutputFormat::Custom(fmt) => reearth_flow_common::datetime::format_with(dt, fmt),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_types::{Attributes, Feature};

    fn create_test_feature(attr_name: &str, value: AttributeValue) -> Feature {
        let mut feature = Feature::new_with_attributes(Attributes::new());
        feature.insert(attr_name.to_string(), value);
        feature
    }

    #[test]
    fn test_parse_rfc3339_to_unix_s() {
        let feature = create_test_feature(
            "timestamp",
            AttributeValue::String("2021-01-01T00:00:00Z".to_string()),
        );

        let input_value = feature.get("timestamp").unwrap();
        let dt = parse_datetime(input_value, &DateTimeInputFormat::Rfc3339).unwrap();
        let result = format_datetime(&dt, &DateTimeOutputFormat::UnixS);

        assert_eq!(result, "1609459200");
    }

    #[test]
    fn test_parse_unix_s_to_rfc3339() {
        let feature = create_test_feature(
            "timestamp",
            AttributeValue::String("1609459200".to_string()),
        );

        let input_value = feature.get("timestamp").unwrap();
        let dt = parse_datetime(input_value, &DateTimeInputFormat::UnixS).unwrap();
        let result = format_datetime(&dt, &DateTimeOutputFormat::Rfc3339);

        assert_eq!(result, "2021-01-01T00:00:00+00:00");
    }

    #[test]
    fn test_parse_unix_ms_to_rfc3339() {
        let feature = create_test_feature(
            "timestamp",
            AttributeValue::String("1609459200000".to_string()),
        );

        let input_value = feature.get("timestamp").unwrap();
        let dt = parse_datetime(input_value, &DateTimeInputFormat::UnixMs).unwrap();
        let result = format_datetime(&dt, &DateTimeOutputFormat::Rfc3339);

        assert_eq!(result, "2021-01-01T00:00:00+00:00");
    }

    #[test]
    fn test_parse_date_to_rfc3339() {
        let feature = create_test_feature("date", AttributeValue::String("2021-01-01".to_string()));

        let input_value = feature.get("date").unwrap();
        let dt = parse_datetime(input_value, &DateTimeInputFormat::Date).unwrap();
        let result = format_datetime(&dt, &DateTimeOutputFormat::Rfc3339);

        assert_eq!(result, "2021-01-01T00:00:00+00:00");
    }

    #[test]
    fn test_custom_format() {
        let feature = create_test_feature(
            "date",
            AttributeValue::String("01/01/2021 12:30".to_string()),
        );

        let input_value = feature.get("date").unwrap();
        let dt = parse_datetime(
            input_value,
            &DateTimeInputFormat::Custom("%d/%m/%Y %H:%M".to_string()),
        )
        .unwrap();
        let result = format_datetime(&dt, &DateTimeOutputFormat::Rfc3339);

        assert_eq!(result, "2021-01-01T12:30:00+00:00");
    }

    #[test]
    fn test_auto_rfc3339() {
        let feature = create_test_feature(
            "timestamp",
            AttributeValue::String("2021-01-01T00:00:00Z".to_string()),
        );

        let input_value = feature.get("timestamp").unwrap();
        let dt = parse_datetime(input_value, &DateTimeInputFormat::Auto).unwrap();
        let result = format_datetime(&dt, &DateTimeOutputFormat::UnixS);

        assert_eq!(result, "1609459200");
    }

    #[test]
    fn test_auto_date_only() {
        let feature = create_test_feature("date", AttributeValue::String("2021-01-01".to_string()));

        let input_value = feature.get("date").unwrap();
        let dt = parse_datetime(input_value, &DateTimeInputFormat::Auto).unwrap();
        let result = format_datetime(&dt, &DateTimeOutputFormat::Rfc3339);

        assert_eq!(result, "2021-01-01T00:00:00+00:00");
    }

    #[test]
    fn test_rfc3339_to_date() {
        let feature = create_test_feature(
            "timestamp",
            AttributeValue::String("2021-01-15T12:30:45Z".to_string()),
        );

        let input_value = feature.get("timestamp").unwrap();
        let dt = parse_datetime(input_value, &DateTimeInputFormat::Rfc3339).unwrap();
        let result = format_datetime(&dt, &DateTimeOutputFormat::Date);

        assert_eq!(result, "2021-01-15");
    }

    #[test]
    fn test_rfc3339_to_custom_format() {
        let feature = create_test_feature(
            "timestamp",
            AttributeValue::String("2021-01-01T00:00:00Z".to_string()),
        );

        let input_value = feature.get("timestamp").unwrap();
        let dt = parse_datetime(input_value, &DateTimeInputFormat::Rfc3339).unwrap();
        let result = format_datetime(
            &dt,
            &DateTimeOutputFormat::Custom("%d/%m/%Y %H:%M".to_string()),
        );

        assert_eq!(result, "01/01/2021 00:00");
    }

    #[test]
    fn test_parse_failure() {
        let feature = create_test_feature(
            "timestamp",
            AttributeValue::String("invalid-datetime".to_string()),
        );

        let input_value = feature.get("timestamp").unwrap();
        let result = parse_datetime(input_value, &DateTimeInputFormat::Rfc3339);

        assert!(result.is_err());
    }

    #[test]
    fn test_numeric_unix_timestamp() {
        // Test with numeric (integer) input
        use serde_json::Number;
        let feature = create_test_feature(
            "timestamp",
            AttributeValue::Number(Number::from(1609459200i64)),
        );

        let input_value = feature.get("timestamp").unwrap();
        let dt = parse_datetime(input_value, &DateTimeInputFormat::UnixS).unwrap();
        let result = format_datetime(&dt, &DateTimeOutputFormat::Rfc3339);

        assert_eq!(result, "2021-01-01T00:00:00+00:00");
    }
}
