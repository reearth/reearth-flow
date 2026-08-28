//! Transportation xlink-reference completeness detector (L-tran-03), shared
//! across PLATEAU generations.
//!
//! A `tran:Road` aggregates the boundary surfaces of its `TrafficArea` /
//! `AuxiliaryTrafficArea` into an aggregate `lodXMultiSurface` whose
//! `gml:surfaceMember` entries reference those polygons by `xlink:href`. This
//! check reads the raw GML and reports, per LOD, every boundary polygon
//! `gml:id` that the aggregate surface does not reference.
//!
//! The generation-independent orchestration (raw-GML load, per-container
//! traversal, set difference, port emission) lives here as a template method.
//! The generation-specific seam — the aggregate-surface namespace prefix, where
//! the boundary polygons live, and which `gml` namespace resolves `gml:id` — is
//! injected as a [`TransportationXlinkStrategy`] trait object, so a generation
//! whose extraction *logic* (not merely its constants) differs can override the
//! behavioral methods without touching this file.

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
use reearth_flow_types::{Attribute, AttributeValue, Code, CompiledCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::errors::{PlateauProcessorError, Result};
use super::PlateauProfile;

/// XML namespace for `xlink:href`. Identical across CityGML 2.0/3.0.
const XLINK_NS: &str = "http://www.w3.org/1999/xlink";

pub static PASSED_PORT: Lazy<Port> = Lazy::new(|| Port::new("passed"));
pub static FAILED_PORT: Lazy<Port> = Lazy::new(|| Port::new("failed"));

/// Generation-specific seam for the transportation xlink check.
///
/// The accessors describe *where* the aggregate references and the boundary
/// definitions live; the `aggregate_*` methods carry the default extraction
/// built from those accessors and are override points for a generation whose
/// extraction logic genuinely differs (rather than only its constants).
pub(crate) trait TransportationXlinkStrategy: Send + Sync + Debug {
    /// Feature containers to scan, as `//{name}` roots (e.g. `tran:Road`).
    fn containers(&self) -> &[&str];
    /// Namespace prefix of the container's own aggregate `lodXMultiSurface`
    /// (`tran` in CityGML 2.0, `core` in 3.0).
    fn aggregate_prefix(&self) -> &str;
    /// XML namespace that resolves `gml:id` (differs by GML version).
    fn gml_namespace(&self) -> &str;

    /// LODs to inspect, as (lod fragment, lod number label). LOD2 and LOD3 carry
    /// aggregate surfaces in both generations (LOD4 is abolished in 3.0 and was
    /// never scanned in 2.0).
    fn lods(&self) -> &[(&str, &str)] {
        &[("lod2", "2"), ("lod3", "3")]
    }

    /// Qualified aggregate multi-surface tag for a LOD (e.g. `tran:lod3MultiSurface`).
    fn aggregate_tag(&self, lod: &str) -> String {
        format!("{}:{lod}MultiSurface", self.aggregate_prefix())
    }

    /// XPath, relative to the container, matching the aggregate surface's
    /// `xlink:href` references for one LOD.
    fn aggregate_xlink_xpath(&self, lod: &str) -> String {
        format!(
            "{}//gml:surfaceMember[@xlink:href]",
            self.aggregate_tag(lod)
        )
    }

    /// XPath, relative to the container, matching the boundary polygons of one
    /// LOD (each keyed by its `gml:id`). Generation-specific: CityGML 2.0 nests
    /// boundaries directly under `tran:trafficArea` / `tran:auxiliaryTrafficArea`,
    /// while 3.0 nests them under `tran:trafficSpace` / `tran:auxiliaryTrafficSpace`
    /// -> `core:boundary`.
    fn child_surface_xpath(&self, lod: &str) -> String;
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum Error {
    #[error("reearth flow common error: {0}")]
    InvalidUri(#[from] reearth_flow_common::Error),
    #[error("Transportation XLink Detector Error: {0}")]
    TransportationXlinkDetector(String),
    #[error("Failed to convert bytes to string")]
    FromUtf8(#[from] std::string::FromUtf8Error),
    #[error("Storage Error: {0}")]
    Storage(#[from] reearth_flow_storage::Error),
    #[error("Object Store Error: {0}")]
    ObjectStore(#[from] object_store::Error),
}

#[derive(Debug, Clone)]
pub(crate) struct TransportationXlinkDetectorFactory {
    name: String,
    strategy: &'static dyn TransportationXlinkStrategy,
}

impl TransportationXlinkDetectorFactory {
    pub(crate) fn new(
        profile: &PlateauProfile,
        strategy: &'static dyn TransportationXlinkStrategy,
    ) -> Self {
        Self {
            name: profile.action_name("TransportationXlinkDetector"),
            strategy,
        }
    }
}

impl ProcessorFactory for TransportationXlinkDetectorFactory {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Detect unreferenced surfaces in PLATEAU transportation models (L-tran-03)"
    }

    fn parameter_schema(&self) -> Option<schemars::schema::RootSchema> {
        Some(schemars::schema_for!(TransportationXlinkDetectorParam))
    }

    fn categories(&self) -> &[&'static str] {
        &["PLATEAU"]
    }

    fn get_input_ports(&self) -> Vec<Port> {
        vec![FEATURES_PORT.clone()]
    }

    fn get_output_ports(&self) -> Vec<Port> {
        vec![PASSED_PORT.clone(), FAILED_PORT.clone()]
    }

    fn build(
        &self,
        _ctx: NodeContext,
        _event_hub: EventHub,
        _action: String,
        with: Option<HashMap<String, Value>>,
    ) -> Result<Box<dyn Processor>, BoxedError> {
        let params: TransportationXlinkDetectorParam = if let Some(with) = with {
            let value: Value = serde_json::to_value(with).map_err(|e| {
                PlateauProcessorError::TransportationXlinkDetectorFactory(format!(
                    "Failed to serialize `with` parameter: {e}"
                ))
            })?;
            serde_json::from_value(value).map_err(|e| {
                PlateauProcessorError::TransportationXlinkDetectorFactory(format!(
                    "Failed to deserialize `with` parameter: {e}"
                ))
            })?
        } else {
            return Err(PlateauProcessorError::TransportationXlinkDetectorFactory(
                "Missing required parameter `with`".to_string(),
            )
            .into());
        };

        let city_gml_path = params.city_gml_path.compile().map_err(|e| {
            PlateauProcessorError::TransportationXlinkDetectorFactory(format!(
                "Failed to compile city_gml_path: {e}"
            ))
        })?;

        let process = TransportationXlinkDetector {
            city_gml_path,
            strategy: self.strategy,
        };
        Ok(Box::new(process))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TransportationXlinkDetectorParam {
    city_gml_path: Code,
}

#[derive(Debug, Clone)]
pub struct TransportationXlinkDetector {
    city_gml_path: CompiledCode,
    strategy: &'static dyn TransportationXlinkStrategy,
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
        let city_gml_path = self
            .city_gml_path
            .eval_string(feature, ctx.variables.clone())
            .map_err(|e| {
                Error::TransportationXlinkDetector(format!(
                    "Failed to evaluate cityGmlPath expression: {e:?}"
                ))
            })?;
        let uri = Uri::from_str(&city_gml_path)?;
        let storage = ctx.storage_resolver.resolve(&uri)?;
        let content = storage.get_sync(uri.path().as_path())?;
        let xml_content = String::from_utf8(content.to_vec())?;

        let stream_error: Rc<RefCell<Option<Error>>> = Rc::new(RefCell::new(None));

        let transformer = Transformer::from(xml_content.as_str())
            .with_root_namespaces()
            .map_err(|e| Error::TransportationXlinkDetector(format!("{e:?}")))?;

        let strategy = self.strategy;
        let ctx = &ctx;

        let mut t = transformer;
        for container in strategy.containers() {
            // The reported feature type is the container's local name (`Road`).
            let feature_type = container
                .rsplit(':')
                .next()
                .unwrap_or(container)
                .to_string();
            let xpath = format!("//{container}");
            let stream_error = Rc::clone(&stream_error);

            t = t.on(&xpath, move |node| {
                if stream_error.borrow().is_some() {
                    return;
                }

                // Gate: the container must carry at least one aggregate LOD surface.
                let has_lod = node.children().iter().any(|c| {
                    let qname = c.qname();
                    strategy
                        .lods()
                        .iter()
                        .any(|(lod, _)| qname == strategy.aggregate_tag(lod))
                });
                if !has_lod {
                    return;
                }

                let doc = node.document();
                let mut xml_ctx = match xml::create_context(doc) {
                    Ok(ctx) => ctx,
                    Err(e) => {
                        *stream_error.borrow_mut() =
                            Some(Error::TransportationXlinkDetector(format!("{e:?}")));
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
                            Some(Error::TransportationXlinkDetector(format!("{e:?}")));
                        return;
                    }
                };

                match extract_unreferenced_surfaces(strategy, &xml_ctx, &root_node) {
                    Ok(Some(result)) => {
                        for (lod, surface_id) in result.unreferenced_surfaces {
                            let mut feature = feature.clone();
                            feature.refresh_id();

                            feature.attributes_mut().insert(
                                Attribute::new("gmlId"),
                                AttributeValue::String(result.road_id.clone()),
                            );
                            feature.attributes_mut().insert(
                                Attribute::new("featureType"),
                                AttributeValue::String(feature_type.clone()),
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
                        *stream_error.borrow_mut() = Some(e);
                    }
                }
            });
        }

        t.for_each()
            .map_err(|e| Error::TransportationXlinkDetector(format!("{e:?}")))?;

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
    strategy: &'static dyn TransportationXlinkStrategy,
    xml_ctx: &XmlContext,
    road_node: &XmlRoNode,
) -> Result<Option<UnreferencedSurfacesResult>, Error> {
    let road_id = road_node
        .get_attribute_ns("id", strategy.gml_namespace())
        .ok_or(Error::TransportationXlinkDetector(
            "Failed to get Road gml:id".to_string(),
        ))?;

    let mut all_unreferenced = Vec::new();

    for (lod_tag, lod_number) in strategy.lods() {
        if let Some(unreferenced) =
            check_lod_surfaces(strategy, xml_ctx, road_node, lod_tag, lod_number)?
        {
            all_unreferenced.extend(unreferenced);
        }
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
    strategy: &'static dyn TransportationXlinkStrategy,
    xml_ctx: &XmlContext,
    road_node: &XmlRoNode,
    lod_tag: &str,
    lod_number: &str,
) -> Result<Option<Vec<(String, String)>>, Error> {
    // All XLink references from the Road's aggregate lodXMultiSurface.
    let xlink_refs = xml::find_readonly_nodes_by_xpath(
        xml_ctx,
        &strategy.aggregate_xlink_xpath(lod_tag),
        road_node,
    )
    .map_err(|e| Error::TransportationXlinkDetector(format!("{e:?}")))?;

    let referenced_surfaces: HashSet<String> = xlink_refs
        .iter()
        .filter_map(|node| {
            let href = node.get_attribute_ns("href", XLINK_NS)?;
            Some(href.trim_start_matches('#').to_string())
        })
        .collect();

    // All child boundary surface IDs from TrafficArea and AuxiliaryTrafficArea.
    let child_surface_nodes = xml::find_readonly_nodes_by_xpath(
        xml_ctx,
        &strategy.child_surface_xpath(lod_tag),
        road_node,
    )
    .map_err(|e| Error::TransportationXlinkDetector(format!("{e:?}")))?;

    let mut child_surfaces = Vec::new();
    for surface_node in child_surface_nodes {
        if let Some(surface_id) = surface_node.get_attribute_ns("id", strategy.gml_namespace()) {
            child_surfaces.push(surface_id);
        }
    }

    // Surfaces defined on the boundaries but not referenced by the aggregate.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plateau4::transportation_xlink_strategy::Plateau4TransportationXlinkStrategy;
    use crate::plateau6::transportation_xlink_strategy::Plateau6TransportationXlinkStrategy;

    #[test]
    fn both_generations_scan_lod2_and_lod3_of_tran_road() {
        let strategies: [&dyn TransportationXlinkStrategy; 2] = [
            &Plateau4TransportationXlinkStrategy,
            &Plateau6TransportationXlinkStrategy,
        ];
        for strategy in strategies {
            assert_eq!(strategy.containers(), &["tran:Road"]);
            assert_eq!(strategy.lods(), &[("lod2", "2"), ("lod3", "3")]);
        }
    }

    #[test]
    fn the_aggregate_surface_moves_from_tran_to_core_in_citygml3() {
        assert_eq!(
            Plateau4TransportationXlinkStrategy.aggregate_tag("lod3"),
            "tran:lod3MultiSurface"
        );
        assert_eq!(
            Plateau6TransportationXlinkStrategy.aggregate_tag("lod3"),
            "core:lod3MultiSurface"
        );
    }

    #[test]
    fn the_xlink_xpath_hangs_off_the_generations_aggregate_tag() {
        assert_eq!(
            Plateau4TransportationXlinkStrategy.aggregate_xlink_xpath("lod2"),
            "tran:lod2MultiSurface//gml:surfaceMember[@xlink:href]"
        );
        assert_eq!(
            Plateau6TransportationXlinkStrategy.aggregate_xlink_xpath("lod2"),
            "core:lod2MultiSurface//gml:surfaceMember[@xlink:href]"
        );
    }

    #[test]
    fn citygml3_reaches_the_boundary_polygons_through_a_space_and_core_boundary() {
        let v2 = Plateau4TransportationXlinkStrategy.child_surface_xpath("lod3");
        assert!(v2.contains("tran:trafficArea/tran:TrafficArea/tran:lod3MultiSurface"));
        assert!(v2
            .contains("tran:auxiliaryTrafficArea/tran:AuxiliaryTrafficArea/tran:lod3MultiSurface"));

        let v3 = Plateau6TransportationXlinkStrategy.child_surface_xpath("lod3");
        assert!(v3.contains(
            "tran:trafficSpace/tran:TrafficSpace/core:boundary/tran:TrafficArea/core:lod3MultiSurface"
        ));
        assert!(v3.contains(
            "tran:auxiliaryTrafficSpace/tran:AuxiliaryTrafficSpace/core:boundary/tran:AuxiliaryTrafficArea/core:lod3MultiSurface"
        ));
    }

    #[test]
    fn gml_id_resolves_in_the_generations_own_gml_namespace() {
        assert_eq!(
            Plateau4TransportationXlinkStrategy.gml_namespace(),
            "http://www.opengis.net/gml"
        );
        assert_eq!(
            Plateau6TransportationXlinkStrategy.gml_namespace(),
            "http://www.opengis.net/gml/3.2"
        );
    }
}
