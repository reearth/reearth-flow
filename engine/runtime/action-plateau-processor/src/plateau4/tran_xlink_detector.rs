use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
    str::FromStr,
};

use fastxml::transform::StreamTransformer;
use once_cell::sync::Lazy;
use reearth_flow_common::{
    uri::Uri,
    xml::{self, XmlContext, XmlRoNode},
};
use reearth_flow_runtime::{
    errors::BoxedError,
    event::EventHub,
    executor_operation::{ExecutorContext, NodeContext},
    forwarder::ProcessorChannelForwarder,
    node::{Port, Processor, ProcessorFactory, DEFAULT_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue, Expr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::{PlateauProcessorError, Result};

pub static PASSED_PORT: Lazy<Port> = Lazy::new(|| Port::new("passed"));
pub static FAILED_PORT: Lazy<Port> = Lazy::new(|| Port::new("failed"));

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("reearth flow common error: {0}")]
    InvalidUri(#[from] reearth_flow_common::Error),
    #[error("Transportation XLink Detector Error: {0}")]
    TranXlinkDetector(String),
    #[error("Failed to convert bytes to string")]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error("Storage Error: {0}")]
    Storage(#[from] reearth_flow_storage::Error),
    #[error("Object Store Error: {0}")]
    ObjectStore(#[from] object_store::Error),
}

#[derive(Debug, Clone, Default)]
pub struct TransportationXlinkDetectorFactory;

impl ProcessorFactory for TransportationXlinkDetectorFactory {
    fn name(&self) -> &str {
        "PLATEAU4.TransportationXlinkDetector"
    }

    fn description(&self) -> &str {
        "Detect unreferenced surfaces in PLATEAU transportation models (L-TRAN-03)"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(TransportationXlinkDetectorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![DEFAULT_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![PASSED_PORT.clone(), FAILED_PORT.clone()]
    }

    fn build(
        &self,
        ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: TransportationXlinkDetectorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::TranXlinkDetectorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::TranXlinkDetectorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(PlateauProcessorError::TranXlinkDetectorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let city_gml_path = ctx
            .expr_engine
            .compile(params.city_gml_path.as_ref())
            .map_err(|e| {
                PlateauProcessorError::TranXlinkDetectorFactory(format!(
                    "Failed to compile city_gml_path: {e}"
                ))
            })?;

        let process = TransportationXlinkDetector { city_gml_path };
        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TransportationXlinkDetectorParam {
    city_gml_path: Expr,
}

#[derive(Debug, Clone)]
pub struct TransportationXlinkDetector {
    city_gml_path: rhai::AST,
}

impl Processor for TransportationXlinkDetector {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        self.process_impl(ctx, fw).map_err(Into::into)
    }

    fn finish(
        &mut self,
        _ctx: NodeContext,
        _fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "TransportationXlinkDetector"
    }
}

impl TransportationXlinkDetector {
    fn process_impl(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), Error> {
        let feature = &ctx.feature;
        let city_gml_path = {
            let scope = feature.new_scope(ctx.expr_engine.clone(), &None);
            scope.eval_ast::<String>(&self.city_gml_path).map_err(|e| {
                Error::TranXlinkDetector(format!(
                    "Failed to evaluate cityGmlPath expression: {e:?}"
                ))
            })?
        };
        let uri = Uri::from_str(&city_gml_path)?;
        let storage = ctx.storage_resolver.resolve(&uri)?;
        let content = storage.get_sync(uri.path().as_path())?;
        let xml_content = String::from_utf8(content.to_vec())?;

        let stream_error: Rc<RefCell<Option<Error>>> = Rc::new(RefCell::new(None));

        let transformer = StreamTransformer::new(&xml_content)
            .with_root_namespaces()
            .map_err(|e| Error::TranXlinkDetector(format!("{e:?}")))?;

        let stream_error_clone = Rc::clone(&stream_error);
        let ctx = &ctx;
        transformer
            .on("//tran:Road", move |node| {
                if stream_error_clone.borrow().is_some() {
                    return;
                }

                // Check LOD condition: element must have lod2 or lod3 MultiSurface child
                let has_lod = node.children().iter().any(|c| {
                    let qname = c.qname();
                    qname == "tran:lod2MultiSurface" || qname == "tran:lod3MultiSurface"
                });
                if !has_lod {
                    return;
                }

                let doc = node.document();
                let mut xml_ctx = match xml::create_context(doc) {
                    Ok(ctx) => ctx,
                    Err(e) => {
                        *stream_error_clone.borrow_mut() =
                            Some(Error::TranXlinkDetector(format!("{e:?}")));
                        return;
                    }
                };
                for (prefix, uri) in node.namespaces() {
                    let _ = xml_ctx.register_namespace(prefix, uri);
                }
                let root_node = match xml::get_root_readonly_node(doc) {
                    Ok(n) => n,
                    Err(e) => {
                        *stream_error_clone.borrow_mut() =
                            Some(Error::TranXlinkDetector(format!("{e:?}")));
                        return;
                    }
                };

                match extract_unreferenced_surfaces(&xml_ctx, &root_node) {
                    Ok(Some(unreferenced_surfaces)) => {
                        for (lod, surface_id) in unreferenced_surfaces.unreferenced_surfaces {
                            let mut feature = feature.clone();
                            feature.refresh_id();

                            feature.attributes_mut().insert(
                                Attribute::new("gmlId"),
                                AttributeValue::String(unreferenced_surfaces.road_id.clone()),
                            );
                            feature.attributes_mut().insert(
                                Attribute::new("featureType"),
                                AttributeValue::String("Road".to_string()),
                            );
                            feature
                                .attributes_mut()
                                .insert(Attribute::new("lod"), AttributeValue::String(lod));
                            feature.attributes_mut().insert(
                                Attribute::new("unreferenced"),
                                AttributeValue::String(surface_id),
                            );

                            fw.send(ctx.new_with_feature_and_port(feature, FAILED_PORT.clone()));
                        }
                    }
                    Ok(None) => {
                        let feature = feature.clone();
                        fw.send(ctx.new_with_feature_and_port(feature, PASSED_PORT.clone()));
                    }
                    Err(e) => {
                        *stream_error_clone.borrow_mut() = Some(e);
                    }
                }
            })
            .for_each()
            .map_err(|e| Error::TranXlinkDetector(format!("{e:?}")))?;

        if let Some(err) = Rc::try_unwrap(stream_error)
            .expect("all callback references should be dropped after for_each()")
            .into_inner()
        {
            return Err(err);
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct UnreferencedSurfacesResult {
    road_id: String,
    unreferenced_surfaces: Vec<(String, String)>, // (lod, surface_id)
}

fn extract_unreferenced_surfaces(
    xml_ctx: &XmlContext,
    road_node: &XmlRoNode,
) -> Result<Option<UnreferencedSurfacesResult>, Error> {
    // Get Road gml:id
    let road_id = road_node
        .get_attribute_ns("id", "http://www.opengis.net/gml")
        .ok_or(Error::TranXlinkDetector(
            "Failed to get Road gml:id".to_string(),
        ))?;

    let mut all_unreferenced = Vec::new();

    // Process LOD2
    if let Some(unreferenced) = check_lod_surfaces(xml_ctx, road_node, "lod2", "2")? {
        all_unreferenced.extend(unreferenced);
    }

    // Process LOD3
    if let Some(unreferenced) = check_lod_surfaces(xml_ctx, road_node, "lod3", "3")? {
        all_unreferenced.extend(unreferenced);
    }

    if all_unreferenced.is_empty() {
        Ok(None)
    } else {
        Ok(Some(UnreferencedSurfacesResult {
            road_id,
            unreferenced_surfaces: all_unreferenced,
        }))
    }
}

fn check_lod_surfaces(
    xml_ctx: &XmlContext,
    road_node: &XmlRoNode,
    lod_tag: &str,
    lod_number: &str,
) -> Result<Option<Vec<(String, String)>>, Error> {
    // Get all XLink references from Road's lodXMultiSurface
    let xlink_refs = xml::find_readonly_nodes_by_xpath(
        xml_ctx,
        &format!("tran:{lod_tag}MultiSurface//gml:surfaceMember[@xlink:href]"),
        road_node,
    )?;

    let referenced_surfaces: HashSet<String> = xlink_refs
        .iter()
        .filter_map(|node| {
            let href = node.get_attribute_ns("href", "http://www.w3.org/1999/xlink")?;
            Some(href.trim_start_matches('#').to_string())
        })
        .collect();

    // Get all child surface IDs from TrafficArea and AuxiliaryTrafficArea
    let child_surface_nodes = xml::find_readonly_nodes_by_xpath(
        xml_ctx,
        &format!(
            "tran:trafficArea/tran:TrafficArea/tran:{lod_tag}MultiSurface//gml:Polygon[@gml:id] | \
             tran:auxiliaryTrafficArea/tran:AuxiliaryTrafficArea/tran:{lod_tag}MultiSurface//gml:Polygon[@gml:id]"
        ),
        road_node,
    )?;

    let mut child_surfaces = Vec::new();
    for surface_node in child_surface_nodes {
        if let Some(surface_id) = surface_node.get_attribute_ns("id", "http://www.opengis.net/gml")
        {
            child_surfaces.push(surface_id);
        }
    }

    // Find unreferenced surfaces
    let unreferenced: Vec<(String, String)> = child_surfaces
        .into_iter()
        .filter(|surface_id| !referenced_surfaces.contains(surface_id))
        .map(|surface_id| (lod_number.to_string(), surface_id))
        .collect();

    if unreferenced.is_empty() {
        Ok(None)
    } else {
        Ok(Some(unreferenced))
    }
}
