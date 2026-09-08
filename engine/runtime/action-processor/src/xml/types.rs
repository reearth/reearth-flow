use std::collections::HashMap;

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
    /// # XML File
    /// Reads the document from the file path or URL held in the attribute.
    File,
    /// # XML Text
    /// Uses the attribute value itself as the document.
    Text,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ValidationType {
    /// # Syntax
    /// Checks that the document is well-formed XML.
    Syntax,
    /// # Syntax and Namespace
    /// Also checks that every namespace prefix used by an element is declared on that element or
    /// an ancestor, and that unprefixed elements have a default namespace.
    SyntaxAndNamespace,
    /// # Syntax and Schema
    /// Also validates the document against the XSD schemas named by its `xsi:schemaLocation`.
    /// Unreachable remote schemas are skipped, as those locations are hints per the XML Schema
    /// specification.
    SyntaxAndSchema,
}

/// # XML Validator Parameters
///
/// Configures which XML document is validated and how thoroughly.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct XmlValidatorParam {
    /// # XML Attribute
    /// Attribute holding the XML document: the file path or URL to read, or the XML text itself,
    /// depending on the input type.
    pub attribute: Attribute,
    /// # Input Type
    /// Whether the attribute holds the location of the document or the document itself.
    pub input_type: XmlInputType,
    /// # Validation Type
    /// Checks to run against the document.
    pub validation_type: ValidationType,
}
