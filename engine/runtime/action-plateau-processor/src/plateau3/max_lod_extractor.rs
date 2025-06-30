use super::errors::PlateauProcessorError;
use once_cell::sync::Lazy;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;

static DIGITS_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d+").unwrap());

#[derive(Debug, Clone, Default)]
pub struct MaxLodExtractorFactory;

impl ProcessorFactory for MaxLodExtractorFactory {
    fn name(&self) -> &str {
        "PLATEAU3.MaxLodExtractor"
    }

    fn description(&self) -> &str {
        "Extracts maxLod"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
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
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let process = MaxLodExtractor {};
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct MaxLodExtractor {}

impl Processor for MaxLodExtractor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        let package = feature.attributes.get(&Attribute::new("package")).ok_or(
            PlateauProcessorError::MaxLodExtractor("package key empty".to_string()),
        )?;

        let city_gml_path = feature
            .attributes
            .get(&Attribute::new("cityGmlPath"))
            .unwrap()
            .clone();

        let path_uri = Uri::from_str(city_gml_path.to_string().as_str()).map_err(|err| {
            PlateauProcessorError::MaxLodExtractor(format!(
                "cityGmlPath is not a valid uri: {err}"
            ))
        })?;

        let file_name = path_uri
            .file_name()
            .ok_or(PlateauProcessorError::MaxLodExtractor(
                "file_name is empty".to_string(),
            ))?;

        let file_name_str = file_name
            .to_str()
            .ok_or(PlateauProcessorError::MaxLodExtractor(
                "file_name is not a valid string".to_string(),
            ))?;

        if !DIGITS_PATTERN.is_match(file_name_str) {
            return Ok(());
        }

        let code = feature.attributes.get(&Attribute::new("meshCode")).ok_or(
            PlateauProcessorError::MaxLodExtractor("meshCode key empty".to_string()),
        )?;

        let max_lod = feature.attributes.get(&Attribute::new("maxLod")).ok_or(
            PlateauProcessorError::MaxLodExtractor("maxLod key empty".to_string()),
        )?;

        let attribute_code = Attribute::new("code");
        let attribute_type = Attribute::new("type");
        let attribute_max_lod = Attribute::new("maxLod");
        let attribute_file = Attribute::new("file");

        let mut attributes = feature.attributes.clone();

        for (k, _) in feature.attributes.iter() {
            attributes.swap_remove(k);
        }

        attributes.insert(attribute_code, AttributeValue::String(code.to_string()));
        attributes.insert(attribute_type, AttributeValue::String(package.to_string()));
        attributes.insert(
            attribute_max_lod,
            AttributeValue::String(max_lod.to_string()),
        );
        attributes.insert(
            attribute_file,
            AttributeValue::String(file_name_str.to_string()),
        );

        let feature = Feature {
            attributes,
            ..feature.clone()
        };

        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "MaxLodExtractor"
    }
}
