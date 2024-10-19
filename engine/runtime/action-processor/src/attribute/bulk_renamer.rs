use super::errors::AttributeProcessorError;
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, Feature};
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
#[derive(Debug, Clone, Default)]
pub struct BulkAttributeRenamerFactory;

impl ProcessorFactory for BulkAttributeRenamerFactory {
    fn name(&self) -> &str {
        "BulkAttributeRenamer"
    }

    fn description(&self) -> &str {
        "Renames attributes by adding/removing prefixes or suffixes, or replacing text"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(BulkAttributeRenamerParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["Attribute"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: BulkAttributeRenamerParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                AttributeProcessorError::BulkRenamerFactory(format!(
                    "Failed to serialize `with` parameter: {}",
                    e
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                AttributeProcessorError::BulkRenamerFactory(format!(
                    "Failed to deserialize `with` parameter: {}",
                    e
                ))
            })?
        } else {
            return Err(AttributeProcessorError::BulkRenamerFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let process = BulkAttributeRenamer { params };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct BulkAttributeRenamer {
    params: BulkAttributeRenamerParam,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BulkAttributeRenamerParam {
    pub rename_type: RenameType,
    pub rename_action: RenameAction,
    pub text_to_find: Option<String>,
    pub rename_value: String,
    pub selected_attributes: Option<Vec<String>>,
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub enum RenameType {
    All,
    Selected,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub enum RenameAction {
    AddPrefix,
    AddSuffix,
    RemovePrefix,
    RemoveSuffix,
    StringReplace,
}

impl Processor for BulkAttributeRenamer {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let mut feature = ctx.feature.clone();

        let attributes_to_rename = self.select_attributes(&feature)?;
        let attributes_to_remove = self.rename_attributes(&mut feature, attributes_to_rename)?;

        for attr in attributes_to_remove {
            feature.attributes.remove(&attr);
        }

        fw.send(ctx.new_with_feature_and_port(feature.clone(), DEFAULT_PORT.clone()));
        Ok(())
    }

    fn finish(
        &self,
        _ctx: NodeContext,
        _fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "BulkAttributeRenamer"
    }
}

impl BulkAttributeRenamer {
    fn select_attributes(&self, feature: &Feature) -> Result<Vec<Attribute>, BoxedError> {
        match self.params.rename_type {
            RenameType::All => Ok(feature.attributes.keys().cloned().collect()),
            RenameType::Selected => {
                if let Some(attrs) = &self.params.selected_attributes {
                    Ok(attrs.iter().map(Attribute::new).collect())
                } else {
                    Err(AttributeProcessorError::BulkRenamer(
                        "No attributes selected for renaming".to_string(),
                    )
                    .into())
                }
            }
        }
    }

    fn rename_attributes(
        &self,
        feature: &mut Feature,
        attributes: Vec<Attribute>,
    ) -> Result<Vec<Attribute>, BoxedError> {
        let mut attributes_to_remove = vec![];

        for attr in attributes {
            if let Some(value) = feature.attributes.get(&attr) {
                let new_name = self.get_new_name(&attr.inner())?;
                if new_name.is_empty() {
                    feature.attributes.remove(&attr);
                } else {
                    let new_attr = Attribute::new(new_name);
                    feature.attributes.insert(new_attr, value.clone());
                    attributes_to_remove.push(attr.clone());
                }
            }
        }
        Ok(attributes_to_remove)
    }

    fn get_new_name(&self, attr_name: &str) -> Result<String, BoxedError> {
        match self.params.rename_action {
            RenameAction::AddPrefix => Ok(format!("{}{}", self.params.rename_value, attr_name)),
            RenameAction::AddSuffix => Ok(format!("{}{}", attr_name, self.params.rename_value)),
            RenameAction::RemovePrefix => self.remove_prefix(attr_name),
            RenameAction::RemoveSuffix => self.remove_suffix(attr_name),
            RenameAction::StringReplace => self.string_replace(attr_name),
        }
    }

    fn remove_prefix(&self, attr_name: &str) -> Result<String, BoxedError> {
        if attr_name.starts_with(&self.params.rename_value) {
            Ok(attr_name
                .strip_prefix(&self.params.rename_value)
                .unwrap_or(attr_name)
                .to_string())
        } else {
            Err(AttributeProcessorError::BulkRenamer(format!(
                "Attribute '{}' does not start with prefix '{}'",
                attr_name, self.params.rename_value
            ))
            .into())
        }
    }

    fn remove_suffix(&self, attr_name: &str) -> Result<String, BoxedError> {
        if attr_name.ends_with(&self.params.rename_value) {
            Ok(attr_name
                .strip_suffix(&self.params.rename_value)
                .unwrap_or(attr_name)
                .to_string())
        } else {
            Err(AttributeProcessorError::BulkRenamer(format!(
                "Attribute '{}' does not end with suffix '{}'",
                attr_name, self.params.rename_value
            ))
            .into())
        }
    }

    fn string_replace(&self, attr_name: &str) -> Result<String, BoxedError> {
        if let Some(ref find) = self.params.text_to_find {
            let re = Regex::new(find).map_err(|e| {
                AttributeProcessorError::BulkRenamer(format!(
                    "Invalid regex pattern '{}': {}",
                    find, e
                ))
            })?;

            if re.is_match(attr_name) {
                Ok(re
                    .replace_all(attr_name, &self.params.rename_value as &str)
                    .to_string())
            } else {
                Err(AttributeProcessorError::BulkRenamer(format!(
                    "Attribute '{}' does not match the regex pattern '{}'",
                    attr_name, find
                ))
                .into())
            }
        } else {
            Err(AttributeProcessorError::BulkRenamer(
                "Missing 'text_to_find' parameter for StringReplace action".to_string(),
            )
            .into())
        }
    }
}
