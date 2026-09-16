//! PLATEAU 4 (CityGML 2.0) extraction seam for the common transportation
//! xlink check.

use crate::common::transportation_xlink_detector::TransportationXlinkStrategy;

/// Zero-sized strategy; passed as `&Plateau4TransportationXlinkStrategy` (rvalue
/// static promotion yields the `&'static dyn` the factory needs), so no named
/// static.
#[derive(Debug)]
pub(crate) struct Plateau4TransportationXlinkStrategy;

impl TransportationXlinkStrategy for Plateau4TransportationXlinkStrategy {
    fn containers(&self) -> &[&str] {
        &["tran:Road"]
    }
    fn aggregate_prefix(&self) -> &str {
        "tran"
    }
    fn gml_namespace(&self) -> &str {
        "http://www.opengis.net/gml"
    }
    fn child_surface_xpath(&self, lod: &str) -> String {
        format!(
            "tran:trafficArea/tran:TrafficArea/tran:{lod}MultiSurface//gml:Polygon[@gml:id] | \
             tran:auxiliaryTrafficArea/tran:AuxiliaryTrafficArea/tran:{lod}MultiSurface//gml:Polygon[@gml:id]"
        )
    }
}
