use std::{collections::HashMap, str::FromStr, sync::Arc};

use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Attributes, Expr, Feature};
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Number, Value};

use super::errors::PlateauProcessorError;

/// Output port for gml:name code space validation errors
static GML_NAME_ERRORS_PORT: once_cell::sync::Lazy<Port> =
    once_cell::sync::Lazy::new(|| Port::new("gmlNameErrors"));

/// Output port for per-file statistics
static STATS_PORT: once_cell::sync::Lazy<Port> = once_cell::sync::Lazy::new(|| Port::new("stats"));

/// # GmlNameCodeSpaceValidator Parameters
///
/// Configuration for validating gml:name elements to ensure they have codeSpace attributes.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct GmlNameCodeSpaceValidatorParam {
    /// Expression to get the path to the CityGML file
    #[serde(skip_serializing_if = "Option::is_none")]
    city_gml_path: Option<Expr>,
}

#[derive(Debug, Clone, Default)]
pub struct GmlNameCodeSpaceValidatorFactory;

impl ProcessorFactory for GmlNameCodeSpaceValidatorFactory {
    fn name(&self) -> &str {
        "PLATEAU4.GmlNameCodeSpaceValidator"
    }

    fn description(&self) -> &str {
        "Validates that gml:name elements have codeSpace attributes (coded values)"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GmlNameCodeSpaceValidatorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            DEFAULT_PORT.clone(),
            GML_NAME_ERRORS_PORT.clone(),
            STATS_PORT.clone(),
        ]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let global_params = with.clone();
        let params: GmlNameCodeSpaceValidatorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::GmlNameCodeSpaceValidator(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::GmlNameCodeSpaceValidator(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            GmlNameCodeSpaceValidatorParam::default()
        };

        let expr_engine = Arc::clone(&ctx.expr_engine);
        let city_gml_path_expr = if let Some(expr) = params.city_gml_path {
            Some(expr_engine.compile(expr.as_ref()).map_err(|e| {
                PlateauProcessorError::GmlNameCodeSpaceValidator(format!(
                    "Failed to compile cityGmlPath expression: {e}"
                ))
            })?)
        } else {
            None
        };

        let process = GmlNameCodeSpaceValidator {
            city_gml_path_expr,
            global_params,
            processed_files: HashMap::new(),
        };
        Ok(Box::new(process))
    }
}

/// Stores error information for a single uncoded gml:name
#[derive(Debug, Clone)]
struct UncodedGmlNameError {
    gml_id: String,
    gml_name: String,
}

/// Stores per-file statistics
#[derive(Debug, Clone, Default)]
struct FileStats {
    filename: String,
    error_count: usize,
}

#[derive(Debug, Clone)]
pub struct GmlNameCodeSpaceValidator {
    city_gml_path_expr: Option<rhai::AST>,
    global_params: Option<HashMap<String, serde_json::Value>>,
    /// Map from filename to error count for statistics output
    processed_files: HashMap<String, FileStats>,
}

impl Processor for GmlNameCodeSpaceValidator {
    fn num_threads(&self) -> usize {
        1
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        // Get city GML path from expression or attribute
        let city_gml_path = if let Some(expr) = &self.city_gml_path_expr {
            let scope = feature.new_scope(Arc::clone(&ctx.expr_engine), &self.global_params);
            scope.eval_ast::<String>(expr).map_err(|e| {
                PlateauProcessorError::GmlNameCodeSpaceValidator(format!(
                    "Failed to evaluate cityGmlPath expression: {e:?}"
                ))
            })?
        } else {
            // Try to get from gmlPath attribute
            feature
                .get(Attribute::new("gmlPath"))
                .and_then(|v| match v {
                    AttributeValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .ok_or_else(|| {
                    PlateauProcessorError::GmlNameCodeSpaceValidator(
                        "No cityGmlPath expression and no gmlPath attribute found".to_string(),
                    )
                })?
        };

        // Get file info from feature
        let folder = feature
            .get(Attribute::new("udxDirs"))
            .and_then(|v| match v {
                AttributeValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "gen".to_string());

        let file_index = feature
            .get(Attribute::new("fileIndex"))
            .and_then(|v| match v {
                AttributeValue::Number(n) => n.as_u64(),
                AttributeValue::String(s) => s.parse().ok(),
                _ => None,
            })
            .unwrap_or(1);

        let filename = feature
            .get(Attribute::new("gmlFilename"))
            .and_then(|v| match v {
                AttributeValue::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| {
                // Extract filename from path
                city_gml_path
                    .split('/')
                    .next_back()
                    .unwrap_or("unknown.gml")
                    .to_string()
            });

        // Parse XML and find uncoded gml:name elements
        let errors = self.validate_gml_names(&ctx, &city_gml_path)?;

        // Track statistics for this file
        let stats = self
            .processed_files
            .entry(filename.clone())
            .or_insert_with(|| FileStats {
                filename: filename.clone(),
                error_count: 0,
            });
        stats.error_count += errors.len();

        // Output error details for each uncoded gml:name
        for error in errors {
            let mut error_feature = Feature::new_with_attributes(Attributes::new());
            error_feature.insert("Folder", AttributeValue::String(folder.clone()));
            error_feature.insert("Index", AttributeValue::Number(Number::from(file_index)));
            error_feature.insert("Filename", AttributeValue::String(filename.clone()));
            error_feature.insert("gml_id", AttributeValue::String(error.gml_id));
            error_feature.insert(
                "コード化されていないgml_name",
                AttributeValue::String(error.gml_name),
            );

            fw.send(ctx.new_with_feature_and_port(error_feature, GML_NAME_ERRORS_PORT.clone()));
        }

        // Forward original feature
        fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));

        Ok(())
    }

    fn finish(&self, ctx: NodeContext, fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        // Output per-file statistics
        for stats in self.processed_files.values() {
            let mut stats_feature = Feature::new_with_attributes(Attributes::new());
            stats_feature.insert("Filename", AttributeValue::String(stats.filename.clone()));
            stats_feature.insert(
                "_num_uncoded_gml_name",
                AttributeValue::Number(Number::from(stats.error_count)),
            );

            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                stats_feature,
                STATS_PORT.clone(),
            ));
        }

        Ok(())
    }

    fn name(&self) -> &str {
        "GmlNameCodeSpaceValidator"
    }
}

impl GmlNameCodeSpaceValidator {
    fn validate_gml_names(
        &self,
        ctx: &ExecutorContext,
        city_gml_path: &str,
    ) -> super::errors::Result<Vec<UncodedGmlNameError>> {
        let storage_resolver = Arc::clone(&ctx.storage_resolver);

        let city_gml_uri = Uri::from_str(city_gml_path).map_err(|e| {
            PlateauProcessorError::GmlNameCodeSpaceValidator(format!("Invalid URI: {e:?}"))
        })?;

        let storage = storage_resolver.resolve(&city_gml_uri).map_err(|e| {
            PlateauProcessorError::GmlNameCodeSpaceValidator(format!(
                "Failed to resolve storage: {e:?}"
            ))
        })?;

        let xml_content = storage
            .get_sync(city_gml_uri.path().as_path())
            .map_err(|e| {
                PlateauProcessorError::GmlNameCodeSpaceValidator(format!(
                    "Failed to read file: {e:?}"
                ))
            })?;

        // Convert to string for regex-based analysis
        let xml_str = String::from_utf8_lossy(&xml_content);

        // Use regex to find uncoded gml:name elements directly
        // This bypasses libxml serialization issues
        self.find_uncoded_gml_names_regex(&xml_str)
    }

    /// Find uncoded gml:name elements using regex on raw XML content
    /// This is more reliable than using libxml's attribute access methods
    fn find_uncoded_gml_names_regex(
        &self,
        xml_str: &str,
    ) -> super::errors::Result<Vec<UncodedGmlNameError>> {
        let mut errors = Vec::new();

        // Regex to match GenericCityObject elements with their gml:id
        // Pattern: <gen:GenericCityObject gml:id="...">...</gen:GenericCityObject>
        let generic_city_object_re = Regex::new(
            r#"<gen:GenericCityObject[^>]*gml:id="([^"]*)"[^>]*>([\s\S]*?)</gen:GenericCityObject>"#,
        )
        .map_err(|e| {
            PlateauProcessorError::GmlNameCodeSpaceValidator(format!("Failed to compile regex: {e}"))
        })?;

        // Regex to match ONLY uncoded gml:name elements (no attributes at all)
        // This pattern matches: <gml:name>value</gml:name>
        // But NOT: <gml:name codeSpace="...">value</gml:name>
        let uncoded_gml_name_re = Regex::new(r#"<gml:name>([^<]*)</gml:name>"#).map_err(|e| {
            PlateauProcessorError::GmlNameCodeSpaceValidator(format!(
                "Failed to compile regex: {e}"
            ))
        })?;

        // Find all GenericCityObject elements
        for cap in generic_city_object_re.captures_iter(xml_str) {
            let gml_id = cap.get(1).map(|m| m.as_str()).unwrap_or_default();
            let inner_content = cap.get(2).map(|m| m.as_str()).unwrap_or_default();

            // Find ONLY uncoded gml:name elements within this GenericCityObject
            for name_cap in uncoded_gml_name_re.captures_iter(inner_content) {
                let gml_name_value = name_cap.get(1).map(|m| m.as_str()).unwrap_or_default();

                errors.push(UncodedGmlNameError {
                    gml_id: gml_id.to_string(),
                    gml_name: gml_name_value.to_string(),
                });
            }
        }

        Ok(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_name() {
        let factory = GmlNameCodeSpaceValidatorFactory;
        assert_eq!(factory.name(), "PLATEAU4.GmlNameCodeSpaceValidator");
    }

    #[test]
    fn test_factory_ports() {
        let factory = GmlNameCodeSpaceValidatorFactory;
        let input_ports = factory.get_input_ports();
        let output_ports = factory.get_output_ports();

        assert_eq!(input_ports.len(), 1);
        assert_eq!(output_ports.len(), 3);
    }

    #[test]
    fn test_regex_detects_uncoded_gml_name() {
        let validator = GmlNameCodeSpaceValidator {
            city_gml_path_expr: None,
            global_params: None,
            processed_files: HashMap::new(),
        };

        // XML with uncoded gml:name (no codeSpace attribute)
        let xml_uncoded = r#"
        <gen:GenericCityObject gml:id="gen_001">
            <gml:name>20</gml:name>
        </gen:GenericCityObject>
        "#;

        let errors = validator.find_uncoded_gml_names_regex(xml_uncoded).unwrap();
        assert_eq!(errors.len(), 1, "Should detect 1 uncoded gml:name");
        assert_eq!(errors[0].gml_id, "gen_001");
        assert_eq!(errors[0].gml_name, "20");
    }

    #[test]
    fn test_regex_ignores_coded_gml_name() {
        let validator = GmlNameCodeSpaceValidator {
            city_gml_path_expr: None,
            global_params: None,
            processed_files: HashMap::new(),
        };

        // XML with coded gml:name (HAS codeSpace attribute)
        let xml_coded = r#"
        <gen:GenericCityObject gml:id="gen_002">
            <gml:name codeSpace="../../codelists/GenericCityObject_name.xml">20</gml:name>
        </gen:GenericCityObject>
        "#;

        let errors = validator.find_uncoded_gml_names_regex(xml_coded).unwrap();
        assert_eq!(
            errors.len(),
            0,
            "Should NOT detect any uncoded gml:name when codeSpace is present"
        );
    }

    #[test]
    fn test_regex_mixed_coded_and_uncoded() {
        let validator = GmlNameCodeSpaceValidator {
            city_gml_path_expr: None,
            global_params: None,
            processed_files: HashMap::new(),
        };

        // XML with both coded and uncoded gml:name
        let xml_mixed = r#"
        <gen:GenericCityObject gml:id="gen_coded">
            <gml:name codeSpace="../../codelists/test.xml">CodedValue</gml:name>
        </gen:GenericCityObject>
        <gen:GenericCityObject gml:id="gen_uncoded">
            <gml:name>UncodedValue</gml:name>
        </gen:GenericCityObject>
        "#;

        let errors = validator.find_uncoded_gml_names_regex(xml_mixed).unwrap();
        assert_eq!(errors.len(), 1, "Should detect only the uncoded gml:name");
        assert_eq!(errors[0].gml_id, "gen_uncoded");
        assert_eq!(errors[0].gml_name, "UncodedValue");
    }
}
