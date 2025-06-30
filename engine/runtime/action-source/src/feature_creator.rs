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
        "Creates features from expressions"
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

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeatureCreator {
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
