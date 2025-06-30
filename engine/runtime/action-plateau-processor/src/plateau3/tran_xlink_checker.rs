use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use itertools::Itertools;
use nusamai_citygml::{GML31_NS, XLINK_NS};
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
use reearth_flow_types::{Attribute, AttributeValue};
use serde_json::Value;

use super::errors::PlateauProcessorError;

#[derive(Debug, Clone, Default)]
pub struct TranXLinkCheckerFactory;

impl ProcessorFactory for TranXLinkCheckerFactory {
    fn name(&self) -> &str {
        "PLATEAU3.TranXLinkChecker"
    }

    fn description(&self) -> &str {
        "Check Xlink for Tran"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        None
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
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
        _with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let process = TranXLinkChecker {};
        Ok(Box::new(process))
    }
}

#[derive(Debug, Clone)]
pub struct TranXLinkChecker {}

impl Processor for TranXLinkChecker {
    fn num_threads(&self) -> usize {
        2
    }

    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        let feature = &ctx.feature;

        let city_gml_path = feature
            .attributes
            .get(&Attribute::new("cityGmlPath"))
            .ok_or(PlateauProcessorError::TranXLinkChecker(
                "cityGmlPath key empty".to_string(),
            ))?;

        let uri = Uri::from_str(city_gml_path.to_string().as_str()).map_err(|err| {
            PlateauProcessorError::TranXLinkChecker(format!(
                "cityGmlPath is not a valid uri: {err}"
            ))
        })?;
        let storage = ctx
            .storage_resolver
            .resolve(&uri)
            .map_err(|e| PlateauProcessorError::TranXLinkChecker(format!("{e:?}")))?;
        let content = storage
            .get_sync(uri.path().as_path())
            .map_err(|e| PlateauProcessorError::TranXLinkChecker(format!("{e:?}")))?;
        let xml_content = String::from_utf8(content.to_vec())
            .map_err(|_| PlateauProcessorError::TranXLinkChecker("Invalid UTF-8".to_string()))?;
        let Ok(document) = xml::parse(xml_content) else {
            return Err(
                PlateauProcessorError::TranXLinkChecker("Failed to parse XML".to_string()).into(),
            );
        };
        let xml_ctx = xml::create_context(&document).map_err(|e| {
            PlateauProcessorError::TranXLinkChecker(format!("Failed to create xml context: {e}"))
        })?;
        let root_node = match xml::get_root_readonly_node(&document) {
            Ok(node) => node,
            Err(e) => {
                return Err(PlateauProcessorError::TranXLinkChecker(format!(
                    "Failed to get root node: {e}"
                ))
                .into());
            }
        };
        let nodes = xml::find_readonly_nodes_by_xpath(&xml_ctx, "*//tran:Road", &root_node)
            .map_err(|e| {
                PlateauProcessorError::TranXLinkChecker(format!(
                    "Failed to find_readonly_nodes_in_elements with {e}"
                ))
            })?;
        for road in &nodes {
            let gml_id = road
                .get_attribute_ns("id", std::str::from_utf8(GML31_NS.into_inner()).unwrap())
                .unwrap_or_default();
            let traffix_areas =
                xml::find_readonly_nodes_by_xpath(&xml_ctx, ".//tran:TrafficArea", road).map_err(
                    |e| {
                        PlateauProcessorError::TranXLinkChecker(format!(
                            "Failed to find_readonly_nodes_in_elements with {e}"
                        ))
                    },
                )?;
            let mut lod2_trf_gml_ids = Vec::new();
            let mut lod3_trf_gml_ids = Vec::new();
            for traffix_area in &traffix_areas {
                lod2_trf_gml_ids.extend(extract_gml_ids_by_xpath(
                    &xml_ctx,
                    "./tran:lod2MultiSurface/gml:MultiSurface/gml:surfaceMember/*[@gml:id]",
                    traffix_area,
                ));
                lod3_trf_gml_ids.extend(extract_gml_ids_by_xpath(
                    &xml_ctx,
                    "./tran:lod3MultiSurface/gml:MultiSurface/gml:surfaceMember/*[@gml:id]",
                    traffix_area,
                ));
            }
            let aux_traffix_areas =
                xml::find_readonly_nodes_by_xpath(&xml_ctx, ".//tran:AuxiliaryTrafficArea", road)
                    .map_err(|e| {
                    PlateauProcessorError::TranXLinkChecker(format!(
                        "Failed to find_readonly_nodes_in_elements with {e}"
                    ))
                })?;
            let mut lod2_aux_gml_ids = Vec::new();
            let mut lod3_aux_gml_ids = Vec::new();
            for aux_traffix_area in &aux_traffix_areas {
                lod2_aux_gml_ids.extend(extract_gml_ids_by_xpath(
                    &xml_ctx,
                    "./tran:lod2MultiSurface/gml:MultiSurface/gml:surfaceMember/*[@gml:id]",
                    aux_traffix_area,
                ));
                lod3_aux_gml_ids.extend(extract_gml_ids_by_xpath(
                    &xml_ctx,
                    "./tran:lod3MultiSurface/gml:MultiSurface/gml:surfaceMember/*[@gml:id]",
                    aux_traffix_area,
                ));
            }
            let lod2xlinks =
                extract_xlink_by_xpath(&xml_ctx, "./tran:lod2MultiSurface//*[@xlink:href]", road);
            let lod3xlinks =
                extract_xlink_by_xpath(&xml_ctx, "./tran:lod3MultiSurface//*[@xlink:href]", road);
            if !lod2_trf_gml_ids.is_empty() {
                let lod2_trf_gml_ids: HashSet<_> = lod2_trf_gml_ids.into_iter().collect();
                let lod2_aux_gml_ids: HashSet<_> = lod2_aux_gml_ids.into_iter().collect();
                let lod2_xlinks: HashSet<_> = lod2xlinks.into_iter().collect();
                let lod2_gml_ids: HashSet<_> = lod2_trf_gml_ids
                    .union(&lod2_aux_gml_ids)
                    .cloned()
                    .collect::<HashSet<_>>();
                let lod2_gml_ids_difference: HashSet<_> = lod2_gml_ids
                    .difference(&lod2_xlinks)
                    .cloned()
                    .collect::<HashSet<_>>();
                let mut feature = feature.clone();
                feature.refresh_id();
                feature.insert("gmlId", AttributeValue::String(gml_id.clone()));
                feature.insert("featureType", AttributeValue::String("Road".to_string()));
                feature.insert("lod", AttributeValue::String("2".to_string()));
                feature.insert(
                    "unreferencedSurfaceNum",
                    AttributeValue::Number(lod2_gml_ids_difference.len().into()),
                );
                feature.insert(
                    "unreferencedIds",
                    AttributeValue::Array(
                        lod2_gml_ids_difference
                            .into_iter()
                            .map(AttributeValue::String)
                            .collect(),
                    ),
                );
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
            if !lod3_trf_gml_ids.is_empty() {
                let lod3_trf_gml_ids: HashSet<_> = lod3_trf_gml_ids.into_iter().collect();
                let lod3_aux_gml_ids: HashSet<_> = lod3_aux_gml_ids.into_iter().collect();
                let lod3_xlinks: HashSet<_> = lod3xlinks.into_iter().collect();
                let lod3_gml_ids: HashSet<_> = lod3_trf_gml_ids
                    .union(&lod3_aux_gml_ids)
                    .cloned()
                    .collect::<HashSet<_>>();
                let lod3_gml_ids_difference: HashSet<_> = lod3_gml_ids
                    .difference(&lod3_xlinks)
                    .cloned()
                    .collect::<HashSet<_>>();
                let mut feature = feature.clone();
                feature.refresh_id();
                feature.insert("gmlId", AttributeValue::String(gml_id.clone()));
                feature.insert("featureType", AttributeValue::String("Road".to_string()));
                feature.insert("lod", AttributeValue::String("3".to_string()));
                feature.insert(
                    "unreferencedSurfaceNum",
                    AttributeValue::Number(lod3_gml_ids_difference.len().into()),
                );
                feature.insert(
                    "unreferencedIds",
                    AttributeValue::Array(
                        lod3_gml_ids_difference
                            .into_iter()
                            .map(AttributeValue::String)
                            .collect(),
                    ),
                );
                fw.send(ctx.new_with_feature_and_port(feature, DEFAULT_PORT.clone()));
            }
        }
        Ok(())
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "TranXLinkChecker"
    }
}

fn extract_gml_ids_by_xpath(ctx: &XmlContext, xpath: &str, node: &XmlRoNode) -> Vec<String> {
    let nodes = xml::find_readonly_nodes_by_xpath(ctx, xpath, node).unwrap_or_default();
    nodes
        .iter()
        .flat_map(|node| {
            node.get_attribute_ns("id", std::str::from_utf8(GML31_NS.into_inner()).unwrap())
        })
        .collect_vec()
}

fn extract_xlink_by_xpath(ctx: &XmlContext, xpath: &str, node: &XmlRoNode) -> Vec<String> {
    let nodes = xml::find_readonly_nodes_by_xpath(ctx, xpath, node).unwrap_or_default();
    nodes
        .iter()
        .flat_map(|node| {
            let href =
                node.get_attribute_ns("href", std::str::from_utf8(XLINK_NS.into_inner()).unwrap())?;
            Some(href[1..].to_string())
        })
        .collect_vec()
}
