use std::{collections::HashMap, str::FromStr, sync::Arc};

use reearth_flow_common::{
    uri::Uri,
    xml::{self, XmlDocument},
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Expr, Feature};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::{Result, XmlProcessorError};

#[derive(Debug, Clone, Default)]
pub struct XmlFragmenterFactory;

impl ProcessorFactory for XmlFragmenterFactory {
    fn name(&self) -> &str {
        "XMLFragmenter"
    }

    fn description(&self) -> &str {
        "Fragments large XML documents into smaller pieces based on specified element patterns"
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
        let params: XmlFragmenterParam = if let Some(with) = with.clone() {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                XmlProcessorError::FragmenterFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                XmlProcessorError::FragmenterFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
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
                    "Failed to comple expr engine with {e:?}"
                ))
            })?;
        let elements_to_exclude_ast = expr_engine
            .compile(property.elements_to_exclude.to_string().as_str())
            .map_err(|e| {
                XmlProcessorError::FragmenterFactory(format!(
                    "Failed to comple expr engine with {e:?}"
                ))
            })?;
        let process = XmlFragmenter {
            global_params: with,
            params,
            elements_to_match_ast,
            elements_to_exclude_ast,
        };
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct XmlFragmenter {
    global_params: Option<HashMap<String, serde_json::Value>>,
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

/// # XMLFragmenter Parameters
///
/// Configuration for fragmenting XML documents into smaller pieces.
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(tag = "source", rename_all = "camelCase")]
pub enum XmlFragmenterParam {
    #[serde(rename = "url")]
    /// URL-based source configuration for XML fragmenting
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
    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        match &self.params {
            XmlFragmenterParam::Url { property } => {
                send_xml_fragment(
                    &ctx,
                    fw,
                    &self.global_params,
                    &ctx.feature,
                    &property.attribute,
                    &self.elements_to_match_ast,
                    &self.elements_to_exclude_ast,
                )?;
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "XmlFragmenter"
    }
}

fn send_xml_fragment(
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
    global_params: &Option<HashMap<String, serde_json::Value>>,
    feature: &Feature,
    attribute: &Attribute,
    elements_to_match_ast: &rhai::AST,
    elements_to_exclude_ast: &rhai::AST,
) -> Result<()> {
    let storage_resolver = Arc::clone(&ctx.storage_resolver);
    let expr_engine = Arc::clone(&ctx.expr_engine);

    let scope = feature.new_scope(expr_engine.clone(), global_params);
    let elements_to_match = scope
        .eval_ast::<rhai::Array>(elements_to_match_ast)
        .map_err(|e| {
            XmlProcessorError::Fragmenter(format!("Failed expr engine error with {e:?}"))
        })?;
    let elements_to_match = elements_to_match
        .iter()
        .map(|v| v.clone().into_string().unwrap_or_default())
        .collect::<Vec<String>>();
    if elements_to_match.is_empty() {
        return Ok(());
    }

    let elements_to_exclude = scope
        .eval_ast::<rhai::Array>(elements_to_exclude_ast)
        .map_err(|e| {
            XmlProcessorError::Fragmenter(format!("Failed expr engine error with {e:?}"))
        })?;
    let elements_to_exclude = elements_to_exclude
        .iter()
        .map(|v| v.clone().into_string().unwrap_or_default())
        .collect::<Vec<String>>();

    let url = match feature.get(attribute) {
        Some(AttributeValue::String(url)) => {
            Uri::from_str(url).map_err(|e| XmlProcessorError::Fragmenter(format!("{e:?}")))?
        }
        _ => return Err(XmlProcessorError::Fragmenter("No url found".to_string())),
    };
    let storage = storage_resolver
        .resolve(&url)
        .map_err(|e| XmlProcessorError::Fragmenter(format!("{e:?}")))?;
    let bytes = storage
        .get_sync(&url.path())
        .map_err(|e| XmlProcessorError::Fragmenter(format!("{e:?}")))?;
    let raw_xml = String::from_utf8(bytes.to_vec())
        .map_err(|e| XmlProcessorError::Fragmenter(format!("{e:?}")))?;
    let document = xml::parse(raw_xml.as_str())
        .map_err(|e| XmlProcessorError::Fragmenter(format!("{e:?}")))?;

    generate_fragment(
        ctx,
        fw,
        feature,
        &url,
        &document,
        &elements_to_match,
        &elements_to_exclude,
    )
}

fn generate_fragment(
    ctx: &ExecutorContext,
    fw: &ProcessorChannelForwarder,
    feature: &Feature,
    uri: &Uri,
    document: &XmlDocument,
    elements_to_match: &[String],
    elements_to_exclude: &[String],
) -> Result<()> {
    let elements_to_match = elements_to_match
        .iter()
        .map(|element| format!("name()='{element}'"))
        .collect::<Vec<_>>();
    let elements_to_match_query = elements_to_match.join(" or ");
    let elements_to_match_query = format!("({elements_to_match_query})");
    let elements_to_exclude_query = {
        if elements_to_exclude.is_empty() {
            "".to_string()
        } else {
            let elements_to_exclude = elements_to_exclude
                .iter()
                .map(|element| format!("name()='{element}'"))
                .collect::<Vec<_>>();
            let elements_to_exclude_query = elements_to_exclude.join(" or ");
            format!("({elements_to_exclude_query})")
        }
    };
    let xpath = {
        if elements_to_exclude_query.is_empty() {
            format!("//*[{elements_to_match_query}]")
        } else {
            format!("//*[{elements_to_match_query} and not({elements_to_exclude_query})]")
        }
    };
    let xctx = xml::create_context(document)
        .map_err(|e| XmlProcessorError::Fragmenter(format!("{e:?}")))?;
    let root = xml::get_root_node(document)
        .map_err(|e| XmlProcessorError::Fragmenter(format!("{e:?}")))?;
    let mut nodes = xml::find_nodes_by_xpath(&xctx, &xpath, &root)
        .map_err(|_| XmlProcessorError::Fragmenter("Failed to evaluate xpath".to_string()))?;
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
                                "Failed to set namespace with {e:?}"
                            ))
                        });
                }
                xml::node_to_xml_string(document, node)
                    .map_err(|e| XmlProcessorError::Fragmenter(format!("{e:?}")))?
            };
            let xml_parent_id = node
                .get_parent()
                .map(|parent| xml::get_node_id(uri, &parent));

            let fragment = XmlFragment {
                xml_id,
                fragment,
                matched_tag: tag,
                xml_parent_id,
            };
            let mut value = Feature::new_with_attributes(feature.attributes.clone());
            XmlFragment::to_hashmap(fragment)
                .into_iter()
                .for_each(|(k, v)| {
                    value.attributes.insert(k, v);
                });
            fw.send(ctx.new_with_feature_and_port(value, DEFAULT_PORT.clone()));
        }
    }
    Ok(())
}
