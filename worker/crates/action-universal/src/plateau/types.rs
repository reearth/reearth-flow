use std::collections::HashMap;

use reearth_flow_action::{ActionValue, error};
use serde::{Deserialize, Serialize};


pub(crate) static DICTIONARIES_INITIATOR_SETTINGS_PORT: &str = "settings";

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SchemaFeature {
    pub(crate) name: String,
    pub(crate) r#type: String,
    pub(crate) min_occurs: String,
    pub(crate) max_occurs: String,
    pub(crate) flag: Option<String>,
    pub(crate) children: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Settings {
    pub(crate) xpath_to_properties: HashMap<String, HashMap<String, SchemaFeature>>,
    pub(crate) except_feature_types: Vec<String>,
    pub(crate) codelists: HashMap<String, HashMap<String, String>>,
}

impl Settings {
    pub(crate) fn new(
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

impl TryFrom<Settings> for ActionValue {
    type Error = error::Error;
    fn try_from(value: Settings) -> Result<Self, error::Error> {
        let value = serde_json::to_value(value).map_err(|e| {
            error::Error::output(format!("Cannot convert to json with error = {:?}", e))
        })?;
        Ok(ActionValue::from(value))
    }
}

impl TryFrom<ActionValue> for Settings {
    type Error = error::Error;
    fn try_from(value: ActionValue) -> Result<Self, error::Error> {
        let value = serde_json::to_value(value).map_err(|e| {
            error::Error::output(format!("Cannot convert to json with error = {:?}", e))
        })?;
        serde_json::from_value(value).map_err(|e| {
            error::Error::output(format!("Cannot convert to json with error = {:?}", e))
        })
    }
}
