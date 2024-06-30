use super::errors::PlateauProcessorError;
use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

static DIGITS_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\d+").unwrap());

#[derive(Debug, Clone, Default)]
pub struct MaxLodExtractorFactory;

impl ProcessorFactory for MaxLodExtractorFactory {
    fn name(&self) -> &str {
        "PLATEAU.MaxLodExtractor"
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
    fn initialize(&mut self, _ctx: NodeContext) {}

    fn num_threads(&self) -> usize {
        5
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        let package = feature.attributes.get(&Attribute::new("package")).ok_or(
            PlateauProcessorError::DomainOfDefinitionValidator("package key empty".to_string()),
        )?;

        let package_lenght = package.to_string().len();
        let out_length = package_lenght + 1;

        let city_gml_path = feature
            .attributes
            .get(&Attribute::new("cityGmlPath"))
            .unwrap()
            .clone();

        let city_gml_path = city_gml_path.to_string();
        let path = Path::new(&city_gml_path);
        let file_name = if let Some(file_name) = path.file_name() {
            if let Some(file_name_str) = file_name.to_str() {
                file_name_str
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        };

        if !DIGITS_PATTERN.is_match(file_name) {
            return Ok(());
        }

        let parts: Vec<&str> = file_name.split('_').collect();
        let out_code = parts[0];
        let out_type = parts[1];

        let attribute_code = Attribute::new("code");
        let attribute_type = Attribute::new("type");
        let attribute_max_lod = Attribute::new("maxLod");
        let attribute_length = Attribute::new("length");
        let attribute_file = Attribute::new("file");

        let mut attributes = feature.attributes.clone();

        for (k, _) in feature.attributes.iter() {
            attributes.remove(k);
        }

        attributes.insert(attribute_code, AttributeValue::String(out_code.to_string()));
        attributes.insert(attribute_type, AttributeValue::String(out_type.to_string()));
        attributes.insert(attribute_max_lod, AttributeValue::String("1".to_string()));
        attributes.insert(
            attribute_length,
            AttributeValue::String(out_length.to_string()),
        );
        attributes.insert(
            attribute_file,
            AttributeValue::String(file_name.to_string()),
        );

        let feature = Feature {
            attributes,
            ..feature.clone()
        };

        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(
        &self,
        _ctx: NodeContext,
        _fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "MaxLodExtractor"
    }
}
