use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature, GeometryValue};
use serde_json::Value;

pub static UNFILTERED_PORT: Lazy<Port> = Lazy::new(|| Port::new("unfiltered"));

#[derive(Debug, Clone, Default)]
pub struct GeometrySplitterFactory;

impl ProcessorFactory for GeometrySplitterFactory {
    fn name(&self) -> &str {
        "GeometrySplitter"
    }

    fn description(&self) -> &str {
        "Split geometry by type"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
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
        let process = GeometrySplitter {};
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct GeometrySplitter;

impl Processor for GeometrySplitter {
    fn initialize(&mut self, _ctx: NodeContext) {}

    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let Some(geometry) = &feature.geometry else {
            return Ok(());
        };
        match &geometry.value {
            GeometryValue::CityGmlGeometry(city_gml_geometry) => {
                if city_gml_geometry.features.len() < 2 {
                    fw.send(
                        ctx.new_with_feature_and_port(ctx.feature.clone(), DEFAULT_PORT.clone()),
                    );
                    return Ok(());
                }
                for split_feature in city_gml_geometry.split_feature() {
                    let mut geometry = geometry.clone();
                    let mut attributes = feature.attributes.clone();
                    let geometry_name = if let Some(feature) = split_feature.features.first() {
                        feature.to_string()
                    } else {
                        "unknown".to_string()
                    };
                    attributes.insert(
                        Attribute::new("geometryName"),
                        AttributeValue::String(geometry_name),
                    );
                    geometry.value = GeometryValue::CityGmlGeometry(split_feature);
                    fw.send(ctx.new_with_feature_and_port(
                        Feature::new_with_attributes_and_geometry(attributes, geometry),
                        DEFAULT_PORT.clone(),
                    ));
                }
            }
            _ => unimplemented!(),
        }
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
        "GeometrySplitter"
    }
}
