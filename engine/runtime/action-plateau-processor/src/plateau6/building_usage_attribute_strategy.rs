//! PLATEAU 6 (CityGML 3.0 / i-UR 4.0) seam for the common building
//! usage-attribute validator.
//!
//! i-UR attributes hang off one or more `bldg:adeOfAbstractBuilding` hooks (a
//! single hook is stored as a Map, multiple as an Array); the survey year is an
//! ISO date and the city code is `uro:city`. Extraction is lenient: a building
//! without the relevant attributes simply produces no findings (rather than an
//! error), and buildings are always emitted (never dropped).

use std::collections::HashMap;

use reearth_flow_runtime::errors::BoxedError;
use reearth_flow_types::{Attribute, AttributeValue, Attributes, Feature};

use crate::common::building_usage_attribute_validator::{
    classify_city_code, usage_violation_messages, BuildingUsageAttributeStrategy, UsageAnalysis,
};

/// Zero-sized strategy; passed as `&Plateau6BuildingUsageStrategy` (rvalue static
/// promotion yields the `&'static dyn` the factory needs), so no named static.
#[derive(Debug)]
pub(crate) struct Plateau6BuildingUsageStrategy;

impl BuildingUsageAttributeStrategy for Plateau6BuildingUsageStrategy {
    fn analyze(
        &self,
        feature: &Feature,
        city_code_to_name: &HashMap<String, String>,
    ) -> Result<Option<UsageAnalysis>, BoxedError> {
        let ade = feature
            .attributes
            .get(&Attribute::new("bldg:adeOfAbstractBuilding"));

        let mut usage_messages = Vec::<String>::new();
        if let Some(building_detail_attr) = ade_child_map(ade, "uro:BuildingDetailAttribute") {
            let survey_year = match building_detail_attr.get("uro:surveyYear") {
                // i-UR 4.0 surveyYear is an ISO date (e.g. "2020-04-01"); take the year.
                Some(AttributeValue::String(date)) => {
                    date.split('-').next().unwrap_or("").to_string()
                }
                _ => String::new(),
            };
            usage_messages = usage_violation_messages(building_detail_attr, &survey_year);
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

        Ok(Some(UsageAnalysis {
            usage_messages,
            city_code_error,
        }))
    }

    fn set_usage_error(&self, attributes: &mut Attributes, messages: &[String]) {
        attributes.insert(
            Attribute::new("usageError"),
            AttributeValue::String(messages.join("\n")),
        );
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
