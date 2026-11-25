//! CityGML 3.0 specific namespace constants (GML 3.2 based).

use super::spec::CityGmlSpec;

pub(crate) const APP_NS: &str = "http://www.opengis.net/citygml/appearance/3.0";
pub(crate) const BLDG_NS: &str = "http://www.opengis.net/citygml/building/3.0";
pub(crate) const BRID_NS: &str = "http://www.opengis.net/citygml/bridge/3.0";
pub(crate) const CON_NS: &str = "http://www.opengis.net/citygml/construction/3.0";
pub(crate) const CORE_NS: &str = "http://www.opengis.net/citygml/3.0";
pub(crate) const DEM_NS: &str = "http://www.opengis.net/citygml/relief/3.0";
pub(crate) const DYN_NS: &str = "http://www.opengis.net/citygml/dynamizer/3.0";
pub(crate) const FRN_NS: &str = "http://www.opengis.net/citygml/cityfurniture/3.0";
pub(crate) const GEN_NS: &str = "http://www.opengis.net/citygml/generics/3.0";
pub(crate) const GML_NS: &str = "http://www.opengis.net/gml/3.2";
pub(crate) const GRP_NS: &str = "http://www.opengis.net/citygml/cityobjectgroup/3.0";
pub(crate) const LUSE_NS: &str = "http://www.opengis.net/citygml/landuse/3.0";
pub(crate) const PCL_NS: &str = "http://www.opengis.net/citygml/pointcloud/3.0";
pub(crate) const SCH_NS: &str = "http://www.ascc.net/xml/schematron";
pub(crate) const SMIL20_NS: &str = "http://www.w3.org/2001/SMIL20/";
pub(crate) const SMIL20LANG_NS: &str = "http://www.w3.org/2001/SMIL20/Language";
pub(crate) const TRAN_NS: &str = "http://www.opengis.net/citygml/transportation/3.0";
pub(crate) const TUN_NS: &str = "http://www.opengis.net/citygml/tunnel/3.0";
pub(crate) const VEG_NS: &str = "http://www.opengis.net/citygml/vegetation/3.0";
pub(crate) const VERS_NS: &str = "http://www.opengis.net/citygml/versioning/3.0";
pub(crate) const WTR_NS: &str = "http://www.opengis.net/citygml/waterbody/3.0";
pub(crate) const XAL_NS: &str = "urn:oasis:names:tc:ciq:xsdschema:xAL:2.0";
pub(crate) const XLINK_NS: &str = "http://www.w3.org/1999/xlink";
pub(crate) const XSI_NS: &str = "http://www.w3.org/2001/XMLSchema-instance";

/// Namespace specification for CityGML 3.0.
///
/// Main differences from CityGML 2.0:
/// - `gml` changes from GML 3.1.1 to GML 3.2 (`.../gml/3.2`)
/// - each citygml module changes from `/2.0` to `/3.0`
/// - the construction module `con` is added
/// - the dynamizer module `dyn` and the point cloud module `pcl` are added
/// - `pbase` / `tex` are not used in 3.0
pub static CITYGML3: CityGmlSpec = CityGmlSpec {
    gml_ns: GML_NS,
    core_ns: CORE_NS,
    namespaces: &[
        ("app", APP_NS),
        ("bldg", BLDG_NS),
        ("brid", BRID_NS),
        ("con", CON_NS),
        ("core", CORE_NS),
        ("dem", DEM_NS),
        ("dyn", DYN_NS),
        ("frn", FRN_NS),
        ("gen", GEN_NS),
        ("gml", GML_NS),
        ("grp", GRP_NS),
        ("luse", LUSE_NS),
        ("pcl", PCL_NS),
        ("sch", SCH_NS),
        ("smil20", SMIL20_NS),
        ("smil20lang", SMIL20LANG_NS),
        ("tran", TRAN_NS),
        ("tun", TUN_NS),
        ("veg", VEG_NS),
        ("wtr", WTR_NS),
        ("xAL", XAL_NS),
        ("xlink", XLINK_NS),
        ("xsi", XSI_NS),
    ],
};
