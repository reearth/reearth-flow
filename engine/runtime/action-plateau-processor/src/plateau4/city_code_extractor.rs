use crate::types::dictionary::Dictionary;

use super::errors::PlateauProcessorError;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone, Default)]
pub struct CityCodeExtractorFactory;

impl ProcessorFactory for CityCodeExtractorFactory {
    fn name(&self) -> &str {
        "PLATEAU4.CityCodeExtractor"
    }

    fn description(&self) -> &str {
        "Extracts Codelist"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(CityCodeExtractorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
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
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: CityCodeExtractorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::CityCodeExtractorFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::CityCodeExtractorFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(PlateauProcessorError::CityCodeExtractorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let process = CityCodeExtractor {
            city_code_attribute: params.city_code_attribute,
            codelists_path_attribute: params.codelists_path_attribute,
        };
        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CityCodeExtractorParam {
    city_code_attribute: Attribute,
    codelists_path_attribute: Attribute,
}

#[derive(Debug, Clone)]
pub(crate) struct CityCodeExtractor {
    city_code_attribute: Attribute,
    codelists_path_attribute: Attribute,
}

impl Processor for CityCodeExtractor {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let AttributeValue::String(city_code) = feature
            .attributes
            .get(&self.city_code_attribute)
            .ok_or(PlateauProcessorError::CityCodeExtractor(
                "cityCode attribute empty".to_string(),
            ))
            .cloned()?
        else {
            return Err(PlateauProcessorError::CityCodeExtractor(
                "cityCode attribute error".to_string(),
            )
            .into());
        };
        let AttributeValue::String(codelists_path) = feature
            .attributes
            .get(&self.codelists_path_attribute)
            .ok_or(PlateauProcessorError::CityCodeExtractor(
                "codelists path attribute empty".to_string(),
            ))
            .cloned()?
        else {
            return Err(PlateauProcessorError::CityCodeExtractor(
                "codelists path attribute error".to_string(),
            )
            .into());
        };
        let codelists_path = Uri::from_str(&codelists_path)?;
        let authorities_path = codelists_path.join("Common_localPublicAuthorities.xml")?;
        let storage = ctx.storage_resolver.resolve(&authorities_path)?;
        let bytes = storage.get_sync(&authorities_path.as_path())?;
        let dic: Dictionary = quick_xml::de::from_str(&String::from_utf8(bytes.to_vec())?)?;
        let city_name = dic
            .entries
            .iter()
            .find(|entry| entry.definition.name.value == city_code)
            .map(|entry| entry.definition.description.clone());
        let mut feature = feature.clone();
        if let Some(city_name) = city_name {
            feature.insert("cityName", AttributeValue::String(city_name.value));
        }
        fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "CityCodeExtractor"
    }
}
