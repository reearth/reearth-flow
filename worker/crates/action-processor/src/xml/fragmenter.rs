use std::{collections::HashMap, str::FromStr, sync::Arc};

use reearth_flow_common::{
    uri::Uri,
    xml::{self, XmlDocument},
};
use reearth_flow_runtime::{
    channels::ProcessorChannelForwarder,
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::errors::{Result, XmlProcessorError};

#[derive(Debug, Clone, Default)]
pub struct XmlFragmenterFactory;

impl ProcessorFactory for XmlFragmenterFactory {
    fn name(&self) -> &str {
        "XMLFragmenter"
    }

    fn description(&self) -> &str {
        "Fragment XML"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(XmlFragmenterParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["XML"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: XmlFragmenterParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                XmlProcessorError::FragmenterFactory(format!("Failed to serialize with: {}", e))
            })?;
            serde_json::from_value(value).map_err(|e| {
                XmlProcessorError::FragmenterFactory(format!("Failed to deserialize with: {}", e))
            })?
        } else {
            return Err(XmlProcessorError::FragmenterFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let XmlFragmenterParam::Url { property } = &params;
        let expr_engine = Arc::clone(&ctx.expr_engine);
        let elements_to_match_ast = expr_engine
            .compile(property.elements_to_match.to_string().as_str())
            .map_err(|e| {
                XmlProcessorError::FragmenterFactory(format!(
                    "Failed to comple expr engine with {:?}",
                    e
                ))
            })?;
        let elements_to_exclude_ast = expr_engine
            .compile(property.elements_to_exclude.to_string().as_str())
            .map_err(|e| {
                XmlProcessorError::FragmenterFactory(format!(
                    "Failed to comple expr engine with {:?}",
                    e
                ))
            })?;
        let process = XmlFragmenter {
            params,
            elements_to_match_ast,
            elements_to_exclude_ast,
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct XmlFragmenter {
    params: XmlFragmenterParam,
    elements_to_match_ast: rhai::AST,
    elements_to_exclude_ast: rhai::AST,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PropertySchema {
    pub(super) elements_to_match: Expr,
    pub(super) elements_to_exclude: Expr,
    pub(super) attribute: Attribute,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "source", rename_all = "camelCase")]
pub enum XmlFragmenterParam {
    #[serde(rename = "url")]
    Url {
        #[serde(flatten)]
        property: PropertySchema,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct XmlFragment {
    pub(super) xml_id: String,
    pub(super) fragment: String,
    pub(super) matched_tag: String,
    pub(super) xml_parent_id: Option<String>,
}

impl XmlFragment {
    fn to_hashmap(fragment: XmlFragment) -> HashMap<Attribute, AttributeValue> {
        let mut map = HashMap::new();
        map.insert(
            Attribute::new("xmlId"),
            AttributeValue::String(fragment.xml_id),
        );
        map.insert(
            Attribute::new("xmlFragment"),
            AttributeValue::String(fragment.fragment),
        );
        map.insert(
            Attribute::new("matchedTag"),
            AttributeValue::String(fragment.matched_tag),
        );
        let attribute = if let Some(xml_parent_id) = fragment.xml_parent_id {
            AttributeValue::String(xml_parent_id)
        } else {
            AttributeValue::Null
        };
        map.insert(Attribute::new("xmlParentId"), attribute);
        map
    }
}

impl Processor for XmlFragmenter {
    fn initialize(&mut self, _ctx: NodeContext) {}

    fn num_threads(&self) -> usize {
        20
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &mut dyn ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        match &self.params {
            XmlFragmenterParam::Url { property } => {
                let fragments = action_value_to_fragment(
                    &ctx,
                    &ctx.feature,
                    &property.attribute,
                    &self.elements_to_match_ast,
                    &self.elements_to_exclude_ast,
                )?;
                for fragment in fragments {
                    fw.send(ctx.new_with_feature_and_port(fragment, DEFAULT_PORT.clone()));
                }
            }
        }
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
        "XmlFragmenter"
    }
}

fn action_value_to_fragment(
    ctx: &ExecutorContext,
    row: &Feature,
    attribute: &Attribute,
    elements_to_match_ast: &rhai::AST,
    elements_to_exclude_ast: &rhai::AST,
) -> Result<Vec<Feature>> {
    let mut result = Vec::<Feature>::new();
    let storage_resolver = Arc::clone(&ctx.storage_resolver);
    let expr_engine = Arc::clone(&ctx.expr_engine);

    let scope = expr_engine.new_scope();
    for (k, v) in &row.attributes {
        scope.set(k.clone().into_inner().as_str(), v.clone().into());
    }
    let elements_to_match = scope
        .eval_ast::<rhai::Array>(elements_to_match_ast)
        .map_err(|e| {
            XmlProcessorError::Fragmenter(format!("Failed expr engine error with {:?}", e))
        })?;
    let elements_to_match = elements_to_match
        .iter()
        .map(|v| v.clone().into_string().unwrap_or_default())
        .collect::<Vec<String>>();
    if elements_to_match.is_empty() {
        return Ok(result);
    }

    let elements_to_exclude = scope
        .eval_ast::<rhai::Array>(elements_to_exclude_ast)
        .map_err(|e| {
            XmlProcessorError::Fragmenter(format!("Failed expr engine error with {:?}", e))
        })?;
    let elements_to_exclude = elements_to_exclude
        .iter()
        .map(|v| v.clone().into_string().unwrap_or_default())
        .collect::<Vec<String>>();

    let url = match row.get(attribute) {
        Some(AttributeValue::String(url)) => {
            Uri::from_str(url).map_err(|e| XmlProcessorError::Fragmenter(format!("{:?}", e)))?
        }
        _ => return Err(XmlProcessorError::Fragmenter("No url found".to_string())),
    };
    let storage = storage_resolver
        .resolve(&url)
        .map_err(|e| XmlProcessorError::Fragmenter(format!("{:?}", e)))?;
    let bytes = storage
        .get_sync(&url.path())
        .map_err(|e| XmlProcessorError::Fragmenter(format!("{:?}", e)))?;
    let raw_xml = String::from_utf8(bytes.to_vec())
        .map_err(|e| XmlProcessorError::Fragmenter(format!("{:?}", e)))?;
    let document = xml::parse(raw_xml.as_str())
        .map_err(|e| XmlProcessorError::Fragmenter(format!("{:?}", e)))?;

    let fragments = generate_fragment(&url, &document, &elements_to_match, &elements_to_exclude)?;
    let xml_id_maps = fragments
        .into_iter()
        .map(|fragment| (fragment.xml_id.clone(), (uuid::Uuid::new_v4(), fragment)))
        .collect::<HashMap<String, (Uuid, XmlFragment)>>();
    for (_xml_id, (feature_id, fragment)) in xml_id_maps.into_iter() {
        let mut value = Feature::new_with_id_and_attributes(feature_id, row.attributes.clone());
        XmlFragment::to_hashmap(fragment)
            .into_iter()
            .for_each(|(k, v)| {
                value.attributes.insert(k, v);
            });
        result.push(value);
    }
    Ok(result)
}

fn generate_fragment(
    uri: &Uri,
    document: &XmlDocument,
    elements_to_match: &[String],
    elements_to_exclude: &[String],
) -> Result<Vec<XmlFragment>> {
    let elements_to_match = elements_to_match
        .iter()
        .map(|element| format!("name()='{}'", element))
        .collect::<Vec<_>>();
    let elements_to_match_query = elements_to_match.join(" or ");
    let elements_to_match_query = format!("({})", elements_to_match_query);
    let elements_to_exclude_query = {
        if elements_to_exclude.is_empty() {
            "".to_string()
        } else {
            let elements_to_exclude = elements_to_exclude
                .iter()
                .map(|element| format!("name()='{}'", element))
                .collect::<Vec<_>>();
            let elements_to_exclude_query = elements_to_exclude.join(" or ");
            format!("({})", elements_to_exclude_query)
        }
    };
    let xpath = {
        if elements_to_exclude_query.is_empty() {
            format!("//*[{}]", elements_to_match_query)
        } else {
            format!(
                "//*[{} and not({})]",
                elements_to_match_query, elements_to_exclude_query
            )
        }
    };
    let ctx = xml::create_context(document)
        .map_err(|e| XmlProcessorError::Fragmenter(format!("{:?}", e)))?;
    let root = xml::get_root_node(document)
        .map_err(|e| XmlProcessorError::Fragmenter(format!("{:?}", e)))?;
    let mut nodes = xml::find_nodes_by_xpath(&ctx, &xpath, &root)
        .map_err(|_| XmlProcessorError::Fragmenter("Failed to evaluate xpath".to_string()))?;
    let mut result = Vec::<XmlFragment>::new();
    for node in nodes.iter_mut() {
        let node_type = node
            .get_type()
            .ok_or(XmlProcessorError::Fragmenter("No node type".to_string()))?;
        if node_type == xml::XmlNodeType::ElementNode {
            let xml_id = xml::get_node_id(uri, node);
            let tag = xml::get_node_tag(node);
            let fragment = {
                for ns in root.get_namespace_declarations().iter() {
                    let _ = node
                        .set_attribute(
                            format!("xmlns:{}", ns.get_prefix()).as_str(),
                            ns.get_href().as_str(),
                        )
                        .map_err(|e| {
                            XmlProcessorError::Fragmenter(format!(
                                "Failed to set namespace with {:?}",
                                e
                            ))
                        });
                }
                xml::node_to_xml_string(document, node)
                    .map_err(|e| XmlProcessorError::Fragmenter(format!("{:?}", e)))?
            };
            let xml_parent_id = node
                .get_parent()
                .map(|parent| xml::get_node_id(uri, &parent));
            result.push(XmlFragment {
                xml_id,
                fragment,
                matched_tag: tag,
                xml_parent_id,
            });
        }
    }
    Ok(result)
}
