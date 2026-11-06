//! The ADE schemas `reearth-flow-citygml` builds in, each offered as a preset
//! an `adeSchemas` entry can name instead of giving a schema of its own.
//!
//! Each preset is one module of one version, so modules of different versions
//! can be combined, and bundling another ADE adds presets here and nothing to
//! the writer's parameters.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// # Built-in ADE Module
/// An ADE schema module built into Flow, named after its ADE, module and version.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, JsonSchema)]
pub enum BuiltInAde {
    /// # i-UR 3.0 Urban Object (uro)
    #[serde(rename = "iur-uro-3.0")]
    IurUro3_0,
    /// # i-UR 3.1 Urban Object (uro)
    #[serde(rename = "iur-uro-3.1")]
    IurUro3_1,
    /// # i-UR 3.2 Urban Object (uro)
    #[serde(rename = "iur-uro-3.2")]
    IurUro3_2,
    /// # i-UR 3.0 Urban Function (urf)
    #[serde(rename = "iur-urf-3.0")]
    IurUrf3_0,
    /// # i-UR 3.1 Urban Function (urf)
    #[serde(rename = "iur-urf-3.1")]
    IurUrf3_1,
    /// # i-UR 3.2 Urban Function (urf)
    #[serde(rename = "iur-urf-3.2")]
    IurUrf3_2,
}

impl BuiltInAde {
    #[cfg(test)]
    const ALL: [Self; 6] = [
        Self::IurUro3_0,
        Self::IurUro3_1,
        Self::IurUro3_2,
        Self::IurUrf3_0,
        Self::IurUrf3_1,
        Self::IurUrf3_2,
    ];

    /// What this module binds when an entry overrides nothing.
    pub(crate) fn preset(self) -> Preset {
        match self {
            Self::IurUro3_0 => iur(IurModule::Uro, "3.0"),
            Self::IurUro3_1 => iur(IurModule::Uro, "3.1"),
            Self::IurUro3_2 => iur(IurModule::Uro, "3.2"),
            Self::IurUrf3_0 => iur(IurModule::Urf, "3.0"),
            Self::IurUrf3_1 => iur(IurModule::Urf, "3.1"),
            Self::IurUrf3_2 => iur(IurModule::Urf, "3.2"),
        }
    }
}

/// The defaults a built-in module supplies to an `adeSchemas` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Preset {
    /// The prefix the module's own data uses, which an entry may override.
    pub prefix: &'static str,
    /// The module's `targetNamespace`, fixed by its schema.
    pub namespace: String,
    /// The published URL of the schema, which the built-in bundle holds a copy
    /// of, so it is read from there.
    pub location: String,
    /// The `xsi:schemaLocation` to declare unless the entry gives one.
    pub declared_location: String,
}

/// An i-UR module: its prefix and its schema file's name.
#[derive(Clone, Copy)]
enum IurModule {
    Uro,
    Urf,
}

/// An i-UR module of `version`, declared where a PLATEAU dataset keeps it,
/// relative to a file in `udx/<type>/`, so the output validates against the
/// dataset's own copy.
fn iur(module: IurModule, version: &str) -> Preset {
    let (prefix, file) = match module {
        IurModule::Uro => ("uro", "urbanObject"),
        IurModule::Urf => ("urf", "urbanFunction"),
    };
    Preset {
        prefix,
        namespace: format!("https://www.geospatial.jp/iur/{prefix}/{version}"),
        location: format!("https://www.geospatial.jp/iur/schemas/{prefix}/{version}/{file}.xsd"),
        declared_location: format!("../../schemas/iur/{prefix}/{version}/{file}.xsd"),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use reearth_flow_citygml::schema::{AdeSource, SchemaSet};

    use super::*;

    #[test]
    fn every_built_in_module_loads_from_the_bundle() {
        let never = |url: &str| -> Result<Vec<u8>, String> { Err(format!("read {url}")) };
        for module in BuiltInAde::ALL {
            let preset = module.preset();
            let sources = [AdeSource {
                namespace: preset.namespace,
                location: preset.location,
            }];
            if let Err(e) = SchemaSet::with_ade_from(&sources, &never) {
                panic!("{module:?}: {e}");
            }
        }
    }

    // Modules are chosen one by one, so versions may differ between them.
    #[test]
    fn modules_of_different_versions_load_together() {
        let never = |url: &str| -> Result<Vec<u8>, String> { Err(format!("read {url}")) };
        let sources: Vec<AdeSource> = [BuiltInAde::IurUro3_2, BuiltInAde::IurUrf3_1]
            .into_iter()
            .map(|module| {
                let preset = module.preset();
                AdeSource {
                    namespace: preset.namespace,
                    location: preset.location,
                }
            })
            .collect();

        SchemaSet::with_ade_from(&sources, &never).unwrap();
    }

    #[test]
    fn a_module_presets_its_prefix_namespace_and_dataset_location() {
        let uro = BuiltInAde::IurUro3_2.preset();

        assert_eq!(uro.prefix, "uro");
        assert_eq!(uro.namespace, "https://www.geospatial.jp/iur/uro/3.2");
        assert_eq!(
            uro.declared_location,
            "../../schemas/iur/uro/3.2/urbanObject.xsd"
        );
    }

    #[test]
    fn a_module_is_named_after_its_ade_module_and_version() {
        assert_eq!(
            serde_json::to_value(BuiltInAde::IurUrf3_1).unwrap(),
            serde_json::json!("iur-urf-3.1")
        );
    }
}
