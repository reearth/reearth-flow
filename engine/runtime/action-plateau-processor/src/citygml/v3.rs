//! CityGML 3.0 specific namespace constants (GML 3.2 based).

use super::spec::CityGmlSpec;

/// Namespace specification for CityGML 3.0.
///
/// Main differences from CityGML 2.0:
/// - `gml` changes from GML 3.1.1 to GML 3.2 (`.../gml/3.2`)
/// - each citygml module changes from `/2.0` to `/3.0`
/// - the construction module `con` is added
/// - `pbase` / `tex` are not used in 3.0
pub static CITYGML3: CityGmlSpec = CityGmlSpec {
    gml_ns: "http://www.opengis.net/gml/3.2",
    core_ns: "http://www.opengis.net/citygml/3.0",
    namespaces: &[
        ("app", "http://www.opengis.net/citygml/appearance/3.0"),
        ("bldg", "http://www.opengis.net/citygml/building/3.0"),
        ("brid", "http://www.opengis.net/citygml/bridge/3.0"),
        ("con", "http://www.opengis.net/citygml/construction/3.0"),
        ("core", "http://www.opengis.net/citygml/3.0"),
        ("dem", "http://www.opengis.net/citygml/relief/3.0"),
        ("frn", "http://www.opengis.net/citygml/cityfurniture/3.0"),
        ("gen", "http://www.opengis.net/citygml/generics/3.0"),
        ("gml", "http://www.opengis.net/gml/3.2"),
        ("grp", "http://www.opengis.net/citygml/cityobjectgroup/3.0"),
        ("luse", "http://www.opengis.net/citygml/landuse/3.0"),
        ("sch", "http://www.ascc.net/xml/schematron"),
        ("smil20", "http://www.w3.org/2001/SMIL20/"),
        ("smil20lang", "http://www.w3.org/2001/SMIL20/Language"),
        ("tran", "http://www.opengis.net/citygml/transportation/3.0"),
        ("tun", "http://www.opengis.net/citygml/tunnel/3.0"),
        ("veg", "http://www.opengis.net/citygml/vegetation/3.0"),
        ("wtr", "http://www.opengis.net/citygml/waterbody/3.0"),
        ("xAL", "urn:oasis:names:tc:ciq:xsdschema:xAL:2.0"),
        ("xlink", "http://www.w3.org/1999/xlink"),
        ("xsi", "http://www.w3.org/2001/XMLSchema-instance"),
    ],
};
