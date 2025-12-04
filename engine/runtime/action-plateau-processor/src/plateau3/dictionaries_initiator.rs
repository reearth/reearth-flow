use std::{collections::HashMap, str::FromStr, sync::Arc};

use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT, REJECTED_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    errors::PlateauProcessorError,
    types::SchemaFeature,
    utils::{create_codelist_map, generate_xpath_to_properties},
};

static ADMIN_CODE_LIST: &str = "Common_localPublicAuthorities.xml";

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Schema {
    features: HashMap<String, Vec<SchemaFeature>>,
    complex_types: HashMap<String, Vec<SchemaFeature>>,
}

#[derive(Debug, Clone, Default)]
pub struct DictionariesInitiatorFactory;

impl ProcessorFactory for DictionariesInitiatorFactory {
    fn name(&self) -> &str {
        "PLATEAU3.DictionariesInitiator"
    }

    fn description(&self) -> &str {
        "Initializes dictionaries for PLATEAU"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
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
        let params: DictionariesInitiatorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::DictionariesInitiatorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::DictionariesInitiatorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(PlateauProcessorError::DictionariesInitiatorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let xpath_to_properties = {
            let schema_json = params.schema_json.clone().ok_or(
                PlateauProcessorError::DictionariesInitiatorFactory(
                    "Missing required parameter `with`".to_string(),
                ),
            )?;
            let dm_geom_to_xml = params
                .extract_dm_geometry_as_xml_fragment
                .unwrap_or_default();
            generate_xpath_to_properties(schema_json, dm_geom_to_xml)?
        };
        let except_feature_types = params.except_feature_types.clone().unwrap_or_default();
        let process = DictionariesInitiator {
            params,
            xpath_to_properties,
            except_feature_types,
            codelists_map: HashMap::new(),
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct DictionariesInitiator {
    params: DictionariesInitiatorParam,
    xpath_to_properties: HashMap<String, HashMap<String, SchemaFeature>>,
    except_feature_types: Vec<String>,
    codelists_map: HashMap<String, HashMap<String, HashMap<String, String>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DictionariesInitiatorParam {
    city_code: Option<String>,
    target_packages: Option<Vec<String>>,
    add_nsprefix_to_feature_types: Option<bool>,
    except_feature_types: Option<Vec<String>>,
    extract_dm_geometry_as_xml_fragment: Option<bool>,
    schema_json: Option<String>,
}

impl Processor for DictionariesInitiator {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        // Codelist dictionary creation
        let dir_codelists = match feature.get("dirCodelists") {
            Some(AttributeValue::String(dir)) => dir,
            v => {
                return Err(PlateauProcessorError::DictionariesInitiator(format!(
                    "No dirCodelists value with {v:?}"
                ))
                .into())
            }
        };
        if !self.codelists_map.contains_key(dir_codelists) {
            let dir = Uri::from_str(dir_codelists).map_err(|e| {
                PlateauProcessorError::DictionariesInitiator(format!(
                    "Cannot parse uri with error = {e:?}"
                ))
            })?;
            if dir.is_dir() {
                let codelists = create_codelist_map(Arc::clone(&ctx.storage_resolver), &dir)?;
                if !codelists.is_empty() {
                    self.codelists_map
                        .insert(dir_codelists.to_string(), codelists);
                }
            }
        }
        let mut result_value = feature.clone();
        // Municipality name acquisition
        if let Some(file) = self.codelists_map.get(dir_codelists) {
            if let Some(city_code) = &self.params.city_code {
                if let Some(name) = file.get(ADMIN_CODE_LIST) {
                    if let Some(city_name) = name.get(city_code) {
                        result_value.insert(
                            Attribute::new("cityName"),
                            AttributeValue::String(city_name.clone()),
                        );
                        result_value.insert(
                            Attribute::new("cityCode"),
                            AttributeValue::String(city_code.clone()),
                        );
                    }
                }
            }
        }

        result_value.insert(
            Attribute::new("featureTypesWithPrefix"),
            AttributeValue::Array(
                self.xpath_to_properties
                    .keys()
                    .map(|v| AttributeValue::String(v.clone()))
                    .collect::<Vec<_>>(),
            ),
        );
        let ftypes = self.xpath_to_properties.keys().collect::<Vec<_>>();
        let out_ftypes = ftypes
            .iter()
            .flat_map(|v| {
                if !self.except_feature_types.contains(v) {
                    if let Some(true) = self.params.add_nsprefix_to_feature_types {
                        Some(AttributeValue::String(v.replace(':', "_")))
                    } else {
                        Some(AttributeValue::String(
                            v.split(':')
                                .map(|v| v.to_string())
                                .nth(1)
                                .unwrap_or_default(),
                        ))
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        result_value.insert(
            Attribute::new("featureTypes"),
            AttributeValue::Array(out_ftypes),
        );
        fw.send(ctx.new_with_feature_and_port(result_value, DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "DictionariesInitiator"
    }
}
