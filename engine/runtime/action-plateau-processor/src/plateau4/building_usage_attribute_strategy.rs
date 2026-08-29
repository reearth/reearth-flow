//! PLATEAU 4 (CityGML 2.0 / i-UR 3.x) seam for the common building
//! usage-attribute validator.
//!
//! Reads the reader's attribute tree directly: each i-UR attribute is a
//! property wrapper (`uro:buildingIDAttribute`) holding one type element
//! (`uro:BuildingIDAttribute`) whose fields are the leaf values. The reader
//! resolves code-typed leaves into a label plus a `{name}_code` sibling; an
//! unresolved code keeps its `$`/`@codeSpace` map shape. A building without
//! `uro:BuildingDetailAttribute` is dropped from all output, and missing
//! structural attributes are hard errors.

use std::collections::HashMap;

use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_types::{Attribute, AttributeValue, Feature};

use crate::common::building_usage_attribute_validator::{
    classify_city_code, usage_violation_messages, BuildingUsageAttributeStrategy, UsageAnalysis,
};
use crate::common::errors::PlateauProcessorError;

/// Zero-sized strategy; passed as `&Plateau4BuildingUsageStrategy` (rvalue static
/// promotion yields the `&'static dyn` the factory needs), so no named static.
#[derive(Debug)]
pub(crate) struct Plateau4BuildingUsageStrategy;

/// The map content of a value that is either a map or an array whose first
/// element is a map.
fn first_map(value: &AttributeValue) -> Option<&HashMap<String, AttributeValue>> {
    match value {
        AttributeValue::Map(map) => Some(map),
        AttributeValue::Array(array) => match array.first() {
            Some(AttributeValue::Map(map)) => Some(map),
            _ => None,
        },
        _ => None,
    }
}

/// Steps from the property wrapper `attrs[wrapper]` into its type element
/// `type_name`, tolerating a repeated wrapper or type element.
fn type_element<'a>(
    attrs: &'a HashMap<String, AttributeValue>,
    wrapper: &str,
    type_name: &str,
) -> Option<&'a HashMap<String, AttributeValue>> {
    first_map(attrs.get(wrapper)?)
        .and_then(|wrapper_map| wrapper_map.get(type_name))
        .and_then(first_map)
}

impl BuildingUsageAttributeStrategy for Plateau4BuildingUsageStrategy {
    fn analyze(
        &self,
        feature: &Feature,
        city_code_to_name: &HashMap<String, String>,
    ) -> Result<Option<UsageAnalysis>, BoxedError> {
        let Some(AttributeValue::Map(gml_attributes)) =
            feature.attributes.get(&Attribute::new("cityGmlAttributes"))
        else {
            return Err(PlateauProcessorError::BuildingUsageAttributeValidator(
                "cityGmlAttributes key empty".to_string(),
            )
            .into());
        };

        // A building without uro:BuildingDetailAttribute carries no usage or city
        // checks in this layout: drop it from all output (matching plateau4).
        let Some(building_detail_attr) = type_element(
            gml_attributes,
            "uro:buildingDetailAttribute",
            "uro:BuildingDetailAttribute",
        ) else {
            return Ok(None);
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

        let usage_messages = usage_violation_messages(building_detail_attr, &survey_year);

        let Some(id_attr) = type_element(
            gml_attributes,
            "uro:buildingIDAttribute",
            "uro:BuildingIDAttribute",
        ) else {
            return Err(PlateauProcessorError::BuildingUsageAttributeValidator(
                "uro:BuildingIDAttribute must be specified as per cityGML specification, but it is not.".to_string(),
            )
            .into());
        };
        let city_code_error = match id_attr.get("uro:city_code") {
            Some(AttributeValue::String(city_code)) => {
                classify_city_code(city_code, city_code_to_name)
            }
            Some(_) => {
                return Err(PlateauProcessorError::BuildingUsageAttributeValidator(
                    "uro:city_code must be a string, but it is not".to_string(),
                )
                .into())
            }
            // No resolved code: an unresolvable codeSpace leaves `uro:city` as a
            // `$`/`@codeSpace` map carrying the raw code, so classify that.
            None => match id_attr.get("uro:city").and_then(|city| match city {
                AttributeValue::Map(city_map) => city_map.get("$"),
                _ => None,
            }) {
                Some(AttributeValue::String(raw_code)) => {
                    classify_city_code(raw_code, city_code_to_name)
                }
                _ => Some("<未設定>".to_string()),
            },
        };

        Ok(Some(UsageAnalysis {
            usage_messages,
            city_code_error,
        }))
    }
}
