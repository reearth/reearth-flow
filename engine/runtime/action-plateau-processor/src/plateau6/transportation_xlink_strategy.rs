//! PLATEAU 6 (CityGML 3.0) extraction seam for the common transportation
//! xlink check.
//!
//! In the Space/SpaceBoundary model the Road aggregate surface hangs off
//! `core:` (`core:lodXMultiSurface`) and the boundary polygons live under
//! `tran:trafficSpace` / `tran:auxiliaryTrafficSpace` -> `core:boundary` ->
//! `tran:TrafficArea` / `tran:AuxiliaryTrafficArea`; `gml:id` resolves in the
//! GML 3.2 namespace. LOD4 is abolished, so only LOD2/LOD3 are scanned (the
//! common default). The difference from PLATEAU 4 is the aggregate namespace
//! prefix, the boundary nesting, and the `gml` namespace, so only those seams
//! are provided.

use crate::common::transportation_xlink_detector::TransportationXlinkStrategy;

/// Zero-sized strategy; passed as `&Plateau6TransportationXlinkStrategy` (rvalue
/// static promotion yields the `&'static dyn` the factory needs), so no named
/// static.
#[derive(Debug)]
pub(crate) struct Plateau6TransportationXlinkStrategy;

impl TransportationXlinkStrategy for Plateau6TransportationXlinkStrategy {
    fn containers(&self) -> &[&str] {
        &["tran:Road"]
    }
    fn aggregate_prefix(&self) -> &str {
        "core"
    }
    fn gml_namespace(&self) -> &str {
        "http://www.opengis.net/gml/3.2"
    }
    fn child_surface_xpath(&self, lod: &str) -> String {
        format!(
            "tran:trafficSpace/tran:TrafficSpace/core:boundary/tran:TrafficArea/core:{lod}MultiSurface//gml:Polygon[@gml:id] | \
             tran:auxiliaryTrafficSpace/tran:AuxiliaryTrafficSpace/core:boundary/tran:AuxiliaryTrafficArea/core:{lod}MultiSurface//gml:Polygon[@gml:id]"
        )
    }
}
