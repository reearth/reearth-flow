use std::collections::HashMap;

use once_cell::sync::Lazy;
use reearth_flow_action::{error, AttributeValue, Dataframe, Port};
use serde::{Deserialize, Serialize};

pub(super) static DICTIONARIES_INITIATOR_SETTINGS_PORT: Lazy<Port> =
    Lazy::new(|| Port::new("settings"));

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct SchemaFeature {
    pub(super) name: String,
    pub(super) r#type: String,
    pub(super) min_occurs: String,
    pub(super) max_occurs: String,
    pub(super) flag: Option<String>,
    pub(super) children: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct Settings {
    pub(super) xpath_to_properties: HashMap<String, HashMap<String, SchemaFeature>>,
    pub(super) except_feature_types: Vec<String>,
    pub(super) codelists: HashMap<String, HashMap<String, String>>,
}

impl Settings {
    pub(super) fn new(
        xpath_to_properties: HashMap<String, HashMap<String, SchemaFeature>>,
        except_feature_types: Vec<String>,
        codelists: HashMap<String, HashMap<String, String>>,
    ) -> Self {
        Self {
            xpath_to_properties,
            except_feature_types,
            codelists,
        }
    }
}

impl TryFrom<Settings> for AttributeValue {
    type Error = error::Error;
    fn try_from(value: Settings) -> Result<Self, error::Error> {
        let value = serde_json::to_value(value).map_err(|e| {
            error::Error::output(format!("Cannot convert to json with error = {:?}", e))
        })?;
        Ok(AttributeValue::from(value))
    }
}

impl TryFrom<Dataframe> for Settings {
    type Error = error::Error;
    fn try_from(value: Dataframe) -> Result<Self, error::Error> {
        let settings = value.features.first();
        if settings.is_none() {
            return Err(error::Error::output("No settings"));
        }
        let feature = settings.unwrap();
        let value = serde_json::to_value(&feature.attributes).map_err(|e| {
            error::Error::output(format!("Cannot convert to json with error = {:?}", e))
        })?;
        serde_json::from_value(value).map_err(|e| {
            error::Error::output(format!("Cannot convert to json with error = {:?}", e))
        })
    }
}
