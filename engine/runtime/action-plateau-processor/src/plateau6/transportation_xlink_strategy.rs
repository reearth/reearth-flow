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
    /// Descendant axis, because the traffic spaces may sit directly under the
    /// container or be nested through any depth of `tran:Section` /
    /// `tran:Intersection`.
    ///
    /// Limitation: a nested space that carries its own aggregate surface is not
    /// distinguished, so its boundary surfaces are still matched against the
    /// outermost container's references.
    fn child_surface_xpath(&self, lod: &str) -> String {
        format!(
            ".//tran:TrafficArea/core:{lod}MultiSurface//gml:Polygon[@gml:id] | \
             .//tran:AuxiliaryTrafficArea/core:{lod}MultiSurface//gml:Polygon[@gml:id]"
        )
    }
}
