//! PLATEAU 4 (CityGML 2.0 / i-UR 3.x) seam for the common building
//! usage-attribute validator.
//!
//! i-UR attributes nest under a `cityGmlAttributes` map, and each i-UR attribute
//! is an array whose first element is the attribute map. The survey year is a
//! plain string and the city code is `uro:city_code` under
//! `uro:BuildingIDAttribute`. A building without `uro:BuildingDetailAttribute`
//! is dropped from all output, and missing structural attributes are hard errors
//! (mirrors the original plateau4 behaviour).

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
        let Some(detail_value) = gml_attributes.get("uro:BuildingDetailAttribute") else {
            return Ok(None);
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

        let usage_messages = usage_violation_messages(building_detail_attr, &survey_year);

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

        Ok(Some(UsageAnalysis {
            usage_messages,
            city_code_error,
        }))
    }
}
