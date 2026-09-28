mod citygml;
mod csv;
pub(super) mod geojson;
mod json;

use std::collections::HashMap;

use indexmap::IndexMap;
use reearth_flow_common::csv::Delimiter;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::lod::LodMask;
use reearth_flow_types::{Attribute, AttributeValue, Code, CodeType, CompiledCode, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::FeatureProcessorError;

#[derive(Debug, Clone, Default)]
pub(super) struct FeatureWriterFactory;

impl ProcessorFactory for FeatureWriterFactory {
    fn name(&self) -> &str {
        "Feature Writer"
    }

    fn description(&self) -> &str {
        "Writes the features it receives to files, grouping them by the evaluated output path. \
Emits one feature per file written, carrying that file's path and row count, in place of the \
features it consumed."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureWriterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Feature"]
    }

    fn tags(&self) -> &[&'static str] {
        &["csv", "json"]
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
        let params: FeatureWriterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                FeatureProcessorError::FeatureWriterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                FeatureProcessorError::FeatureWriterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(FeatureProcessorError::FeatureWriterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let output = params
            .output
            .compile()
            .map_err(|e| FeatureProcessorError::FeatureWriterFactory(format!("{e:?}")))?;
        let format = match params.format {
            FeatureWriterFormat::Csv => CompiledFormat::Csv,
            FeatureWriterFormat::Tsv => CompiledFormat::Tsv,
            FeatureWriterFormat::Json { converter } => CompiledFormat::Json {
                converter: converter
                    .map(|code| code.compile())
                    .transpose()
                    .map_err(|e| FeatureProcessorError::FeatureWriterFactory(format!("{e:?}")))?,
            },
            #[cfg(not(feature = "new-geometry"))]
            FeatureWriterFormat::CityGml {
                lod_filter,
                epsg_code,
                pretty_print,
            } => CompiledFormat::CityGml {
                lod_mask: citygml::build_lod_mask(&lod_filter),
                epsg_code,
                pretty_print: pretty_print.unwrap_or(true),
            },
        };

        Ok(Box::new(FeatureWriter {
            format,
            output,
            buffer: HashMap::new(),
        }))
    }
}

#[derive(Debug, Clone)]
struct FeatureWriter {
    format: CompiledFormat,
    output: CompiledCode,
    pub(super) buffer: HashMap<String, Vec<Feature>>,
}

/// # Feature Writer Parameters
///
/// Configures the file format written and where each file goes.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct FeatureWriterParam {
    /// # Format
    ///
    /// The file format to write, with the settings that format takes. Every
    /// format writes attribute values only; geometry is not included in the
    /// output.
    format: FeatureWriterFormat,
    /// # Output Path
    ///
    /// Where to write, relative to the job's output directory. Evaluated once per
    /// feature, so an expression over the feature's attributes splits the stream
    /// into one file per distinct result.
    output: Code,
}

/// # Format
///
/// The file format written, and the settings belonging to it.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
enum FeatureWriterFormat {
    /// # CSV
    ///
    /// Comma-separated values. The column list comes from the first feature
    /// written to a file, so every later feature must carry those same
    /// attributes.
    Csv,
    /// # TSV
    ///
    /// Tab-separated values, with the same column rules as CSV.
    Tsv,
    /// # JSON
    ///
    /// An array of objects, one per feature, holding its attribute values.
    #[serde(rename_all = "camelCase")]
    Json {
        /// # Converter
        ///
        /// Builds the JSON document from all the features destined for one file,
        /// replacing the default array of attribute objects.
        #[serde(default)]
        converter: Option<Code<{ CodeType::FlowExpr as u32 }>>,
    },
    // TODO(new-geometry): the CityGML arm shares `write_citygml_to_storage` with
    // the `CityGML Writer` sink. Restore this variant when that sink is ported.
    #[cfg(not(feature = "new-geometry"))]
    #[serde(rename = "citygml", rename_all = "camelCase")]
    CityGml {
        /// # LOD Filter
        ///
        /// LOD levels to include. When omitted or empty, every LOD is included.
        #[serde(default)]
        lod_filter: Option<Vec<u8>>,
        /// # EPSG Code
        ///
        /// EPSG code of the coordinate reference system to declare.
        #[serde(default)]
        epsg_code: Option<u32>,
        /// # Pretty Print
        ///
        /// Whether to indent the output. Defaults to indenting.
        #[serde(default = "default_pretty_print")]
        pretty_print: Option<bool>,
    },
}

#[cfg(not(feature = "new-geometry"))]
fn default_pretty_print() -> Option<bool> {
    Some(true)
}

#[derive(Debug, Clone)]
enum CompiledFormat {
    Csv,
    Tsv,
    Json {
        converter: Option<CompiledCode>,
    },
    #[cfg(not(feature = "new-geometry"))]
    CityGml {
        lod_mask: LodMask,
        epsg_code: Option<u32>,
        pretty_print: bool,
    },
}

impl Processor for FeatureWriter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let path = self
            .output
            .eval_string(feature, ctx.variables.clone())
            .map_err(|e| FeatureProcessorError::FeatureWriter(format!("{e:?}")))?;
        // Validation happens at flush time via SinkOutput::new; nothing to
        // pre-check here. The buffer is keyed by the raw relative-path string.
        let buffer = self.buffer.entry(path).or_default();
        buffer.push(ctx.feature);
        Ok(())
    }

    fn finish(
        &mut self,
        ctx: NodeContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        for (rel_path, features) in &self.buffer {
            // SinkOutput::new validates the path and acquires the storage backend,
            // providing the sandbox gate at flush time.
            let sink_output = reearth_flow_action_sink::SinkOutput::new(
                &ctx.sandbox_root,
                rel_path,
                &ctx.storage_resolver,
            )
            .map_err(|e| {
                FeatureProcessorError::FeatureWriter(format!(
                    "sink output {rel_path:?} rejected by sandbox: {e}"
                ))
            })?;
            let output = sink_output.uri();
            let feature: Feature = IndexMap::<Attribute, AttributeValue>::from([
                (
                    Attribute::new("filePath".to_string()),
                    AttributeValue::String(output.path().to_str().unwrap_or_default().to_string()),
                ),
                (
                    Attribute::new("rowCount".to_string()),
                    AttributeValue::Number(serde_json::Number::from(features.len())),
                ),
            ])
            .into();
            match self.format {
                CompiledFormat::Csv => {
                    csv::write_csv(output, Delimiter::Comma, &ctx.storage_resolver, features)?;
                }
                CompiledFormat::Tsv => {
                    csv::write_csv(output, Delimiter::Tab, &ctx.storage_resolver, features)?;
                }
                CompiledFormat::Json { ref converter } => {
                    json::write_json(
                        output,
                        converter,
                        &ctx.storage_resolver,
                        ctx.variables.clone(),
                        features,
                    )?;
                }
                #[cfg(not(feature = "new-geometry"))]
                CompiledFormat::CityGml {
                    lod_mask,
                    epsg_code,
                    pretty_print,
                } => {
                    citygml::write_citygml(
                        output,
                        &ctx.sandbox_root,
                        features,
                        &lod_mask,
                        &epsg_code,
                        &pretty_print,
                        &ctx.storage_resolver,
                        ctx.diagnostics.as_deref(),
                    )?;
                }
            }
            fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                &ctx,
                feature,
                FEATURES_PORT.clone(),
            ));
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "Feature Writer"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output() -> Value {
        serde_json::json!({ "type": "string", "value": "out.csv" })
    }

    fn parse(with: Value) -> Result<FeatureWriterParam, serde_json::Error> {
        serde_json::from_value(with)
    }

    #[test]
    fn a_format_carrying_no_settings_needs_only_its_type() {
        for name in ["csv", "tsv"] {
            let params = parse(serde_json::json!({
                "format": { "type": name },
                "output": output(),
            }))
            .unwrap_or_else(|e| panic!("{name} should parse: {e}"));
            match (name, params.format) {
                ("csv", FeatureWriterFormat::Csv) | ("tsv", FeatureWriterFormat::Tsv) => {}
                (_, other) => panic!("{name} parsed as {other:?}"),
            }
        }
    }

    #[test]
    fn the_converter_is_optional_and_belongs_to_the_json_format() {
        let bare = parse(serde_json::json!({
            "format": { "type": "json" },
            "output": output(),
        }))
        .expect("json without a converter should parse");
        assert!(matches!(
            bare.format,
            FeatureWriterFormat::Json { converter: None }
        ));

        let with_converter = parse(serde_json::json!({
            "format": {
                "type": "json",
                "converter": { "type": "flowExpr", "value": "attributes" },
            },
            "output": output(),
        }))
        .expect("json with a converter should parse");
        assert!(matches!(
            with_converter.format,
            FeatureWriterFormat::Json { converter: Some(_) }
        ));
    }

    /// The converter used to sit beside `format`, where nothing stopped it being
    /// set against CSV. It now belongs to the variant that reads it — but a
    /// leftover copy at the old position is *ignored*, not reported, because no
    /// action in the workspace sets `deny_unknown_fields`. Pinned here so the
    /// migration trap is visible; see cross-cutting finding 10.
    #[test]
    fn a_converter_left_at_the_old_position_is_silently_ignored() {
        let params = parse(serde_json::json!({
            "format": { "type": "csv" },
            "output": output(),
            "converter": { "type": "flowExpr", "value": "attributes" },
        }))
        .expect("an unknown key does not fail the parse");
        assert!(matches!(params.format, FeatureWriterFormat::Csv));
    }

    /// §7.3: the arm cannot run under new geometry, so it must not be offered.
    #[cfg(feature = "new-geometry")]
    #[test]
    fn citygml_is_not_a_format_in_the_shipped_build() {
        let err = parse(serde_json::json!({
            "format": { "type": "citygml" },
            "output": output(),
        }))
        .unwrap_err();
        assert!(
            err.to_string().contains("citygml"),
            "expected the unknown variant to be named, got: {err}"
        );
    }

    #[cfg(not(feature = "new-geometry"))]
    #[test]
    fn citygml_carries_its_own_settings_in_the_legacy_build() {
        let params = parse(serde_json::json!({
            "format": {
                "type": "citygml",
                "lodFilter": [1, 2],
                "epsgCode": 6697,
                "prettyPrint": true,
            },
            "output": output(),
        }))
        .expect("citygml should parse in the legacy build");
        let FeatureWriterFormat::CityGml {
            lod_filter,
            epsg_code,
            pretty_print,
        } = params.format
        else {
            panic!("expected the citygml format");
        };
        assert_eq!(lod_filter, Some(vec![1, 2]));
        assert_eq!(epsg_code, Some(6697));
        assert_eq!(pretty_print, Some(true));
    }

    /// Omitted entirely, every citygml setting falls back rather than failing.
    #[cfg(not(feature = "new-geometry"))]
    #[test]
    fn citygml_settings_all_default() {
        let params = parse(serde_json::json!({
            "format": { "type": "citygml" },
            "output": output(),
        }))
        .expect("citygml should parse with no settings");
        let FeatureWriterFormat::CityGml {
            lod_filter,
            pretty_print,
            ..
        } = params.format
        else {
            panic!("expected the citygml format");
        };
        assert_eq!(lod_filter, None);
        // No filter means every LOD, not none of them.
        let mask = citygml::build_lod_mask(&lod_filter);
        assert_eq!(format!("{mask:?}"), format!("{:?}", LodMask::all()));
        assert_eq!(pretty_print, Some(true));
    }
}
