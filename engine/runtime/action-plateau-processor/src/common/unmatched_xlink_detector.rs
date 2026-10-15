//! Unmatched-xlink detector (L-bldg-06), shared across PLATEAU generations.
//!
//! A building solid (`bldg:lod2Solid` in CityGML 2.0, `core:lod2Solid` in 3.0)
//! references its boundary-surface polygons by `xlink:href`. This check reads the
//! raw GML and reports references that do not resolve:
//!
//! - **unmatched "from"**: an `xlink:href` in the solid that points to a
//!   `gml:id` which is not defined among the boundary polygons.
//! - **unmatched "to"**: a boundary `gml:Polygon[@gml:id]` that no solid
//!   `xlink:href` references.
//!
//! The generation-independent orchestration (raw-GML load, per-container
//! traversal, set difference, port emission, per-file summary) lives here as a
//! template method. The generation-specific seam — which elements to scan, which
//! LOD geometry tags carry the references, where boundary surfaces live and which
//! `gml` namespace resolves `gml:id` — is injected as a [`UnmatchedXlinkStrategy`]
//! trait object, so a generation whose extraction *logic* (not merely its
//! constants) differs can override the behavioral methods without touching this
//! file.

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt::Debug,
    rc::Rc,
    str::FromStr,
};

use fastxml::transform::Transformer;
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
    node::{Port, Processor, ProcessorFactory, FEATURES_PORT},
};
use reearth_flow_types::{Attribute, AttributeValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::{PlateauProcessorError, Result};
use super::PlateauProfile;

/// XML namespace for `xlink:href`. Identical across CityGML 2.0/3.0.
const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

pub static SUMMARY_PORT: Lazy<Port> = Lazy::new(|| Port::new("summary"));
pub static UNMATCHED_XLINK_FROM: Lazy<Port> = Lazy::new(|| Port::new("unMatchedXlinkFrom"));
pub static UNMATCHED_XLINK_TO: Lazy<Port> = Lazy::new(|| Port::new("unMatchedXlinkTo"));

/// A single feature container (`bldg:Building` etc.) with its resolved xlink sets.
#[derive(Debug, Default, Clone)]
pub(crate) struct XlinkGmlElement {
    pub gml_id: String,
    /// `xlink:href` target id -> LOD tag (local name) that carries the reference.
    pub from: HashMap<String, String>,
    /// boundary `gml:id` -> element tag (`gml:Polygon`).
    pub to: HashMap<String, String>,
}

/// Generation-specific seam for the unmatched-xlink check.
///
/// The four accessors describe *where* the references and definitions live; the
/// `extract_*` methods carry the default extraction built from those accessors
/// and are the override points for a generation whose extraction logic genuinely
/// differs (rather than only its constants).
pub(crate) trait UnmatchedXlinkStrategy: Send + Sync + Debug {
    /// Feature containers to scan, as `//{name}` roots (e.g. `bldg:Building`).
    fn containers(&self) -> &[&str];
    /// Qualified LOD geometry tags whose descendant `xlink:href` reference
    /// boundary polygons (e.g. `core:lod2Solid`). Used for the has-geometry gate,
    /// the "from" XPath, and the reported tag (its local name).
    fn lod_geometry_tags(&self) -> &[&str];
    /// Qualified container element holding boundary surfaces (e.g. `core:boundary`).
    fn boundary_container(&self) -> &str;
    /// XML namespace that resolves `gml:id` (differs by GML version).
    fn gml_namespace(&self) -> &str;

    /// Extract the "from" set: `xlink:href` target id -> LOD tag local name.
    fn extract_from(
        &self,
        xml_ctx: &XmlContext,
        node: &XmlRoNode,
    ) -> Result<HashMap<String, String>, Error> {
        let mut from = HashMap::<String, String>::new();
        for tag in self.lod_geometry_tags() {
            let local = tag.rsplit(':').next().unwrap_or(tag);
            let elements = xml::find_readonly_nodes_by_xpath(
                xml_ctx,
                format!(
                    "{tag}//gml:surfaceMember[@xlink:href] | {tag}//gml:baseSurface[@xlink:href]"
                )
                .as_str(),
                node,
            )
            .map_err(|e| Error::UnmatchedXlinkDetector(format!("{e:?}")))?;
            for element in &elements {
                if let Some(xlink) = element.get_attribute_ns("href", XLINK_NS) {
                    from.insert(xlink.replace('#', ""), local.to_string());
                }
            }
        }
        Ok(from)
    }

    /// Extract the "to" set: boundary `gml:id` -> element tag (`gml:Polygon`).
    fn extract_to(
        &self,
        xml_ctx: &XmlContext,
        node: &XmlRoNode,
    ) -> Result<HashMap<String, String>, Error> {
        let mut to = HashMap::<String, String>::new();
        let elements = xml::find_readonly_nodes_by_xpath(
            xml_ctx,
            format!("{}/*//gml:Polygon[@gml:id]", self.boundary_container()).as_str(),
            node,
        )
        .map_err(|e| Error::UnmatchedXlinkDetector(format!("{e:?}")))?;
        for element in &elements {
            let gml_id = element.get_attribute_ns("id", self.gml_namespace()).ok_or(
                Error::UnmatchedXlinkDetector("Failed to get gml id".to_string()),
            )?;
            to.insert(gml_id, xml::get_readonly_node_tag(element));
        }
        Ok(to)
    }

    /// Build the resolved xlink element for one container node: its `gml:id`
    /// plus the "from" / "to" sets. Composes the accessors above, so a generation
    /// only overrides this when the assembly itself differs.
    fn build_element(
        &self,
        xml_ctx: &XmlContext,
        node: &XmlRoNode,
    ) -> Result<XlinkGmlElement, Error> {
        let gml_id = node.get_attribute_ns("id", self.gml_namespace()).ok_or(
            Error::UnmatchedXlinkDetector("Failed to get gml id".to_string()),
        )?;
        let from = self.extract_from(xml_ctx, node)?;
        let to = self.extract_to(xml_ctx, node)?;
        Ok(XlinkGmlElement { gml_id, from, to })
    }
}

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
pub(crate) enum Error {
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

#[derive(Debug, Clone)]
pub(crate) struct UnmatchedXlinkDetectorFactory {
    name: String,
    strategy: &'static dyn UnmatchedXlinkStrategy,
}

impl UnmatchedXlinkDetectorFactory {
    pub(crate) fn new(
        profile: &PlateauProfile,
        strategy: &'static dyn UnmatchedXlinkStrategy,
    ) -> Self {
        Self {
            name: profile.action_name("UnmatchedXlinkDetector"),
            strategy,
        }
    }
}

impl ProcessorFactory for UnmatchedXlinkDetectorFactory {
    fn name(&self) -> &str {
        &self.name
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
        vec![FEATURES_PORT.clone()]
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
        let process = UnmatchedXlinkDetector {
            params,
            strategy: self.strategy,
        };
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
    strategy: &'static dyn UnmatchedXlinkStrategy,
}

impl Processor for UnmatchedXlinkDetector {
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

        let summary = Rc::new(RefCell::new(Summary::default()));
        let stream_error: Rc<RefCell<Option<Error>>> = Rc::new(RefCell::new(None));

        let transformer = Transformer::from(xml_content.as_str())
            .with_root_namespaces()
            .map_err(|e| Error::UnmatchedXlinkDetector(format!("{e:?}")))?;

        let strategy = self.strategy;
        let lod_tags = strategy.lod_geometry_tags();

        let mut t = transformer;
        for target in strategy.containers() {
            let xpath = format!("//{target}");
            let summary = Rc::clone(&summary);
            let stream_error = Rc::clone(&stream_error);
            let ctx = &ctx;

            t = t.on(&xpath, move |node| {
                if stream_error.borrow().is_some() {
                    return;
                }

                // Gate: the container must carry at least one target LOD geometry.
                let has_lod = node
                    .children()
                    .iter()
                    .any(|c| lod_tags.contains(&c.qname().as_str()));
                if !has_lod {
                    return;
                }

                let doc = node.document();
                let mut xml_ctx = match xml::create_context(doc) {
                    Ok(ctx) => ctx,
                    Err(e) => {
                        *stream_error.borrow_mut() =
                            Some(Error::UnmatchedXlinkDetector(format!("{e:?}")));
                        return;
                    }
                };
                for (prefix, uri) in node.namespaces() {
                    let _ = xml_ctx.register_namespace(prefix, uri);
                }
                let root_node = match xml::get_root_readonly_node(doc) {
                    Ok(n) => n,
                    Err(e) => {
                        *stream_error.borrow_mut() =
                            Some(Error::UnmatchedXlinkDetector(format!("{e:?}")));
                        return;
                    }
                };

                match strategy.build_element(&xml_ctx, &root_node) {
                    Ok(xlink_gml_element) => {
                        let mut s = summary.borrow_mut();
                        s.xlink_from_count += xlink_gml_element.from.len() as u32;
                        s.xlink_to_count += xlink_gml_element.to.len() as u32;
                        let response = Response::from(xlink_gml_element);
                        s.unmatched_xlink_from_count += response.unmatched_xlink_from_count;
                        s.unmatched_xlink_to_count += response.unmatched_xlink_to_count;
                        drop(s);

                        let mut ports = Vec::<Port>::new();
                        if response.unmatched_xlink_from_count > 0 {
                            ports.push(UNMATCHED_XLINK_FROM.clone());
                        }
                        if response.unmatched_xlink_to_count > 0 {
                            ports.push(UNMATCHED_XLINK_TO.clone());
                        }
                        for port in ports {
                            let mut f = feature.clone();
                            f.refresh_id();
                            let attributes: HashMap<Attribute, AttributeValue> =
                                response.clone().into();
                            f.attributes_mut().extend(attributes);
                            fw.send(ctx.new_with_feature_and_port(f, port.clone()));
                        }
                    }
                    Err(e) => {
                        *stream_error.borrow_mut() = Some(e);
                    }
                }
            });
        }

        t.for_each()
            .map_err(|e| Error::UnmatchedXlinkDetector(format!("{e:?}")))?;

        if let Some(err) = Rc::try_unwrap(stream_error)
            .expect("all callback references should be dropped after for_each()")
            .into_inner()
        {
            return Err(err);
        }

        let summary = Rc::try_unwrap(summary).unwrap().into_inner();
        let mut feature = feature.clone();
        let attributes: HashMap<Attribute, AttributeValue> = summary.clone().into();
        feature.attributes_mut().extend(attributes);
        fw.send(ctx.new_with_feature_and_port(feature.clone(), SUMMARY_PORT.clone()));
        Ok(())
    }
}
