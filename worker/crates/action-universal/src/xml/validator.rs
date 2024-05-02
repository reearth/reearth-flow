use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use reearth_flow_common::{uri::Uri, xml};
use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, ActionContext, ActionDataframe, ActionResult, Attribute, AttributeValue, Feature,
    Result, SyncAction, DEFAULT_PORT,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
enum XmlInputType {
    File,
    Text,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
enum ValidationType {
    Syntax,
    SyntaxAndNamespace,
    SyntaxAndSchema,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct XmlValidator {
    attribute: Attribute,
    input_type: XmlInputType,
    validation_type: ValidationType,
}

#[async_trait::async_trait]
#[typetag::serde(name = "XMLValidator")]
impl SyncAction for XmlValidator {
    fn run(&self, ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        let input = inputs
            .get(&DEFAULT_PORT)
            .ok_or(Error::input("No Default Port"))?;

        let targets = &input.features;
        let mut success = Vec::<Feature>::new();
        let mut failed = Vec::<Feature>::new();
        let mut schema_store = HashMap::<String, xml::XmlSchemaValidationContext>::new();
        for feature in targets {
            let xml_content = self.get_xml_content(&ctx, feature)?;
            let Ok(document) = xml::parse(xml_content) else {
                failed.push(feature.clone());
                continue;
            };
            match self.validation_type {
                ValidationType::Syntax => continue,
                ValidationType::SyntaxAndNamespace => {
                    if xml::create_context(&document).is_ok() {
                        success.push(feature.clone());
                    } else {
                        failed.push(feature.clone());
                    }
                    continue;
                }
                ValidationType::SyntaxAndSchema => (),
            }
            let schema_locations =
                xml::parse_schema_locations(&document).map_err(Error::internal_runtime)?;
            let target_locations = schema_locations
                .difference(&HashSet::from_iter(schema_store.keys().cloned()))
                .cloned()
                .collect::<Vec<_>>();
            if !target_locations.is_empty() {
                let schema_contents =
                    ctx.get_contents_by_uris(self.get_base_path(feature), &target_locations);
                for (uri, content) in schema_contents {
                    let schema_context = xml::create_xml_schema_validation_context(content)
                        .map_err(Error::internal_runtime)?;
                    schema_store.insert(uri, schema_context);
                }
            }
            for locations in schema_locations {
                let schema_context = schema_store
                    .get_mut(&locations)
                    .ok_or(Error::input("No Schema"))?;
                if xml::validate_document_by_schema_context(&document, schema_context).is_ok() {
                    success.push(feature.clone());
                } else {
                    failed.push(feature.clone());
                }
            }
        }

        Ok(inputs)
    }
}

impl XmlValidator {
    fn get_base_path(&self, feature: &Feature) -> String {
        match self.input_type {
            XmlInputType::File => feature
                .attributes
                .get(&self.attribute)
                .and_then(|v| {
                    if let AttributeValue::String(s) = v {
                        Some(s.to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or("".to_string()),
            XmlInputType::Text => "".to_string(),
        }
    }

    fn get_xml_content(&self, ctx: &ActionContext, feature: &Feature) -> Result<String> {
        match self.input_type {
            XmlInputType::File => {
                let uri = feature
                    .attributes
                    .get(&self.attribute)
                    .ok_or(Error::input("No Attribute"))?;
                let uri = match uri {
                    AttributeValue::String(s) => {
                        Uri::from_str(s).map_err(|_| Error::input("Invalid URI"))?
                    }
                    _ => return Err(Error::input("Invalid Attribute")),
                };
                let storage = ctx.resolve_uri(&uri)?;
                let content = storage
                    .get_sync(uri.path().as_path())
                    .map_err(Error::internal_runtime)?;
                String::from_utf8(content.to_vec()).map_err(|_| Error::input("Invalid UTF-8"))
            }
            XmlInputType::Text => {
                let content = feature
                    .attributes
                    .get(&self.attribute)
                    .ok_or(Error::input("No Attribute"))?;
                let content = match content {
                    AttributeValue::String(s) => s,
                    _ => return Err(Error::input("Invalid Attribute")),
                };
                Ok(content.to_string())
            }
        }
    }
}
