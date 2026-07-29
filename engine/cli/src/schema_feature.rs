use clap::{ArgMatches, Command};

/// The `schema-feature` subcommand.
pub fn build_schema_feature_command() -> Command {
    Command::new("schema-feature")
        .about("Show the production intermediate-data (feature) JSON schema.")
        .long_about(
            "Emit the JSON schema for the production intermediate-data form of a feature: the \
             record each line of intermediate-data JSONL holds, i.e. its id, attributes and \
             geometry. Requires the CLI to be built with `--features schema`.",
        )
}

/// One record of intermediate-data JSONL: a feature's id, its attributes, and its
/// geometry. Must stay in step with the serialized form of
/// `reearth_flow_types::Feature`, which it mirrors for schema generation only.
#[cfg(feature = "schema")]
#[derive(schemars::JsonSchema)]
#[schemars(rename = "Feature")]
#[allow(dead_code)]
struct FeatureSchema {
    id: uuid::Uuid,
    /// Open, string-keyed attribute values; untagged JSON, so unconstrained.
    attributes: std::collections::HashMap<String, serde_json::Value>,
    geometry: reearth_flow_geometry::Geometry,
}

#[derive(Debug, Eq, PartialEq)]
pub struct SchemaFeatureCliCommand;

impl SchemaFeatureCliCommand {
    pub fn parse_cli_args(_matches: ArgMatches) -> crate::Result<Self> {
        Ok(SchemaFeatureCliCommand)
    }

    #[cfg(feature = "schema")]
    pub fn execute(&self) -> crate::Result<()> {
        let schema = schemars::schema_for!(FeatureSchema);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
        Ok(())
    }

    #[cfg(not(feature = "schema"))]
    pub fn execute(&self) -> crate::Result<()> {
        Err(crate::errors::Error::init(
            "rebuild the CLI with `--features schema` to generate the intermediate-data schema"
                .to_string(),
        ))
    }
}

#[cfg(all(test, feature = "schema"))]
mod tests {
    /// `FeatureSchema` is a hand mirror, so nothing stops `Feature` from growing
    /// a field the schema never learns about. Compare the two field sets on a
    /// real value.
    #[cfg(feature = "new-geometry")]
    #[test]
    fn mirror_covers_every_field_feature_serializes() {
        use std::collections::BTreeSet;

        let feature = reearth_flow_types::Feature::new_with_attributes(Default::default());
        let serialized: serde_json::Value = serde_json::to_value(&feature).unwrap();
        let actual: BTreeSet<&str> = serialized
            .as_object()
            .expect("a feature serializes as an object")
            .keys()
            .map(String::as_str)
            .collect();

        let schema = serde_json::to_value(schemars::schema_for!(super::FeatureSchema)).unwrap();
        let mirrored: BTreeSet<&str> = schema["properties"]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();

        assert_eq!(
            actual, mirrored,
            "FeatureSchema no longer mirrors reearth_flow_types::Feature"
        );
    }

    /// The committed schema must match the current wire form. If this fails, run
    /// `cargo make schema-feature` and commit the result.
    #[test]
    fn committed_schema_is_up_to_date() {
        let generated =
            serde_json::to_string_pretty(&schemars::schema_for!(super::FeatureSchema)).unwrap();
        let committed = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../schema/feature-intermediate.schema.json"
        ));
        assert_eq!(
            generated.trim(),
            committed.trim(),
            "intermediate-data schema is stale; run `cargo make schema-feature` and commit"
        );
    }
}
