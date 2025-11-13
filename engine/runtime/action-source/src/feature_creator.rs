use std::{collections::HashMap, sync::Arc};

use indexmap::IndexMap;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::NodeContext,
    node::{IngestionMessage, Port, Source, SourceFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature};
use rhai::Dynamic;
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
        &["Feature"]
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
        Ok(Box::new(processor))
    }
}

/// # FeatureCreator Parameters
/// Configure how to generate custom features using script expressions
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCreator {
    /// # Script Expression
    /// Write a script expression that returns a map (single feature) or array of maps (multiple features). Each map represents feature attributes as key-value pairs.
    creator: Expr,
}

#[async_trait::async_trait]
impl Source for FeatureCreator {
    async fn initialize(&self, _ctx: NodeContext) {}

    fn name(&self) -> &str {
        "FeatureCreator"
    }

    async fn serialize_state(&self) -> Result<Vec<u8>, BoxedError> {
        Ok(vec![])
    }

    async fn start(
        &mut self,
        ctx: NodeContext,
        sender: Sender<(Port, IngestionMessage)>,
    ) -> Result<(), BoxedError> {
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let scope = expr_engine.new_scope();
        let new_value = scope
            .eval::<Dynamic>(self.creator.to_string().as_str())
            .map_err(|e| {
                crate::errors::SourceError::FeatureCreator(format!("Failed to evaluate: {e}"))
            })?;
        if new_value.is::<rhai::Map>() {
            if let Ok(AttributeValue::Map(new_value)) = new_value.try_into() {
                let attributes = new_value
                    .iter()
                    .map(|(k, v)| (Attribute::new(k.clone()), v.clone()))
                    .collect::<IndexMap<Attribute, AttributeValue>>();
                let feature = Feature::from(attributes);
                sender
                    .send((
                        DEFAULT_PORT.clone(),
                        IngestionMessage::OperationEvent { feature },
                    ))
                    .await
                    .map_err(|e| crate::errors::SourceError::FeatureCreator(format!("{e:?}")))?;
            } else {
                return Err(
                    SourceError::FeatureCreator("Failed to convert to map".to_string()).into(),
                );
            }
        } else if new_value.is::<rhai::Array>() {
            let array_values = new_value.clone().into_array().map_err(|e| {
                crate::errors::SourceError::FeatureCreator(format!("Failed to convert: {e}"))
            })?;
            for new_value in array_values {
                if let Ok(AttributeValue::Map(new_value)) = new_value.try_into() {
                    let attributes = new_value
                        .iter()
                        .map(|(k, v)| (Attribute::new(k.clone()), v.clone()))
                        .collect::<IndexMap<Attribute, AttributeValue>>();
                    let feature = Feature::from(attributes);
                    sender
                        .send((
                            DEFAULT_PORT.clone(),
                            IngestionMessage::OperationEvent { feature },
                        ))
                        .await
                        .map_err(|e| {
                            crate::errors::SourceError::FeatureCreator(format!("{e:?}"))
                        })?;
                }
            }
            return Ok(());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_runtime::node::SourceFactory;

    #[test]
    fn test_factory_name() {
        let factory = FeatureCreatorFactory::default();
        assert_eq!(factory.name(), "FeatureCreator");
    }

    #[test]
    fn test_factory_description() {
        let factory = FeatureCreatorFactory::default();
        assert!(!factory.description().is_empty());
        assert!(factory.description().contains("Custom Features"));
    }

    #[test]
    fn test_factory_categories() {
        let factory = FeatureCreatorFactory::default();
        assert!(factory.categories().contains(&"Feature"));
    }

    #[test]
    fn test_factory_output_ports() {
        let factory = FeatureCreatorFactory::default();
        assert_eq!(factory.get_output_ports().len(), 1);
    }

    #[test]
    fn test_factory_parameter_schema() {
        let factory = FeatureCreatorFactory::default();
        assert!(factory.parameter_schema().is_some());
    }

    #[test]
    fn test_factory_build_without_params() {
        let factory = FeatureCreatorFactory::default();
        let node_ctx = NodeContext::default();
        let event_hub = EventHub::new(30);
        
        let result = factory.build(node_ctx, event_hub, "test".to_string(), None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_factory_build_with_params() {
        let factory = FeatureCreatorFactory::default();
        let node_ctx = NodeContext::default();
        let event_hub = EventHub::new(30);
        
        let mut params = HashMap::new();
        params.insert("creator".to_string(), serde_json::json!("#{name: 'test'}"));
        
        let result = factory.build(node_ctx, event_hub, "test".to_string(), Some(params), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_feature_creator_name() {
        let creator = FeatureCreator {
            creator: Expr::new("#{name: 'test'}"),
        };
        
        assert_eq!(creator.name(), "FeatureCreator");
    }
}

