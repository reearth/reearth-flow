//! PLATEAU product-specification-level profile.
//!
//! Composes the CityGML specification ([`CityGmlSpec`]: version-specific
//! namespaces) with the iur (uro/urf) namespaces and action-name prefix that
//! vary by PLATEAU generation. The common check logic (`common/*.rs`) receives
//! this profile to absorb the generational differences. The concrete profile
//! instances are defined on the generation side (`plateau4` / `plateau6`).

use crate::citygml::CityGmlSpec;

/// i-UR 3.x namespaces, which the generation built on CityGML 2.0 accepts in
/// every minor version.
pub(crate) const IUR3_NAMESPACES: &[(&str, &str)] = &[
    ("uro", "https://www.geospatial.jp/iur/uro/3.0"),
    ("uro", "https://www.geospatial.jp/iur/uro/3.1"),
    ("uro", "https://www.geospatial.jp/iur/uro/3.2"),
    ("urf", "https://www.geospatial.jp/iur/urf/3.0"),
    ("urf", "https://www.geospatial.jp/iur/urf/3.1"),
    ("urf", "https://www.geospatial.jp/iur/urf/3.2"),
];

/// i-UR 4.0 namespaces. `urc` first appears in 4.0.
pub(crate) const IUR4_NAMESPACES: &[(&str, &str)] = &[
    ("uro", "https://www.geospatial.jp/iur/uro/4.0"),
    ("urf", "https://www.geospatial.jp/iur/urf/4.0"),
    ("urc", "https://www.geospatial.jp/iur/urc/4.0"),
];

/// A profile representing a single PLATEAU generation.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PlateauProfile {
    /// The CityGML specification referenced (2.0 / 3.0).
    pub citygml: &'static CityGmlSpec,
    /// Each standard prefix and namespace URI of the i-UR versions the
    /// generation accepts. A prefix may have more than one URI.
    pub iur_namespaces: &'static [(&'static str, &'static str)],
    /// Action-name prefix (e.g. `"PLATEAU4"` / `"PLATEAU6"`).
    pub action_prefix: &'static str,
}

impl PlateauProfile {
    /// Each standard prefix and its namespace URI, the i-UR ones included.
    fn namespaces(&self) -> impl Iterator<Item = (&'static str, &'static str)> {
        self.citygml
            .namespaces
            .iter()
            .chain(self.iur_namespaces)
            .copied()
    }

    /// Returns the namespace URI of a standard prefix, the i-UR ones included.
    /// For an i-UR prefix with more than one version, the oldest one.
    pub fn namespace(&self, prefix: &str) -> Option<&'static str> {
        self.namespaces()
            .find(|(p, _)| *p == prefix)
            .map(|(_, uri)| uri)
    }

    /// Returns the standard prefix of a namespace URI, or `None` when the URI
    /// is not one of the versions the generation accepts. The objectlist and
    /// the checks use the standard prefix whatever prefix a document binds.
    pub fn prefix_of(&self, namespace: &str) -> Option<&'static str> {
        self.namespaces()
            .find(|(_, uri)| *uri == namespace)
            .map(|(prefix, _)| prefix)
    }

    /// Builds an action name (e.g. `"PLATEAU6.UDXFolderExtractor"`).
    pub fn action_name(&self, action: &str) -> String {
        format!("{}.{}", self.action_prefix, action)
    }

    /// i-UR namespace prefix used for DataQuality attributes (C07/C08 labels).
    ///
    /// i-UR 4.0 (shipped with CityGML 3.0) relocated the DataQuality package from
    /// the `uro` namespace to the new `urc` namespace, so the generation built on
    /// CityGML 3.0 reports DataQuality error paths with the `urc` prefix.
    pub fn dataquality_prefix(&self) -> &'static str {
        if self.is_citygml3() {
            "urc"
        } else {
            "uro"
        }
    }

    /// Property through which a root feature type (e.g. `bldg:Building`) holds
    /// its parts, renamed in CityGML 3.0 (e.g. `bldg:consistsOfBuildingPart` ->
    /// `bldg:buildingPart`).
    pub fn part_property(&self, feature_type: &str) -> Option<&'static str> {
        let properties: &[(&str, &str)] = if self.is_citygml3() {
            &[
                ("bldg:Building", "bldg:buildingPart"),
                ("brid:Bridge", "brid:bridgePart"),
                ("tun:Tunnel", "tun:tunnelPart"),
                // No `uro:UndergroundBuilding`: CityGML 3.0 declares
                // `bldg:buildingPart` on `bldg:Building` only, not on the
                // abstract building type it inherits from.
            ]
        } else {
            &[
                ("bldg:Building", "bldg:consistsOfBuildingPart"),
                ("brid:Bridge", "brid:consistsOfBridgePart"),
                ("tun:Tunnel", "tun:consistsOfTunnelPart"),
                ("uro:UndergroundBuilding", "bldg:consistsOfBuildingPart"),
            ]
        };
        properties
            .iter()
            .find(|(root, _)| *root == feature_type)
            .map(|(_, property)| *property)
    }

    /// Local names of the generic city object types in the `gen` namespace.
    /// CityGML 3.0 split `GenericCityObject` into one type per kind of space
    /// and one for surfaces.
    pub fn generic_city_object_types(&self) -> &'static [&'static str] {
        if self.is_citygml3() {
            &[
                "GenericOccupiedSpace",
                "GenericUnoccupiedSpace",
                "GenericLogicalSpace",
                "GenericThematicSurface",
            ]
        } else {
            &["GenericCityObject"]
        }
    }

    pub fn is_citygml3(&self) -> bool {
        std::ptr::eq(self.citygml, &crate::citygml::CITYGML3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static CITYGML2: PlateauProfile = PlateauProfile {
        citygml: &crate::citygml::CITYGML2,
        iur_namespaces: IUR3_NAMESPACES,
        action_prefix: "PLATEAU4",
    };
    static CITYGML3: PlateauProfile = PlateauProfile {
        citygml: &crate::citygml::CITYGML3,
        iur_namespaces: IUR4_NAMESPACES,
        action_prefix: "PLATEAU6",
    };

    #[test]
    // Only the i-UR versions of the generation have a standard prefix.
    fn test_prefix_of_iur_versions() {
        for version in ["3.0", "3.1", "3.2"] {
            let uro = format!("https://www.geospatial.jp/iur/uro/{version}");
            assert_eq!(CITYGML2.prefix_of(&uro), Some("uro"));
            assert_eq!(CITYGML3.prefix_of(&uro), None);
        }
        assert_eq!(
            CITYGML2.prefix_of("https://www.geospatial.jp/iur/uro/4.0"),
            None
        );
        assert_eq!(
            CITYGML3.prefix_of("https://www.geospatial.jp/iur/urc/4.0"),
            Some("urc")
        );
        assert_eq!(
            CITYGML3.prefix_of("http://www.opengis.net/citygml/building/2.0"),
            None
        );
    }
}
