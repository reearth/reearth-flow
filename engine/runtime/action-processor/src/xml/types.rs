use std::collections::HashMap;

use reearth_flow_common::xml;
use reearth_flow_types::{Attribute, AttributeValue};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub error_type: String,
    pub message: String,
    pub line: Option<i32>,
    pub col: Option<i32>,
}

impl ValidationResult {
    pub fn new(error_type: &str, message: &str) -> Self {
        ValidationResult {
            error_type: error_type.to_string(),
            message: message.to_string(),
            line: None,
            col: None,
        }
    }

    pub fn new_with_line_and_col(
        error_type: &str,
        message: &str,
        line: Option<i32>,
        col: Option<i32>,
    ) -> Self {
        ValidationResult {
            error_type: error_type.to_string(),
            message: message.to_string(),
            line,
            col,
        }
    }
}

impl From<ValidationResult> for HashMap<String, AttributeValue> {
    fn from(result: ValidationResult) -> Self {
        let mut map = HashMap::new();
        map.insert(
            "errorType".to_string(),
            AttributeValue::String(result.error_type),
        );
        map.insert(
            "message".to_string(),
            AttributeValue::String(result.message),
        );
        map.insert(
            "line".to_string(),
            AttributeValue::String(result.line.unwrap_or_default().to_string()),
        );
        map.insert(
            "col".to_string(),
            AttributeValue::String(result.col.unwrap_or_default().to_string()),
        );
        map
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum XmlInputType {
    File,
    Text,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ValidationType {
    Syntax,
    SyntaxAndNamespace,
    SyntaxAndSchema,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct XmlValidatorParam {
    pub attribute: Attribute,
    pub input_type: XmlInputType,
    pub validation_type: ValidationType,
}

pub type SchemaStore = HashMap<Vec<(String, String)>, xml::XmlSchemaValidationContext>;
