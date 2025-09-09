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

pub static L_04_05_BLDG_ERROR_PORT: Lazy<Port> = Lazy::new(|| Port::new("l0405BldgError"));
pub static CITY_CODE_ERROR_PORT: Lazy<Port> = Lazy::new(|| Port::new("cityCodeError"));

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
        "PLATEAU4.BuildingUsageAttributeValidator"
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
            L_04_05_BLDG_ERROR_PORT.clone(),
            CITY_CODE_ERROR_PORT.clone(),
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
        let mut city_code_to_name = HashMap::new();
        let codelists = param.codelists.ok_or(
            PlateauProcessorError::BuildingUsageAttributeValidatorFactory(
                "codelistsPath is required".to_string(),
            ),
        )?;
        
        let dir = Uri::from_str(&codelists).map_err(|e| {
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
                city_code_to_name.insert(code, name);
            }
        }
        Ok(Box::new(BuildingUsageAttributeValidator {
            city_code_to_name,
        }))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BuildingUsageAttributeValidatorParam {
    codelists: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BuildingUsageAttributeValidator {
    city_code_to_name: HashMap<String, String>,
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
        let survey_year = {
            let Some(attr) = gml_attributes.get("uro:BuildingDetailAttribute") else {
                // No BuildingDetailAttribute means no usage attributes, so no error.
                return Ok(())
            };
            if let AttributeValue::Map(attr) = attr {
                let Some(year_attr) = attr.get("uro:surveyYear") else {
                    return Err(PlateauProcessorError::BuildingUsageAttributeValidator(
                        "uro:surveyYear must be specified as per cityGML specification, but it is not.".to_string(),
                    ))?;
                };
                if let AttributeValue::String(year) = year_attr {
                    year
                } else {
                    return Err(PlateauProcessorError::BuildingUsageAttributeValidator(
                        "uro:surveyYear must be a string, but it is not".to_string(),
                    ))?;
                }
            } else {
                return Err(PlateauProcessorError::BuildingUsageAttributeValidator(
                    "uro:BuildingDetailAttribute must be a map, but it is not".to_string(),
                ))?;
            }
        };
        let mut error_messages = Vec::<String>::new();
        // Check usage attributes only inside uro:BuildingDetailAttribute
        if let Some(AttributeValue::Map(building_detail_attr)) = 
            gml_attributes.get("uro:BuildingDetailAttribute") {
            let detail_keys: Vec<String> = building_detail_attr
                .keys()
                .map(|key| key.to_string())
                .collect();
            
            for (key, value) in USAGE_ATTRIBUTES.iter() {
                if detail_keys.contains(&key.to_string()) && !detail_keys.contains(&value.to_string()) {
                    error_messages.push(format!(
                        "{}年建物利用現況: '{}' が存在しますが '{}' が存在しません。",
                        survey_year,
                        key,
                        value
                    ));
                }
            }
        }
        let Some(AttributeValue::Map(id_attr)) =
            gml_attributes.get("uro:BuildingIDAttribute")
         else {
            Err(PlateauProcessorError::BuildingUsageAttributeValidator(
                "uro:BuildingIDAttribute must be specified as per cityGML specification, but it is not.".to_string()
            ))?
        };

        let city_code_error = if let Some(city_code) = id_attr.get("uro:city_code") {
            let AttributeValue::String(city_code) = city_code else {
                return Err(PlateauProcessorError::BuildingUsageAttributeValidator(
                    "uro:city must be a string, but it is not".to_string(),
                ))?;
            };
            if let Some(city_name) = self.city_code_to_name.get(city_code) {
                if MAJOR_CITY_CODES.contains(&city_code.as_str()) {
                    Some(format!("{city_code} {city_name}:要修正（区のコードとする）"))
                } else {
                    None
                }
            } else {
                Some(format!("{city_code}: コードリストに該当なし"))
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
            ports.push(L_04_05_BLDG_ERROR_PORT.clone());
        }
        if let Some(city_code_error) = city_code_error {
            attributes.insert(
                Attribute::new("cityCodeError"),
                AttributeValue::String(city_code_error),
            );
            ports.push(CITY_CODE_ERROR_PORT.clone());
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
