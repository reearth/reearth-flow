use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
};

use nusamai_citygml::GML31_NS;
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
use reearth_flow_types::{Attribute, AttributeValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::{PlateauProcessorError, Result};

pub static SUMMARY_PORT: Lazy<Port> = Lazy::new(|| Port::new("summary"));
pub static UNMATCHED_XLINK_FROM: Lazy<Port> = Lazy::new(|| Port::new("unMatchedXlinkFrom"));
pub static UNMATCHED_XLINK_TO: Lazy<Port> = Lazy::new(|| Port::new("unMatchedXlinkTo"));

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct Summary {
    xlink_from_count: u32,
    xlink_to_count: u32,
    unmatched_xlink_from_count: u32,
    unmatched_xlink_to_count: u32,
}

impl From<Summary> for HashMap<Attribute, AttributeValue> {
    fn from(value: Summary) -> Self {
        serde_json::to_value(value)
            .unwrap()
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (Attribute::new(k), AttributeValue::from(v.clone())))
            .collect()
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct Response {
    gml_id: String,
    unmatched_xlink_from_id: Vec<String>,
    unmatched_xlink_from_tag: Vec<String>,
    unmatched_xlink_from_count: u32,
    unmatched_xlink_to_id: Vec<String>,
    unmatched_xlink_to_tag: Vec<String>,
    unmatched_xlink_to_count: u32,
}

impl From<Response> for HashMap<Attribute, AttributeValue> {
    fn from(value: Response) -> Self {
        serde_json::to_value(value)
            .unwrap()
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (Attribute::new(k), AttributeValue::from(v.clone())))
            .collect()
    }
}

impl From<XlinkGmlElement> for Response {
    fn from(value: XlinkGmlElement) -> Self {
        let from_ids: HashSet<String> = value.from.keys().cloned().collect();
        let to_ids: HashSet<String> = value.to.keys().cloned().collect();
        let fi = from_ids.difference(&to_ids);
        let ti = to_ids.difference(&from_ids);
        let fi: Vec<String> = fi.cloned().collect();
        let ti: Vec<String> = ti.cloned().collect();
        Response {
            gml_id: value.gml_id,
            unmatched_xlink_from_id: fi.clone(),
            unmatched_xlink_from_tag: fi
                .iter()
                .map(|id| value.from.get(id).unwrap().clone())
                .collect(),
            unmatched_xlink_from_count: fi.len() as u32,
            unmatched_xlink_to_id: ti.clone(),
            unmatched_xlink_to_tag: ti
                .iter()
                .map(|id| value.to.get(id).unwrap().clone())
                .collect(),
            unmatched_xlink_to_count: ti.len() as u32,
        }
    }
}

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("reearth flow common error: {0}")]
    InvalidUri(#[from] reearth_flow_common::Error),
    #[error("Unmatched Xlink Detector Error: {0}")]
    UnmatchedXlinkDetector(String),
    #[error("Failed to convert bytes to string")]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error("Storage Error: {0}")]
    Storage(#[from] reearth_flow_storage::Error),
    #[error("Object Store Error: {0}")]
    ObjectStore(#[from] object_store::Error),
}

#[derive(Debug, Default, Clone)]
struct XlinkGmlElement {
    gml_id: String,
    from: HashMap<String, String>,
    to: HashMap<String, String>,
}

#[derive(Debug, Clone, Default)]
pub struct UnmatchedXlinkDetectorFactory;

impl ProcessorFactory for UnmatchedXlinkDetectorFactory {
    fn name(&self) -> &str {
        "PLATEAU4.UnmatchedXlinkDetector"
    }

    fn description(&self) -> &str {
        "Detect unmatched Xlinks for PLATEAU"
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
        vec![
            SUMMARY_PORT.clone(),
            UNMATCHED_XLINK_FROM.clone(),
            UNMATCHED_XLINK_TO.clone(),
        ]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: UnmatchedXlinkDetectorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::UnmatchedXlinkDetectorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::UnmatchedXlinkDetectorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(PlateauProcessorError::UnmatchedXlinkDetectorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };
        let process = UnmatchedXlinkDetector { params };
        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnmatchedXlinkDetectorParam {
    attribute: Attribute,
}

#[derive(Debug, Clone)]
pub struct UnmatchedXlinkDetector {
    params: UnmatchedXlinkDetectorParam,
}

impl Processor for UnmatchedXlinkDetector {
    fn process(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), BoxedError> {
        self.process_impl(ctx, fw).map_err(Into::into)
    }

    fn finish(&self, _ctx: NodeContext, _fw: &ProcessorChannelForwarder) -> Result<(), BoxedError> {
        Ok(())
    }

    fn name(&self) -> &str {
        "UnmatchedXlinkDetector"
    }
}

impl UnmatchedXlinkDetector {
    fn process_impl(
        &mut self,
        ctx: ExecutorContext,
        fw: &ProcessorChannelForwarder,
    ) -> Result<(), Error> {
        let feature = &ctx.feature;
        let uri = feature
            .attributes
            .get(&self.params.attribute)
            .ok_or(Error::UnmatchedXlinkDetector("Required URI".to_string()))?;
        let uri = match uri {
            AttributeValue::String(s) => Uri::from_str(s)?,
            _ => {
                return Err(Error::UnmatchedXlinkDetector(
                    "Invalid Attribute".to_string(),
                ))
            }
        };
        let storage = ctx.storage_resolver.resolve(&uri)?;
        let content = storage.get_sync(uri.path().as_path())?;
        let xml_content = String::from_utf8(content.to_vec())?;
        let document = xml::parse(xml_content)?;
        let nodes = xml::find_readonly_nodes_by_xpath(&xml_ctx, BUILDING_XPATH_QUERY, &root_node)?;
        let mut summary = Summary::default();
        for node in nodes {
            let xlink_gml_element = extract_xlink_gml_element(&xml_ctx, &node)?;
            let Some(xlink_gml_element) = xlink_gml_element else {
                continue;
            };
            summary.xlink_from_count += xlink_gml_element.from.len() as u32;
            summary.xlink_to_count += xlink_gml_element.to.len() as u32;
            let response = Response::from(xlink_gml_element);
            summary.unmatched_xlink_from_count += response.unmatched_xlink_from_count;
            summary.unmatched_xlink_to_count += response.unmatched_xlink_to_count;
            let mut ports = Vec::<Port>::new();
            if response.unmatched_xlink_from_count > 0 {
                ports.push(UNMATCHED_XLINK_FROM.clone());
            }
            if response.unmatched_xlink_to_count > 0 {
                ports.push(UNMATCHED_XLINK_TO.clone());
            }
            for port in ports {
                let mut feature = feature.clone();
                feature.refresh_id();
                let attributes: HashMap<Attribute, AttributeValue> = response.clone().into();
                feature.attributes.extend(attributes);
                fw.send(ctx.new_with_feature_and_port(feature, port.clone()));
            }
        }
        let mut feature = feature.clone();
        let attributes: HashMap<Attribute, AttributeValue> = summary.clone().into();
        feature.attributes.extend(attributes);
        fw.send(ctx.new_with_feature_and_port(feature.clone(), SUMMARY_PORT.clone()));
        Ok(())
    }
}

fn extract_xlink_gml_element(
    xml_ctx: &XmlContext,
    node: &XmlRoNode,
) -> Result<Option<XlinkGmlElement>, Error> {
    let gml_id = node
        .get_attribute_ns(
            "id",
            String::from_utf8(GML31_NS.into_inner().to_vec())?.as_str(),
        )
        .ok_or(Error::UnmatchedXlinkDetector(
            "Failed to get gml id".to_string(),
        ))?;
    let mut xlink_from = HashMap::<String, String>::new();
    let mut xlink_to = HashMap::<String, String>::new();
    for tag in ["lod2Solid", "lod3Solid", "lod4Solid", "lod4MultiSurface"] {
        let elements = xml::find_readonly_nodes_by_xpath(
            xml_ctx,
            format!("bldg:{tag}//gml:surfaceMember[@xlink:href] | bldg:{tag}//gml:baseSurface[@xlink:href]").as_str(),
            node,
        )?;
        let from = elements
            .iter()
            .flat_map(|element| {
                let xlink = element.get_attribute_ns("href", XLINK_NS)?;
                Some((xlink.replace("#", ""), tag.to_string()))
            })
            .collect::<HashMap<String, String>>();
        xlink_from.extend(from);
    }
    let elements =
        xml::find_readonly_nodes_by_xpath(xml_ctx, "bldg:boundedBy/*//gml:Polygon[@gml:id]", node)?;
    for element in &elements {
        let gml_id = element
            .get_attribute_ns(
                "id",
                String::from_utf8(GML31_NS.into_inner().to_vec())?.as_str(),
            )
            .ok_or(Error::UnmatchedXlinkDetector(
                "Failed to get gml id".to_string(),
            ))?;

        xlink_to.insert(gml_id, xml::get_readonly_node_tag(element));
    }
    Ok(Some(XlinkGmlElement {
        gml_id,
        from: xlink_from,
        to: xlink_to,
    }))
}
