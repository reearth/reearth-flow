use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use serde_json::Value;

#[cfg(not(feature = "new-geometry"))]
use std::sync::Arc;

#[cfg(not(feature = "new-geometry"))]
use reearth_flow_types::{Feature, GeometryValue};

#[cfg(feature = "new-geometry")]
use reearth_flow_geometry::ops::RemoveAppearance;

#[derive(Debug, Clone, Default)]
pub struct AppearanceRemoverFactory;

impl ProcessorFactory for AppearanceRemoverFactory {
    fn name(&self) -> &str {
        "Appearance Remover"
    }

    fn description(&self) -> &str {
        "Discards the materials, textures, and texture coordinates carried by a feature's geometry."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn tags(&self) -> &[&'static str] {
        &["citygml", "3d"]
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
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        Ok(Box::new(AppearanceRemover))
    }
}

#[derive(Debug, Clone)]
pub struct AppearanceRemover;

impl Processor for AppearanceRemover {
    #[cfg(not(feature = "new-geometry"))]
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
                gml.polygon_uvs.0.clear();

                let mut geometry = (*feature.geometry).clone();
                geometry.value = GeometryValue::CityGmlGeometry(gml);
                Feature {
                    geometry: Arc::new(geometry),
                    attributes: feature.attributes.clone(),
                    id: feature.id,
                }
            }
            // For non-CityGML geometry, pass through unchanged
            _ => feature.clone(),
        };

        fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
        Ok(())
    }

    #[cfg(feature = "new-geometry")]
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        feature.geometry_mut().remove_appearance();
        fw.send(ctx.new_with_feature_and_port(feature, FEATURES_PORT.clone()));
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
        "Appearance Remover"
    }
}
