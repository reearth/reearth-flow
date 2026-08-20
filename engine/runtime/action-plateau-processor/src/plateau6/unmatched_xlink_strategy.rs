//! PLATEAU 6 (CityGML 3.0) extraction seam for the common unmatched-xlink check.
//!
//! Solids hang off `core:` and boundary surfaces off `core:boundary` (`con:`
//! surfaces); `gml:id` resolves in the GML 3.2 namespace. LOD4 is abolished, so
//! only LOD2/LOD3 solids are scanned. The difference from PLATEAU 4 is purely in
//! these constants, so only the config accessors are provided and the common
//! default extraction is reused.

use crate::common::unmatched_xlink_detector::UnmatchedXlinkStrategy;

/// Zero-sized strategy; passed as `&Plateau6XlinkStrategy` (rvalue static
/// promotion yields the `&'static dyn` the factory needs), so no named static.
#[derive(Debug)]
pub(crate) struct Plateau6XlinkStrategy;

impl UnmatchedXlinkStrategy for Plateau6XlinkStrategy {
    fn containers(&self) -> &[&str] {
        &["bldg:Building", "bldg:BuildingPart", "bldg:BuildingRoom"]
    }
    fn lod_geometry_tags(&self) -> &[&str] {
        &["core:lod2Solid", "core:lod3Solid"]
    }
    fn boundary_container(&self) -> &str {
        "core:boundary"
    }
    fn gml_namespace(&self) -> &str {
        "http://www.opengis.net/gml/3.2"
    }
}
