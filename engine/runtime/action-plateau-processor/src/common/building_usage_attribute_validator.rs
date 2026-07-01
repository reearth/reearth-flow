use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use once_cell::sync::Lazy;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_types::{Attribute, AttributeValue, Code, CompiledCode, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::dictionary::Dictionary;

use super::errors::PlateauProcessorError;
use super::PlateauProfile;

static L_04_05_BLDG_ERROR_PORT: Lazy<Port> = Lazy::new(|| Port::new("l0405BldgError"));
static CITY_CODE_ERROR_PORT: Lazy<Port> = Lazy::new(|| Port::new("cityCodeError"));

/// Derived usage attribute -> its required parent attribute. A building that
/// carries the derived attribute without its parent is a dependency violation
/// (L-bldg-04,05).
static USAGE_ATTRIBUTES: Lazy<Vec<(&'static str, &'static str)>> = Lazy::new(|| {
    vec![
        ("uro:majorUsage2", "uro:majorUsage"),
        ("uro:detailedUsage2", "uro:detailedUsage"),
        ("uro:detailedUsage3", "uro:detailedUsage2"),
        ("uro:secondFloorUsage", "uro:groundFloorUsage"),
        ("uro:thirdFloorUsage", "uro:secondFloorUsage"),
        ("uro:basementSecondUsage", "uro:basementFirstUsage"),
    ]
});

/// Designated-city (政令指定都市) municipality codes. A building whose city code
/// is one of these must be refined to the corresponding ward code, so the code
/// is reported as an error.
static MAJOR_CITY_CODES: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        "01100", "04100", "11100", "12100", "13100", "14100", "14130", "14150", "15100", "22100",
        "22130", "23100", "26100", "27100", "27140", "28100", "33100", "34100", "40100", "40130",
        "43100",
    ]
});

#[derive(Debug, Clone)]
pub(crate) struct BuildingUsageAttributeValidatorFactory {
    name: String,
    is_citygml3: bool,
}

impl BuildingUsageAttributeValidatorFactory {
    pub(crate) fn new(profile: &PlateauProfile) -> Self {
        Self {
            name: profile.action_name("BuildingUsageAttributeValidator"),
            is_citygml3: profile.is_citygml3(),
        }
    }
}

impl ProcessorFactory for BuildingUsageAttributeValidatorFactory {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Validates building usage attributes (L-bldg-04,05 dependency violations) and the city code against the Common_localPublicAuthorities code list. Usage errors are emitted on the l0405BldgError port and city-code errors on the cityCodeError port."
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
        _ctx: NodeContext,
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

        let codelists_path_expr = param.codelists_path.compile().map_err(|e| {
            PlateauProcessorError::BuildingUsageAttributeValidatorFactory(format!(
                "Failed to compile codelists_path expression: {e}"
            ))
        })?;

        Ok(Box::new(BuildingUsageAttributeValidator {
            codelists_path_expr,
            city_code_to_name: None,
            is_citygml3: self.is_citygml3,
        }))
    }
}

/// # BuildingUsageAttributeValidator Parameters
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BuildingUsageAttributeValidatorParam {
    /// Expression evaluating to the PLATEAU codelists directory path.
    codelists_path: Code,
}

#[derive(Debug, Clone)]
pub(crate) struct BuildingUsageAttributeValidator {
    codelists_path_expr: CompiledCode,
    /// Lazily built code -> name map from Common_localPublicAuthorities.xml.
    city_code_to_name: Option<HashMap<String, String>>,
    /// CityGML 3.0 (i-UR 4.0) vs CityGML 2.0 (i-UR 3.x) attribute layout.
    is_citygml3: bool,
}

impl Processor for BuildingUsageAttributeValidator {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        // Build the code list map once, on the first feature.
        if self.city_code_to_name.is_none() {
            self.city_code_to_name = Some(build_city_code_to_name(
                &ctx.feature,
                &self.codelists_path_expr,
                ctx.env_vars.clone(),
                &ctx.storage_resolver,
            )?);
        }
        if self.is_citygml3 {
            self.process_citygml3(&ctx, fw)
        } else {
            self.process_citygml2(&ctx, fw)
        }
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "BuildingUsageAttributeValidator"
    }
}

impl BuildingUsageAttributeValidator {
    /// CityGML 2.0 / i-UR 3.x layout: i-UR attributes nest under a
    /// `cityGmlAttributes` map, and each i-UR attribute is an array whose first
    /// element is the attribute map. Mirrors the original plateau4 behaviour.
    fn process_citygml2(
        &self,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let city_code_to_name = self.city_code_to_name.as_ref().unwrap();

        let Some(AttributeValue::Map(gml_attributes)) =
            feature.attributes.get(&Attribute::new("cityGmlAttributes"))
        else {
            return Err(PlateauProcessorError::BuildingUsageAttributeValidator(
                "cityGmlAttributes key empty".to_string(),
            )
            .into());
        };

        // A building without uro:BuildingDetailAttribute carries no usage or city
        // checks in this layout (early return, matching plateau4).
        let Some(detail_value) = gml_attributes.get("uro:BuildingDetailAttribute") else {
            return Ok(());
        };
        let AttributeValue::Array(detail_array) = detail_value else {
            return Err(PlateauProcessorError::BuildingUsageAttributeValidator(
                "uro:BuildingDetailAttribute must be an array, but it is not".to_string(),
            )
            .into());
        };
        let Some(AttributeValue::Map(building_detail_attr)) = detail_array.first() else {
            return Err(PlateauProcessorError::BuildingUsageAttributeValidator(
                "uro:BuildingDetailAttribute must be an array with a map element, but it is not"
                    .to_string(),
            )
            .into());
        };
        let survey_year =
            match building_detail_attr.get("uro:surveyYear") {
                Some(AttributeValue::String(year)) => year.clone(),
                Some(_) => {
                    return Err(PlateauProcessorError::BuildingUsageAttributeValidator(
                        "uro:surveyYear must be a string, but it is not".to_string(),
                    )
                    .into())
                }
                None => return Err(PlateauProcessorError::BuildingUsageAttributeValidator(
                    "uro:surveyYear must be specified as per cityGML specification, but it is not."
                        .to_string(),
                )
                .into()),
            };

        let error_messages = usage_violation_messages(building_detail_attr, &survey_year);

        let id_attr = match gml_attributes.get("uro:BuildingIDAttribute") {
            Some(AttributeValue::Array(id_array)) => match id_array.first() {
                Some(AttributeValue::Map(id_attr)) => id_attr,
                _ => {
                    return Err(PlateauProcessorError::BuildingUsageAttributeValidator(
                        "uro:BuildingIDAttribute must be an array with a map element, but it is not"
                            .to_string(),
                    )
                    .into())
                }
            },
            _ => {
                return Err(PlateauProcessorError::BuildingUsageAttributeValidator(
                    "uro:BuildingIDAttribute must be specified as per cityGML specification, but it is not.".to_string(),
                )
                .into())
            }
        };
        let city_code_error = match id_attr.get("uro:city_code") {
            Some(AttributeValue::String(city_code)) => {
                classify_city_code(city_code, city_code_to_name)
            }
            Some(_) => {
                return Err(PlateauProcessorError::BuildingUsageAttributeValidator(
                    "uro:city must be a string, but it is not".to_string(),
                )
                .into())
            }
            None => Some("<未設定>".to_string()),
        };

        self.emit(ctx, fw, error_messages, city_code_error);
        Ok(())
    }

    /// CityGML 3.0 / i-UR 4.0 layout: i-UR attributes hang off one or more
    /// `bldg:adeOfAbstractBuilding` hooks (a single hook is stored as a Map,
    /// multiple as an Array), the survey year is an ISO date, and the city code
    /// is `uro:city`.
    fn process_citygml3(
        &self,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;
        let city_code_to_name = self.city_code_to_name.as_ref().unwrap();

        let ade = feature
            .attributes
            .get(&Attribute::new("bldg:adeOfAbstractBuilding"));

        let mut error_messages = Vec::<String>::new();
        if let Some(building_detail_attr) = ade_child_map(ade, "uro:BuildingDetailAttribute") {
            let survey_year = match building_detail_attr.get("uro:surveyYear") {
                // i-UR 4.0 surveyYear is an ISO date (e.g. "2020-04-01"); take the year.
                Some(AttributeValue::String(date)) => {
                    date.split('-').next().unwrap_or("").to_string()
                }
                _ => String::new(),
            };
            error_messages = usage_violation_messages(building_detail_attr, &survey_year);
        }

        // Only buildings that carry a uro:BuildingIDAttribute are in scope.
        let city_code_error = match ade_child_map(ade, "uro:BuildingIDAttribute") {
            Some(id_attr) => match id_attr.get("uro:city") {
                Some(AttributeValue::String(city_code)) if !city_code.is_empty() => {
                    classify_city_code(city_code, city_code_to_name)
                }
                _ => Some("<未設定>".to_string()),
            },
            None => None,
        };

        self.emit(ctx, fw, error_messages, city_code_error);
        Ok(())
    }

    /// Sends the feature on the error ports it qualifies for (and DEFAULT if
    /// none), attaching `errors`/`cityCodeError` attributes. The usage error is
    /// carried as the `errors` array (CityGML 2.0) or the `usageError` scalar
    /// (CityGML 3.0) to match each generation's downstream CSV wiring.
    fn emit(
        &self,
        ctx: &ExecutorContext,
        fw: &ProcessorChannelForwarder,
        error_messages: Vec<String>,
        city_code_error: Option<String>,
    ) {
        let feature = &ctx.feature;
        let mut attributes = (*feature.attributes).clone();
        let mut ports = Vec::<Port>::new();
        if !error_messages.is_empty() {
            if self.is_citygml3 {
                attributes.insert(
                    Attribute::new("usageError"),
                    AttributeValue::String(error_messages.join("\n")),
                );
            } else {
                attributes.insert(
                    Attribute::new("errors"),
                    AttributeValue::Array(
                        error_messages
                            .into_iter()
                            .map(AttributeValue::String)
                            .collect(),
                    ),
                );
            }
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

        let attributes = Arc::new(attributes);
        for port in ports {
            fw.send(ctx.new_with_feature_and_port(
                Feature {
                    attributes: Arc::clone(&attributes),
                    ..feature.clone()
                },
                port,
            ));
        }
    }
}

/// Builds the L-bldg-04,05 violation messages for a `uro:BuildingDetailAttribute`
/// map: one message per derived attribute present without its parent.
fn usage_violation_messages(
    building_detail_attr: &HashMap<String, AttributeValue>,
    survey_year: &str,
) -> Vec<String> {
    USAGE_ATTRIBUTES
        .iter()
        .filter(|(derived, parent)| {
            building_detail_attr.contains_key(*derived)
                && !building_detail_attr.contains_key(*parent)
        })
        .map(|(derived, parent)| {
            format!("{survey_year}年建物利用現況: '{derived}' が存在しますが '{parent}' が存在しません。")
        })
        .collect()
}

/// Classifies a present city code: `None` when valid, otherwise the error message
/// (missing from the code list, or a designated-city code that needs a ward code).
fn classify_city_code(
    city_code: &str,
    city_code_to_name: &HashMap<String, String>,
) -> Option<String> {
    if let Some(city_name) = city_code_to_name.get(city_code) {
        if MAJOR_CITY_CODES.contains(&city_code) {
            Some(format!(
                "{city_code} {city_name}:要修正（区のコードとする）"
            ))
        } else {
            None
        }
    } else {
        Some(format!("{city_code}: コードリストに該当なし"))
    }
}

/// Returns the inner map of an i-UR attribute found under
/// `bldg:adeOfAbstractBuilding`, which may itself be a single Map or an Array of
/// Maps depending on how many ADE hooks the building carries. When the attribute
/// is stored as an Array, the first Map element is used.
fn ade_child_map<'a>(
    ade: Option<&'a AttributeValue>,
    key: &str,
) -> Option<&'a HashMap<String, AttributeValue>> {
    let child = match ade? {
        AttributeValue::Map(map) => map.get(key),
        AttributeValue::Array(items) => items.iter().find_map(|item| match item {
            AttributeValue::Map(map) => map.get(key),
            _ => None,
        }),
        _ => None,
    }?;
    match child {
        AttributeValue::Map(map) => Some(map),
        AttributeValue::Array(items) => items.iter().find_map(|item| match item {
            AttributeValue::Map(map) => Some(map),
            _ => None,
        }),
        _ => None,
    }
}

/// Reads `Common_localPublicAuthorities.xml` from the codelists directory and
/// returns a map of city code -> city name.
fn build_city_code_to_name(
    feature: &Feature,
    codelists_path_expr: &CompiledCode,
    env_vars: Arc<serde_json::Map<String, serde_json::Value>>,
    storage_resolver: &Arc<StorageResolver>,
) -> Result<HashMap<String, String>, BoxedError> {
    let codelists_path = codelists_path_expr
        .eval_string(feature, env_vars)
        .map_err(|e| {
            PlateauProcessorError::BuildingUsageAttributeValidator(format!(
                "Failed to evaluate codelists_path expression: {e:?}"
            ))
        })?;
    let dir = Uri::from_str(&codelists_path).map_err(|e| {
        PlateauProcessorError::BuildingUsageAttributeValidator(format!(
            "Failed to parse codelists path: {e}"
        ))
    })?;
    let authorities_path = dir.join("Common_localPublicAuthorities.xml").map_err(|e| {
        PlateauProcessorError::BuildingUsageAttributeValidator(format!(
            "Failed to join codelists path: {e}"
        ))
    })?;
    let storage = storage_resolver.resolve(&authorities_path).map_err(|e| {
        PlateauProcessorError::BuildingUsageAttributeValidator(format!(
            "Failed to resolve storage: {e}"
        ))
    })?;
    let bytes = storage
        .get_sync(authorities_path.path().as_path())
        .map_err(|e| {
            PlateauProcessorError::BuildingUsageAttributeValidator(format!(
                "Failed to read code list: {e}"
            ))
        })?;
    let dictionary: Dictionary = quick_xml::de::from_str(&String::from_utf8(bytes.to_vec())?)
        .map_err(|e| {
            PlateauProcessorError::BuildingUsageAttributeValidator(format!(
                "Failed to parse code list: {e}"
            ))
        })?;
    Ok(dictionary
        .entries
        .iter()
        .map(|entry| {
            (
                entry.definition.name.value.clone(),
                entry.definition.description.value.clone(),
            )
        })
        .collect())
}
