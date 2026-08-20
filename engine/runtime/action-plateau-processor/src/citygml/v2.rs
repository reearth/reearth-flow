//! CityGML 2.0 specific namespace constants (GML 3.1.1 based).

use super::spec::CityGmlSpec;

/// Namespace specification for CityGML 2.0.
pub static CITYGML2: CityGmlSpec = CityGmlSpec {
    gml_ns: "http://www.opengis.net/gml",
    core_ns: "http://www.opengis.net/citygml/2.0",
    namespaces: &[
        ("app", "http://www.opengis.net/citygml/appearance/2.0"),
        ("bldg", "http://www.opengis.net/citygml/building/2.0"),
        ("brid", "http://www.opengis.net/citygml/bridge/2.0"),
        ("core", "http://www.opengis.net/citygml/2.0"),
        ("dem", "http://www.opengis.net/citygml/relief/2.0"),
        ("frn", "http://www.opengis.net/citygml/cityfurniture/2.0"),
        ("gen", "http://www.opengis.net/citygml/generics/2.0"),
        ("gml", "http://www.opengis.net/gml"),
        ("grp", "http://www.opengis.net/citygml/cityobjectgroup/2.0"),
        ("luse", "http://www.opengis.net/citygml/landuse/2.0"),
        ("pbase", "http://www.opengis.net/citygml/profiles/base/2.0"),
        ("sch", "http://www.ascc.net/xml/schematron"),
        ("smil20", "http://www.w3.org/2001/SMIL20/"),
        ("smil20lang", "http://www.w3.org/2001/SMIL20/Language"),
        ("tex", "http://www.opengis.net/citygml/texturedsurface/2.0"),
        ("tran", "http://www.opengis.net/citygml/transportation/2.0"),
        ("tun", "http://www.opengis.net/citygml/tunnel/2.0"),
        ("veg", "http://www.opengis.net/citygml/vegetation/2.0"),
        ("wtr", "http://www.opengis.net/citygml/waterbody/2.0"),
        ("xAL", "urn:oasis:names:tc:ciq:xsdschema:xAL:2.0"),
        ("xlink", "http://www.w3.org/1999/xlink"),
        ("xsi", "http://www.w3.org/2001/XMLSchema-instance"),
    ],
};
