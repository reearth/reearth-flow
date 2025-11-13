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
        "Extracts city code information from PLATEAU4 codelists for local public authorities"
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
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::CityCodeExtractorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
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

/// # CityCodeExtractor Parameters
///
/// Configuration for extracting PLATEAU4 city code information from codelists.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CityCodeExtractorParam {
    /// Attribute containing the city code to look up in codelists
    city_code_attribute: Attribute,
    /// Attribute containing the path to the PLATEAU codelists directory
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
            .ok_or(PlateauProcessorError::CityCodeExtractor(format!(
                "cityCode attribute empty: {}",
                self.city_code_attribute
            )))
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

#[cfg(test)]
mod tests {
    use super::*;
    use reearth_flow_runtime::node::ProcessorFactory;

    #[test]
    fn test_factory_name() {
        let factory = CityCodeExtractorFactory::default();
        assert_eq!(factory.name(), "PLATEAU4.CityCodeExtractor");
    }

    #[test]
    fn test_factory_description() {
        let factory = CityCodeExtractorFactory::default();
        assert!(!factory.description().is_empty());
        assert!(factory.description().contains("city code"));
    }

    #[test]
    fn test_factory_categories() {
        let factory = CityCodeExtractorFactory::default();
        assert!(factory.categories().contains(&"PLATEAU"));
    }

    #[test]
    fn test_factory_ports() {
        let factory = CityCodeExtractorFactory::default();
        assert_eq!(factory.get_input_ports().len(), 1);
        assert_eq!(factory.get_output_ports().len(), 1);
    }

    #[test]
    fn test_factory_parameter_schema() {
        let factory = CityCodeExtractorFactory::default();
        assert!(factory.parameter_schema().is_some());
    }

    #[test]
    fn test_factory_build_without_params() {
        let factory = CityCodeExtractorFactory::default();
        let node_ctx = NodeContext::default();
        let event_hub = EventHub::new(30);
        
        let result = factory.build(node_ctx, event_hub, "test".to_string(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_factory_build_with_params() {
        let factory = CityCodeExtractorFactory::default();
        let node_ctx = NodeContext::default();
        let event_hub = EventHub::new(30);
        
        let mut params = HashMap::new();
        params.insert("cityCodeAttribute".to_string(), serde_json::json!("cityCode"));
        params.insert("codelistsPathAttribute".to_string(), serde_json::json!("codelistsPath"));
        
        let result = factory.build(node_ctx, event_hub, "test".to_string(), Some(params));
        assert!(result.is_ok());
    }

    #[test]
    fn test_processor_name() {
        let processor = CityCodeExtractor {
            city_code_attribute: Attribute::new("cityCode"),
            codelists_path_attribute: Attribute::new("codelistsPath"),
        };
        
        assert_eq!(processor.name(), "CityCodeExtractor");
    }
}

