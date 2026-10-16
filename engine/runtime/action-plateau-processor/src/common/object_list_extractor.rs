//! Objectlist extractor, shared across PLATEAU generations.
//!
//! Reads the objectlist workbook named by the input feature and attaches the
//! per-package `featureTypes` and `objectList` attributes consumed by the XML
//! Fragmenter and the missing-attribute detector. The workbook format differs
//! by generation and is read by an [`ObjectListReader`].

use std::{collections::HashMap, fmt::Debug, str::FromStr};

use super::errors::PlateauProcessorError;
use super::PlateauProfile;
use crate::object_list::{self, FeatureTypes, ObjectListMap};
use bytes::Bytes;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::Attribute;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Generation-specific reader of an objectlist workbook.
pub(crate) trait ObjectListReader: Send + Sync + Debug {
    fn read(&self, bytes: Bytes) -> object_list::Result<(FeatureTypes, ObjectListMap)>;
}

#[derive(Debug, Clone)]
pub(crate) struct ObjectListExtractorFactory {
    name: String,
    reader: &'static dyn ObjectListReader,
}

impl ObjectListExtractorFactory {
    pub(crate) fn new(profile: &PlateauProfile, reader: &'static dyn ObjectListReader) -> Self {
        Self {
            name: profile.action_name("ObjectListExtractor"),
            reader,
        }
    }
}

impl ProcessorFactory for ObjectListExtractorFactory {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Extract object list"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(ObjectListExtractorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
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
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: ObjectListExtractorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::ObjectListExtractorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::ObjectListExtractorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(PlateauProcessorError::ObjectListExtractorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let process = ObjectListExtractor {
            object_list_path_attribute: params.object_list_path_attribute,
            reader: self.reader,
        };
        Ok(Box::new(process))
    }
}

/// # ObjectListExtractor Parameters
///
/// Configuration for extracting object lists from PLATEAU data.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ObjectListExtractorParam {
    object_list_path_attribute: Attribute,
}

#[derive(Debug, Clone)]
pub(crate) struct ObjectListExtractor {
    object_list_path_attribute: Attribute,
    reader: &'static dyn ObjectListReader,
}

impl Processor for ObjectListExtractor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();
        let object_list_path = feature
            .attributes
            .get(&self.object_list_path_attribute)
            .and_then(|v| v.as_string())
            .ok_or(PlateauProcessorError::ObjectListExtractor(
                "objectListPath attribute empty".to_string(),
            ))?;
        let object_list_path = Uri::from_str(object_list_path.as_str())
            .map_err(|e| PlateauProcessorError::ObjectListExtractor(format!("{e}")))?;
        let storage_resolver = ctx.storage_resolver.clone();
        let storage = storage_resolver.resolve(&object_list_path).map_err(|e| {
            PlateauProcessorError::ObjectListExtractor(format!(
                "Failed to resolve objectList path: {e}"
            ))
        })?;
        let bytes = storage
            .get_sync(object_list_path.path().as_path())
            .map_err(|e| {
                PlateauProcessorError::ObjectListExtractor(format!(
                    "Failed to get objectList file: {e}"
                ))
            })?;
        let (feature_types, object_list) = self.reader.read(bytes).map_err(|e| {
            PlateauProcessorError::ObjectListExtractor(format!(
                "Failed to parse objectList file: {e}"
            ))
        })?;
        feature.insert("featureTypes", feature_types.into());
        feature.insert("objectList", object_list.into());
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
        "ObjectListExtractor"
    }
}
