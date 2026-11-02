//! The i-UR ADE versions whose schemas `reearth-flow-citygml` builds in.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// # i-UR Version
/// A version of the i-UR ADE whose schemas are built into Flow.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, JsonSchema)]
pub enum IurVersion {
    /// # i-UR 3.0
    /// Writes with the i-UR 3.0 schemas, whose namespaces end in `/3.0`.
    #[serde(rename = "3.0")]
    V3_0,
    /// # i-UR 3.1
    /// Writes with the i-UR 3.1 schemas, whose namespaces end in `/3.1`.
    #[serde(rename = "3.1")]
    V3_1,
    /// # i-UR 3.2
    /// Writes with the i-UR 3.2 schemas, whose namespaces end in `/3.2`.
    #[serde(rename = "3.2")]
    V3_2,
}

impl IurVersion {
    const ALL: [Self; 3] = [Self::V3_0, Self::V3_1, Self::V3_2];

    fn as_str(self) -> &'static str {
        match self {
            Self::V3_0 => "3.0",
            Self::V3_1 => "3.1",
            Self::V3_2 => "3.2",
        }
    }

    /// The `uro` and `urf` modules of this version.
    pub(crate) fn modules(self) -> [BuiltInAde; 2] {
        MODULES.map(|(prefix, file)| BuiltInAde::new(prefix, file, self))
    }
}

/// Each i-UR module's prefix and schema file name.
const MODULES: [(&str, &str); 2] = [("uro", "urbanObject"), ("urf", "urbanFunction")];

/// One built-in i-UR module of one version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BuiltInAde {
    pub prefix: &'static str,
    pub namespace: String,
    /// The published URL of the schema, which the built-in bundle holds a
    /// copy of, so it is read from there.
    pub location: String,
    /// Where a dataset keeps the schema, relative to a file in `udx/<type>/`.
    /// Data that declares it validates against the dataset's own copy.
    pub dataset_location: String,
}

impl BuiltInAde {
    fn new(prefix: &'static str, file: &str, version: IurVersion) -> Self {
        let version = version.as_str();
        Self {
            prefix,
            namespace: format!("https://www.geospatial.jp/iur/{prefix}/{version}"),
            location: format!(
                "https://www.geospatial.jp/iur/schemas/{prefix}/{version}/{file}.xsd"
            ),
            dataset_location: format!("../../schemas/iur/{prefix}/{version}/{file}.xsd"),
        }
    }
}

/// The built-in module whose namespace is `namespace`, of any version.
pub(crate) fn built_in(namespace: &str) -> Option<BuiltInAde> {
    IurVersion::ALL
        .into_iter()
        .flat_map(IurVersion::modules)
        .find(|module| module.namespace == namespace)
}

#[cfg(test)]
mod tests {
    use reearth_flow_citygml::schema::{AdeSource, SchemaSet};

    use super::*;

    #[test]
    fn every_built_in_module_loads_from_the_bundle() {
        let never = |url: &str| -> Result<Vec<u8>, String> { Err(format!("read {url}")) };
        for version in IurVersion::ALL {
            let sources: Vec<AdeSource> = version
                .modules()
                .into_iter()
                .map(|module| AdeSource {
                    namespace: module.namespace,
                    location: module.location,
                })
                .collect();
            if let Err(e) = SchemaSet::with_ade_from(&sources, &never) {
                panic!("i-UR {}: {e}", version.as_str());
            }
        }
    }

    #[test]
    fn a_namespace_finds_its_module_and_dataset_location() {
        let uro = built_in("https://www.geospatial.jp/iur/uro/3.2").unwrap();
        assert_eq!(uro.prefix, "uro");
        assert_eq!(
            uro.dataset_location,
            "../../schemas/iur/uro/3.2/urbanObject.xsd"
        );
        assert!(built_in("https://www.geospatial.jp/iur/uro/4.0").is_none());
    }
}
