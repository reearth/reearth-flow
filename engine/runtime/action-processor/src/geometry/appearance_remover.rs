use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::GeometryValue;
use serde_json::Value;

#[derive(Debug, Clone, Default)]
pub struct AppearanceRemoverFactory;

impl ProcessorFactory for AppearanceRemoverFactory {
    fn name(&self) -> &str {
        "AppearanceRemover"
    }

    fn description(&self) -> &str {
        "Removes appearance information (materials, textures) from CityGML geometry"
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
        Ok(Box::new(AppearanceRemover))
    }
}

#[derive(Debug, Clone)]
pub struct AppearanceRemover;

impl Processor for AppearanceRemover {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        let feature = match &feature.geometry.value {
            GeometryValue::CityGmlGeometry(gml) => {
                let mut gml = gml.clone();
                gml.materials.clear();
                gml.textures.clear();
                gml.polygon_materials.clear();
                gml.polygon_textures.clear();
                gml.polygon_uvs = flatgeom::MultiPolygon::default();

                let mut geometry = feature.geometry.clone();
                geometry.value = GeometryValue::CityGmlGeometry(gml);
                let mut feature = feature.clone();
                feature.geometry = geometry;
                feature
            }
            // For non-CityGML geometry, pass through unchanged
            _ => feature.clone(),
        };

        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "AppearanceRemover"
    }
}
