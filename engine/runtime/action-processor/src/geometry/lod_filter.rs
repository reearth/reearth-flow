use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::GeometryProcessorError;

pub static UNFILTERED_PORT: Lazy<Port> = Lazy::new(|| Port::new("unfiltered"));

#[derive(Debug, Clone, Default)]
pub struct GeometryLodFilterFactory;

impl ProcessorFactory for GeometryLodFilterFactory {
    fn name(&self) -> &str {
        "GeometryLodFilter"
    }

    fn description(&self) -> &str {
        "Filter geometry by lod"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(GeometryLodFilterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Geometry"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone(), UNFILTERED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: GeometryLodFilterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                GeometryProcessorError::GeometryLodFilterFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                GeometryProcessorError::GeometryLodFilterFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(GeometryProcessorError::GeometryLodFilterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let max_lod = params.max_lod.unwrap_or(4);
        let min_lod = params.min_lod.unwrap_or(0);
        if max_lod < min_lod {
            return Err(GeometryProcessorError::GeometryLodFilterFactory(
                "max_lod must be greater than or equal to min_lod".to_string(),
            )
            .into());
        }

        let process = GeometryLodFilter { max_lod, min_lod };
        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeometryLodFilterParam {
    pub max_lod: Option<u8>,
    pub min_lod: Option<u8>,
}

#[derive(Debug, Clone)]
pub struct GeometryLodFilter {
    pub max_lod: u8,
    pub min_lod: u8,
}

impl Processor for GeometryLodFilter {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let Some(lod) = &ctx.feature.metadata.lod else {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), UNFILTERED_PORT.clone()));
            return Ok(());
        };
        let Some(highest_lod) = lod.highest_lod() else {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), UNFILTERED_PORT.clone()));
            return Ok(());
        };
        let Some(lowest_lod) = lod.lowest_lod() else {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), UNFILTERED_PORT.clone()));
            return Ok(());
        };
        if highest_lod <= self.max_lod && lowest_lod >= self.min_lod {
            fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), DEFAULT_PORT.clone()));
            return Ok(());
        }
        fw.send(ctx.new_with_feature_and_port(ctx.feature.clone(), UNFILTERED_PORT.clone()));
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
        "GeometryLodFilter"
    }
}

#[cfg(test)]
mod tests {
    use reearth_flow_types::{lod::LodMask, Feature};

    use crate::tests::utils::{create_default_execute_context, MockProcessorChannelForwarder};

    use super::*;

    #[test]
    fn test_min_lod_default_port() {
        let mut filter = GeometryLodFilter {
            max_lod: 2,
            min_lod: 1,
        };
        let mut fw = Box::new(MockProcessorChannelForwarder::default());
        let mut feature = Feature::default();
        let mut mask = LodMask::default();
        mask.add_lod(1);
        feature.metadata.lod = Some(mask);
        let ctx = create_default_execute_context(&feature);
        let result = filter.process(ctx, &mut *fw);
        assert!(result.is_ok());
        assert_eq!(fw.send_port, DEFAULT_PORT.clone());
    }

    #[test]
    fn test_max_lod_default_port() {
        let mut filter = GeometryLodFilter {
            max_lod: 3,
            min_lod: 1,
        };
        let mut fw = Box::new(MockProcessorChannelForwarder::default());
        let mut feature = Feature::default();
        let mut mask = LodMask::default();
        mask.add_lod(3);
        mask.add_lod(2);
        feature.metadata.lod = Some(mask);
        let ctx = create_default_execute_context(&feature);
        let result = filter.process(ctx, &mut *fw);
        assert!(result.is_ok());
        assert_eq!(fw.send_port, DEFAULT_PORT.clone());
    }

    #[test]
    fn test_min_lod_unfilter_port() {
        let mut filter = GeometryLodFilter {
            max_lod: 3,
            min_lod: 2,
        };
        let mut fw = Box::new(MockProcessorChannelForwarder::default());
        let mut feature = Feature::default();
        let mut mask = LodMask::default();
        mask.add_lod(3);
        mask.add_lod(1);
        feature.metadata.lod = Some(mask);
        let ctx = create_default_execute_context(&feature);
        let result = filter.process(ctx, &mut *fw);
        assert!(result.is_ok());
        assert_eq!(fw.send_port, UNFILTERED_PORT.clone());
    }

    #[test]
    fn test_max_lod_unfilter_port() {
        let mut filter = GeometryLodFilter {
            max_lod: 3,
            min_lod: 2,
        };
        let mut fw = Box::new(MockProcessorChannelForwarder::default());
        let mut feature = Feature::default();
        let mut mask = LodMask::default();
        mask.add_lod(4);
        mask.add_lod(2);
        feature.metadata.lod = Some(mask);
        let ctx = create_default_execute_context(&feature);
        let result = filter.process(ctx, &mut *fw);
        assert!(result.is_ok());
        assert_eq!(fw.send_port, UNFILTERED_PORT.clone());
    }
}
