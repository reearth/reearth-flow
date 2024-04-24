use std::{collections::HashMap, str::FromStr, sync::Arc};

use reearth_flow_action::{Attribute, Dataframe, Feature};
use reearth_flow_common::collection;
use reearth_flow_common::str::to_hash;
use reearth_flow_common::uri::Uri;
use reearth_flow_common::xml::{self, XmlDocument};
use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, ActionContext, ActionDataframe, ActionResult, AttributeValue, Result, SyncAction,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PropertySchema {
    pub(super) elements_to_match: String,
    pub(super) elements_to_exclude: String,
    pub(super) attribute: String,
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
            Attribute::new("fragment"),
            AttributeValue::String(fragment.fragment),
        );
        map.insert(
            Attribute::new("matchedTag"),
            AttributeValue::String(fragment.matched_tag),
        );
        if let Some(xml_parent_id) = fragment.xml_parent_id {
            map.insert(
                Attribute::new("xmlParentId"),
                AttributeValue::String(xml_parent_id),
            );
        }
        map
    }
}

impl From<XmlFragment> for Feature {
    fn from(fragment: XmlFragment) -> Self {
        let mut map = HashMap::new();
        map.insert(
            Attribute::new("xmlId"),
            AttributeValue::String(fragment.xml_id),
        );
        map.insert(
            Attribute::new("fragment"),
            AttributeValue::String(fragment.fragment),
        );
        map.insert(
            Attribute::new("matchedTag"),
            AttributeValue::String(fragment.matched_tag),
        );
        if let Some(xml_parent_id) = fragment.xml_parent_id {
            map.insert(
                Attribute::new("xmlParentId"),
                AttributeValue::String(xml_parent_id),
            );
        }
        Feature::new_with_attributes(map)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "source", rename_all = "camelCase")]
pub enum XmlFragmenter {
    #[serde(rename = "url")]
    Url {
        #[serde(flatten)]
        property: PropertySchema,
    },
}

#[typetag::serde(name = "XMLFragmenter")]
impl SyncAction for XmlFragmenter {
    fn run(&self, ctx: ActionContext, inputs: ActionDataframe) -> ActionResult {
        match self {
            XmlFragmenter::Url { property } => url(ctx, inputs, property),
        }
    }
}

pub(super) fn url(
    ctx: ActionContext,
    inputs: ActionDataframe,
    props: &PropertySchema,
) -> Result<ActionDataframe> {
    let expr_engine = Arc::clone(&ctx.expr_engine);
    let elements_to_match_ast = expr_engine
        .compile(props.elements_to_match.as_str())
        .map_err(Error::internal_runtime)?;
    let elements_to_exclude_ast = expr_engine
        .compile(props.elements_to_exclude.as_str())
        .map_err(Error::internal_runtime)?;

    let mut output = ActionDataframe::new();
    for (port, data) in inputs {
        let result = collection::par_map(&data.features, |row| {
            action_value_to_fragment(
                &ctx,
                row,
                &props.attribute,
                &elements_to_match_ast,
                &elements_to_exclude_ast,
            )
            .unwrap_or_default()
        });
        let result = result.into_iter().flatten().collect::<Vec<Feature>>();
        output.insert(port, Dataframe::new(result));
    }
    Ok(output)
}

fn action_value_to_fragment(
    ctx: &ActionContext,
    row: &Feature,
    attribute: &String,
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
        .map_err(Error::internal_runtime)?;
    let elements_to_match = elements_to_match
        .iter()
        .map(|v| v.clone().into_string().unwrap_or_default())
        .collect::<Vec<String>>();
    if elements_to_match.is_empty() {
        return Ok(result);
    }

    let elements_to_exclude = scope
        .eval_ast::<rhai::Array>(elements_to_exclude_ast)
        .map_err(Error::internal_runtime)?;
    let elements_to_exclude = elements_to_exclude
        .iter()
        .map(|v| v.clone().into_string().unwrap_or_default())
        .collect::<Vec<String>>();

    let url = match row.get(attribute) {
        Some(AttributeValue::String(url)) => Uri::from_str(url).map_err(Error::internal_runtime)?,
        _ => return Err(Error::internal_runtime("No url found")),
    };
    let storage = storage_resolver
        .resolve(&url)
        .map_err(Error::internal_runtime)?;
    let bytes = storage
        .get_sync(&url.path())
        .map_err(Error::internal_runtime)?;
    let raw_xml = String::from_utf8(bytes.to_vec()).map_err(Error::internal_runtime)?;
    ctx.action_log(format!("Parsing XML document: {:?} ...", url));
    let document = xml::parse(raw_xml.as_str()).map_err(Error::internal_runtime)?;

    let fragments = generate_fragment(&document, &elements_to_match, &elements_to_exclude)?;
    for fragment in fragments {
        let mut value = row.attributes.clone();
        value.extend(XmlFragment::to_hashmap(fragment));
        result.push(row.with_attributes(value));
    }
    Ok(result)
}

fn generate_fragment(
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
    let ctx = xml::create_context(document).map_err(Error::internal_runtime)?;
    let root = xml::get_root_node(document).map_err(Error::internal_runtime)?;
    let results = ctx
        .node_evaluate(&xpath, &root)
        .map_err(|_| Error::internal_runtime("Failed to evaluate xpath".to_string()))?;
    let mut nodes = results.get_nodes_as_vec();

    let mut result = Vec::<XmlFragment>::new();
    let mut fragments = HashMap::<String, String>::new();
    for node in nodes.iter_mut() {
        let node_type = node
            .get_type()
            .ok_or(Error::internal_runtime("No node type".to_string()))?;
        if node_type == xml::XmlNodeType::ElementNode {
            let tag = xml::get_node_tag(node);
            let key = format!("{:?}", node);
            let fragment = {
                if let Some(fragment) = fragments.get(&key) {
                    fragment.clone()
                } else {
                    for ns in root.get_namespace_declarations().iter() {
                        let _ = node
                            .set_attribute(
                                format!("xmlns:{}", ns.get_prefix()).as_str(),
                                ns.get_href().as_str(),
                            )
                            .map_err(|e| {
                                Error::internal_runtime(format!(
                                    "Failed to set namespace with {:?}",
                                    e
                                ))
                            });
                    }
                    xml::node_to_xml_string(document, node).map_err(Error::internal_runtime)?
                }
            };
            let xml_id = to_hash(&fragment);
            fragments.insert(format!("{:?}", node), xml_id.clone());
            let xml_parent_id = node.get_parent().map(|mut parent| {
                let key = format!("{:?}", parent);
                if let Some(fragment) = fragments.get(&key) {
                    fragment.clone()
                } else {
                    for ns in root.get_namespace_declarations().iter() {
                        let _ = parent
                            .set_attribute(
                                format!("xmlns:{}", ns.get_prefix()).as_str(),
                                ns.get_href().as_str(),
                            )
                            .map_err(|e| {
                                Error::internal_runtime(format!(
                                    "Failed to set namespace with {:?}",
                                    e
                                ))
                            });
                    }
                    to_hash(&xml::node_to_xml_string(document, &mut parent).unwrap_or_default())
                }
            });
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
