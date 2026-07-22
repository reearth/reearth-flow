//! PLATEAU 4 (CityGML 2.0) extraction seam for the common unmatched-xlink check.
//!
//! Solids and boundary surfaces are `bldg:`-namespaced (`bldg:boundedBy`) and
//! `gml:id` resolves in the GML 3.1.1 namespace. The difference from PLATEAU 6 is
//! purely in these constants, so only the config accessors are provided and the
//! common default extraction is reused.

use crate::common::unmatched_xlink_detector::UnmatchedXlinkStrategy;

/// Zero-sized strategy; passed as `&Plateau4XlinkStrategy` (rvalue static
/// promotion yields the `&'static dyn` the factory needs), so no named static.
#[derive(Debug)]
pub(crate) struct Plateau4XlinkStrategy;

impl UnmatchedXlinkStrategy for Plateau4XlinkStrategy {
    fn containers(&self) -> &[&str] {
        &["bldg:Building", "bldg:BuildingPart", "bldg:Room"]
    }
    fn lod_geometry_tags(&self) -> &[&str] {
        &[
            "bldg:lod2Solid",
            "bldg:lod3Solid",
            "bldg:lod4Solid",
            "bldg:lod4MultiSurface",
        ]
    }
    fn boundary_container(&self) -> &str {
        "bldg:boundedBy"
    }
    fn gml_namespace(&self) -> &str {
        "http://www.opengis.net/gml"
    }
}
