//! PLATEAU product-specification-level profile.
//!
//! Composes the CityGML specification ([`CityGmlSpec`]: version-specific
//! namespaces) with the iur (uro/urf) namespaces and action-name prefix that
//! vary by PLATEAU generation. The common check logic (`common/*.rs`) receives
//! this profile to absorb the generational differences. The concrete profile
//! instances are defined on the generation side (`plateau4` / `plateau6`).

use crate::citygml::CityGmlSpec;

/// A profile representing a single PLATEAU generation.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // uro_ns / urf_ns kept for documentation (only prefix keys are consulted today)
pub(crate) struct PlateauProfile {
    /// The CityGML specification referenced (2.0 / 3.0).
    pub citygml: &'static CityGmlSpec,
    /// i-UR uro namespace.
    pub uro_ns: &'static str,
    /// i-UR urf namespace.
    pub urf_ns: &'static str,
    /// Action-name prefix (e.g. `"PLATEAU4"` / `"PLATEAU6"`).
    pub action_prefix: &'static str,
}

impl PlateauProfile {
    /// Builds an action name (e.g. `"PLATEAU6.UDXFolderExtractor"`).
    pub fn action_name(&self, action: &str) -> String {
        format!("{}.{}", self.action_prefix, action)
    }

    /// Returns whether the prefix belongs to a known CityGML / i-UR namespace.
    pub fn is_known_namespace_prefix(&self, prefix: &str) -> bool {
        self.citygml.namespace(prefix).is_some() || prefix == "uro" || prefix == "urf"
    }

    /// i-UR namespace prefix used for DataQuality attributes (C07/C08 labels).
    ///
    /// i-UR 4.0 (shipped with CityGML 3.0) relocated the DataQuality package from
    /// the `uro` namespace to the new `urc` namespace, so the generation built on
    /// CityGML 3.0 reports DataQuality error paths with the `urc` prefix.
    pub fn dataquality_prefix(&self) -> &'static str {
        if std::ptr::eq(self.citygml, &crate::citygml::CITYGML3) {
            "urc"
        } else {
            "uro"
        }
    }
}
