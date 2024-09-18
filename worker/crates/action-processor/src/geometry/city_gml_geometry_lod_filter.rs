use std::collections::HashMap;

use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::GeometryValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

#[derive(Debug, Clone, Default)]
pub struct CityGmlGeometryLodFilterFactory;

impl ProcessorFactory for CityGmlGeometryLodFilterFactory {
    fn name(&self) -> &str {
        "CityGmlGeometryLodFilter"
    }

    fn description(&self) -> &str {
        "Filters CityGML geometries by LOD"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CityGmlGeometryLodFilter))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), REJECTED_PORT.clone()]
    }
    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let filter: CityGmlGeometryLodFilter = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::CityGmlGeometryLodFilterFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::CityGmlGeometryLodFilterFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(GeometryProcessorError::CityGmlGeometryLodFilterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        if filter.lods.iter().any(|lod| *lod > 4) {
            return Err(GeometryProcessorError::CityGmlGeometryLodFilterFactory(
                "LOD must be between 0 and 4".to_string(),
            )
            .into());
        }
        Ok(Box::new(filter))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CityGmlGeometryLodFilter {
    lods: Vec<u8>,
}

impl Processor for CityGmlGeometryLodFilter {
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
        if let Some(geometry) = &feature.geometry {
            match &geometry.value {
                GeometryValue::CityGmlGeometry(v) => {
                    let mut feature = feature.clone();
                    let mut geometry = geometry.clone();
                    let mut geometry_value = v.clone();
                    geometry_value.gml_geometries.retain(|g| {
                        if let Some(lod) = g.lod {
                            self.lods.contains(&lod)
                        } else {
                            false
                        }
                    });
                    if geometry_value.gml_geometries.is_empty() {
                        fw.send(ctx.new_with_feature_and_port(feature, REJECTED_PORT.clone()));
                        return Ok(());
                    }
                    geometry.value = GeometryValue::CityGmlGeometry(geometry_value);
                    feature.geometry = Some(geometry);
                    fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
                }
                GeometryValue::FlowGeometry2D(_) => {
                    return Err(GeometryProcessorError::CityGmlGeometryLodFilter(
                        "FlowGeometry2D is not supported".to_string(),
                    )
                    .into());
                }
                GeometryValue::FlowGeometry3D(_) => {
                    return Err(GeometryProcessorError::CityGmlGeometryLodFilter(
                        "FlowGeometry3D is not supported".to_string(),
                    )
                    .into());
                }
                GeometryValue::None => {
                    return Err(GeometryProcessorError::CityGmlGeometryLodFilter(
                        "GeometryValue is None".to_string(),
                    )
                    .into());
                }
            }
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
        "CityGmlGeometryLodFilter"
    }
}
