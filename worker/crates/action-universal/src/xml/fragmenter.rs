use std::{collections::HashMap, str::FromStr, sync::Arc};

use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, utils::convert_dataframe_to_scope_params, Action, ActionContext, ActionDataframe,
    ActionResult, ActionValue, Result,
};
use reearth_flow_common::collection;
use reearth_flow_common::uri::Uri;
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_storage::resolve::StorageResolver;
use reearth_flow_xml::{
    convert::as_element,
    parser,
    traits::{Node, NodeType},
    RefNode,
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
    fn to_hashmap(fragment: XmlFragment) -> HashMap<String, ActionValue> {
        let mut map = HashMap::new();
        map.insert("xmlId".to_string(), ActionValue::String(fragment.xml_id));
        map.insert(
            "fragment".to_string(),
            ActionValue::String(fragment.fragment),
        );
        map.insert(
            "matchedTag".to_string(),
            ActionValue::String(fragment.matched_tag),
        );
        if let Some(xml_parent_id) = fragment.xml_parent_id {
            map.insert(
                "xmlParentId".to_string(),
                ActionValue::String(xml_parent_id),
            );
        }
        map
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

#[async_trait::async_trait]
#[typetag::serde(name = "XMLFragmenter")]
impl Action for XmlFragmenter {
    async fn run(&self, ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.ok_or(Error::input("No input dataframe"))?;
        match self {
            XmlFragmenter::Url { property } => url(ctx, inputs, property).await,
        }
    }
}

pub(super) async fn url(
    ctx: ActionContext,
    inputs: ActionDataframe,
    props: &PropertySchema,
) -> Result<ActionDataframe> {
    let expr_engine = Arc::clone(&ctx.expr_engine);
    let params = convert_dataframe_to_scope_params(&inputs);
    let elements_to_match_ast = expr_engine
        .compile(props.elements_to_match.as_str())
        .map_err(Error::internal_runtime)?;
    let elements_to_exclude_ast = expr_engine
        .compile(props.elements_to_exclude.as_str())
        .map_err(Error::internal_runtime)?;

    let mut output = ActionDataframe::new();
    for (port, data) in inputs {
        let data = match data {
            Some(data) => data,
            None => continue,
        };
        let processed_data = match data {
            ActionValue::Array(data) => {
                let result = collection::map(&data, |row| {
                    let fragment = action_value_to_fragment(
                        row,
                        &props.attribute,
                        &elements_to_match_ast,
                        &elements_to_exclude_ast,
                        &params,
                        Arc::clone(&expr_engine),
                        Arc::clone(&ctx.storage_resolver),
                    );
                    match fragment {
                        Ok(fragment) => fragment,
                        Err(_) => {
                            vec![]
                        }
                    }
                });
                ActionValue::Array(result.into_iter().flatten().collect())
            }
            _ => data,
        };
        output.insert(port, Some(processed_data));
    }
    Ok(output)
}

fn action_value_to_fragment(
    row: &ActionValue,
    attribute: &String,
    elements_to_match_ast: &rhai::AST,
    elements_to_exclude_ast: &rhai::AST,
    params: &HashMap<String, ActionValue>,
    expr_engine: Arc<Engine>,
    storage_resolver: Arc<StorageResolver>,
) -> Result<Vec<ActionValue>> {
    let mut result = Vec::<ActionValue>::new();
    let storage_resolver = Arc::clone(&storage_resolver);

    match row {
        ActionValue::Map(row) => {
            let scope = expr_engine.new_scope();
            for (k, v) in params {
                scope.set(k, v.clone().into());
            }
            for (k, v) in row {
                scope.set(k, v.clone().into());
            }
            let elements_to_match_ast = scope
                .eval_ast::<rhai::Array>(elements_to_match_ast)
                .map_err(Error::internal_runtime)?;
            let elements_to_match = elements_to_match_ast
                .iter()
                .map(|v| v.clone().into_string().unwrap_or_default())
                .collect::<Vec<String>>();
            let elements_to_exclude_ast = scope
                .eval_ast::<rhai::Array>(elements_to_exclude_ast)
                .map_err(Error::internal_runtime)?;
            let elements_to_exclude = elements_to_exclude_ast
                .iter()
                .map(|v| v.clone().into_string().unwrap_or_default())
                .collect::<Vec<String>>();
            let url = match row.get(attribute) {
                Some(ActionValue::String(url)) => Uri::from_str(url).map_err(|err| {
                    Error::internal_runtime(format!("{:?} with url = {}", err, url))
                })?,
                _ => return Err(Error::internal_runtime("No url found")),
            };
            let storage = storage_resolver
                .resolve(&url)
                .map_err(Error::internal_runtime)?;
            let content = storage
                .get_sync(&url.path())
                .map_err(Error::internal_runtime)?;
            let raw_xml = String::from_utf8(content.to_vec()).map_err(Error::internal_runtime)?;
            let document = parser::read_xml(&raw_xml).map_err(Error::internal_runtime)?;
            let root = document
                .first_child()
                .ok_or(Error::internal_runtime("No root node"))?;
            let nodes = recursive_fragment(root, &elements_to_match, &elements_to_exclude)?;
            let results = collection::par_map(&nodes, |node| {
                let mut value = row.clone();
                let tag = node.node_name().to_string();
                let element = as_element(node).unwrap();
                let fragment = element
                    .to_xml(&elements_to_match, &elements_to_exclude)
                    .unwrap_or_default();
                let xml_id = element.node_id();
                let xml_parent_id = match node.parent_node() {
                    Some(parent) if parent.borrow().node_type == NodeType::Element => {
                        Some(parent.node_id())
                    }
                    _ => None,
                };
                value.extend(XmlFragment::to_hashmap(XmlFragment {
                    xml_id,
                    fragment,
                    matched_tag: tag,
                    xml_parent_id,
                }));
                ActionValue::Map(value)
            });
            result.extend(results);
        }
        _ => return Ok(result),
    }
    Ok(result)
}

fn recursive_fragment(
    node: RefNode,
    elements_to_match: &Vec<String>,
    elements_to_exclude: &Vec<String>,
) -> Result<Vec<RefNode>> {
    let mut result = Vec::<RefNode>::new();
    let tag = node.node_name().to_string();

    if elements_to_match.contains(&tag)
        && !elements_to_exclude.contains(&tag)
        && node.borrow().node_type == NodeType::Element
    {
        result.push(node.clone());
    }
    let data = node.child_nodes();
    let child_result = data
        .iter()
        .map(|v| {
            let child = v.clone();
            recursive_fragment(child, elements_to_match, elements_to_exclude).unwrap_or_default()
        })
        .collect::<Vec<Vec<RefNode>>>();
    for child in child_result.into_iter() {
        result.extend(child);
    }
    Ok(result)
}
