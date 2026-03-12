use std::collections::HashMap;

use chrono::{Datelike, FixedOffset, NaiveDate};
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

/// Internal enum representing the parsed datetime value.
/// Follows the "correct typing" principle:
/// - NaiveDate: for date-only values (no timezone)
/// - DateTime<Utc>: for absolute timestamps or when timezone is unknown
/// - DateTime<FixedOffset>: for datetime strings that include timezone info
#[derive(Debug, Clone)]
enum DateTimeValue {
    NaiveDate(NaiveDate),
    Utc(chrono::DateTime<chrono::Utc>),
    FixedOffset(chrono::DateTime<FixedOffset>),
}

impl DateTimeValue {
    /// Convert to DateTime<Utc> for formatting to formats that require UTC
    fn to_utc(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            DateTimeValue::NaiveDate(d) => {
                chrono::DateTime::from_naive_utc_and_offset(d.and_hms_opt(0, 0, 0).unwrap(), chrono::Utc)
            }
            DateTimeValue::Utc(dt) => *dt,
            DateTimeValue::FixedOffset(dt) => dt.with_timezone(&chrono::Utc),
        }
    }
}

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
        feature.insert(output_attr, output_value);

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
) -> Result<DateTimeValue, String> {
    // Extract string and/or numeric values based on input type
    match value {
        AttributeValue::String(s) => {
            let i64_val = s.parse::<i64>().ok();
            parse_from_string_and_number(Some(s), i64_val, None, format)
        }
        AttributeValue::Number(n) => {
            let s = n.to_string();
            // Try integer first, then float (truncating to handle values like 1700000000.0)
            let i64_val = n.as_i64();
            let f64_val = n.as_f64();
            parse_from_string_and_number(Some(&s), i64_val, f64_val, format)
        }
        _ => Err("Unsupported value type for datetime conversion".to_string()),
    }
}

fn parse_from_string_and_number(
    str_val: Option<&str>,
    i64_val: Option<i64>,
    f64_val: Option<f64>,
    format: &DateTimeInputFormat,
) -> Result<DateTimeValue, String> {
    // Get effective i64 value: use i64_val if available, otherwise truncate f64_val
    let effective_i64 = i64_val.or_else(|| f64_val.map(|f| f.trunc() as i64));

    match format {
        DateTimeInputFormat::Auto => {
            // Try numeric formats first (Unix timestamps) - handles numeric strings like "1609459200"
            // For auto-detection, numerical values always default to Unix seconds (not milliseconds)
            // to avoid misinterpreting historical data near 1970
            if let Some(n) = effective_i64 {
                // Try seconds first, but validate the result is reasonable (between 1970 and 2100)
                // to avoid misinterpreting millisecond timestamps as seconds
                if let Ok(dt) = reearth_flow_common::datetime::try_from_unix_s(n) {
                    let year = dt.year();
                    if (1970..=2100).contains(&year) {
                        return Ok(DateTimeValue::Utc(dt));
                    }
                }
                // Fallback: try milliseconds
                if let Ok(dt) = reearth_flow_common::datetime::try_from_unix_ms(n) {
                    return Ok(DateTimeValue::Utc(dt));
                }
            }
            // Try string formats (RFC3339, YYYY-MM-DD, etc.)
            if let Some(s) = str_val {
                // Try RFC3339 first - may include timezone info
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                    return Ok(DateTimeValue::FixedOffset(dt));
                }
                // Try other formats that result in Utc
                if let Ok(dt) = reearth_flow_common::datetime::try_from(s) {
                    return Ok(DateTimeValue::Utc(dt));
                }
                // Try date-only format
                if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                    return Ok(DateTimeValue::NaiveDate(d));
                }
            }
            Err("Could not auto-detect datetime format".to_string())
        }
        DateTimeInputFormat::Rfc3339 => {
            let s = str_val.ok_or("Expected string value for RFC3339")?;
            // RFC3339 may include timezone - parse as FixedOffset to preserve it
            chrono::DateTime::parse_from_rfc3339(s)
                .map(DateTimeValue::FixedOffset)
                .map_err(|e| format!("Failed to parse RFC3339: {e}"))
        }
        DateTimeInputFormat::UnixS => {
            let n = effective_i64.ok_or("Expected numeric value for Unix timestamp")?;
            reearth_flow_common::datetime::try_from_unix_s(n)
                .map(DateTimeValue::Utc)
                .map_err(|e| format!("Failed to parse Unix timestamp (seconds): {e}"))
        }
        DateTimeInputFormat::UnixMs => {
            let n = effective_i64.ok_or("Expected numeric value for Unix timestamp")?;
            reearth_flow_common::datetime::try_from_unix_ms(n)
                .map(DateTimeValue::Utc)
                .map_err(|e| format!("Failed to parse Unix timestamp (milliseconds): {e}"))
        }
        DateTimeInputFormat::Date => {
            let s = str_val.ok_or("Expected string value for Date format")?;
            // Parse YYYY-MM-DD format - store as NaiveDate (no timezone)
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map(DateTimeValue::NaiveDate)
                .map_err(|e| format!("Failed to parse Date: {e}"))
        }
        DateTimeInputFormat::Custom(fmt) => {
            let s = str_val.ok_or("Expected string value for custom format")?;
            // Custom formats without timezone info are parsed as NaiveDateTime, stored as Utc
            let naive_dt = chrono::NaiveDateTime::parse_from_str(s, fmt)
                .map_err(|e| format!("Failed to parse custom format: {e}"))?;
            Ok(DateTimeValue::Utc(chrono::DateTime::from_naive_utc_and_offset(
                naive_dt,
                chrono::Utc,
            )))
        }
    }
}

fn format_datetime(dt: &DateTimeValue, format: &DateTimeOutputFormat) -> AttributeValue {
    match format {
        DateTimeOutputFormat::Rfc3339 => {
            // Output RFC3339 - preserve timezone info if available
            match dt {
                DateTimeValue::FixedOffset(dt) => AttributeValue::String(dt.to_rfc3339()),
                _ => AttributeValue::String(reearth_flow_common::datetime::to_rfc3339(&dt.to_utc())),
            }
        }
        DateTimeOutputFormat::UnixS => {
            // Unix timestamps are always UTC
            let ts = dt.to_utc().timestamp();
            AttributeValue::Number(serde_json::Number::from(ts))
        }
        DateTimeOutputFormat::UnixMs => {
            // Unix timestamps are always UTC
            let ts = dt.to_utc().timestamp_millis();
            AttributeValue::Number(serde_json::Number::from(ts))
        }
        DateTimeOutputFormat::Date => {
            // Date-only output
            match dt {
                DateTimeValue::NaiveDate(d) => AttributeValue::String(d.format("%Y-%m-%d").to_string()),
                DateTimeValue::Utc(dt) => AttributeValue::String(dt.format("%Y-%m-%d").to_string()),
                DateTimeValue::FixedOffset(dt) => AttributeValue::String(dt.format("%Y-%m-%d").to_string()),
            }
        }
        DateTimeOutputFormat::Custom(fmt) => {
            // Custom format - use appropriate formatter based on type
            match dt {
                DateTimeValue::NaiveDate(d) => {
                    // Format as midnight of that date for custom datetime formats
                    let naive_dt = d.and_hms_opt(0, 0, 0).unwrap();
                    AttributeValue::String(naive_dt.format(fmt).to_string())
                }
                DateTimeValue::Utc(dt) => {
                    AttributeValue::String(dt.format(fmt).to_string())
                }
                DateTimeValue::FixedOffset(dt) => {
                    AttributeValue::String(dt.format(fmt).to_string())
                }
            }
        }
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

        assert_eq!(result, AttributeValue::Number(1609459200i64.into()));
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

        assert_eq!(result, AttributeValue::String("2021-01-01T00:00:00+00:00".to_string()));
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

        assert_eq!(result, AttributeValue::String("2021-01-01T00:00:00+00:00".to_string()));
    }

    #[test]
    fn test_parse_date_to_rfc3339() {
        let feature = create_test_feature("date", AttributeValue::String("2021-01-01".to_string()));

        let input_value = feature.get("date").unwrap();
        let dt = parse_datetime(input_value, &DateTimeInputFormat::Date).unwrap();
        let result = format_datetime(&dt, &DateTimeOutputFormat::Rfc3339);

        assert_eq!(result, AttributeValue::String("2021-01-01T00:00:00+00:00".to_string()));
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

        assert_eq!(result, AttributeValue::String("2021-01-01T12:30:00+00:00".to_string()));
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

        assert_eq!(result, AttributeValue::Number(1609459200i64.into()));
    }

    #[test]
    fn test_auto_date_only() {
        let feature = create_test_feature("date", AttributeValue::String("2021-01-01".to_string()));

        let input_value = feature.get("date").unwrap();
        let dt = parse_datetime(input_value, &DateTimeInputFormat::Auto).unwrap();
        let result = format_datetime(&dt, &DateTimeOutputFormat::Rfc3339);

        assert_eq!(result, AttributeValue::String("2021-01-01T00:00:00+00:00".to_string()));
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

        assert_eq!(result, AttributeValue::String("2021-01-15".to_string()));
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

        assert_eq!(result, AttributeValue::String("01/01/2021 00:00".to_string()));
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

        assert_eq!(result, AttributeValue::String("2021-01-01T00:00:00+00:00".to_string()));
    }

    #[test]
    fn test_float_unix_timestamp() {
        // Test with float input (e.g., from schema processing that produces 1700000000.0)
        use serde_json::Number;
        let float_value: f64 = 1609459200.0;
        let feature = create_test_feature(
            "timestamp",
            AttributeValue::Number(Number::from_f64(float_value).unwrap()),
        );

        let input_value = feature.get("timestamp").unwrap();
        let dt = parse_datetime(input_value, &DateTimeInputFormat::UnixS).unwrap();
        let result = format_datetime(&dt, &DateTimeOutputFormat::Rfc3339);

        assert_eq!(result, AttributeValue::String("2021-01-01T00:00:00+00:00".to_string()));
    }

    #[test]
    fn test_auto_unix_timestamp_string() {
        // Test auto-detect with Unix timestamp as string (improved behavior)
        let feature = create_test_feature(
            "timestamp",
            AttributeValue::String("1609459200".to_string()),
        );

        let input_value = feature.get("timestamp").unwrap();
        let dt = parse_datetime(input_value, &DateTimeInputFormat::Auto).unwrap();
        let result = format_datetime(&dt, &DateTimeOutputFormat::Rfc3339);

        assert_eq!(result, AttributeValue::String("2021-01-01T00:00:00+00:00".to_string()));
    }

    #[test]
    fn test_auto_unix_ms_timestamp_string() {
        // Test auto-detect with Unix timestamp in milliseconds as string
        // Note: With the updated logic, auto-detection defaults to seconds first,
        // so a 13-digit value like 1609459200000 will be parsed as seconds (invalid/far future)
        // then fall back to milliseconds. For reliable ms parsing, use UnixMs input format.
        let feature = create_test_feature(
            "timestamp",
            AttributeValue::String("1609459200000".to_string()),
        );

        let input_value = feature.get("timestamp").unwrap();
        let dt = parse_datetime(input_value, &DateTimeInputFormat::Auto).unwrap();
        let result = format_datetime(&dt, &DateTimeOutputFormat::Rfc3339);

        assert_eq!(result, AttributeValue::String("2021-01-01T00:00:00+00:00".to_string()));
    }

    #[test]
    fn test_rfc3339_with_timezone_preserved() {
        // Test that RFC3339 strings with timezone info preserve the timezone
        let feature = create_test_feature(
            "timestamp",
            AttributeValue::String("2021-01-01T12:00:00+05:30".to_string()),
        );

        let input_value = feature.get("timestamp").unwrap();
        let dt = parse_datetime(input_value, &DateTimeInputFormat::Rfc3339).unwrap();
        
        // Verify it's stored as FixedOffset
        match dt {
            DateTimeValue::FixedOffset(dt) => {
                assert_eq!(dt.offset().local_minus_utc(), 5 * 3600 + 30 * 60); // +05:30
            }
            _ => panic!("Expected FixedOffset, got {:?}", dt),
        }
        
        // Output as RFC3339 should preserve the original timezone
        let result = format_datetime(&dt, &DateTimeOutputFormat::Rfc3339);
        assert_eq!(result, AttributeValue::String("2021-01-01T12:00:00+05:30".to_string()));
    }

    #[test]
    fn test_rfc3339_utc_normalized_for_unix() {
        // Test that RFC3339 with timezone gets normalized to UTC for unix output
        let feature = create_test_feature(
            "timestamp",
            AttributeValue::String("2021-01-01T00:00:00+05:30".to_string()),
        );

        let input_value = feature.get("timestamp").unwrap();
        let dt = parse_datetime(input_value, &DateTimeInputFormat::Rfc3339).unwrap();
        
        // Output as UnixS should normalize to UTC
        let result = format_datetime(&dt, &DateTimeOutputFormat::UnixS);
        // 2021-01-01T00:00:00+05:30 = 2020-12-31T18:30:00Z = 1609439400
        assert_eq!(result, AttributeValue::Number(1609439400i64.into()));
    }

    #[test]
    fn test_date_only_preserved_as_naive() {
        // Test that date-only input is stored as NaiveDate
        let feature = create_test_feature(
            "date",
            AttributeValue::String("2021-01-15".to_string()),
        );

        let input_value = feature.get("date").unwrap();
        let dt = parse_datetime(input_value, &DateTimeInputFormat::Date).unwrap();
        
        // Verify it's stored as NaiveDate
        match dt {
            DateTimeValue::NaiveDate(d) => {
                assert_eq!(d.year(), 2021);
                assert_eq!(d.month(), 1);
                assert_eq!(d.day(), 15);
            }
            _ => panic!("Expected NaiveDate, got {:?}", dt),
        }
        
        // Output as Date should preserve the date
        let result = format_datetime(&dt, &DateTimeOutputFormat::Date);
        assert_eq!(result, AttributeValue::String("2021-01-15".to_string()));
    }

    #[test]
    fn test_output_to_different_attribute() {
        // Test that outputAttribute writes to a different attribute (leaves input untouched)
        use crate::tests::utils::create_default_execute_context;
        use reearth_flow_runtime::forwarder::NoopChannelForwarder;

        let mut feature = Feature::new_with_attributes(Attributes::new());
        feature.insert(
            "createdAt".to_string(),
            AttributeValue::String("2021-01-01T00:00:00Z".to_string()),
        );

        let params = DateTimeConverterParam {
            attribute: "createdAt".to_string(),
            input_format: DateTimeInputFormat::Rfc3339,
            output_format: DateTimeOutputFormat::UnixS,
            output_attribute: Some("createdAtTimestamp".to_string()),
        };
        let mut processor = DateTimeConverter { params };

        // Use NoopChannelForwarder to capture outputs
        let noop = NoopChannelForwarder::default();
        let fw = ProcessorChannelForwarder::Noop(noop.clone());

        // Create context using the helper
        let ctx = create_default_execute_context(&feature);
        processor.process(ctx, &fw).unwrap();

        // Check the output feature
        let features = noop.send_features.lock().unwrap();
        assert_eq!(features.len(), 1);
        let output_feature = &features[0];

        // Original attribute should be unchanged
        assert_eq!(
            output_feature.get("createdAt"),
            Some(&AttributeValue::String("2021-01-01T00:00:00Z".to_string()))
        );

        // New attribute should have the converted value as a number
        assert_eq!(
            output_feature.get("createdAtTimestamp"),
            Some(&AttributeValue::Number(1609459200i64.into()))
        );
    }
}
