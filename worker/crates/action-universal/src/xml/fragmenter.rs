use std::{collections::HashMap, str::FromStr, sync::Arc};

use reearth_flow_action_log::action_log;
use reearth_flow_action_log::ActionLogger;
use reearth_flow_common::collection;
use reearth_flow_common::str::to_hash;
use reearth_flow_common::uri::Uri;
use reearth_flow_common::xml::{self, XmlDocument};
use reearth_flow_eval_expr::engine::Engine;
use reearth_flow_storage::resolve::StorageResolver;
use serde::{Deserialize, Serialize};

use reearth_flow_action::{
    error::Error, utils::convert_dataframe_to_scope_params, ActionContext, ActionDataframe,
    ActionResult, ActionValue, Result, SyncAction,
};
use tracing::Span;

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

#[typetag::serde(name = "XMLFragmenter")]
impl SyncAction for XmlFragmenter {
    fn run(&self, ctx: ActionContext, inputs: Option<ActionDataframe>) -> ActionResult {
        let inputs = inputs.ok_or(Error::input("No input dataframe"))?;
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
                let result = collection::par_map(&data, |row| {
                    action_value_to_fragment(
                        row,
                        &props.attribute,
                        &elements_to_match_ast,
                        &elements_to_exclude_ast,
                        &params,
                        Arc::clone(&expr_engine),
                        Arc::clone(&ctx.storage_resolver),
                        &ctx.root_span,
                        Arc::clone(&ctx.logger),
                    )
                    .unwrap_or_default()
                });
                ActionValue::Array(result.into_iter().flatten().collect())
            }
            ActionValue::Map(_) => {
                let result = action_value_to_fragment(
                    &data,
                    &props.attribute,
                    &elements_to_match_ast,
                    &elements_to_exclude_ast,
                    &params,
                    Arc::clone(&expr_engine),
                    Arc::clone(&ctx.storage_resolver),
                    &ctx.root_span,
                    Arc::clone(&ctx.logger),
                )?;
                ActionValue::Array(result)
            }
            _ => data,
        };
        output.insert(port, Some(processed_data));
    }
    Ok(output)
}

#[allow(clippy::too_many_arguments)]
fn action_value_to_fragment(
    row: &ActionValue,
    attribute: &String,
    elements_to_match_ast: &rhai::AST,
    elements_to_exclude_ast: &rhai::AST,
    params: &HashMap<String, ActionValue>,
    expr_engine: Arc<Engine>,
    storage_resolver: Arc<StorageResolver>,
    span: &Span,
    logger: Arc<ActionLogger>,
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
                Some(ActionValue::String(url)) => {
                    Uri::from_str(url).map_err(Error::internal_runtime)?
                }
                _ => return Err(Error::internal_runtime("No url found")),
            };
            let storage = storage_resolver
                .resolve(&url)
                .map_err(Error::internal_runtime)?;
            let bytes = storage
                .get_sync(&url.path())
                .map_err(Error::internal_runtime)?;
            let raw_xml = String::from_utf8(bytes.to_vec()).map_err(Error::internal_runtime)?;
            action_log!(
                parent: span,
                logger,
                "Parsing XML document: {:?} ...", url,
            );
            let document = xml::parse(raw_xml.as_str()).map_err(Error::internal_runtime)?;

            let fragments = generate_fragment(&document, &elements_to_match, &elements_to_exclude)?;
            for fragment in fragments {
                let mut value = row.clone();
                value.extend(XmlFragment::to_hashmap(fragment));
                result.push(ActionValue::Map(value));
            }
        }
        _ => return Ok(result),
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
    for node in nodes.iter_mut() {
        let node_type = node
            .get_type()
            .ok_or(Error::internal_runtime("No node type".to_string()))?;
        if node_type == xml::XmlNodeType::ElementNode {
            for ns in root.get_namespace_declarations().iter() {
                let _ = node
                    .set_attribute(
                        format!("xmlns:{}", ns.get_prefix()).as_str(),
                        ns.get_href().as_str(),
                    )
                    .map_err(|e| {
                        Error::internal_runtime(format!("Failed to set namespace with {:?}", e))
                    });
            }
            let tag = xml::get_node_tag(node);
            let fragment =
                xml::node_to_xml_string(document, node).map_err(Error::internal_runtime)?;
            let xml_id = to_hash(&fragment);
            let xml_parent_id = node
                .get_parent()
                .map(|parent| to_hash(format!("{:?}", parent).as_str()));
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
