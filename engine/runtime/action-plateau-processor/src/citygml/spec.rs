//! CityGML / GML specification-level namespace information.
//!
//! Represents only the facts of the OGC CityGML specification itself, independent
//! of the PLATEAU product specification (iur / codelists / action names).
//! Version-specific values are defined as [`CityGmlSpec`] constants in
//! [`super::v2`] / [`super::v3`].

/// The set of namespaces used by a specific CityGML version.
#[derive(Debug, Clone, Copy)]
pub struct CityGmlSpec {
    /// GML namespace used for feature geometry and `gml:id`.
    /// CityGML 2.0 uses GML 3.1.1, CityGML 3.0 uses GML 3.2.
    pub gml_ns: &'static str,
    /// CityGML core namespace (e.g. `core:cityObjectMember`).
    pub core_ns: &'static str,
    /// Namespace prefix -> URI table defined by this version.
    /// Includes the citygml modules, gml, and W3C namespaces. iur (uro/urf) is
    /// PLATEAU product-specification-level information and is added by
    /// [`crate::common::PlateauProfile`] instead.
    pub namespaces: &'static [(&'static str, &'static str)],
}

impl CityGmlSpec {
    /// Returns the namespace URI for the given prefix.
    pub fn namespace(&self, prefix: &str) -> Option<&'static str> {
        self.namespaces
            .iter()
            .find(|(p, _)| *p == prefix)
            .map(|(_, uri)| *uri)
    }
}

/// Namespace used by codelist dictionaries (`gml:Dictionary`).
///
/// PLATEAU codelists use the GML 3.1.1 SimpleDictionary profile namespace
/// regardless of the CityGML version of the data itself, so this is a fixed
/// value independent of the CityGML version ([`CityGmlSpec::gml_ns`]).
pub const GML_DICTIONARY_NS: &str = "http://www.opengis.net/gml";
