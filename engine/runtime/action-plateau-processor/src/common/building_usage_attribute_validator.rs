//! Building usage-attribute validator (L-bldg-04,05) and city-code check,
//! shared across PLATEAU generations.
//!
//! Two independent checks run per building feature:
//!
//! - **usage dependency (L-bldg-04,05)**: a derived usage attribute present
//!   without its required parent attribute is a violation.
//! - **city code**: the building's municipality code is looked up in the
//!   `Common_localPublicAuthorities` code list; a code missing from the list, or
//!   a designated-city code that should have been refined to a ward code, is an
//!   error.
//!
//! The generation-independent orchestration (code-list load, port emission,
//! violation/classification helpers) lives here, and the findings are emitted
//! identically for every generation (usage errors as the `errors` array,
//! city-code errors as the `cityCodeError` scalar). The only generation-specific
//! seam — how the i-UR attributes are laid out in the feature (CityGML 2.0 nests
//! them under a `cityGmlAttributes` map; CityGML 3.0 hangs them off
//! `bldg:adeOfAbstractBuilding`) and where the survey year and city code come
//! from — is injected as a [`BuildingUsageAttributeStrategy`] trait object.

use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use std::sync::Arc;

use once_cell::sync::Lazy;
use reearth_flow_common::uri::Uri;
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
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

/// Result of analyzing one building feature: the L-bldg-04,05 usage-violation
/// messages and, when the building is in scope for the city-code check, its
/// classification error (`None` when the code is valid).
#[derive(Debug, Default)]
pub(crate) struct UsageAnalysis {
    pub usage_messages: Vec<String>,
    pub city_code_error: Option<String>,
}

/// Generation-specific seam for the building usage-attribute validator.
///
/// Only the analysis differs between generations (how the i-UR attributes are
/// laid out in the feature); the findings, once produced, are emitted
/// identically, so this trait carries the single [`analyze`] method.
///
/// [`analyze`]: BuildingUsageAttributeStrategy::analyze
pub(crate) trait BuildingUsageAttributeStrategy: Send + Sync + Debug {
    /// Analyze one building feature against the resolved code list.
    ///
    /// Returns `Ok(None)` to drop the feature from all output entirely (CityGML
    /// 2.0 emits nothing for a building that carries no
    /// `uro:BuildingDetailAttribute`).
    fn analyze(
        &self,
        feature: &Feature,
        city_code_to_name: &HashMap<String, String>,
    ) -> Result<Option<UsageAnalysis>, BoxedError>;
}

#[derive(Debug, Clone)]
pub(crate) struct BuildingUsageAttributeValidatorFactory {
    name: String,
    strategy: &'static dyn BuildingUsageAttributeStrategy,
}

impl BuildingUsageAttributeValidatorFactory {
    pub(crate) fn new(
        profile: &PlateauProfile,
        strategy: &'static dyn BuildingUsageAttributeStrategy,
    ) -> Self {
        Self {
            name: profile.action_name("BuildingUsageAttributeValidator"),
            strategy,
        }
    }
}

impl ProcessorFactory for BuildingUsageAttributeValidatorFactory {
    fn name(&self) -> &str {
        &self.name
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
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![
            L_04_05_BLDG_ERROR_PORT.clone(),
            CITY_CODE_ERROR_PORT.clone(),
            FEATURES_PORT.clone(),
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
            strategy: self.strategy,
        }))
    }
}

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
    /// Generation-specific attribute layout / output seam.
    strategy: &'static dyn BuildingUsageAttributeStrategy,
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
        let city_code_to_name = self.city_code_to_name.as_ref().unwrap();

        let Some(analysis) = self.strategy.analyze(&ctx.feature, city_code_to_name)? else {
            return Ok(());
        };
        self.emit(&ctx, fw, analysis);
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
        "BuildingUsageAttributeValidator"
    }
}

impl BuildingUsageAttributeValidator {
    /// Sends the feature on the error ports it qualifies for (and DEFAULT if
    /// none), attaching the usage-error attribute (via the strategy) and the
    /// `cityCodeError` attribute.
    fn emit(&self, ctx: &ExecutorContext, fw: &ProcessorChannelForwarder, analysis: UsageAnalysis) {
        let feature = &ctx.feature;
        let mut attributes = (*feature.attributes).clone();
        let mut ports = Vec::<Port>::new();
        if !analysis.usage_messages.is_empty() {
            // Carried as the `errors` array; the downstream CSV writer renders a
            // multi-message array comma-joined (`msg1, msg2`).
            attributes.insert(
                Attribute::new("errors"),
                AttributeValue::Array(
                    analysis
                        .usage_messages
                        .iter()
                        .cloned()
                        .map(AttributeValue::String)
                        .collect(),
                ),
            );
            ports.push(L_04_05_BLDG_ERROR_PORT.clone());
        }
        if let Some(city_code_error) = analysis.city_code_error {
            attributes.insert(
                Attribute::new("cityCodeError"),
                AttributeValue::String(city_code_error),
            );
            ports.push(CITY_CODE_ERROR_PORT.clone());
        }
        if ports.is_empty() {
            ports.push(FEATURES_PORT.clone());
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
pub(crate) fn usage_violation_messages(
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
pub(crate) fn classify_city_code(
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
