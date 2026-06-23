use std::collections::HashMap;

use indexmap::IndexMap;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::NodeContext,
    node::{IngestionMessage, Port, Source, SourceFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Code, CodeType, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::Sender;

use crate::errors::SourceError;

#[derive(Debug, Clone, Default)]
pub struct FeatureCreatorFactory;

impl SourceFactory for FeatureCreatorFactory {
    fn name(&self) -> &str {
        "FeatureCreator"
    }

    fn description(&self) -> &str {
        "Generate Custom Features Using Scripts"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureCreator))
    }

    fn categories(&self) -> &[&'static str] {
        &["Input"]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }
    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
        _state: Option<Vec<u8>>,
    ) -> Result<Box<dyn Source>, BoxedError> {
        let processor: FeatureCreator = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                SourceError::FeatureCreatorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                SourceError::FeatureCreatorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(SourceError::FeatureCreatorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let creator = processor
            .creator
            .compile()
            .map_err(|e| {
                SourceError::FeatureCreatorFactory(format!("Failed to compile params: {e:?}"))
            })?
            .eval_env_only(ctx.env_vars.clone())
            .map_err(|e| {
                SourceError::FeatureCreatorFactory(format!("Failed to evaluate creator: {e:?}"))
            })?;
        let compiled = FeatureCreatorCompiledParam { creator };
        Ok(Box::new(FeatureCreatorSource { params: compiled }))
    }
}

/// # FeatureCreator Parameters
/// Configure how to generate custom features using script expressions
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCreator {
    /// # Script Expression
    /// Write a script expression that returns a map (single feature) or array of maps (multiple features). Each map represents feature attributes as key-value pairs.
    creator: Code<{ CodeType::FlowExpr as u32 }>,
}

#[derive(Debug, Clone)]
struct FeatureCreatorCompiledParam {
    creator: AttributeValue,
}

#[derive(Debug, Clone)]
struct FeatureCreatorSource {
    params: FeatureCreatorCompiledParam,
}

#[async_trait::async_trait]
impl Source for FeatureCreatorSource {
    async fn initialize(&self, _ctx: NodeContext) {}

    fn name(&self) -> &str {
        "FeatureCreator"
    }

    async fn serialize_state(&self) -> Result<Vec<u8>, BoxedError> {
        Ok(vec![])
    }

    async fn start(
        &mut self,
        _ctx: NodeContext,
        sender: Sender<(Port, IngestionMessage)>,
    ) -> Result<(), BoxedError> {
        match self.params.creator.clone() {
            AttributeValue::Map(map) => {
                let attributes = map
                    .into_iter()
                    .map(|(k, v)| (Attribute::new(k), v))
                    .collect::<IndexMap<Attribute, AttributeValue>>();
                let feature = Feature::from(attributes);
                sender
                    .send((
                        DEFAULT_PORT.clone(),
                        IngestionMessage::OperationEvent { feature },
                    ))
                    .await
                    .map_err(|e| SourceError::FeatureCreator(format!("{e:?}")))?;
            }
            AttributeValue::Array(arr) => {
                for item in arr {
                    if let AttributeValue::Map(map) = item {
                        let attributes = map
                            .into_iter()
                            .map(|(k, v)| (Attribute::new(k), v))
                            .collect::<IndexMap<Attribute, AttributeValue>>();
                        let feature = Feature::from(attributes);
                        sender
                            .send((
                                DEFAULT_PORT.clone(),
                                IngestionMessage::OperationEvent { feature },
                            ))
                            .await
                            .map_err(|e| SourceError::FeatureCreator(format!("{e:?}")))?;
                    }
                }
            }
            _ => {
                return Err(SourceError::FeatureCreator(
                    "Expected map or array from creator".to_string(),
                )
                .into());
            }
        }
        Ok(())
    }
}
