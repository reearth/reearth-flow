use super::errors::PlateauProcessorError;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{Context, ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{hash_map::Entry, HashMap};

#[derive(Debug, Clone, Default)]
pub struct MaxLodExtractorFactory;

impl ProcessorFactory for MaxLodExtractorFactory {
    fn name(&self) -> &str {
        "PLATEAU4.MaxLodExtractor"
    }

    fn description(&self) -> &str {
        "Extracts maxLod"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(MaxLodExtractorParam))
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
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: MaxLodExtractorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::MaxLodExtractorFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::MaxLodExtractorFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(PlateauProcessorError::MaxLodExtractorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let process = MaxLodExtractor {
            city_gml_path_attribute: params.city_gml_path_attribute,
            max_lod_attribute: params.max_lod_attribute,
            buffer: HashMap::new(),
        };
        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MaxLodExtractorParam {
    city_gml_path_attribute: Attribute,
    max_lod_attribute: Attribute,
}

#[derive(Debug, Clone)]
struct MaxLodBuffer {
    features: Vec<Feature>,
    max_lod: u8,
}

#[derive(Debug, Clone)]
pub(crate) struct MaxLodExtractor {
    city_gml_path_attribute: Attribute,
    max_lod_attribute: Attribute,
    buffer: HashMap<String, MaxLodBuffer>,
}

impl Processor for MaxLodExtractor {
    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let city_gml_path = feature
            .attributes
            .get(&self.city_gml_path_attribute)
            .ok_or(PlateauProcessorError::MaxLodExtractor(
                "cityGmlPath attribute empty".to_string(),
            ))?;
        let lod = feature
            .metadata
            .lod
            .ok_or(PlateauProcessorError::MaxLodExtractor(
                "lod metadata empty".to_string(),
            ))?;
        let highest_lod = lod
            .highest_lod()
            .ok_or(PlateauProcessorError::MaxLodExtractor(
                "highest lod empty".to_string(),
            ))?;
        let flush = match self.buffer.entry(city_gml_path.to_string()) {
            Entry::Occupied(mut entry) => {
                let buffer = entry.get_mut();
                if highest_lod > buffer.max_lod {
                    buffer.max_lod = highest_lod;
                }
                buffer.features.push(feature.clone());
                false
            }
            Entry::Vacant(entry) => {
                entry.insert(MaxLodBuffer {
                    features: vec![feature.clone()],
                    max_lod: highest_lod,
                });
                true
            }
        };
        if flush {
            self.flush_buffer(ctx.as_context(), fw, city_gml_path.to_string());
        }
        Ok(())
    }

    fn finish(
        &self,
        ctx: NodeContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        for (_, buffer) in self.buffer.iter() {
            for feature in buffer.features.iter() {
                let mut feature = feature.clone();
                feature.attributes.insert(
                    self.max_lod_attribute.clone(),
                    AttributeValue::Number(serde_json::Number::from(buffer.max_lod)),
                );
                fw.send(ExecutorContext::new_with_node_context_feature_and_port(
                    &ctx,
                    feature,
                    DEFAULT_PORT.clone(),
                ));
            }
        }
        Ok(())
    }

    fn name(&self) -> &str {
        "MaxLodExtractor"
    }
}

impl MaxLodExtractor {
    pub(crate) fn flush_buffer(
        &mut self,
        ctx: Context,
        fw: &mut dyn ProcessorChannelForwarder,
        ignore_key: String,
    ) {
        for (_, buffer) in self
            .buffer
            .iter()
            .filter(|(k, _)| (*k).clone() != ignore_key)
        {
            for feature in buffer.features.iter() {
                let mut feature = feature.clone();
                feature.attributes.insert(
                    self.max_lod_attribute.clone(),
                    AttributeValue::Number(serde_json::Number::from(buffer.max_lod)),
                );
                fw.send(ctx.as_executor_context(feature, DEFAULT_PORT.clone()));
            }
        }
        let keys: Vec<String> = self
            .buffer
            .keys()
            .filter(|k| **k != ignore_key)
            .cloned()
            .collect();
        let buffer = &mut self.buffer;
        for key in keys {
            buffer.remove(&key);
        }
    }
}
