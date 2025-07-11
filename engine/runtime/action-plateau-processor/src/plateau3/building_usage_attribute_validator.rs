use std::str::FromStr;
use std::{collections::HashMap, path::Path};

use once_cell::sync::Lazy;
use reearth_flow_common::{uri::Uri, xml};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::PlateauProcessorError;

pub static L_BLDG_ERROR_PORT: Lazy<Port> = Lazy::new(|| Port::new("lBldgError"));
pub static CODE_ERROR_PORT: Lazy<Port> = Lazy::new(|| Port::new("codeError"));

static USAGE_ATTRIBUTES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("uro:majorUsage2", "uro:majorUsage"),
        ("uro:detailedUsage2", "uro:detailedUsage"),
        ("uro:detailedUsage3", "uro:detailedUsage2"),
        ("uro:secondFloorUsage", "uro:groundFloorUsage"),
        ("uro:thirdFloorUsage", "uro:secondFloorUsage"),
        ("uro:basementSecondUsage", "uro:basementFirstUsage"),
    ])
});

static MAJOR_CITY_CODES: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "01100", "04100", "11100", "12100", "13100", "14100", "14130", "14150", "15100", "22100",
        "22130", "23100", "26100", "27100", "27140", "28100", "33100", "34100", "40100", "40130",
        "43100",
    ]
});

#[derive(Debug, Clone, Default)]
pub struct BuildingUsageAttributeValidatorFactory;

impl ProcessorFactory for BuildingUsageAttributeValidatorFactory {
    fn name(&self) -> &str {
        "PLATEAU3.BuildingUsageAttributeValidator"
    }

    fn description(&self) -> &str {
        "This processor validates building usage attributes by checking for the presence of required attributes and ensuring the correctness of city codes. It outputs errors through the lBldgError and codeError ports if any issues are found."
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(BuildingUsageAttributeValidatorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            L_BLDG_ERROR_PORT.clone(),
            CODE_ERROR_PORT.clone(),
            DEFAULT_PORT.clone(),
        ]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let param: BuildingUsageAttributeValidatorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::BuildingUsageAttributeValidatorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::BuildingUsageAttributeValidatorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(
                PlateauProcessorError::BuildingUsageAttributeValidatorFactory(
                    "Missing required parameter `with`".to_string(),
                )
                .into(),
            );
        };
        let mut city_name_to_code = HashMap::new();
        if let Some(codelists_path) = param.codelists_path {
            let dir = Uri::from_str(&codelists_path).map_err(|e| {
                PlateauProcessorError::BuildingUsageAttributeValidatorFactory(format!(
                    "Failed to parse codelists path: {e}"
                ))
            })?;
            let code_list_name = dir
                .join(Path::new("Common_localPublicAuthorities.xml"))
                .map_err(|e| {
                    PlateauProcessorError::BuildingUsageAttributeValidatorFactory(format!(
                        "Failed to join codelists path: {e}"
                    ))
                })?;
            let storage = ctx.storage_resolver.resolve(&code_list_name).map_err(|e| {
                PlateauProcessorError::BuildingUsageAttributeValidatorFactory(format!(
                    "Failed to resolve storage: {e}"
                ))
            })?;
            let common_local_public =
                storage
                    .get_sync(code_list_name.path().as_path())
                    .map_err(|e| {
                        PlateauProcessorError::BuildingUsageAttributeValidatorFactory(format!(
                            "Failed to get storage: {e}"
                        ))
                    })?;
            let document = xml::parse(common_local_public).map_err(|e| {
                PlateauProcessorError::BuildingUsageAttributeValidatorFactory(format!(
                    "Failed to parse xml: {e}"
                ))
            })?;
            let root_node = xml::get_root_readonly_node(&document).map_err(|e| {
                PlateauProcessorError::BuildingUsageAttributeValidatorFactory(format!(
                    "Failed to get root node: {e}"
                ))
            })?;
            let xml_ctx = xml::create_context(&document).map_err(|e| {
                PlateauProcessorError::BuildingUsageAttributeValidatorFactory(format!(
                    "Failed to create context: {e}"
                ))
            })?;
            let nodes =
                xml::find_readonly_nodes_by_xpath(&xml_ctx, ".//gml:Definition", &root_node)
                    .map_err(|e| {
                        PlateauProcessorError::BuildingUsageAttributeValidatorFactory(format!(
                            "Failed to find nodes: {e}"
                        ))
                    })?;
            for node in nodes {
                let mut name = None;
                let mut code = None;
                for child in &node.get_child_nodes() {
                    match xml::get_readonly_node_tag(child).as_str() {
                        "gml:description" => {
                            name = Some(child.get_content());
                        }
                        "gml:name" => {
                            code = Some(child.get_content());
                        }
                        _ => {}
                    }
                }
                if let (Some(name), Some(code)) = (name, code) {
                    city_name_to_code.insert(name, code);
                }
            }
        }
        Ok(Box::new(BuildingUsageAttributeValidator {
            city_name_to_code,
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuildingUsageAttributeValidatorParam {
    codelists_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BuildingUsageAttributeValidator {
    city_name_to_code: HashMap<String, String>,
}

impl Processor for BuildingUsageAttributeValidator {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let attributes = &feature.attributes;
        let Some(AttributeValue::Map(gml_attributes)) =
            attributes.get(&Attribute::new("cityGmlAttributes"))
        else {
            return Err(PlateauProcessorError::BuildingUsageAttributeValidator(
                "cityGmlAttributes key empty".to_string(),
            )
            .into());
        };
        let survey_year = gml_attributes
            .get("uro:buildingDetailAttribute")
            .and_then(|attr| {
                if let AttributeValue::Array(detail_attributes) = attr {
                    detail_attributes.first().and_then(|detail_attribute| {
                        if let AttributeValue::Map(details_attribute) = detail_attribute {
                            details_attribute.get("uro:surveyYear").and_then(|year| {
                                if let AttributeValue::String(survey_year) = year {
                                    Some(survey_year.clone())
                                } else {
                                    None
                                }
                            })
                        } else {
                            None
                        }
                    })
                } else {
                    None
                }
            });
        let mut error_messages = Vec::<String>::new();
        let all_keys = gml_attributes
            .keys()
            .map(|key| key.to_string())
            .collect::<Vec<_>>();
        for (key, value) in USAGE_ATTRIBUTES.iter() {
            if all_keys.contains(&key.to_string()) && !all_keys.contains(&value.to_string()) {
                error_messages.push(format!(
                    "{}年建物利用現況: '{}' が存在しますが '{}' が存在しません。",
                    survey_year.clone().unwrap_or_default(),
                    key,
                    value
                ));
            }
        }
        let city_name = if let Some(AttributeValue::Array(detail_attributes)) =
            gml_attributes.get("uro:buildingIDAttribute")
        {
            detail_attributes.first().and_then(|detail_attribute| {
                if let AttributeValue::Map(details_attribute) = detail_attribute {
                    if let Some(AttributeValue::String(city)) = details_attribute.get("uro:city") {
                        Some(city.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
        } else {
            None
        };
        let city_code_error = if let Some(city) = city_name {
            if let Some(code) = self.city_name_to_code.get(&city) {
                if MAJOR_CITY_CODES.contains(&code.as_str()) {
                    Some(format!("{code} {city}:要修正（区のコードとする）"))
                } else {
                    None
                }
            } else {
                Some(format!("{city}: コードリストに該当なし"))
            }
        } else {
            Some("<未設定>".to_string())
        };
        let mut attributes = feature.attributes.clone();
        let mut ports = Vec::<Port>::new();
        if !error_messages.is_empty() {
            attributes.insert(
                Attribute::new("errors"),
                AttributeValue::Array(
                    error_messages
                        .into_iter()
                        .map(AttributeValue::String)
                        .collect(),
                ),
            );
            ports.push(L_BLDG_ERROR_PORT.clone());
        }
        if let Some(city_code_error) = city_code_error {
            attributes.insert(
                Attribute::new("cityCodeError"),
                AttributeValue::String(city_code_error),
            );
            ports.push(CODE_ERROR_PORT.clone());
        }

        if ports.is_empty() {
            ports.push(DEFAULT_PORT.clone());
        }

        for port in ports {
            fw.send(ctx.new_with_feature_and_port(
                Feature {
                    attributes: attributes.clone(),
                    ..feature.clone()
                },
                port,
            ));
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "BuildingUsageAttributeValidator"
    }
}
