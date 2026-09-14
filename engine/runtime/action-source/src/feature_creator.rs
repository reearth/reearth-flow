use std::collections::HashMap;

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::NodeContext,
    node::{IngestionMessage, Port, Source, SourceFactory, FEATURES_PORT},
};
use reearth_flow_types::{AttributeValue, Code, CodeType, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::Sender;

use crate::errors::SourceError;

#[derive(Debug, Clone, Default)]
pub struct FeatureCreatorFactory;

impl SourceFactory for FeatureCreatorFactory {
    fn name(&self) -> &str {
        "Feature Creator"
    }

    fn description(&self) -> &str {
        "Creates features from a script expression that returns one or more attribute maps."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(FeatureCreator))
    }

    fn categories(&self) -> &[&'static str] {
        &["Input"]
    }

    fn tags(&self) -> &[&'static str] {
        &["scripting"]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
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
            .eval_variables_only(ctx.variables.clone())
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
    /// Script expression that returns a map (single feature) or an array of maps (multiple features). Each map holds feature attributes as key-value pairs.
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
        "Feature Creator"
    }

    async fn serialize_state(&self) -> Result<Vec<u8>, BoxedError> {
        Ok(vec![])
    }

    async fn start(
        &mut self,
        _ctx: NodeContext,
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
                let feature = to_feature(new_value);
                sender
                    .send((
                        FEATURES_PORT.clone(),
                        IngestionMessage::OperationEvent { feature },
                    ))
                    .await
                    .map_err(|e| SourceError::FeatureCreator(format!("{e:?}")))?;
            }
        } else if new_value.is::<rhai::Array>() {
            let array_values = new_value.clone().into_array().map_err(|e| {
                crate::errors::SourceError::FeatureCreator(format!("Failed to convert: {e}"))
            })?;
            for new_value in array_values {
                if let Ok(AttributeValue::Map(new_value)) = new_value.try_into() {
                    let feature = to_feature(new_value);
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

/// Builds a Feature from created attributes. A `__feature_type` key, if present, is
/// consumed to set the feature's metadata feature_type (never surfaced as an attribute) —
/// the only way a FeatureCreator-made feature can carry one, since Feature::from(attributes)
/// otherwise leaves it unset.
fn to_feature(new_value: HashMap<String, AttributeValue>) -> Feature {
    let mut attributes = new_value
        .iter()
        .map(|(k, v)| (Attribute::new(k.clone()), v.clone()))
        .collect::<IndexMap<Attribute, AttributeValue>>();
    let feature_type = attributes
        .shift_remove(&Attribute::new("__feature_type".to_string()))
        .and_then(|v| v.as_string());
    let mut feature = Feature::from(attributes);
    if let Some(feature_type) = feature_type {
        feature.update_feature_type(feature_type);
    }
    feature
}
