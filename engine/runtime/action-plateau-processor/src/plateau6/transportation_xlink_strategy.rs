//! PLATEAU 6 (CityGML 3.0) extraction seam for the common transportation
//! xlink check.

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
