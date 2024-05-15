use std::{collections::HashMap, fmt::Debug};

use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::NodeContext,
    node::{IngestionMessage, Port, Source, SourceFactory, DEFAULT_PORT},
};
use serde_json::Value;
use tokio::sync::mpsc::Sender;

use crate::errors::UniversalSourceError;

#[async_trait::async_trait]
#[typetag::serde(tag = "action", content = "with")]
pub trait UniversalSource: Send + Sync + Debug + UniversalSourceClone {
    async fn initialize(&self, ctx: NodeContext);
    async fn serialize_state(&self) -> Result<Vec<u8>, BoxedError>;
    async fn start(
        &mut self,
        ctx: NodeContext,
        sender: Sender<(Port, IngestionMessage)>,
    ) -> Result<(), BoxedError>;
}

pub trait UniversalSourceClone {
    fn clone_box(&self) -> Box<dyn UniversalSource>;
}

impl<T> UniversalSourceClone for T
where
    T: 'static + UniversalSource + Clone,
{
    fn clone_box(&self) -> Box<dyn UniversalSource> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn UniversalSource> {
    fn clone(&self) -> Box<dyn UniversalSource> {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
pub struct UniversalOperator(Box<dyn UniversalSource>);

#[async_trait::async_trait]
impl Source for UniversalOperator {
    async fn initialize(&self, ctx: NodeContext) {
        self.0.initialize(ctx).await
    }

    async fn serialize_state(&self) -> Result<Vec<u8>, BoxedError> {
        self.0.serialize_state().await
    }

    async fn start(
        &mut self,
        ctx: NodeContext,
        sender: Sender<(Port, IngestionMessage)>,
    ) -> Result<(), BoxedError> {
        self.0.start(ctx, sender).await
    }
}

#[derive(Debug, Clone, Default)]
pub struct UniversalSourceFactory {}

impl SourceFactory for UniversalSourceFactory {
    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }
    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        action: String,
        with: Option<HashMap<String, Value>>,
        _state: Option<Vec<u8>>,
    ) -> Result<Box<dyn Source>, BoxedError> {
        let mut params = vec![(
            "action".to_owned(),
            serde_json::Value::String(action.to_string()),
        )];

        if let Some(with) = with {
            let value = serde_json::to_value(with).map_err(|e| {
                UniversalSourceError::BuildFactory(format!("Failed to serialize with: {}", e))
            })?;
            params.push(("with".to_owned(), value));
        }

        let processor: serde_json::Result<Box<dyn UniversalSource>> =
            serde_json::from_value(serde_json::Value::Object(
                params
                    .clone()
                    .into_iter()
                    .collect::<serde_json::Map<_, _>>(),
            ));
        let processor = processor.map_err(|e| {
            UniversalSourceError::BuildFactory(format!("Failed to deserialize processor: {}", e))
        })?;
        Ok(Box::new(UniversalOperator(processor.clone())))
    }
}
